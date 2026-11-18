//! GitHub GraphQL observation. One named operation per POST; dispatch in the
//! acceptance double is a substring match on the query text.

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::model::{CiResult, PrState, StackInfo};

const USER_AGENT: &str = "passuff";

#[derive(Debug, Clone)]
pub struct GithubError {
    pub message: String,
}

impl std::fmt::Display for GithubError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GithubError {}

fn unavailable(detail: impl std::fmt::Display) -> GithubError {
    GithubError {
        message: format!("GitHub is unavailable: {detail}"),
    }
}

fn authentication_failed() -> GithubError {
    GithubError {
        message: "GitHub authentication failed".to_string(),
    }
}

fn rate_limited(reset: Option<String>) -> GithubError {
    GithubError {
        message: match reset {
            Some(reset) => format!("GitHub rate limit reached until {reset}"),
            None => "GitHub rate limit reached".to_string(),
        },
    }
}

fn malformed(detail: impl std::fmt::Display) -> GithubError {
    GithubError {
        message: format!("GitHub returned a malformed response: {detail}"),
    }
}

#[derive(Debug, Clone, Default)]
pub struct RateLimit {
    pub remaining: Option<u64>,
    pub reset_at: Option<String>,
    pub cost: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CommentObservation {
    pub id: String,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub author_is_bot: bool,
}

#[derive(Debug, Clone)]
pub struct ReviewObservation {
    pub id: String,
    pub body: Option<String>,
    pub state: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub author_is_bot: bool,
}

#[derive(Debug, Clone)]
pub struct PullRequestObservation {
    pub key: String,
    pub node_id: Option<String>,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: PrState,
    pub is_draft: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub url: Option<String>,
    pub head_ref_oid: Option<String>,
    pub author: Option<String>,
    pub author_is_bot: bool,
    pub assignees: Vec<String>,
    pub ci: CiResult,
    pub comments: Vec<CommentObservation>,
    pub reviews: Vec<ReviewObservation>,
    pub review_comments: Vec<CommentObservation>,
    pub stack: Option<StackInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryObservation {
    pub authored: Vec<PullRequestObservation>,
    pub assigned: Vec<PullRequestObservation>,
    pub review_requested: Vec<PullRequestObservation>,
    pub team_review_requested: Vec<PullRequestObservation>,
    pub rate_limit: RateLimit,
}

#[derive(Debug, Clone, Default)]
pub struct ProbeObservation {
    pub nodes: Vec<PullRequestObservation>,
    pub rate_limit: RateLimit,
}

pub struct GithubClient {
    agent: ureq::Agent,
}

struct RawResponse {
    root: Value,
    errors: Vec<String>,
}

impl Default for GithubClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GithubClient {
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .proxy(None)
            .timeout_global(Some(Duration::from_secs(20)))
            .timeout_connect(Some(Duration::from_secs(5)))
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
        }
    }

