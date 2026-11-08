"""Activity classification for one observed pull request."""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from .config import EventsConfig, FocusConfig
from .github import PullRequest


def _iso(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def _review_state(state: str) -> str:
    return {
        "APPROVED": "approved",
        "CHANGES_REQUESTED": "changes requested",
        "COMMENTED": "commented",
        "DISMISSED": "dismissed",
        "PENDING": "pending",
    }.get(state, state.lower())


def is_bot(login: str) -> bool:
    return login.endswith("[bot]")


def snapshot(pr: PullRequest) -> dict[str, Any]:
    return {
        "comments": {str(item["id"]): item for item in pr.comments},
        "reviews": {str(item["id"]): item for item in pr.reviews},
        "review_comments": {str(item["id"]): item for item in pr.review_comments},
        "head_sha": pr.head_sha,
        "ci": pr.ci,
        "state": pr.state,
        "updated_at": pr.updated_at,
    }


def event(kind: str, summary: str, actor: str, occurred_at: str | None, body: str = "", notify: bool = True, read: bool = False) -> dict[str, Any]:
    return {
        "kind": kind,
        "summary": summary,
        "actor": actor,
        "occurred_at": occurred_at or None,
        "body": body,
        "notify": notify,
        "read": read,
    }


def classify(
    previous: dict[str, Any] | None,
    current: PullRequest,
    *,
    login_set: set[str],
    focus: FocusConfig,
    events: EventsConfig,
    state_change_reported: bool,
    last_observed_at: str | None,
) -> list[dict[str, Any]]:
    """Return the activity events for one observation."""

    prev = previous or {}
    prev_comments = {str(key): value for key, value in (prev.get("comments") or {}).items()}
    prev_reviews = {str(key): value for key, value in (prev.get("reviews") or {}).items()}
    prev_review_comments = {str(key): value for key, value in (prev.get("review_comments") or {}).items()}
    prev_state = prev.get("state")
    prev_ci_terminal = prev.get("ci_terminal")
    if prev_ci_terminal is None and prev.get("ci") in {"failing", "passing"}:
        prev_ci_terminal = prev.get("ci")
    focused = (not current.relations) or bool(set(current.relations) & set(focus.relations))
    output: list[dict[str, Any]] = []

    def add(kind: str, summary: str, actor: str, occurred_at: str | None, body: str = "", *, notify: bool = True, read: bool = False) -> None:
        output.append(event(kind, summary, actor, occurred_at, body, notify=notify and focused, read=read))

    first = not prev
    if first:
        created = _iso(current.created_at)
        boundary = _iso(last_observed_at)
        if current.state in {"merged", "closed"}:
            if boundary is not None and created is not None and created > boundary:
                add("state_change", "Merged" if current.state == "merged" else "Closed", current.author, current.updated_at)
            return output
        if boundary is not None and created is not None and created > boundary:
            add("new_pull_request", f"New pull request by @{current.author}", current.author, current.created_at)
        return output

    if prev_state in {"merged", "closed"} and current.state == "open":
        return output
    if prev_state in {"merged", "closed"}:
        return output

    for item in current.comments:
        identifier = str(item["id"])
        if identifier in prev_comments:
            continue
        actor = item.get("author", "")
        body = item.get("body", "")
        occurred = item.get("created_at")
        if actor in login_set:
            add("comment", f"New comment from @{actor}", actor, occurred, body, notify=False, read=True)
        elif is_bot(actor) and not events.bot_comments:
            add("comment", f"New comment from @{actor}", actor, occurred, body, notify=False)
        elif events.comments:
            add("comment", f"New comment from @{actor}", actor, occurred, body)
        else:
            add("comment", f"New comment from @{actor}", actor, occurred, body, notify=False)

    for item in current.reviews:
        identifier = str(item["id"])
        if identifier in prev_reviews:
            continue
        actor = item.get("author", "")
        state = _review_state(str(item.get("state", "COMMENTED")))
        occurred = item.get("submitted_at")
        body = item.get("body", "")
        summary = f"Review {state} by @{actor}"
        if actor in login_set:
            add("review", summary, actor, occurred, body, notify=False, read=True)
        elif events.reviews:
            add("review", summary, actor, occurred, body)
        else:
            add("review", summary, actor, occurred, body, notify=False)

    for item in current.review_comments:
        identifier = str(item["id"])
        if identifier in prev_review_comments:
            continue
        actor = item.get("author", "")
        body = item.get("body", "")
        occurred = item.get("created_at")
        if actor in login_set:
            add("review_comment", f"Review comment from @{actor}", actor, occurred, body, notify=False, read=True)
        elif is_bot(actor) and not events.bot_comments:
            add("review_comment", f"Review comment from @{actor}", actor, occurred, body, notify=False)
        elif events.comments:
            add("review_comment", f"Review comment from @{actor}", actor, occurred, body)
        else:
            add("review_comment", f"Review comment from @{actor}", actor, occurred, body, notify=False)

    ci = current.ci
    if ci in {"failing", "error"}:
        if prev_ci_terminal != "failing":
            name = current.ci_name or "build"
            if events.ci_failure:
                add("ci_failure", f"CI failing: {name}", "", None, name)
            else:
                add("ci_failure", f"CI failing: {name}", "", None, name, notify=False)
    elif ci == "passing" and prev_ci_terminal == "failing":
        if events.ci_recovery:
            add("ci_recovery", "CI recovered", "", None)
        else:
            add("ci_recovery", "CI recovered", "", None, notify=False)

    if current.state in {"merged", "closed"}:
        summary = "Merged" if current.state == "merged" else "Closed"
        if events.state_changes:
            add("state_change", summary, "", current.updated_at)
        else:
            add("state_change", summary, "", current.updated_at, notify=False)

    return output
