use async_trait::async_trait;
use chrono::Utc;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;
use url::Url;

use crate::domain::{
    AccountFailureKind, ApiAllowance, AuthoredPullRequest, CiResult, CiState, Comment, PrState,
    ProbePullRequest, PullRequestKey, PullRequestStack, Review, ReviewComment,
};

#[derive(Debug, Clone)]
pub struct AccountObservation {
    pub login: String,
    pub pull_requests: Vec<AuthoredPullRequest>,
    pub involved_node_ids: Vec<String>,
    pub checked_at: String,
}

/// One account's cheap change probe: open pull requests without observations.
#[derive(Debug, Clone)]
pub struct AccountProbe {
    pub login: String,
    pub pull_requests: Vec<ProbePullRequest>,
    pub allowance: Option<ApiAllowance>,
}

#[derive(Debug, Error, Clone)]
pub enum GitHubError {
    #[error("authentication failed")]
    Authentication,
    #[error("rate limited until {reset_at}")]
    RateLimited { reset_at: String },
    #[error("request timed out")]
    Timeout,
    #[error("transport error: {0}")]
    Transport(String),
    #[error("malformed response: {0}")]
    Malformed(String),
    #[error("GraphQL error: {0}")]
    GraphQl(String),
}

impl GitHubError {
    pub fn account_failure_kind(&self) -> AccountFailureKind {
        match self {
            GitHubError::Authentication => AccountFailureKind::Authentication,
            GitHubError::RateLimited { reset_at } => AccountFailureKind::RateLimited {
                reset_at: reset_at.clone(),
            },
            GitHubError::Timeout => AccountFailureKind::Timeout,
            GitHubError::Transport(_) => AccountFailureKind::Transport,
            GitHubError::Malformed(_) | GitHubError::GraphQl(_) => {
                AccountFailureKind::MalformedResponse
            }
        }
    }
}

#[async_trait]
pub trait GitHubGateway: Send + Sync {
    async fn observe(
        &self,
        token: &str,
        graphql_url: &Url,
        known_node_ids: &[String],
    ) -> Result<AccountObservation, GitHubError>;

    /// Query the open authored pull requests and the GraphQL allowance without
    /// building observations.
    async fn probe(&self, token: &str, graphql_url: &Url) -> Result<AccountProbe, GitHubError>;
}

#[derive(Debug, Clone)]
pub struct GitHubApiAdapter {
    client: reqwest::Client,
}

impl GitHubApiAdapter {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("passuff/0.1")
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("reqwest client");
        Self { client }
    }

    async fn post(
        &self,
        token: &str,
        graphql_url: &Url,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<String, GitHubError> {
        let response = self
            .client
            .post(graphql_url.clone())
            .bearer_auth(token)
            .json(&serde_json::json!({ "query": query, "variables": variables }))
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    GitHubError::Timeout
                } else {
                    GitHubError::Transport(error.to_string())
                }
            })?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(GitHubError::Authentication);
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            let reset_at = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|value| value.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            return Err(GitHubError::RateLimited { reset_at });
        }
        if !status.is_success() {
            return Err(GitHubError::Transport(format!("HTTP {status}")));
        }
        response
            .text()
            .await
            .map_err(|error| GitHubError::Transport(error.to_string()))
    }
}

#[async_trait]
impl GitHubGateway for GitHubApiAdapter {
    async fn observe(
        &self,
        token: &str,
        graphql_url: &Url,
        known_node_ids: &[String],
    ) -> Result<AccountObservation, GitHubError> {
        match self
            .observe_with(token, graphql_url, known_node_ids, MAIN_QUERY, NODES_QUERY)
            .await
        {
            Err(GitHubError::GraphQl(message)) if mentions_stack(&message) => {
                tracing::debug!("stack metadata unavailable; retrying without stack fields");
                self.observe_with(
                    token,
                    graphql_url,
                    known_node_ids,
                    &without_stack_fields(MAIN_QUERY),
                    &without_stack_fields(NODES_QUERY),
                )
                .await
            }
            result => result,
        }
    }

    async fn probe(&self, token: &str, graphql_url: &Url) -> Result<AccountProbe, GitHubError> {
        self.probe_with(token, graphql_url).await
    }
}