    pub fn viewer(&self, url: &str, token: &str) -> Result<String, GithubError> {
        let query = "query PassuffViewer { viewer { login } }";
        let root = self.call(url, token, query, json!({}), false)?;
        root.pointer("/data/viewer/login")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| malformed("viewer login is missing"))
    }

    pub fn authored(
        &self,
        url: &str,
        token: &str,
    ) -> Result<(Vec<PullRequestObservation>, RateLimit), GithubError> {
        let root = self.call_with_stack_retry(url, token, authored_query, json!({}), false)?;
        let nodes = nodes_at(&root, "/data/viewer/pullRequests/nodes");
        let observations = nodes.iter().filter_map(parse_pull_request).collect();
        Ok((observations, rate_limit(&root)))
    }

    pub fn discovery(&self, url: &str, token: &str) -> Result<DiscoveryObservation, GithubError> {
        let root = self.call_with_stack_retry(url, token, discovery_query, json!({}), true)?;
        let mut discovery = DiscoveryObservation {
            rate_limit: rate_limit(&root),
            ..DiscoveryObservation::default()
        };
        discovery.assigned = nodes_at(&root, "/data/assigned/nodes")
            .iter()
            .filter_map(parse_pull_request)
            .collect();
        discovery.review_requested = nodes_at(&root, "/data/reviewRequested/nodes")
            .iter()
            .filter_map(parse_pull_request)
            .collect();
        discovery.team_review_requested = nodes_at(&root, "/data/teamReviewRequested/nodes")
            .iter()
            .filter_map(parse_pull_request)
            .collect();
        Ok(discovery)
    }

    pub fn probe(&self, url: &str, token: &str) -> Result<ProbeObservation, GithubError> {
        let root = self.call(url, token, &probe_query(), json!({}), false)?;
        let nodes = nodes_at(&root, "/data/viewer/pullRequests/nodes")
            .iter()
            .filter_map(parse_pull_request)
            .collect();
        Ok(ProbeObservation {
            nodes,
            rate_limit: rate_limit(&root),
        })
    }

    pub fn observe(
        &self,
        url: &str,
        token: &str,
        ids: &[String],
    ) -> Result<Vec<PullRequestObservation>, GithubError> {
        let query = observe_query(true);
        let root = self.call_with_stack_retry(
            url,
            token,
            |with_stack| {
                if with_stack {
                    query.clone()
                } else {
                    observe_query(false)
                }
            },
            json!({ "ids": ids }),
            false,
        )?;
        let nodes = nodes_at(&root, "/data/nodes");
        Ok(nodes.iter().filter_map(parse_pull_request).collect())
    }

    fn call(
        &self,
        url: &str,
        token: &str,
        query: &str,
        variables: Value,
        allow_partial_errors: bool,
    ) -> Result<Value, GithubError> {
        let response = self.send(url, token, query, variables)?;
        self.finish(response, allow_partial_errors)
    }

    fn call_with_stack_retry<F>(
        &self,
        url: &str,
        token: &str,
        build_query: F,
        variables: Value,
        allow_partial_errors: bool,
    ) -> Result<Value, GithubError>
    where
        F: Fn(bool) -> String,
    {
        let first = build_query(true);
        match self.send(url, token, &first, variables.clone()) {
            Ok(response) if !contains_stack(&response.errors) => {
                return self.finish(response, allow_partial_errors);
            }
            Ok(_) => {}
            Err(error) if error.message.contains("stack") => {}
            Err(error) => return Err(error),
        }
        let second = build_query(false);
        let response = self.send(url, token, &second, variables)?;
        self.finish(response, allow_partial_errors)
    }

    fn finish(
        &self,
        response: RawResponse,
        allow_partial_errors: bool,
    ) -> Result<Value, GithubError> {
        if response.errors.is_empty() {
            return Ok(response.root);
        }
        if allow_partial_errors {
            return Ok(response.root);
        }
        let joined = response.errors.join("; ");
        if joined.to_ascii_lowercase().contains("rate limit") {
            return Err(rate_limited(reset_from(&response.root)));
        }
        Err(malformed(joined))
    }

    fn send(
        &self,
        url: &str,
        token: &str,
        query: &str,
        variables: Value,
    ) -> Result<RawResponse, GithubError> {
        let body = json!({ "query": query, "variables": variables });
        let authorization = format!("Bearer {token}");
        let response = self
            .agent
            .post(url)
            .header("Authorization", authorization)
            .header("User-Agent", USER_AGENT)
            .send_json(&body)
            .map_err(|error| match error {
                ureq::Error::StatusCode(status) => status_error(status, None),
                other => unavailable(other),
            })?;

        let status = response.status().as_u16();
        let mut response = response;
        let text = response
            .body_mut()
            .read_to_string()
            .map_err(|error| malformed(format!("cannot read response body: {error}")))?;

        let root: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if status == 401 || status == 403 {
            return Err(authentication_failed());
        }
        if status == 429 {
            return Err(rate_limited(reset_from(&root)));
        }
        if !(200..300).contains(&status) {
            let excerpt: String = text.chars().take(200).collect();
            return Err(malformed(format!("HTTP status {status}: {excerpt}")));
        }
        if root.is_null() {
            return Err(malformed("response body is not JSON"));
        }

        let errors = root
            .get("errors")
            .and_then(Value::as_array)
            .map(|errors| {
                errors
                    .iter()
                    .filter_map(|error| {
                        error
                            .get("message")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(RawResponse { root, errors })
    }
}

fn status_error(status: u16, reset: Option<String>) -> GithubError {
    match status {
        401 | 403 => authentication_failed(),
        429 => rate_limited(reset),
        other => malformed(format!("HTTP status {other}")),
    }
}

fn contains_stack(errors: &[String]) -> bool {
    errors.iter().any(|error| error.contains("stack"))
}

fn nodes_at<'a>(root: &'a Value, pointer: &str) -> &'a [Value] {
    root.pointer(pointer)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn reset_from(root: &Value) -> Option<String> {
    root.pointer("/rateLimit/resetAt")
        .or_else(|| root.pointer("/data/rateLimit/resetAt"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn rate_limit(root: &Value) -> RateLimit {
    let value = root
        .pointer("/rateLimit")
        .or_else(|| root.pointer("/data/rateLimit"));
    let Some(value) = value else {
        return RateLimit::default();
    };
    RateLimit {
        remaining: value.get("remaining").and_then(Value::as_u64),
        reset_at: value
            .get("resetAt")
            .and_then(Value::as_str)
            .map(str::to_string),
        cost: value.get("cost").and_then(Value::as_u64),
    }
}

fn shared_selection(with_stack: bool) -> String {
    let mut fields = vec![
        "id",
        "number",
        "title",
        "body",
        "state",
        "isDraft",
        "createdAt",
        "updatedAt",
        "mergedAt",
        "closedAt",
        "url",
        "headRefOid",
        "author { login __typename }",
        "assignees(first: 10) { nodes { login } }",
        "statusCheckRollup { state }",
        "comments(first: 100) { nodes { id body createdAt author { login __typename } } }",
        "reviews(first: 100) { nodes { id body state submittedAt author { login __typename } } }",
        "reviewThreads(first: 50) { nodes { comments(first: 50) { nodes { id body createdAt author { login __typename } } } } }",
        "repository { name owner { login } }",
    ];
    if with_stack {
        fields.push("stack { number size baseRefName }");
        fields.push("stackEntry { position }");
    }
    fields
        .into_iter()
        .map(|field| format!("        {field}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn probe_query() -> String {
    r#"query PassuffProbe {
  viewer {
    login
    pullRequests(first: 100, orderBy: {field: UPDATED_AT, direction: DESC}) {
      nodes {
        id
        number
        state
        createdAt
        updatedAt
        headRefOid
        statusCheckRollup { state }
        repository { name owner { login } }
      }
    }
  }
  rateLimit { remaining resetAt cost }
}"#
    .to_string()
}

fn authored_query(with_stack: bool) -> String {
    format!(
        "query PassuffAuthored {{\n  viewer {{\n    pullRequests(first: 100, orderBy: {{field: UPDATED_AT, direction: DESC}}) {{\n      nodes {{\n{}\n      }}\n    }}\n  }}\n}}",
        shared_selection(with_stack)
    )
}

fn discovery_query(with_stack: bool) -> String {
    let selection = shared_selection(with_stack);
    format!(
        "query PassuffDiscovery {{\n  assigned: search(query: \"is:open is:pr assignee:@me\", type: ISSUE, first: 100) {{\n    nodes {{ ... on PullRequest {{\n{}\n    }} }}\n  }}\n  reviewRequested: search(query: \"is:open is:pr review-requested:@me\", type: ISSUE, first: 100) {{\n    nodes {{ ... on PullRequest {{\n{}\n    }} }}\n  }}\n  teamReviewRequested: search(query: \"is:open is:pr team-review-requested:@me\", type: ISSUE, first: 100) {{\n    nodes {{ ... on PullRequest {{\n{}\n    }} }}\n  }}\n}}",
        selection, selection, selection
    )
}

fn observe_query(with_stack: bool) -> String {
    format!(
        "query PassuffObserve($ids: [ID!]!) {{\n  nodes(ids: $ids) {{\n    ... on PullRequest {{\n{}\n    }}\n  }}\n}}",
        shared_selection(with_stack)
    )
}

fn parse_pull_request(value: &Value) -> Option<PullRequestObservation> {
    let number = value.get("number").and_then(Value::as_u64)?;
    let repository = value.get("repository")?;
    let name = repository.get("name").and_then(Value::as_str)?;
    let owner = repository
        .get("owner")
        .and_then(|owner| owner.get("login"))
        .and_then(Value::as_str)?;
    let key = format!("{owner}/{name}#{number}");

    let author = value
        .get("author")
        .and_then(|author| author.get("login"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let author_is_bot = value
        .get("author")
        .map(|author| {
            author.get("__typename").and_then(Value::as_str) == Some("Bot")
                || author
                    .get("login")
                    .and_then(Value::as_str)
                    .is_some_and(|login| login.ends_with("[bot]"))
        })
        .unwrap_or(false);

    let comments = comments_at(value, "/comments/nodes");
    let reviews = reviews_at(value, "/reviews/nodes");
    let mut review_comments = Vec::new();
    if let Some(threads) = value
        .pointer("/reviewThreads/nodes")
        .and_then(Value::as_array)
    {
        for thread in threads {
            review_comments.extend(comments_at(thread, "/comments/nodes"));
        }
    }

    Some(PullRequestObservation {
        key,
        node_id: value.get("id").and_then(Value::as_str).map(str::to_string),
        number,
        title: value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        body: value
            .get("body")
            .and_then(Value::as_str)
            .filter(|body| !body.is_empty())
            .map(str::to_string),
        state: PrState::from_graphql(value.get("state").and_then(Value::as_str).unwrap_or("OPEN")),
        is_draft: value
            .get("isDraft")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        created_at: timestamp(value, "createdAt"),
        updated_at: timestamp(value, "updatedAt"),
        merged_at: timestamp(value, "mergedAt"),
        closed_at: timestamp(value, "closedAt"),
        url: value.get("url").and_then(Value::as_str).map(str::to_string),
        head_ref_oid: value
            .get("headRefOid")
            .and_then(Value::as_str)
            .map(str::to_string),
        author,
        author_is_bot,
        assignees: value
            .pointer("/assignees/nodes")
            .and_then(Value::as_array)
            .map(|nodes| {
                nodes
                    .iter()
                    .filter_map(|node| node.get("login").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        ci: CiResult::from_graphql(
            value
                .pointer("/statusCheckRollup/state")
                .and_then(Value::as_str),
        ),
        comments,
        reviews,
        review_comments,
        stack: parse_stack(value),
    })
}

fn parse_stack(value: &Value) -> Option<StackInfo> {
    let stack = value.get("stack")?;
    if stack.is_null() {
        return None;
    }
    let size = stack.get("size").and_then(Value::as_u64)?;
    if size == 0 {
        return None;
    }
    let position = value
        .pointer("/stackEntry/position")
        .and_then(Value::as_u64)?;
    Some(StackInfo {
        number: stack.get("number").and_then(Value::as_u64)?,
        size,
        base: stack
            .get("baseRefName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        position,
    })
}

fn comments_at(value: &Value, pointer: &str) -> Vec<CommentObservation> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|node| {
                    Some(CommentObservation {
                        id: node.get("id").and_then(Value::as_str)?.to_string(),
                        body: node
                            .get("body")
                            .and_then(Value::as_str)
                            .filter(|body| !body.is_empty())
                            .map(str::to_string),
                        created_at: timestamp(node, "createdAt"),
                        author: node
                            .get("author")
                            .and_then(|author| author.get("login"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        author_is_bot: node
                            .get("author")
                            .map(|author| {
                                author.get("__typename").and_then(Value::as_str) == Some("Bot")
                                    || author
                                        .get("login")
                                        .and_then(Value::as_str)
                                        .is_some_and(|login| login.ends_with("[bot]"))
                            })
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn reviews_at(value: &Value, pointer: &str) -> Vec<ReviewObservation> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|node| {
                    Some(ReviewObservation {
                        id: node.get("id").and_then(Value::as_str)?.to_string(),
                        body: node
                            .get("body")
                            .and_then(Value::as_str)
                            .filter(|body| !body.is_empty())
                            .map(str::to_string),
                        state: node
                            .get("state")
                            .and_then(Value::as_str)
                            .unwrap_or("COMMENTED")
                            .to_string(),
                        submitted_at: timestamp(node, "submittedAt"),
                        author: node
                            .get("author")
                            .and_then(|author| author.get("login"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        author_is_bot: node
                            .get("author")
                            .map(|author| {
                                author.get("__typename").and_then(Value::as_str) == Some("Bot")
                                    || author
                                        .get("login")
                                        .and_then(Value::as_str)
                                        .is_some_and(|login| login.ends_with("[bot]"))
                            })
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn timestamp(value: &Value, field: &str) -> Option<DateTime<Utc>> {
    value
        .get(field)
        .and_then(Value::as_str)
        .and_then(|text| DateTime::parse_from_rfc3339(text).ok())
        .map(|time| time.with_timezone(&Utc))
}
