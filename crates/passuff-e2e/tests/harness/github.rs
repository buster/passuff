//! GraphQL boundary mock. It answers the five operations the product issues and
//! exposes only the fixture state a scenario needs to change.
//!
//! Each fixture is claimed by the token in the `Authorization` header, so a
//! scenario can run two accounts against one mock and tell their data apart.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use crate::harness::bin::{PERSONAL_TOKEN, TOKEN};

pub const LOGIN: &str = "tester";
pub const PERSONAL_LOGIN: &str = "tester-personal";
pub const REVIEWER: &str = "reviewer";
pub const BOT: &str = "dependabot[bot]";

const CREATED_AT: &str = "2026-11-01T09:00:00Z";
const BASELINE_UPDATED_AT: &str = "2026-11-01T10:00:00Z";
const COMMENT_AT: &str = "2026-11-01T11:00:00Z";

#[derive(Debug, Clone)]
struct Comment {
    id: String,
    body: String,
    author: String,
    bot: bool,
    created_at: &'static str,
}

#[derive(Debug, Clone)]
struct Fixture {
    login: String,
    owner: &'static str,
    repo: &'static str,
    number: u64,
    title: &'static str,
    body: &'static str,
    state: &'static str,
    ci: &'static str,
    comments: Vec<Comment>,
    updated_at: &'static str,
}

impl Fixture {
    fn work() -> Self {
        Self {
            login: LOGIN.to_string(),
            owner: "octo",
            repo: "repo",
            number: 7,
            title: "Add retry loop",
            body: "Adds a retry loop to the upload path.",
            state: "OPEN",
            ci: "SUCCESS",
            comments: Vec::new(),
            updated_at: BASELINE_UPDATED_AT,
        }
    }

    fn personal() -> Self {
        Self {
            login: PERSONAL_LOGIN.to_string(),
            owner: "octo",
            repo: "other",
            number: 9,
            title: "Fix login",
            body: "Fixes the login redirect.",
            state: "OPEN",
            ci: "SUCCESS",
            comments: Vec::new(),
            updated_at: BASELINE_UPDATED_AT,
        }
    }

    fn key(&self) -> String {
        format!("{}/{}#{}", self.owner, self.repo, self.number)
    }

    fn node_id(&self) -> String {
        format!("PR_node_{}", self.number)
    }
}

struct MockState {
    fixtures: BTreeMap<&'static str, Fixture>,
    hidden_from_authored: bool,
    fail_everything: bool,
    viewer_login: Option<String>,
}

impl MockState {
    fn fixture(&self, token: &str) -> Option<&Fixture> {
        self.fixtures.get(token)
    }
}

pub struct GitHubMock {
    server: MockServer,
    state: Arc<Mutex<MockState>>,
}

impl GitHubMock {
    pub async fn start() -> Result<Self, String> {
        let server = MockServer::start().await;
        let mut fixtures = BTreeMap::new();
        fixtures.insert(TOKEN, Fixture::work());
        fixtures.insert(PERSONAL_TOKEN, Fixture::personal());
        let state = Arc::new(Mutex::new(MockState {
            fixtures,
            hidden_from_authored: false,
            fail_everything: false,
            viewer_login: None,
        }));
        let responder = GitHubResponder {
            state: state.clone(),
        };
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(responder)
            .mount(&server)
            .await;
        Ok(Self { server, state })
    }

    pub fn graphql_url(&self) -> String {
        format!("{}/graphql", self.server.uri())
    }

    pub fn add_comment(&self) {
        self.push_comment(TOKEN, "comment", REVIEWER, false, "Looks good now.");
    }

    pub fn add_own_comment(&self) {
        self.push_comment(TOKEN, "own-comment", LOGIN, false, "Pushing a fix.");
    }

    pub fn add_bot_comment(&self) {
        self.push_comment(TOKEN, "bot-comment", BOT, true, "Coverage report.");
    }

    pub fn add_personal_comment(&self) {
        self.push_comment(
            PERSONAL_TOKEN,
            "other-comment",
            REVIEWER,
            false,
            "Please rebase.",
        );
    }

    pub fn fail_ci(&self) {
        self.update(TOKEN, |fixture| fixture.ci = "FAILURE");
    }

    pub fn pass_ci(&self) {
        self.update(TOKEN, |fixture| fixture.ci = "SUCCESS");
    }

    /// New activity is visible to `PassuffObserve` only, so a targeted pass is
    /// the only operation that can make it reach the product.
    pub fn observe_only(&self) {
        self.mutate(|state| state.hidden_from_authored = true);
    }

    pub fn fail_everything(&self) {
        self.mutate(|state| state.fail_everything = true);
    }

    /// Override the login `PassuffViewer` reports for every account.
    pub fn set_viewer_login(&self, login: &str) {
        self.mutate(|state| state.viewer_login = Some(login.to_string()));
    }

    fn push_comment(&self, token: &'static str, prefix: &str, author: &str, bot: bool, body: &str) {
        self.update(token, |fixture| {
            let id = format!("{prefix}-{}", fixture.comments.len() + 1);
            fixture.comments.push(Comment {
                id,
                body: body.to_string(),
                author: author.to_string(),
                bot,
                created_at: COMMENT_AT,
            });
            fixture.updated_at = COMMENT_AT;
        });
    }