impl GitHubApiAdapter {
    async fn observe_with(
        &self,
        token: &str,
        graphql_url: &Url,
        known_node_ids: &[String],
        main_query: &str,
        nodes_query: &str,
    ) -> Result<AccountObservation, GitHubError> {
        tracing::debug!(
            host = graphql_url.host_str().unwrap_or("unknown"),
            "querying GitHub"
        );
        let mut login: Option<String> = None;
        let mut pull_requests = Vec::new();
        let after: Option<String> = None;
        let mut seen_nodes = std::collections::HashSet::new();

        for _ in 0..10 {
            let body = self
                .post(
                    token,
                    graphql_url,
                    main_query,
                    serde_json::json!({ "after": after }),
                )
                .await?;
            let response: GqlResponse = serde_json::from_str(&body)
                .map_err(|error| GitHubError::Malformed(error.to_string()))?;
            if let Some(errors) = response.errors.as_ref().filter(|errors| !errors.is_empty()) {
                if response.data.is_none() {
                    return Err(GitHubError::GraphQl(
                        errors
                            .iter()
                            .map(|error| error.message.clone())
                            .collect::<Vec<_>>()
                            .join("; "),
                    ));
                }
            }
            let data = response
                .data
                .ok_or_else(|| GitHubError::Malformed("missing data".to_string()))?;
            login.get_or_insert_with(|| data.viewer.login.clone());
            let Some(connection) = data.viewer.pull_requests else {
                break;
            };
            for node in connection.nodes {
                if seen_nodes.insert(node.id.clone()) {
                    pull_requests.push(map_pull_request(node));
                }
            }
            // One page bounds the response size; older authored pull requests
            // live in the watch state and are hydrated by node id.
            break;
        }

        if !known_node_ids.is_empty() {
            if let Ok(body) = self
                .post(
                    token,
                    graphql_url,
                    nodes_query,
                    serde_json::json!({ "ids": known_node_ids }),
                )
                .await
            {
                if let Ok(response) = serde_json::from_str::<GqlNodesResponse>(&body) {
                    if let Some(errors) =
                        response.errors.as_ref().filter(|errors| !errors.is_empty())
                    {
                        let message = errors
                            .iter()
                            .map(|error| error.message.clone())
                            .collect::<Vec<_>>()
                            .join("; ");
                        if response.data.is_none() && mentions_stack(&message) {
                            return Err(GitHubError::GraphQl(message));
                        }
                    }
                    if let Some(data) = response.data {
                        for node in data.nodes.into_iter().flatten() {
                            if seen_nodes.insert(node.id.clone()) {
                                pull_requests.push(map_pull_request(node));
                            }
                        }
                    }
                }
            }
        }

        let mut discovered: Vec<String> = Vec::new();
        for query in ["is:pr assignee:@me", "is:pr review-requested:@me"] {
            for id in self.search_pull_request_ids(token, graphql_url, query).await {
                if !seen_nodes.contains(&id) && !discovered.contains(&id) {
                    discovered.push(id);
                }
            }
        }
        if !discovered.is_empty() {
            if let Ok(body) = self
                .post(
                    token,
                    graphql_url,
                    nodes_query,
                    serde_json::json!({ "ids": discovered }),
                )
                .await
            {
                if let Ok(response) = serde_json::from_str::<GqlNodesResponse>(&body) {
                    if let Some(data) = response.data {
                        for node in data.nodes.into_iter().flatten() {
                            if seen_nodes.insert(node.id.clone()) {
                                pull_requests.push(map_pull_request(node));
                            }
                        }
                    }
                }
            }
        }

        let login =
            login.ok_or_else(|| GitHubError::Malformed("missing viewer login".to_string()))?;
        tracing::debug!(
            pull_requests = pull_requests.len(),
            "GitHub observation parsed"
        );
        Ok(AccountObservation {
            login,
            pull_requests,
            involved_node_ids: discovered,
            checked_at: Utc::now().to_rfc3339(),
        })
    }

