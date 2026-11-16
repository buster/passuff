use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use passuff_config::{AccountConfig, FocusRelation};
use serde_json::{json, Value};

use crate::model::{
    Allowance, CiResult, CommentData, ProbePullRequest, ProbeReport, PullRequestKey,
    PullRequestSnapshot, PullRequestState, ReviewCommentData, ReviewData, ReviewState, StackInfo,
};

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("GitHub authentication failed")]
    Authentication,
    #[error("GitHub rate limit reached{}", .0.as_deref().map(|at| format!(" until {at}")).unwrap_or_default())]
    RateLimited(Option<String>),
    #[error("GitHub is unavailable: {0}")]
    Unavailable(String),
    #[error("GitHub returned a malformed response: {0}")]
    Malformed(String),
}

pub trait GitHubGateway: Send {
    fn viewer_login(&mut self) -> Result<String, GatewayError>;
    fn probe(&mut self) -> Result<ProbeReport, GatewayError>;
    fn list_authored(&mut self) -> Result<Vec<PullRequestSnapshot>, GatewayError>;
    fn discover_involved(&mut self) -> Result<Vec<PullRequestSnapshot>, GatewayError>;
    fn observe(
        &mut self,
        keys: &[PullRequestKey],
    ) -> Result<Vec<PullRequestSnapshot>, GatewayError>;
    fn allowance(&self) -> Allowance;
}

pub trait GatewayFactory: Send + Sync {
    fn create(
        &self,
        label: &str,
        account: &AccountConfig,
        token: &str,
    ) -> Result<Box<dyn GitHubGateway>, GatewayError>;
}

#[derive(Debug, Default)]
pub struct SystemGatewayFactory;

impl GatewayFactory for SystemGatewayFactory {
    fn create(
        &self,
        label: &str,
        account: &AccountConfig,
        token: &str,
    ) -> Result<Box<dyn GitHubGateway>, GatewayError> {
        let endpoint = if let Ok(override_url) = std::env::var("PASSUFF_GRAPHQL_URL") {
            override_url
        } else {
            account.graphql_url().to_string()
        };
        Ok(Box::new(GraphQlGateway::new(label, &endpoint, token)?))
    }
}

pub struct GraphQlGateway {
    label: String,
    endpoint: String,
    token: String,
    client: reqwest::blocking::Client,
    node_ids: BTreeMap<PullRequestKey, String>,
    allowance: Allowance,
}

impl GraphQlGateway {
    pub fn new(label: &str, endpoint: &str, token: &str) -> Result<Self, GatewayError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| GatewayError::Unavailable(error.to_string()))?;
        Ok(Self {
            label: label.to_string(),
            endpoint: endpoint.to_string(),
            token: token.to_string(),
            client,
            node_ids: BTreeMap::new(),
            allowance: Allowance::default(),
        })
    }

    fn request(&mut self, query: &str, variables: Value) -> Result<Value, GatewayError> {
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.token)
            .header("User-Agent", "passuff")
            .json(&json!({"query": query, "variables": variables}))
            .send()
            .map_err(|error| GatewayError::Unavailable(error.to_string()))?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(GatewayError::Authentication);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(GatewayError::RateLimited(None));
        }
        let body = response
            .text()
            .map_err(|error| GatewayError::Unavailable(error.to_string()))?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|error| GatewayError::Malformed(format!("{error}: {}", self.label)))?;
        if let Some(errors) = value.get("errors").and_then(Value::as_array) {
            if !errors.is_empty() {
                let message = errors
                    .iter()
                    .filter_map(|error| error.get("message").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("; ");
                if message.to_ascii_lowercase().contains("rate limit") {
                    return Err(GatewayError::RateLimited(None));
                }
                // A failed search alias (for example a repository without the
                // team-review search qualifier) must not fail the account's
                // authored observation. Use whatever partial data arrived.
                if !query.contains("PassuffDiscovery") {
                    return Err(GatewayError::Malformed(message));
                }
            }
        }
        if let Some(rate) = value.pointer("/data/rateLimit") {
            self.allowance = parse_allowance(rate);
        }
        Ok(value)
    }

    fn request_full(
        &mut self,
        query_with_stack: &str,
        variables: Value,
    ) -> Result<Value, GatewayError> {
        match self.request(query_with_stack, variables.clone()) {
            Ok(value) => Ok(value),
            Err(GatewayError::Malformed(message))
                if message.to_ascii_lowercase().contains("stack") =>
            {
                let without_stack = strip_stack_fields(query_with_stack);
                self.request(&without_stack, variables)
            }
            Err(error) => Err(error),
        }
    }
}

