//! GraphQL boundary mock. It answers the five operations the product issues and
//! exposes only the fixture state a scenario needs to change.

use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

pub const LOGIN: &str = "tester";
pub const REVIEWER: &str = "reviewer";

const CREATED_AT: &str = "2026-11-01T09:00:00Z";
const BASELINE_UPDATED_AT: &str = "2026-11-01T10:00:00Z";
const COMMENT_AT: &str = "2026-11-01T11:00:00Z";

#[derive(Debug, Clone, Default)]
struct MockState {
    pending_comment: bool,
    authored_includes_pending: bool,
    fail_everything: bool,
}

pub struct GitHubMock {
    server: MockServer,
    state: Arc<Mutex<MockState>>,
}

impl GitHubMock {
    pub async fn start() -> Result<Self, String> {
        let server = MockServer::start().await;
        let state = Arc::new(Mutex::new(MockState {
            pending_comment: false,
            authored_includes_pending: true,
            fail_everything: false,
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
        self.update(|state| state.pending_comment = true);
    }

    /// New activity is visible to `PassuffObserve` only, so a targeted pass is
    /// the only operation that can make it reach the product.
    pub fn observe_only(&self) {
        self.update(|state| state.authored_includes_pending = false);
    }

    pub fn fail_everything(&self) {
        self.update(|state| state.fail_everything = true);
    }

    fn update(&self, change: impl FnOnce(&mut MockState)) {
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
            Ok(state) => state.clone(),
            Err(_) => return ResponseTemplate::new(500),
        };
        if state.fail_everything {
            return ResponseTemplate::new(500).set_body_string("");
        }
        let body: Value = match serde_json::from_slice(&request.body) {
            Ok(body) => body,
            Err(_) => return ResponseTemplate::new(400).set_body_string("bad request body"),
        };
        let query = body
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or_default();
        ResponseTemplate::new(200).set_body_json(payload(query, &state))
    }
}

fn payload(query: &str, state: &MockState) -> Value {
    if query.contains("PassuffViewer") {
        json!({"data": {"viewer": {"login": LOGIN}}})
    } else if query.contains("PassuffProbe") {
        json!({
            "data": {"viewer": {"login": LOGIN, "pullRequests": {"nodes": [probe_node(state)]}}},
            "rateLimit": rate_limit(),
        })
    } else if query.contains("PassuffAuthored") {
        json!({
            "data": {"viewer": {"login": LOGIN, "pullRequests": {
                "nodes": [full_node(state, state.authored_includes_pending)]
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
        json!({
            "data": {"nodes": [full_node(state, true)]},
            "rateLimit": rate_limit(),
        })
    } else {
        json!({"data": {}})
    }
}

fn rate_limit() -> Value {
    json!({"remaining": 4000, "resetAt": "2099-01-01T00:00:00Z", "cost": 1})
}

fn updated_at(state: &MockState, include_pending: bool) -> &'static str {
    if include_pending && state.pending_comment {
        COMMENT_AT
    } else {
        BASELINE_UPDATED_AT
    }
}

fn repository() -> Value {
    json!({"name": "repo", "owner": {"login": "octo"}})
}

fn probe_node(state: &MockState) -> Value {
    json!({
        "id": "PR_node_7",
        "number": 7,
        "state": "OPEN",
        "createdAt": CREATED_AT,
        "updatedAt": updated_at(state, true),
        "headRefOid": "abc123",
        "statusCheckRollup": {"state": "SUCCESS"},
        "repository": repository(),
    })
}

fn full_node(state: &MockState, include_pending: bool) -> Value {
    let comments = if include_pending && state.pending_comment {
        json!([{
            "id": "comment_1",
            "body": "Looks good now.",
            "createdAt": COMMENT_AT,
            "author": {"login": REVIEWER, "__typename": "User"},
        }])
    } else {
        json!([])
    };
    json!({
        "id": "PR_node_7",
        "number": 7,
        "title": "Add retry loop",
        "body": "Adds a retry loop to the upload path.",
        "state": "OPEN",
        "isDraft": false,
        "createdAt": CREATED_AT,
        "updatedAt": updated_at(state, include_pending),
        "mergedAt": null,
        "closedAt": null,
        "url": "https://github.com/octo/repo/pull/7",
        "headRefOid": "abc123",
        "author": {"login": LOGIN, "__typename": "User"},
        "assignees": {"nodes": []},
        "statusCheckRollup": {"state": "SUCCESS"},
        "comments": {"nodes": comments},
        "reviews": {"nodes": []},
        "reviewThreads": {"nodes": []},
        "repository": repository(),
        "stack": null,
        "stackEntry": null,
    })
}