    async fn search_pull_request_ids(
        &self,
        token: &str,
        graphql_url: &Url,
        query: &str,
    ) -> Vec<String> {
        let Ok(body) = self
            .post(
                token,
                graphql_url,
                SEARCH_QUERY,
                serde_json::json!({ "query": query }),
            )
            .await
        else {
            return Vec::new();
        };
        let Ok(response) = serde_json::from_str::<GqlSearchResponse>(&body) else {
            return Vec::new();
        };
        if response
            .errors
            .as_ref()
            .is_some_and(|errors| !errors.is_empty())
        {
            tracing::debug!("involved pull request search failed; skipping it");
            return Vec::new();
        }
        response
            .data
            .map(|data| {
                data.search
                    .nodes
                    .into_iter()
                    .flatten()
                    .map(|node| node.id)
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn probe_with(
        &self,
        token: &str,
        graphql_url: &Url,
    ) -> Result<AccountProbe, GitHubError> {
        let mut login: Option<String> = None;
        let mut pull_requests = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut allowance: Option<ApiAllowance> = None;
        let mut after: Option<String> = None;

        for _ in 0..10 {
            let body = self
                .post(
                    token,
                    graphql_url,
                    PROBE_QUERY,
                    serde_json::json!({ "after": after }),
                )
                .await?;
            let response: GqlProbeResponse = serde_json::from_str(&body)
                .map_err(|error| GitHubError::Malformed(error.to_string()))?;
            if let Some(errors) = response.errors.as_ref().filter(|errors| !errors.is_empty()) {
                if response.data.is_none() {
                    return Err(GitHubError::GraphQl(
                        errors
                            .iter()
                            .map(|error| error.message.clone())
                            .collect::<Vec<_>>()
                            .join("; "),
                    ));
                }
            }
            let data = response
                .data
                .ok_or_else(|| GitHubError::Malformed("missing data".to_string()))?;
            if let Some(rate) = data.rate_limit {
                let cost = rate.cost.min(u64::from(u32::MAX)) as u32;
                let limit = rate.limit.min(u64::from(u32::MAX)) as u32;
                let remaining = rate.remaining.min(u64::from(u32::MAX)) as u32;
                allowance = Some(match allowance {
                    Some(previous) => ApiAllowance {
                        cost: previous.cost.saturating_add(cost),
                        limit,
                        remaining,
                        reset_at: rate.reset_at,
                    },
                    None => ApiAllowance {
                        cost,
                        limit,
                        remaining,
                        reset_at: rate.reset_at,
                    },
                });
            }
            login.get_or_insert_with(|| data.viewer.login.clone());
            let Some(connection) = data.viewer.pull_requests else {
                break;
            };
            for node in connection.nodes {
                if !seen.insert(node.id.clone()) {
                    continue;
                }
                let ci_state = node
                    .commits
                    .nodes
                    .first()
                    .and_then(|commit| commit.commit.status_check_rollup.as_ref())
                    .map(|rollup| map_ci(rollup).state);
                pull_requests.push(ProbePullRequest {
                    repository: node.repository.name_with_owner,
                    number: node.number,
                    node_id: node.id,
                    updated_at: node.updated_at,
                    created_at: node.created_at,
                    head_oid: node
                        .commits
                        .nodes
                        .first()
                        .map(|commit| commit.commit.oid.clone())
                        .unwrap_or_default(),
                    ci_state,
                    state: map_pr_state(&node.state, node.merged),
                });
            }
            let page_info = connection.page_info.unwrap_or_default();
            if page_info.has_next_page {
                match page_info.end_cursor {
                    Some(cursor) if !cursor.is_empty() => after = Some(cursor),
                    _ => break,
                }
            } else {
                break;
            }
        }

        let login =
            login.ok_or_else(|| GitHubError::Malformed("missing viewer login".to_string()))?;
        tracing::debug!(
            pull_requests = pull_requests.len(),
            "GitHub change probe parsed"
        );
        Ok(AccountProbe {
            login,
            pull_requests,
            allowance,
        })
    }
}

fn mentions_stack(message: &str) -> bool {
    message.contains("stack") || message.contains("stackEntry")
}

fn without_stack_fields(query: &str) -> String {
    query
        .replace(
            "        stack { number size baseRefName }\n        stackEntry { position }\n",
            "",
        )
        .replace(
            "      stack { number size baseRefName }\n      stackEntry { position }\n",
            "",
        )
}

fn map_pr_state(state: &str, merged: bool) -> PrState {
    if merged || state.eq_ignore_ascii_case("MERGED") {
        PrState::Merged
    } else if state.eq_ignore_ascii_case("CLOSED") {
        PrState::Closed
    } else {
        PrState::Open
    }
}

fn map_pull_request(node: GqlPr) -> AuthoredPullRequest {
    let state = map_pr_state(&node.state, node.merged);
    let ci = node
        .commits
        .nodes
        .first()
        .and_then(|node| node.commit.status_check_rollup.as_ref())
        .map(map_ci)
        .unwrap_or_default();
    let head_oid = node
        .commits
        .nodes
        .first()
        .map(|node| node.commit.oid.clone())
        .unwrap_or_default();
    let mut review_comments = Vec::new();
    for thread in node.review_threads.nodes {
        for comment in thread.comments.nodes {
            review_comments.push(ReviewComment {
                id: comment.id,
                author: comment
                    .author
                    .and_then(|author| author.login)
                    .unwrap_or_else(|| "ghost".to_string()),
                body: comment.body,
                created_at: comment.created_at,
            });
        }
    }
    let stack = map_stack(node.stack, node.stack_entry);
    AuthoredPullRequest {
        key: PullRequestKey::new(node.repository.name_with_owner, node.number),
        node_id: node.id,
        author: node
            .author
            .and_then(|author| author.login)
            .unwrap_or_else(|| "ghost".to_string()),
        title: node.title,
        body: node.body.unwrap_or_default(),
        url: node.url,
        created_at: node.created_at,
        updated_at: node.updated_at,
        state,
        merged: node.merged,
        is_draft: node.is_draft,
        assignees: node
            .assignees
            .nodes
            .into_iter()
            .filter_map(|author| author.login)
            .collect(),
        labels: node
            .labels
            .nodes
            .into_iter()
            .map(|label| label.name)
            .collect(),
        milestone: node.milestone.and_then(|milestone| milestone.title),
        base_ref: node.base_ref_name,
        merged_by: node.merged_by.and_then(|actor| actor.login),
        stack,
        head_oid,
        ci,
        comments: node
            .comments
            .nodes
            .into_iter()
            .map(|comment| Comment {
                id: comment.id,
                author: comment
                    .author
                    .and_then(|author| author.login)
                    .unwrap_or_else(|| "ghost".to_string()),
                body: comment.body,
                created_at: comment.created_at,
            })
            .collect(),
        reviews: node
            .reviews
            .nodes
            .into_iter()
            .map(|review| Review {
                id: review.id,
                author: review
                    .author
                    .and_then(|author| author.login)
                    .unwrap_or_else(|| "ghost".to_string()),
                state: review.state,
                body: review.body,
                submitted_at: review.submitted_at.unwrap_or_default(),
            })
            .collect(),
        review_comments,
    }
}

fn map_stack(stack: Option<GqlStack>, entry: Option<GqlStackEntry>) -> Option<PullRequestStack> {
    let stack = stack?;
    let entry = entry?;
    let number = stack.number?;
    let size = stack.size?;
    let base = stack.base_ref_name?;
    let position = entry.position?;
    if size >= 1 && position >= 1 && position <= size {
        Some(PullRequestStack {
            number,
            size,
            position,
            base,
        })
    } else {
        None
    }
}

fn map_ci(rollup: &GqlRollup) -> CiResult {
    let state = match rollup.state.to_ascii_uppercase().as_str() {
        "SUCCESS" => CiState::Success,
        "FAILURE" => CiState::Failure,
        "ERROR" => CiState::Error,
        "PENDING" => CiState::Pending,
        _ => CiState::None,
    };
    let mut failed_checks = Vec::new();
    if let Some(contexts) = rollup.contexts.as_ref() {
        for context in &contexts.nodes {
            let failed = context.conclusion.as_deref().is_some_and(|value| {
                value.eq_ignore_ascii_case("FAILURE") || value.eq_ignore_ascii_case("ERROR")
            }) || context.state.as_deref().is_some_and(|value| {
                value.eq_ignore_ascii_case("FAILURE") || value.eq_ignore_ascii_case("ERROR")
            });
            if failed {
                if let Some(name) = context.name.as_ref().or(context.context.as_ref()) {
                    failed_checks.push(name.clone());
                }
            }
        }
    }
    CiResult {
        state,
        failed_checks,
    }
}

const MAIN_QUERY: &str = r#"
query($after: String) {
  viewer {
    login
    pullRequests(first: 50, after: $after, states: [OPEN, MERGED, CLOSED], orderBy: {field: UPDATED_AT, direction: DESC}) {
      pageInfo { hasNextPage endCursor }
      nodes {
        id
        number
        title
        body
        url
        state
        merged
        isDraft
        createdAt
        updatedAt
        baseRefName
        mergedBy { login }
        stack { number size baseRefName }
        stackEntry { position }
        author { login }
        repository { nameWithOwner }
        assignees(first: 20) { nodes { login } }
        labels(first: 20) { nodes { name } }
        milestone { title }
        comments(last: 100) { nodes { id author { login } body createdAt } }
        reviews(last: 100) { nodes { id author { login } state body submittedAt } }
        reviewThreads(last: 50) { nodes { comments(last: 20) { nodes { id author { login } body createdAt } } } }
        commits(last: 1) { nodes { commit { oid statusCheckRollup { state contexts(first: 50) { nodes { __typename ... on CheckRun { name conclusion } ... on StatusContext { context state } } } } } } }
      }
    }
  }
}
"#;

const NODES_QUERY: &str = r#"
query($ids: [ID!]!) {
  nodes(ids: $ids) {
    ... on PullRequest {
      id
      number
      title
      body
      url
      state
      merged
      isDraft
      createdAt
      updatedAt
      baseRefName
      mergedBy { login }
      stack { number size baseRefName }
      stackEntry { position }
      author { login }
      repository { nameWithOwner }
      assignees(first: 20) { nodes { login } }
      labels(first: 20) { nodes { name } }
      milestone { title }
      comments(last: 100) { nodes { id author { login } body createdAt } }
      reviews(last: 100) { nodes { id author { login } state body submittedAt } }
      reviewThreads(last: 50) { nodes { comments(last: 20) { nodes { id author { login } body createdAt } } } }
      commits(last: 1) { nodes { commit { oid statusCheckRollup { state contexts(first: 50) { nodes { __typename ... on CheckRun { name conclusion } ... on StatusContext { context state } } } } } } }
    }
  }
}
"#;

const PROBE_QUERY: &str = r#"
query($after: String) {
  rateLimit { cost limit remaining resetAt }
  viewer {
    login
    pullRequests(first: 100, after: $after, states: [OPEN, MERGED, CLOSED], orderBy: {field: UPDATED_AT, direction: DESC}) {
      pageInfo { hasNextPage endCursor }
      nodes {
        id
        number
        updatedAt
        createdAt
        state
        merged
        repository { nameWithOwner }
        commits(last: 1) { nodes { commit { oid statusCheckRollup { state } } } }
      }
    }
  }
}
"#;

#[derive(Debug, Deserialize)]
struct GqlResponse {
    data: Option<GqlData>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Debug, Deserialize)]
struct GqlData {
    viewer: GqlViewer,
}

#[derive(Debug, Deserialize)]
struct GqlViewer {
    login: String,
    #[serde(rename = "pullRequests")]
    pull_requests: Option<GqlConnection<GqlPr>>,
}

#[derive(Debug, Deserialize)]
struct GqlConnection<T> {
    nodes: Vec<T>,
}

#[derive(Debug, Default, Deserialize)]
struct GqlPageInfo {
    #[serde(rename = "hasNextPage", default)]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GqlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct GqlNodesResponse {
    data: Option<GqlNodesData>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Debug, Deserialize)]
struct GqlNodesData {
    #[serde(default)]
    nodes: Vec<Option<GqlPr>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlPr {
    id: String,
    number: u64,
    title: String,
    #[serde(default)]
    body: Option<String>,
    url: String,
    state: String,
    #[serde(default)]
    merged: bool,
    #[serde(default)]
    is_draft: bool,
    created_at: String,
    updated_at: String,
    base_ref_name: String,
    #[serde(rename = "mergedBy")]
    merged_by: Option<GqlActor>,
    stack: Option<GqlStack>,
    stack_entry: Option<GqlStackEntry>,
    author: Option<GqlActor>,
    repository: GqlRepo,
    #[serde(default)]
    assignees: GqlActors,
    #[serde(default)]
    labels: GqlLabels,
    milestone: Option<GqlMilestone>,
    #[serde(default)]
    comments: GqlComments,
    #[serde(default)]
    reviews: GqlReviews,
    #[serde(default, rename = "reviewThreads")]
    review_threads: GqlThreads,
    #[serde(default)]
    commits: GqlCommits,
}

#[derive(Debug, Default, Deserialize)]
struct GqlActors {
    #[serde(default)]
    nodes: Vec<GqlActor>,
}

#[derive(Debug, Deserialize)]
struct GqlActor {
    login: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct GqlLabels {
    #[serde(default)]
    nodes: Vec<GqlLabel>,
}

#[derive(Debug, Deserialize)]
struct GqlLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GqlMilestone {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GqlRepo {
    #[serde(rename = "nameWithOwner")]
    name_with_owner: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlStack {
    number: Option<u64>,
    size: Option<u64>,
    base_ref_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GqlStackEntry {
    position: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct GqlComments {
    #[serde(default)]
    nodes: Vec<GqlComment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlComment {
    id: String,
    author: Option<GqlActor>,
    body: String,
    created_at: String,
}

#[derive(Debug, Default, Deserialize)]
struct GqlReviews {
    #[serde(default)]
    nodes: Vec<GqlReview>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlReview {
    id: String,
    author: Option<GqlActor>,
    state: String,
    body: String,
    submitted_at: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct GqlThreads {
    #[serde(default)]
    nodes: Vec<GqlThread>,
}

#[derive(Debug, Default, Deserialize)]
struct GqlThread {
    #[serde(default)]
    comments: GqlComments,
}

#[derive(Debug, Default, Deserialize)]
struct GqlCommits {
    #[serde(default)]
    nodes: Vec<GqlCommitNode>,
}

#[derive(Debug, Deserialize)]
struct GqlCommitNode {
    commit: GqlCommit,
}

#[derive(Debug, Deserialize)]
struct GqlCommit {
    oid: String,
    #[serde(rename = "statusCheckRollup")]
    status_check_rollup: Option<GqlRollup>,
}

#[derive(Debug, Deserialize)]
struct GqlRollup {
    state: String,
    contexts: Option<GqlContexts>,
}

#[derive(Debug, Deserialize)]
struct GqlContexts {
    #[serde(default)]
    nodes: Vec<GqlContext>,
}

const SEARCH_QUERY: &str = r#"
query($query: String!) {
  search(query: $query, type: ISSUE, first: 100) {
    nodes { ... on PullRequest { id } }
  }
}
"#;

#[derive(Debug, Deserialize)]
struct GqlSearchResponse {
    data: Option<GqlSearchData>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Debug, Deserialize)]
struct GqlSearchData {
    search: GqlSearchConnection,
}

#[derive(Debug, Deserialize)]
struct GqlSearchConnection {
    nodes: Vec<Option<GqlSearchNode>>,
}

#[derive(Debug, Deserialize)]
struct GqlSearchNode {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GqlProbeResponse {
    data: Option<GqlProbeData>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Debug, Deserialize)]
struct GqlProbeData {
    #[serde(rename = "rateLimit")]
    rate_limit: Option<GqlRateLimit>,
    viewer: GqlProbeViewer,
}

#[derive(Debug, Deserialize)]
struct GqlRateLimit {
    cost: u64,
    limit: u64,
    remaining: u64,
    #[serde(rename = "resetAt")]
    reset_at: String,
}

#[derive(Debug, Deserialize)]
struct GqlProbeViewer {
    login: String,
    #[serde(rename = "pullRequests")]
    pull_requests: Option<GqlProbeConnection>,
}

#[derive(Debug, Deserialize)]
struct GqlProbeConnection {
    nodes: Vec<GqlProbePr>,
    #[serde(rename = "pageInfo")]
    page_info: Option<GqlPageInfo>,
}

#[derive(Debug, Deserialize)]
struct GqlProbePr {
    id: String,
    number: u64,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    state: String,
    merged: bool,
    repository: GqlRepo,
    commits: GqlCommits,
}

#[derive(Debug, Deserialize)]
struct GqlContext {
    name: Option<String>,
    conclusion: Option<String>,
    context: Option<String>,
    state: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn endpoint(server: &MockServer) -> Url {
        Url::parse(&format!("{}/graphql", server.uri())).expect("url")
    }

    async fn mount(server: &MockServer, status: u16, body: serde_json::Value) {
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .mount(server)
            .await;
    }

    fn node() -> serde_json::Value {
        serde_json::json!({
            "id": "node-1",
            "number": 42,
            "title": "Pull request",
            "body": "PR description",
            "url": "https://github.com/acme/api/pull/42",
            "state": "OPEN",
            "merged": false,
            "isDraft": true,
            "createdAt": "2026-11-02T10:00:00Z",
            "updatedAt": "2026-11-02T11:00:00Z",
            "baseRefName": "main",
            "author": { "login": "octo" },
            "repository": { "nameWithOwner": "acme/api" },
            "assignees": { "nodes": [{ "login": "alice" }] },
            "labels": { "nodes": [{ "name": "bug" }] },
            "milestone": { "title": "v1" },
            "comments": { "nodes": [{
                "id": "c1", "author": { "login": "alice" }, "body": "hi",
                "createdAt": "2026-11-02T10:30:00Z"
            }] },
            "reviews": { "nodes": [{
                "id": "r1", "author": { "login": "bob" }, "state": "APPROVED",
                "body": "", "submittedAt": "2026-11-02T10:40:00Z"
            }] },
            "reviewThreads": { "nodes": [{ "comments": { "nodes": [{
                "id": "rc1", "author": { "login": "carol" }, "body": "nit",
                "createdAt": "2026-11-02T10:50:00Z"
            }] } }] },
            "commits": { "nodes": [{ "commit": {
                "oid": "abc123",
                "statusCheckRollup": {
                    "state": "FAILURE",
                    "contexts": { "nodes": [
                        { "__typename": "CheckRun", "name": "build", "conclusion": "FAILURE" },
                        { "__typename": "StatusContext", "context": "unit", "state": "ERROR" }
                    ] }
                }
            } }] }
        })
    }

    #[tokio::test]
    async fn observe_hydrates_assigned_pull_requests() {
        let server = MockServer::start().await;
        let mut assigned = node();
        assigned["id"] = serde_json::json!("node-2");
        assigned["number"] = serde_json::json!(7);
        assigned["repository"] = serde_json::json!({ "nameWithOwner": "acme/web" });
        assigned["author"] = serde_json::json!({ "login": "alice" });
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("search("))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "search": { "nodes": [{ "id": "node-2" }] } }
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("nodes(ids:"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "nodes": [assigned] }
            })))
            .mount(&server)
            .await;
        mount(
            &server,
            200,
            serde_json::json!({
                "data": { "viewer": { "login": "octo", "pullRequests": {
                    "pageInfo": { "hasNextPage": false, "endCursor": null },
                    "nodes": [node()]
                } } }
            }),
        )
        .await;
        let adapter = GitHubApiAdapter::new();
        let observation = adapter
            .observe("token", &endpoint(&server), &[])
            .await
            .expect("observation");
        assert_eq!(observation.pull_requests.len(), 2);
        assert!(observation
            .pull_requests
            .iter()
            .any(|pull_request| pull_request.key.to_string() == "acme/web#7"));
    }