    fn update(&self, token: &'static str, change: impl FnOnce(&mut Fixture)) {
        self.mutate(|state| {
            if let Some(fixture) = state.fixtures.get_mut(token) {
                change(fixture);
            }
        });
    }

    fn mutate(&self, change: impl FnOnce(&mut MockState)) {
        if let Ok(mut state) = self.state.lock() {
            change(&mut state);
        }
    }
}

struct GitHubResponder {
    state: Arc<Mutex<MockState>>,
}

impl Respond for GitHubResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => return ResponseTemplate::new(500),
        };
        if state.fail_everything {
            return ResponseTemplate::new(500).set_body_string("");
        }
        let token = request
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .unwrap_or_default()
            .to_string();
        let body: Value = match serde_json::from_slice(&request.body) {
            Ok(body) => body,
            Err(_) => return ResponseTemplate::new(400).set_body_string("bad request body"),
        };
        let query = body
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let variables = body.get("variables").cloned().unwrap_or(Value::Null);
        ResponseTemplate::new(200).set_body_json(payload(query, &variables, &token, &state))
    }
}

fn payload(query: &str, variables: &Value, token: &str, state: &MockState) -> Value {
    let fixture = state.fixture(token);
    let viewer_login = state
        .viewer_login
        .clone()
        .or_else(|| fixture.map(|fixture| fixture.login.clone()))
        .unwrap_or_else(|| LOGIN.to_string());
    if query.contains("PassuffViewer") {
        return json!({"data": {"viewer": {"login": viewer_login}}});
    }
    let Some(fixture) = fixture else {
        return json!({"data": {}});
    };
    if query.contains("PassuffProbe") {
        json!({
            "data": {"viewer": {"login": viewer_login, "pullRequests": {
                "nodes": [probe_node(fixture)]
            }}},
            "rateLimit": rate_limit(),
        })
    } else if query.contains("PassuffAuthored") {
        json!({
            "data": {"viewer": {"login": viewer_login, "pullRequests": {
                "nodes": [full_node(fixture, !state.hidden_from_authored)]
            }}},
            "rateLimit": rate_limit(),
        })
    } else if query.contains("PassuffDiscovery") {
        json!({
            "data": {
                "assigned": {"nodes": []},
                "reviewRequested": {"nodes": []},
                "teamReviewRequested": {"nodes": []},
            },
            "rateLimit": rate_limit(),
        })
    } else if query.contains("PassuffObserve") {
        let wanted: Vec<String> = variables
            .get("ids")
            .and_then(Value::as_array)
            .map(|ids| {
                ids.iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let nodes = if wanted.iter().any(|id| *id == fixture.node_id()) {
            json!([full_node(fixture, true)])
        } else {
            json!([])
        };
        json!({
            "data": {"nodes": nodes},
            "rateLimit": rate_limit(),
        })
    } else {
        json!({"data": {}})
    }
}

fn rate_limit() -> Value {
    json!({"remaining": 4000, "resetAt": "2099-01-01T00:00:00Z", "cost": 1})
}

fn repository(fixture: &Fixture) -> Value {
    json!({"name": fixture.repo, "owner": {"login": fixture.owner}})
}

fn probe_node(fixture: &Fixture) -> Value {
    json!({
        "id": fixture.node_id(),
        "number": fixture.number,
        "state": fixture.state,
        "createdAt": CREATED_AT,
        "updatedAt": fixture.updated_at,
        "headRefOid": "abc123",
        "statusCheckRollup": {"state": fixture.ci},
        "repository": repository(fixture),
    })
}

fn full_node(fixture: &Fixture, include_activity: bool) -> Value {
    // A hidden activity must also keep the authored `updatedAt` at the
    // baseline, otherwise the open inbox probe sees no change and never issues
    // the observe that exposes the activity.
    let updated_at = if include_activity {
        fixture.updated_at
    } else {
        BASELINE_UPDATED_AT
    };
    let comments = if include_activity {
        Value::Array(
            fixture
                .comments
                .iter()
                .map(|comment| {
                    json!({
                        "id": comment.id,
                        "body": comment.body,
                        "createdAt": comment.created_at,
                        "author": {
                            "login": comment.author,
                            "__typename": if comment.bot { "Bot" } else { "User" },
                        },
                    })
                })
                .collect(),
        )
    } else {
        json!([])
    };
    json!({
        "id": fixture.node_id(),
        "number": fixture.number,
        "title": fixture.title,
        "body": fixture.body,
        "state": fixture.state,
        "isDraft": false,
        "createdAt": CREATED_AT,
        "updatedAt": updated_at,
        "mergedAt": null,
        "closedAt": null,
        "url": format!("https://github.com/{}", fixture.key()),
        "headRefOid": "abc123",
        "author": {"login": fixture.login, "__typename": "User"},
        "assignees": {"nodes": []},
        "statusCheckRollup": {"state": fixture.ci},
        "comments": {"nodes": comments},
        "reviews": {"nodes": []},
        "reviewThreads": {"nodes": []},
        "repository": repository(fixture),
        "stack": null,
        "stackEntry": null,
    })
}