impl GitHubGateway for GraphQlGateway {
    fn viewer_login(&mut self) -> Result<String, GatewayError> {
        let value = self.request("query PassuffViewer { viewer { login } }", json!({}))?;
        value
            .pointer("/data/viewer/login")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| GatewayError::Malformed("missing viewer login".to_string()))
    }

    fn probe(&mut self) -> Result<ProbeReport, GatewayError> {
        let value = self.request(PROBE_QUERY, json!({}))?;
        let nodes = value
            .pointer("/data/viewer/pullRequests/nodes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut pull_requests = Vec::new();
        for node in nodes {
            let Some(key) = key_from_node(&node) else {
                continue;
            };
            if let Some(id) = node.get("id").and_then(Value::as_str) {
                self.node_ids.insert(key.clone(), id.to_string());
            }
            pull_requests.push(ProbePullRequest {
                key,
                node_id: node
                    .get("id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                created_at: parse_time(node.get("createdAt")),
                updated_at: parse_time(node.get("updatedAt")),
                state: parse_state(node.get("state")),
                head_oid: node
                    .get("headRefOid")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                ci: parse_ci(node.get("statusCheckRollup")),
            });
        }
        Ok(ProbeReport {
            pull_requests,
            allowance: self.allowance.clone(),
        })
    }

    fn list_authored(&mut self) -> Result<Vec<PullRequestSnapshot>, GatewayError> {
        let value = self.request_full(AUTHORED_QUERY.as_str(), json!({}))?;
        let nodes = value
            .pointer("/data/viewer/pullRequests/nodes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(self.snapshots_from_nodes(nodes, &[]))
    }

    fn discover_involved(&mut self) -> Result<Vec<PullRequestSnapshot>, GatewayError> {
        let value = self.request_full(DISCOVERY_QUERY.as_str(), json!({}))?;
        let mut by_key: BTreeMap<PullRequestKey, PullRequestSnapshot> = BTreeMap::new();
        for (pointer, relation) in [
            ("/data/assigned/nodes", FocusRelation::Assigned),
            (
                "/data/reviewRequested/nodes",
                FocusRelation::ReviewRequested,
            ),
            (
                "/data/teamReviewRequested/nodes",
                FocusRelation::TeamReviewRequested,
            ),
        ] {
            let nodes = value
                .pointer(pointer)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for mut snapshot in self.snapshots_from_nodes(nodes, &[relation]) {
                if let Some(existing) = by_key.get_mut(&snapshot.key) {
                    for relation in snapshot.relations.drain(..) {
                        if !existing.relations.contains(&relation) {
                            existing.relations.push(relation);
                        }
                    }
                } else {
                    by_key.insert(snapshot.key.clone(), snapshot);
                }
            }
        }
        Ok(by_key.into_values().collect())
    }

    fn observe(
        &mut self,
        keys: &[PullRequestKey],
    ) -> Result<Vec<PullRequestSnapshot>, GatewayError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<String> = keys
            .iter()
            .filter_map(|key| self.node_ids.get(key).cloned())
            .collect();
        if ids.is_empty() {
            return Err(GatewayError::Malformed(
                "no known node ids for the selected pull requests".to_string(),
            ));
        }
        let value = self.request_full(OBSERVE_QUERY.as_str(), json!({"ids": ids}))?;
        let nodes = value
            .pointer("/data/nodes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(self.snapshots_from_nodes(nodes, &[]))
    }

    fn allowance(&self) -> Allowance {
        self.allowance.clone()
    }
}

impl GraphQlGateway {
    fn snapshots_from_nodes(
        &mut self,
        nodes: Vec<Value>,
        relations: &[FocusRelation],
    ) -> Vec<PullRequestSnapshot> {
        let mut snapshots = Vec::new();
        for node in nodes {
            let Some(key) = key_from_node(&node) else {
                continue;
            };
            if node.get("__typename").and_then(Value::as_str) == Some("PullRequest") {
                // Search nodes carry a typename; authored nodes may omit it.
            }
            if let Some(id) = node.get("id").and_then(Value::as_str) {
                self.node_ids.insert(key.clone(), id.to_string());
            }
            let author_login = node
                .pointer("/author/login")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let author_is_bot = is_bot(&node, "/author");
            let comments = nodes_from(&node, "/comments/nodes")
                .into_iter()
                .filter_map(|comment| {
                    Some(CommentData {
                        id: comment.get("id")?.as_str()?.to_string(),
                        author: comment
                            .pointer("/author/login")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        body: comment
                            .get("body")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        created_at: parse_time(comment.get("createdAt")),
                        is_bot: is_bot(comment, "/author"),
                    })
                })
                .collect();
            let reviews = nodes_from(&node, "/reviews/nodes")
                .into_iter()
                .filter_map(|review| {
                    Some(ReviewData {
                        id: review.get("id")?.as_str()?.to_string(),
                        author: review
                            .pointer("/author/login")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        state: ReviewState::parse(
                            review
                                .get("state")
                                .and_then(Value::as_str)
                                .unwrap_or("COMMENTED"),
                        ),
                        body: review
                            .get("body")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        submitted_at: parse_time(review.get("submittedAt")),
                        is_bot: is_bot(review, "/author"),
                    })
                })
                .collect();
            let mut review_comments = Vec::new();
            if let Some(threads) = node
                .pointer("/reviewThreads/nodes")
                .and_then(Value::as_array)
            {
                for thread in threads {
                    for comment in nodes_from(thread, "/comments/nodes") {
                        if let Some(id) = comment.get("id").and_then(Value::as_str) {
                            review_comments.push(ReviewCommentData {
                                id: id.to_string(),
                                author: comment
                                    .pointer("/author/login")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default()
                                    .to_string(),
                                body: comment
                                    .get("body")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default()
                                    .to_string(),
                                created_at: parse_time(comment.get("createdAt")),
                                is_bot: is_bot(comment, "/author"),
                            });
                        }
                    }
                }
            }
            let stack = parse_stack(&node);
            let mut all_relations = relations.to_vec();
            if author_is_bot {
                all_relations.clear();
            }
            snapshots.push(PullRequestSnapshot {
                key: key.clone(),
                node_id: node
                    .get("id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                title: node
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                body: node
                    .get("body")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                state: parse_state(node.get("state")),
                created_at: parse_time(node.get("createdAt")),
                updated_at: parse_time(node.get("updatedAt")),
                merged_at: parse_time(node.get("mergedAt")),
                closed_at: parse_time(node.get("closedAt")),
                head_oid: node
                    .get("headRefOid")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                ci: parse_ci(node.get("statusCheckRollup")),
                author: author_login,
                assignees: nodes_from(&node, "/assignees/nodes")
                    .into_iter()
                    .filter_map(|assignee| assignee.get("login").and_then(Value::as_str))
                    .map(ToString::to_string)
                    .collect(),
                url: key.url(),
                comments,
                reviews,
                review_comments,
                stack,
                relations: all_relations,
            });
        }
        snapshots
    }
}

fn nodes_from<'a>(node: &'a Value, pointer: &str) -> Vec<&'a Value> {
    node.pointer(pointer)
        .and_then(Value::as_array)
        .map(|nodes| nodes.iter().collect())
        .unwrap_or_default()
}

fn key_from_node(node: &Value) -> Option<PullRequestKey> {
    let number = node.get("number")?.as_u64()?;
    let owner = node
        .pointer("/repository/owner/login")
        .and_then(Value::as_str)?;
    let name = node.pointer("/repository/name").and_then(Value::as_str)?;
    Some(PullRequestKey::new(format!("{owner}/{name}"), number))
}

fn parse_time(value: Option<&Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
}

fn parse_state(value: Option<&Value>) -> PullRequestState {
    PullRequestState::parse(value.and_then(Value::as_str).unwrap_or("OPEN"))
}

fn parse_ci(value: Option<&Value>) -> CiResult {
    let state = value
        .and_then(|value| value.get("state").or(Some(value)))
        .and_then(Value::as_str);
    CiResult::parse(state)
}

fn parse_allowance(value: &Value) -> Allowance {
    Allowance {
        remaining: value
            .get("remaining")
            .and_then(Value::as_u64)
            .unwrap_or_default() as u32,
        reset_at: parse_time(value.get("resetAt")),
        cost: value
            .get("cost")
            .and_then(Value::as_u64)
            .unwrap_or_default() as u32,
    }
}

fn parse_stack(node: &Value) -> Option<StackInfo> {
    let stack = node.get("stack")?;
    if stack.is_null() {
        return None;
    }
    let number = stack.get("number")?.as_u64()?;
    let size = stack.get("size")?.as_u64()?;
    let base = stack
        .get("baseRefName")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let position = node
        .pointer("/stackEntry/position")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    Some(StackInfo {
        number,
        size,
        position,
        base,
    })
}

fn is_bot(node: &Value, pointer: &str) -> bool {
    let typenames_bot = node
        .pointer(&format!("{pointer}/__typename"))
        .and_then(Value::as_str)
        .map(|typename| typename.eq_ignore_ascii_case("bot"))
        .unwrap_or(false);
    let login_is_bot = node
        .pointer(&format!("{pointer}/login"))
        .and_then(Value::as_str)
        .map(|login| login.ends_with("[bot]"))
        .unwrap_or(false);
    typenames_bot || login_is_bot
}

fn strip_stack_fields(query: &str) -> String {
    let mut output = query
        .replace("stack { number size baseRefName }", "")
        .replace("stackEntry { position }", "");
    output.push_str("\n# stack fields omitted after a preview rejection");
    output
}

const PROBE_QUERY: &str = r#"
query PassuffProbe {
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
}
"#;

fn full_fields() -> &'static str {
    r#"
        id
        number
        title
        body
        state
        isDraft
        createdAt
        updatedAt
        mergedAt
        closedAt
        url
        headRefOid
        author { login __typename }
        assignees(first: 10) { nodes { login } }
        statusCheckRollup { state }
        comments(first: 100) { nodes { id body createdAt author { login __typename } } }
        reviews(first: 100) { nodes { id body state submittedAt author { login __typename } } }
        reviewThreads(first: 50) { nodes { comments(first: 50) { nodes { id body createdAt author { login __typename } } } } }
        repository { name owner { login } }
        stack { number size baseRefName }
        stackEntry { position }
"#
}