    #[tokio::test]
    async fn observe_accepts_null_review_submission_and_body() {
        let server = MockServer::start().await;
        let mut node = node();
        node["body"] = serde_json::json!(null);
        node["reviews"]["nodes"][0]["submittedAt"] = serde_json::json!(null);
        mount(
            &server,
            200,
            serde_json::json!({
                "data": { "viewer": { "login": "octo", "pullRequests": {
                    "pageInfo": { "hasNextPage": false, "endCursor": null },
                    "nodes": [node]
                } } }
            }),
        )
        .await;
        let adapter = GitHubApiAdapter::new();
        let observation = adapter
            .observe("token", &endpoint(&server), &[])
            .await
            .expect("observation");
        assert_eq!(observation.pull_requests[0].body, "");
        assert_eq!(observation.pull_requests[0].reviews[0].submitted_at, "");
    }

    #[tokio::test]
    async fn observe_maps_authored_pull_requests() {
        let server = MockServer::start().await;
        mount(
            &server,
            200,
            serde_json::json!({
                "data": { "viewer": { "login": "octo", "pullRequests": {
                    "pageInfo": { "hasNextPage": false, "endCursor": null },
                    "nodes": [node()]
                } } }
            }),
        )
        .await;
        let observation = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect("observation");
        assert_eq!(observation.login, "octo");
        let pull_request = &observation.pull_requests[0];
        assert_eq!(pull_request.key.to_string(), "acme/api#42");
        assert_eq!(pull_request.author, "octo");
        assert_eq!(pull_request.body, "PR description");
        assert_eq!(pull_request.state, PrState::Open);
        assert!(pull_request.is_draft);
        assert_eq!(pull_request.head_oid, "abc123");
        assert_eq!(pull_request.ci.state, CiState::Failure);
        assert_eq!(pull_request.ci.failed_checks, vec!["build", "unit"]);
        assert_eq!(pull_request.comments[0].author, "alice");
        assert_eq!(pull_request.reviews[0].state, "APPROVED");
        assert_eq!(pull_request.review_comments[0].id, "rc1");
        assert_eq!(pull_request.labels, vec!["bug"]);
        assert_eq!(pull_request.assignees, vec!["alice"]);
        assert_eq!(pull_request.milestone.as_deref(), Some("v1"));
    }

