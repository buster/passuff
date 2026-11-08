"""Transcript, status, inbox, and digest rendering."""

from __future__ import annotations

from typing import Any

from .inbox import format_when, record_approximate
from .core import RefreshReport


def event_lines(group) -> list[str]:
    return [item["summary"] for item in group.events]


def render_check(report: RefreshReport) -> list[str]:
    lines: list[str] = []
    for key in report.baselines:
        lines.append(f"Fresh baseline recorded for {key}")
    for label, reason in report.failures:
        lines.append(f"account {label} failed: {reason}")
    groups = report.notifications
    total = sum(len(group.events) for group in groups)
    if len(groups) == 1:
        group = groups[0]
        if len(group.events) == 1:
            lines.append(f"{group.key}: {group.events[0]['summary']}")
        else:
            lines.append(f"{len(group.events)} activities on {group.key}")
            lines.extend(f"{group.key}: {line}" for line in event_lines(group))
    elif len(groups) > 1:
        lines.append(f"{total} activities in {len(groups)} pull requests")
        for group in groups:
            count = len(group.events)
            lines.append(f"{group.key}: {count} {'activity' if count == 1 else 'activities'}")
        for group in groups:
            lines.extend(f"{group.key}: {line}" for line in event_lines(group))
    if not groups:
        lines.append("no notifications")
    return lines


def digest(report: RefreshReport) -> tuple[str, str] | None:
    groups = report.notifications
    if not groups:
        return None
    total = sum(len(group.events) for group in groups)
    if len(groups) == 1:
        group = groups[0]
        if len(group.events) == 1:
            return group.events[0]["summary"], ""
        body = "\n".join(event_lines(group))
        return f"{len(group.events)} activities on {group.key}", body
    body_lines = []
    for group in groups[:5]:
        count = len(group.events)
        body_lines.append(f"{group.key}: {count} {'activity' if count == 1 else 'activities'}")
    remaining = len(groups) - 5
    if remaining > 0:
        body_lines.append(f"and {remaining} more pull requests")
    return f"{total} activities in {len(groups)} pull requests", "\n".join(body_lines)


def render_status(view: dict[str, Any]) -> list[str]:
    lines: list[str] = []
    for label, login in view["accounts"]:
        lines.append(f"{label} @{login}" if login else f"{label} not resolved")
    lines.append(view["scope"])
    if view.get("watching_nothing"):
        lines.append("watching nothing")
    lines.append(f"{view['unread']} unread of {view['total']}")
    last = view.get("last_check")
    lines.append("last check " + (format_when(last) if last else "never"))
    watched = view["watched"]
    if not watched:
        lines.append("no watched pull requests")
    for pr in watched:
        lines.append(f"Pull request {pr.get('key')}")
        lines.append(f"{pr.get('key')} {pr.get('state')} CI {pr.get('ci')}")
    return lines


def render_inbox(rows: list[dict[str, Any]], unread: int, total: int, include_read: bool) -> list[str]:
    lines: list[str] = []
    lines.append(f"{unread} unread of {total}")
    if not rows:
        lines.append("no unread activity" if not include_read else "no activity")
        return lines
    for record in rows:
        key = record.get("pull_request", "")
        when = format_when(record.get("occurred_at") or record.get("observed_at"))
        if record_approximate(record):
            when = "~" + when
        state = "read" if record.get("read") else "unread"
        lines.append(f"[{record.get('id')}] {record.get('summary')}")
        lines.append(f"  {key}  {when}  {state}")
    return lines


def render_activity_detail(record: dict[str, Any], pr: dict[str, Any] | None) -> list[str]:
    lines = [
        f"Activity {record.get('id')}",
        record.get("summary", ""),
        f"When: {format_when(record.get('occurred_at') or record.get('observed_at'))}",
    ]
    if record.get("observed_at") and record.get("occurred_at"):
        lines.append(f"Fetched: {format_when(record.get('observed_at'))}")
    if pr:
        lines.append(f"Pull request {pr.get('key')}")
        lines.append(f"Status: {pr.get('state')}   CI: {pr.get('ci')}")
        lines.append(f"Creator: @{pr.get('creator')}")
        if pr.get("assignees"):
            lines.append("Assignee: " + ", ".join("@" + login for login in pr["assignees"]))
        stack = pr.get("stack") or {}
        if stack:
            lines.append(f"stack {stack.get('number')} {stack.get('position')}/{stack.get('size')} base {stack.get('base')}")
    lines.append("read" if record.get("read") else "unread")
    if record.get("body"):
        lines.append("")
        lines.extend(str(record["body"]).splitlines())
    lines.append("Enter open in browser")
    return lines
