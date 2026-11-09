"""GitHub GraphQL port and adapter."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

import httpx

VIEWER_QUERY = "query ViewerLogin { viewer { login } rateLimit { remaining resetAt } }"
PROBE_QUERY = "query Probe { viewer { login pullRequests(first: 100, states: [OPEN, MERGED, CLOSED], orderBy: {field: UPDATED_AT, direction: DESC}) { nodes { ... on PullRequest { id number title state isDraft createdAt updatedAt headRefOid repository { name owner { login } } author { login } commits(last: 1) { nodes { commit { statusCheckRollup { state } } } } stack { number size baseRefName } stackEntry { position } } } } } rateLimit { remaining resetAt } }"
OBSERVE_QUERY = "query Observe($ids: [ID!]!, $includeStack: Boolean!) { nodes(ids: $ids) { ... on PullRequest { id number title body state isDraft createdAt updatedAt headRefOid repository { name owner { login } } author { login } assignees(first: 20) { nodes { login } } comments(first: 100) { nodes { id author { login } body createdAt } } reviews(first: 100) { nodes { id author { login } state body submittedAt } } reviewThreads(first: 100) { nodes { comments(first: 100) { nodes { id author { login } body createdAt } } } } commits(last: 1) { nodes { commit { oid statusCheckRollup { state } } } } stack @include(if: $includeStack) { number size baseRefName } stackEntry @include(if: $includeStack) { position } } } rateLimit { remaining resetAt } }"
DISCOVER_QUERY = "query Discover { search(first: 100, type: ISSUE, query: \"is:pr is:open\") { nodes { ... on PullRequest { id number title state isDraft createdAt updatedAt headRefOid repository { name owner { login } } author { login } assignees(first: 20) { nodes { login } } reviewRequests(first: 20) { nodes { requestedReviewer { ... on User { login } ... on Team { slug } } } } commits(last: 1) { nodes { commit { statusCheckRollup { state } } } } stack { number size baseRefName } stackEntry { position } } } } viewer { login } rateLimit { remaining resetAt } }"


class GitHubError(Exception):
    """Base GitHub failure."""


class GitHubTransportError(GitHubError):
    pass


class GitHubResponseError(GitHubError):
    pass


class StackRejected(GitHubError):
    pass


@dataclass
class RateLimit:
    remaining: int = 5000
    reset_at: str = ""


@dataclass
class PullRequest:
    id: str
    key: str
    number: int
    title: str
    body: str
    state: str
    author: str
    created_at: str
    updated_at: str
    head_sha: str
    ci: str
    stack: dict[str, Any] | None
    ci_name: str = "build"
    comments: list[dict[str, Any]] = field(default_factory=list)
    reviews: list[dict[str, Any]] = field(default_factory=list)
    review_comments: list[dict[str, Any]] = field(default_factory=list)
    assignees: list[str] = field(default_factory=list)
    relations: list[str] = field(default_factory=list)


def _ci_from_state(state: str | None) -> str:
    if state == "SUCCESS":
        return "passing"
    if state in {"FAILURE", "ERROR"}:
        return "failing"
    if state == "PENDING":
        return "pending"
    return "pending"


def _state_from_graphql(state: str) -> str:
    return {"OPEN": "open", "CLOSED": "closed", "MERGED": "merged"}.get(state, "open")


def _node_key(node: dict[str, Any]) -> str:
    repo = node.get("repository") or {}
    owner = (repo.get("owner") or {}).get("login", "")
    name = repo.get("name", "")
    return f"{owner}/{name}#{node.get('number')}"


def _commits_ci(node: dict[str, Any]) -> str:
    commits = ((node.get("commits") or {}).get("nodes") or [])
    if not commits:
        return "pending"
    rollup = ((commits[0].get("commit") or {}).get("statusCheckRollup") or {})
    return _ci_from_state(rollup.get("state"))


def _ci_name(node: dict[str, Any]) -> str:
    commits = ((node.get("commits") or {}).get("nodes") or [])
    if not commits:
        return "build"
    rollup = ((commits[0].get("commit") or {}).get("statusCheckRollup") or {})
    contexts = ((rollup.get("contexts") or {}).get("nodes") or [])
    if contexts:
        return str(contexts[0].get("name") or "build")
    return "build"


def _stack(node: dict[str, Any]) -> dict[str, Any] | None:
    stack = node.get("stack")
    if not stack:
        return None
    entry = node.get("stackEntry") or {}
    return {
        "number": stack.get("number"),
        "size": stack.get("size"),
        "position": entry.get("position"),
        "base": stack.get("baseRefName"),
    }


def _summary(node: dict[str, Any]) -> dict[str, Any]:
    assignees = [
        (item or {}).get("login", "")
        for item in ((node.get("assignees") or {}).get("nodes") or [])
    ]
    relations = []
    for request in ((node.get("reviewRequests") or {}).get("nodes") or []):
        reviewer = request.get("requestedReviewer") or {}
        if reviewer.get("login"):
            relations.append("review-requested")
        elif reviewer.get("slug"):
            relations.append("team-review-requested")
    return {
        "id": node.get("id"),
        "key": _node_key(node),
        "number": node.get("number"),
        "title": node.get("title") or "",
        "state": _state_from_graphql(node.get("state")),
        "author": (node.get("author") or {}).get("login", ""),
        "created_at": node.get("createdAt") or "",
        "updated_at": node.get("updatedAt") or "",
        "head_sha": node.get("headRefOid") or "",
        "ci": _commits_ci(node),
        "ci_name": _ci_name(node),
        "stack": _stack(node),
        "assignees": [login for login in assignees if login],
        "relations": relations,
    }


def _full(node: dict[str, Any]) -> PullRequest:
    summary = _summary(node)
    comments = [
        {
            "id": item.get("id"),
            "author": (item.get("author") or {}).get("login", ""),
            "body": item.get("body") or "",
            "created_at": item.get("createdAt") or "",
        }
        for item in ((node.get("comments") or {}).get("nodes") or [])
    ]
    reviews = [
        {
            "id": item.get("id"),
            "author": (item.get("author") or {}).get("login", ""),
            "state": item.get("state") or "COMMENTED",
            "body": item.get("body") or "",
            "submitted_at": item.get("submittedAt") or "",
        }
        for item in ((node.get("reviews") or {}).get("nodes") or [])
    ]
    review_comments = []
    for thread in ((node.get("reviewThreads") or {}).get("nodes") or []):
        for item in ((thread.get("comments") or {}).get("nodes") or []):
            review_comments.append(
                {
                    "id": item.get("id"),
                    "author": (item.get("author") or {}).get("login", ""),
                    "body": item.get("body") or "",
                    "created_at": item.get("createdAt") or "",
                }
            )
    return PullRequest(
        **summary,
        body=node.get("body") or "",
        comments=comments,
        reviews=reviews,
        review_comments=review_comments,
    )


class GitHubApiAdapter:
    def __init__(self, base_url: str, token: str, timeout: float = 60.0) -> None:
        self.base_url = base_url
        self.token = token
        self.client = httpx.Client(timeout=timeout)

    def close(self) -> None:
        self.client.close()

    def _post(self, operation: str, query: str, variables: dict[str, Any] | None = None) -> dict[str, Any]:
        try:
            response = self.client.post(
                self.base_url,
                json={"query": query, "operationName": operation, "variables": variables or {}},
                headers={"Authorization": f"bearer {self.token}", "Content-Type": "application/json"},
            )
        except httpx.HTTPError as exc:
            raise GitHubTransportError(str(exc)) from exc
        if response.status_code >= 400:
            raise GitHubTransportError(f"HTTP {response.status_code}")
        try:
            payload = response.json()
        except ValueError as exc:
            raise GitHubResponseError("malformed JSON response") from exc
        if not isinstance(payload, dict):
            raise GitHubResponseError("malformed response")
        errors = payload.get("errors")
        if errors:
            text = str(errors)
            if "stack" in text.lower():
                raise StackRejected(text)
            raise GitHubResponseError(text)
        data = payload.get("data")
        if not isinstance(data, dict):
            raise GitHubResponseError("response has no data")
        return data

    def viewer(self) -> tuple[str, RateLimit]:
        data = self._post("ViewerLogin", VIEWER_QUERY)
        viewer = data.get("viewer") or {}
        rate = data.get("rateLimit") or {}
        login = viewer.get("login")
        if not login:
            raise GitHubResponseError("viewer login is missing")
        return login, RateLimit(int(rate.get("remaining", 5000)), str(rate.get("resetAt", "")))

    def probe(self) -> tuple[list[dict[str, Any]], RateLimit]:
        data = self._post("Probe", PROBE_QUERY)
        viewer = data.get("viewer")
        connection = viewer.get("pullRequests") if isinstance(viewer, dict) else None
        if not isinstance(connection, dict) or not isinstance(connection.get("nodes"), list):
            raise GitHubResponseError("partial response: viewer pull requests are missing")
        nodes = connection.get("nodes") or []
        rate = data.get("rateLimit") or {}
        return [_summary(node) for node in nodes if node], RateLimit(int(rate.get("remaining", 5000)), str(rate.get("resetAt", "")))

    def discover(self) -> list[dict[str, Any]]:
        data = self._post("Discover", DISCOVER_QUERY)
        search = data.get("search")
        if not isinstance(search, dict) or not isinstance(search.get("nodes"), list):
            raise GitHubResponseError("partial response: search is missing")
        nodes = search.get("nodes") or []
        return [_summary(node) for node in nodes if node]

    def observe(self, ids: list[str]) -> list[PullRequest]:
        if not ids:
            return []
        try:
            data = self._post("Observe", OBSERVE_QUERY, {"ids": ids, "includeStack": True})
        except StackRejected:
            data = self._post("Observe", OBSERVE_QUERY, {"ids": ids, "includeStack": False})
        nodes = data.get("nodes") or []
        return [_full(node) for node in nodes if node]