    #[tokio::test]
    async fn observe_maps_http_failures() {
        let server = MockServer::start().await;
        mount(&server, 401, serde_json::json!({})).await;
        let error = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect_err("authentication error");
        assert!(matches!(error, GitHubError::Authentication));
        assert_eq!(
            error.account_failure_kind(),
            AccountFailureKind::Authentication
        );

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(
                ResponseTemplate::new(429).insert_header("x-ratelimit-reset", "1700000000"),
            )
            .mount(&server)
            .await;
        let error = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect_err("rate limit error");
        assert!(matches!(
            error,
            GitHubError::RateLimited { reset_at } if reset_at == "1700000000"
        ));

        let server = MockServer::start().await;
        mount(&server, 500, serde_json::json!({})).await;
        let error = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect_err("transport error");
        assert!(matches!(error, GitHubError::Transport(_)));
    }

    #[tokio::test]
    async fn observe_maps_graphql_and_body_failures() {
        let server = MockServer::start().await;
        mount(
            &server,
            200,
            serde_json::json!({ "errors": [{ "message": "Bad query" }] }),
        )
        .await;
        let error = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect_err("graphql error");
        assert!(matches!(error, GitHubError::GraphQl(message) if message == "Bad query"));

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;
        let error = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect_err("malformed response");
        assert!(matches!(error, GitHubError::Malformed(_)));
    }