fn authored_query() -> String {
    format!(
        r#"
query PassuffAuthored {{
  viewer {{
    login
    pullRequests(first: 100, orderBy: {{field: UPDATED_AT, direction: DESC}}) {{
      nodes {{ {} }}
    }}
  }}
  rateLimit {{ remaining resetAt cost }}
}}
"#,
        full_fields()
    )
}

fn discovery_query() -> String {
    format!(
        r#"
query PassuffDiscovery {{
  assigned: search(query: "is:open is:pr assignee:@me", type: ISSUE, first: 100) {{
    nodes {{ ... on PullRequest {{ {} }} }}
  }}
  reviewRequested: search(query: "is:open is:pr review-requested:@me", type: ISSUE, first: 100) {{
    nodes {{ ... on PullRequest {{ {} }} }}
  }}
  teamReviewRequested: search(query: "is:open is:pr team-review-requested:@me", type: ISSUE, first: 100) {{
    nodes {{ ... on PullRequest {{ {} }} }}
  }}
  rateLimit {{ remaining resetAt cost }}
}}
"#,
        full_fields(),
        full_fields(),
        full_fields()
    )
}

fn observe_query() -> String {
    format!(
        r#"
query PassuffObserve($ids: [ID!]!) {{
  nodes(ids: $ids) {{
    ... on PullRequest {{ {} }}
  }}
  rateLimit {{ remaining resetAt cost }}
}}
"#,
        full_fields()
    )
}

static AUTHORED_QUERY: std::sync::LazyLock<String> = std::sync::LazyLock::new(authored_query);
static DISCOVERY_QUERY: std::sync::LazyLock<String> = std::sync::LazyLock::new(discovery_query);
static OBSERVE_QUERY: std::sync::LazyLock<String> = std::sync::LazyLock::new(observe_query);
