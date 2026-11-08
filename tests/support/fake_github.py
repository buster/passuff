"""Deterministic local GitHub GraphQL twin."""

from __future__ import annotations

import json
import threading
from collections import Counter
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any


def _state_gql(value: str) -> str:
    return {"open": "OPEN", "closed": "CLOSED", "merged": "MERGED"}.get(value, "OPEN")


def _ci_gql(value: str) -> str:
    return {"passing": "SUCCESS", "failing": "FAILURE", "pending": "PENDING", "error": "ERROR"}.get(value, "PENDING")


class _Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("content-length", "0"))
        raw = self.rfile.read(length)
        try:
            body = json.loads(raw)
        except ValueError:
            body = {}
        status, payload, as_text = self.server.fake.handle(body, self.headers)
        if as_text:
            data = str(payload).encode()
        else:
            data = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, *args):
        pass


class FakeGitHub:
    def __init__(self, world):
        self.world = world
        self.prs: list[dict[str, Any]] = []
        self.tokens: dict[str, str] = {}
        self.counts = Counter()
        self.faults: dict[str, str] = {}
        self.fault_once: dict[str, str] = {}
        self.stack_rejected = False
        self.repeat_node = False
        self.observed_ids: list[str] = []
        self.stack_attempts = 0
        self.rate_remaining = 5000
        self.rate_reset_at = ""
        self.server = None
        self.port = None
        self.thread = None
        self._next_ids = Counter()

    def start(self) -> None:
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), _Handler)
        self.server.fake = self
        self.port = self.server.server_address[1]
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def stop(self) -> None:
        if self.server:
            self.server.shutdown()
            self.server.server_close()
            self.server = None

    def url(self) -> str:
        return f"http://127.0.0.1:{self.port}/graphql"

    def add_pr(self, repo: str, number: int, author: str, **fields: Any) -> dict[str, Any]:
        pr = {
            "id": f"PR_{repo.replace('/', '_')}_{number}",
            "repo": repo,
            "number": number,
            "author": author,
            "title": fields.get("title", f"PR {number}"),
            "body": fields.get("body", ""),
            "state": fields.get("state", "open"),
            "ci": fields.get("ci", "passing"),
            "ci_name": fields.get("ci_name", "build"),
            "created_at": fields.get("created_at", "2026-11-01T09:00:00Z"),
            "updated_at": fields.get("updated_at", "2026-11-01T09:00:00Z"),
            "head_sha": fields.get("head_sha", f"sha-{number}"),
            "comments": [],
            "reviews": [],
            "review_comments": [],
            "assignees": list(fields.get("assignees", [])),
            "review_requests": list(fields.get("review_requests", [])),
            "team_requests": list(fields.get("team_requests", [])),
            "stack": fields.get("stack"),
        }
        self.prs.append(pr)
        return pr

    def find(self, key: str) -> dict[str, Any] | None:
        for pr in self.prs:
            if f"{pr['repo']}#{pr['number']}" == key:
                return pr
        return None

    def _token_login(self, token: str) -> str | None:
        if token in self.tokens:
            return self.tokens[token]
        if token.startswith("gh-token-"):
            return token[len("gh-token-"):]
        return None

    def _rate(self) -> dict[str, Any]:
        return {"remaining": self.rate_remaining, "resetAt": self.rate_reset_at or "2026-11-08T12:00:00Z"}

    def _pr_node(self, pr: dict[str, Any], full: bool, include_stack: bool) -> dict[str, Any]:
        owner, repo = pr["repo"].split("/", 1)
        node: dict[str, Any] = {
            "id": pr["id"],
            "number": pr["number"],
            "title": pr["title"],
            "body": pr["body"],
            "state": _state_gql(pr["state"]),
            "isDraft": False,
            "createdAt": pr["created_at"],
            "updatedAt": pr["updated_at"],
            "headRefOid": pr["head_sha"],
            "repository": {"name": repo, "owner": {"login": owner}},
            "author": {"login": pr["author"]},
            "assignees": {"nodes": [{"login": login} for login in pr["assignees"]]},
            "reviewRequests": {
                "nodes": [{"requestedReviewer": {"login": login}} for login in pr["review_requests"]]
                + [{"requestedReviewer": {"slug": team}} for team in pr["team_requests"]]
            },
            "commits": {
                "nodes": [
                    {
                        "commit": {
                            "oid": pr["head_sha"],
                            "statusCheckRollup": {
                                "state": _ci_gql(pr["ci"]),
                                "contexts": {"nodes": [{"name": pr["ci_name"]}]},
                            },
                        }
                    }
                ]
            },
        }
        if include_stack and pr.get("stack"):
            stack = pr["stack"]
            node["stack"] = {
                "number": stack.get("number"),
                "size": stack.get("size"),
                "position": stack.get("position"),
                "baseRef": stack.get("base"),
            }
        if full:
            node["comments"] = {
                "nodes": [
                    {
                        "id": c["id"],
                        "author": {"login": c["author"]},
                        "body": c["body"],
                        "createdAt": c["created_at"],
                    }
                    for c in pr["comments"]
                ]
            }
            node["reviews"] = {
                "nodes": [
                    {
                        "id": r["id"],
                        "author": {"login": r["author"]},
                        "state": r["state"],
                        "body": r.get("body", ""),
                        "submittedAt": r["submitted_at"],
                    }
                    for r in pr["reviews"]
                ]
            }
            node["reviewThreads"] = {
                "nodes": [
                    {
                        "comments": {
                            "nodes": [
                                {
                                    "id": c["id"],
                                    "author": {"login": c["author"]},
                                    "body": c["body"],
                                    "createdAt": c["created_at"],
                                }
                                for c in pr["review_comments"]
                            ]
                        }
                    }
                ]
            } if pr["review_comments"] else {"nodes": []}
        return node

    def handle(self, body: dict[str, Any], headers) -> tuple[int, Any, bool]:
        token = (headers.get("authorization") or "").replace("bearer ", "").strip()
        login = self._token_login(token)
        operation = body.get("operationName") or "Unknown"
        self.counts[operation] += 1
        if login is None:
            return 401, {"errors": [{"message": "unknown token"}]}, False
        fault = self.fault_once.pop(login, None) or self.faults.get(login)
        if fault == "malformed":
            return 200, "this is not json", True
        if fault == "partial":
            return 200, {"data": {"viewer": {"login": login}}}, False
        if fault == "timeout":
            return 504, {"errors": [{"message": "timeout"}]}, False
        if fault == "non2xx":
            return 502, {"errors": [{"message": "boom"}]}, False
        if fault == "graphql":
            return 200, {"errors": [{"message": "boom"}]}, False
        data: dict[str, Any] = {"viewer": {"login": login}, "rateLimit": self._rate()}
        if operation == "ViewerLogin":
            return 200, {"data": data}, False
        if operation == "Probe":
            data["search"] = {"nodes": [self._pr_node(pr, False, False) for pr in self.prs]}
            self.world.on_probe()
            return 200, {"data": data}, False
        if operation == "Discover":
            data["search"] = {"nodes": [self._pr_node(pr, False, False) for pr in self.prs if pr["state"] == "open"]}
            return 200, {"data": data}, False
        if operation == "Observe":
            variables = body.get("variables") or {}
            ids = variables.get("ids") or []
            include_stack = bool(variables.get("includeStack"))
            self.observed_ids.extend(ids)
            if include_stack and self.stack_rejected:
                self.stack_attempts += 1
                return 200, {"errors": [{"message": "stack is not available"}]}, False
            self.stack_attempts += 1
            nodes = [self._pr_node(pr, True, include_stack) for pr in self.prs if pr["id"] in ids]
            if self.repeat_node and nodes:
                nodes.append(nodes[0])
            self.world.on_observe()
            return 200, {"data": {"nodes": nodes, "rateLimit": self._rate()}}, False
        return 200, {"data": data}, False