    #[tokio::test]
    async fn observe_maps_terminal_states_and_missing_authors() {
        let server = MockServer::start().await;
        let mut merged = node();
        merged["id"] = serde_json::json!("node-1");
        merged["number"] = serde_json::json!(1);
        merged["state"] = serde_json::json!("MERGED");
        merged["merged"] = serde_json::json!(true);
        merged["author"] = serde_json::Value::Null;
        merged["commits"]["nodes"][0]["commit"]["statusCheckRollup"]["state"] =
            serde_json::json!("ERROR");
        let mut closed = node();
        closed["id"] = serde_json::json!("node-2");
        closed["number"] = serde_json::json!(2);
        closed["state"] = serde_json::json!("CLOSED");
        closed["commits"]["nodes"][0]["commit"]["statusCheckRollup"]["state"] =
            serde_json::json!("PENDING");
        let mut unknown = node();
        unknown["id"] = serde_json::json!("node-3");
        unknown["number"] = serde_json::json!(3);
        unknown["commits"]["nodes"][0]["commit"]["statusCheckRollup"]["state"] =
            serde_json::json!("SOMETHING");
        mount(
            &server,
            200,
            serde_json::json!({
                "data": { "viewer": { "login": "octo", "pullRequests": {
                    "pageInfo": { "hasNextPage": false, "endCursor": null },
                    "nodes": [merged, closed, unknown]
                } } }
            }),
        )
        .await;
        let observation = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect("observation");
        assert_eq!(observation.pull_requests[0].state, PrState::Merged);
        assert_eq!(observation.pull_requests[0].author, "ghost");
        assert_eq!(observation.pull_requests[0].ci.state, CiState::Error);
        assert_eq!(observation.pull_requests[1].state, PrState::Closed);
        assert_eq!(observation.pull_requests[1].ci.state, CiState::Pending);
        assert_eq!(observation.pull_requests[2].ci.state, CiState::None);
    }

    #[tokio::test]
    async fn observe_reports_transport_failures_and_missing_data() {
        let unreachable = Url::parse("http://127.0.0.1:1/graphql").expect("url");
        let error = GitHubApiAdapter::new()
            .observe("token", &unreachable, &[])
            .await
            .expect_err("transport error");
        assert!(matches!(error, GitHubError::Transport(_)));

        let server = MockServer::start().await;
        mount(&server, 200, serde_json::json!({ "data": null })).await;
        let error = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect_err("missing data");
        assert!(
            matches!(error, GitHubError::Malformed(message) if message.contains("missing data"))
        );
    }

    #[tokio::test]
    async fn observe_accepts_a_viewer_without_pull_requests() {
        let server = MockServer::start().await;
        mount(
            &server,
            200,
            serde_json::json!({
                "data": { "viewer": { "login": "octo", "pullRequests": null } }
            }),
        )
        .await;
        let observation = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect("observation");
        assert_eq!(observation.login, "octo");
        assert!(observation.pull_requests.is_empty());
    }
    #[tokio::test]
    async fn maps_stack_membership() {
        let server = MockServer::start().await;
        let mut stacked = node();
        stacked["stack"] = serde_json::json!({
            "number": 7, "size": 2, "baseRefName": "main"
        });
        stacked["stackEntry"] = serde_json::json!({ "position": 1 });
        mount(
            &server,
            200,
            serde_json::json!({
                "data": { "viewer": { "login": "octo", "pullRequests": {
                    "pageInfo": { "hasNextPage": false, "endCursor": null },
                    "nodes": [stacked]
                } } }
            }),
        )
        .await;
        let observation = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect("observation");
        assert_eq!(
            observation.pull_requests[0].stack,
            Some(PullRequestStack {
                number: 7,
                size: 2,
                position: 1,
                base: "main".to_string(),
            })
        );
    }

    #[tokio::test]
    async fn incomplete_stack_membership_is_standalone() {
        for (stack, entry) in [
            (
                Some(serde_json::json!({ "number": 7, "size": 2, "baseRefName": "main" })),
                None,
            ),
            (
                Some(serde_json::json!({ "number": 7, "size": 2, "baseRefName": "main" })),
                Some(serde_json::json!({ "position": 0 })),
            ),
            (
                Some(serde_json::json!({ "number": 7, "size": 2, "baseRefName": "main" })),
                Some(serde_json::json!({ "position": 3 })),
            ),
            (None, Some(serde_json::json!({ "position": 1 }))),
        ] {
            let server = MockServer::start().await;
            let mut incomplete = node();
            if let Some(stack) = stack {
                incomplete["stack"] = stack;
            }
            if let Some(entry) = entry {
                incomplete["stackEntry"] = entry;
            }
            mount(
                &server,
                200,
                serde_json::json!({
                    "data": { "viewer": { "login": "octo", "pullRequests": {
                        "pageInfo": { "hasNextPage": false, "endCursor": null },
                        "nodes": [incomplete]
                    } } }
                }),
            )
            .await;
            let observation = GitHubApiAdapter::new()
                .observe("token", &endpoint(&server), &[])
                .await
                .expect("observation");
            assert!(observation.pull_requests[0].stack.is_none());
        }
    }

    struct RetryResponder {
        calls: std::sync::atomic::AtomicUsize,
    }

    impl wiremock::Respond for RetryResponder {
        fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
            let call = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if call == 0 {
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "errors": [{ "message": "Field 'stack' doesn't exist on type 'PullRequest'" }]
                }))
            } else {
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": { "viewer": { "login": "octo", "pullRequests": {
                        "pageInfo": { "hasNextPage": false, "endCursor": null },
                        "nodes": [node()]
                    } } }
                }))
            }
        }
    }

    #[tokio::test]
    async fn rejects_stack_fields_and_retries_without_them() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(RetryResponder {
                calls: std::sync::atomic::AtomicUsize::new(0),
            })
            .mount(&server)
            .await;
        let observation = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &[])
            .await
            .expect("retry observation");
        assert_eq!(observation.login, "octo");
        assert_eq!(observation.pull_requests.len(), 1);
        assert!(observation.pull_requests[0].stack.is_none());
    }

    #[test]
    fn queries_are_read_only() {
        assert!(!MAIN_QUERY.to_ascii_lowercase().contains("mutation"));
        assert!(!NODES_QUERY.to_ascii_lowercase().contains("mutation"));
        assert!(!without_stack_fields(MAIN_QUERY).contains("stackEntry"));
        assert!(!without_stack_fields(NODES_QUERY).contains("stack {"));
    }
    struct NodesRetryResponder {
        calls: std::sync::atomic::AtomicUsize,
    }

    impl wiremock::Respond for NodesRetryResponder {
        fn respond(&self, request: &wiremock::Request) -> ResponseTemplate {
            let call = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap_or_default();
            let is_nodes = body
                .get("variables")
                .and_then(|variables| variables.get("ids"))
                .is_some();
            if is_nodes && call == 1 {
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "errors": [{ "message": "Field 'stackEntry' doesn't exist on type 'PullRequest'" }]
                }))
            } else if is_nodes {
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": { "nodes": [node()] }
                }))
            } else {
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": { "viewer": { "login": "octo", "pullRequests": {
                        "pageInfo": { "hasNextPage": false, "endCursor": null },
                        "nodes": [node()]
                    } } }
                }))
            }
        }
    }

    #[tokio::test]
    async fn nodes_stack_error_triggers_retry() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(NodesRetryResponder {
                calls: std::sync::atomic::AtomicUsize::new(0),
            })
            .mount(&server)
            .await;
        let observation = GitHubApiAdapter::new()
            .observe("token", &endpoint(&server), &["node-1".to_string()])
            .await
            .expect("retry observation");
        assert_eq!(observation.login, "octo");
        assert_eq!(observation.pull_requests.len(), 1);
        assert!(observation.pull_requests[0].stack.is_none());
    }
}
