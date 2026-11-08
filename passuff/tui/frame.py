"""Pure text frame renderer for the terminal inbox."""

from __future__ import annotations

import re
from typing import Any

from ..config import AppConfig
from ..inbox import ActivityInboxService, format_when, record_approximate
from ..state import load_state
from ..status import WatchStatusReader


def markdown_lines(text: str) -> list[str]:
    lines = []
    in_code = False
    for raw in str(text or "").splitlines():
        line = raw.rstrip()
        stripped = line.strip()
        if stripped.startswith("```"):
            in_code = not in_code
            continue
        if in_code:
            lines.append("  " + line)
            continue
        line = re.sub(r"<!--.*?-->", "", line)
        line = re.sub(r"^#{1,6}\s+", "", line)
        line = line.replace("**", "").replace("__", "").replace("`", "")
        line = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", line)
        line = re.sub(r"^\s*[-*]\s+", "• ", line)
        line = re.sub(r"^\s*\d+\.\s+", "• ", line)
        line = re.sub(r"^\s*---+\s*$", "", line)
        lines.append(line)
    return lines


def _key(pr: dict[str, Any]) -> str:
    key = str(pr.get("key", ""))
    if len(key) > 20:
        return "…" + key[-15:]
    return key


def _when(record: dict[str, Any]) -> str:
    value = format_when(record.get("occurred_at") or record.get("observed_at"))
    if record_approximate(record):
        value = "~" + value
    return value


def _activity_type(record: dict[str, Any]) -> str:
    kind = record.get("kind")
    actor = record.get("actor") or ""
    if kind == "comment":
        return f"comment by @{actor}"
    if kind == "review":
        state = str(record.get("summary", "")).split(" by ")[0].replace("Review ", "")
        return f"review {state} by @{actor}"
    if kind == "review_comment":
        return f"review comment by @{actor}"
    if kind == "ci_failure":
        body = record.get("body") or ""
        return "CI failing: " + (body or "checks")
    if kind == "ci_recovery":
        return "CI recovered"
    if kind in {"state_change", "merged", "closed"}:
        return str(record.get("summary", "state change"))
    if kind == "new_pull_request":
        return f"new pull request by @{actor}"
    return str(record.get("summary", ""))


def _sort_prs(prs: list[dict[str, Any]], sort: str, direction: int) -> list[dict[str, Any]]:
    def key(pr: dict[str, Any]):
        if sort == "name":
            return pr.get("key", "")
        if sort == "state":
            return str(pr.get("state", ""))
        if sort == "age":
            return pr.get("last_observed_at") or ""
        return pr.get("last_observed_at") or ""

    return sorted(prs, key=key, reverse=direction < 0)


def render(config: AppConfig, state: dict[str, Any], view: dict[str, Any]) -> list[str]:
    reader = WatchStatusReader(config, state)
    service = ActivityInboxService(config, state)
    prs = [pr for pr in reader.watched()]
    prs = _sort_prs(prs, view.get("sort", "updated"), view.get("sort_direction", 1))
    activities = service.view(include_read=True)
    ignored = {item.casefold() for item in config.watch.ignored_pull_requests}
    if view.get("filter"):
        filter_key = view["filter"]
        filter_stack = view.get("filter_stack")
        if filter_stack:
            stack = next((pr.get("stack") for pr in prs if pr.get("key") == filter_key), None)
            activities = [
                record for record in activities
                if (state.get("pull_requests", {}).get(record.get("pull_request")) or {}).get("stack", {})
                and (state.get("pull_requests", {}).get(record.get("pull_request")) or {})["stack"].get("number") == (stack or {}).get("number")
            ]
        else:
            activities = [record for record in activities if record.get("pull_request") == filter_key]
    if view.get("unread_only"):
        activities = [record for record in activities if not record.get("read")]
    if not view.get("show_ignored"):
        activities = [
            record for record in activities
            if record.get("pull_request", "").casefold() not in ignored
        ]
    total_activity = len(service.view(include_read=True))
    unread = sum(1 for record in service.view(include_read=True) if not record.get("read"))
    lines: list[str] = []
    lines.append("passuff — Inbox")
    for label, login in reader.accounts():
        lines.append(f"{label} @{login}" if login else f"{label} not resolved")
    lines.append(reader.scope_line())
    if config.watch.mode == "selected" and not config.watch.repositories:
        lines.append("watching nothing")
    filtered = bool(view.get("filter") or view.get("unread_only"))
    if filtered:
        filtered_unread = sum(1 for record in activities if not record.get("read"))
        lines.append(f"{filtered_unread} unread of {len(activities)}")
    else:
        lines.append(f"{unread} unread of {total_activity}")
    lines.append(f"sort: {view.get('sort', 'updated')} {'↑' if view.get('sort_direction', 1) > 0 else '↓'}")
    lines.append("─" * 60)
    if not prs:
        lines.append("no watched pull requests")
    for index, pr in enumerate(prs):
        marker = "▶" if index == view.get("pr_index", 0) else " "
        key = _key(pr)
        stack = pr.get("stack") or {}
        stack_text = f" ☰ {stack.get('position')}/{stack.get('size')}" if stack else ""
        people = ""
        if pr.get("creator") or pr.get("assignees"):
            people = " ☺"
            if pr.get("creator") and pr.get("assignees"):
                people += f" @{pr['creator']} → @{pr['assignees'][0]}"
            elif pr.get("creator"):
                people += f" @{pr['creator']}"
            elif pr.get("assignees"):
                people += f" @{pr['assignees'][0]}"
        ignored_mark = " ⊘" if pr.get("key", "").casefold() in ignored else ""
        lines.append(f"{marker} {key}  {pr.get('state')}  CI {pr.get('ci')}{stack_text}{people}{ignored_mark}")
        if stack:
            lines.append(f"    stack {stack.get('number')} base {stack.get('base')}")
    if prs:
        selected = prs[min(view.get("pr_index", 0), len(prs) - 1)]
        lines.append(f"Pull request {selected.get('key')}")
        lines.append(f"Selected pull request: {selected.get('key')}")
    if not activities:
        lines.append("no activity yet")
    for record in activities[:40]:
        state_text = "read" if record.get("read") else "unread"
        lines.append(f"  {_activity_type(record)}  {_when(record)}  {state_text}")
        if record.get("kind") == "comment":
            lines.append(f"    Comment by @{record.get('actor')}")
        excerpt = " ".join(str(record.get("body") or "").split())[:80]
        if excerpt:
            lines.append(f"    {excerpt}")
        if record.get("summary") and record.get("summary") != _activity_type(record):
            lines.append(f"    {record.get('summary')}")
    if view.get("filter"):
        if view.get("filter_stack"):
            selected_pr = next((pr for pr in prs if pr.get("key") == view["filter"]), None)
            stack = (selected_pr or {}).get("stack") or {}
            lines.append(f"Activity · {str(view['filter']).split('#')[0]} stack {stack.get('number')}")
        else:
            lines.append(f"Activity · {view['filter']}")
    elif view.get("unread_only"):
        lines.append("Activity · unread")
    if view.get("filter") or view.get("unread_only"):
        lines.append(f"(filtered from {total_activity})")
    if activities:
        record = activities[min(view.get("activity_index", 0), len(activities) - 1)]
        lines.append(f"Selected activity: {record.get('summary')}")
    lines.append("mark visible read")
    ignored_selected = False
    if prs:
        current = prs[min(view.get("pr_index", 0), len(prs) - 1)]
        ignored_selected = current.get("key", "").casefold() in ignored
    lines.append(("x unignore" if ignored_selected else "x ignore") + "   tab switch   h help   q quit")
    lines.append("Focus: " + ("PRs" if view.get("focus", "prs") == "prs" else "activity"))
    if view.get("status"):
        lines.append(str(view["status"]))
    overlay = view.get("overlay")
    if overlay == "help":
        lines.append("Pull request pane" if view.get("focus", "activity") == "prs" else "Activity pane")
        lines.append("Key reference")
        lines.append("mark visible read  enter open  r mark read  a mark visible read")
        lines.append("tab switch  j/k move  s sort  f filter  u unread  x ignore  , settings  p watch scope  q close")
    elif overlay == "settings":
        lines.append("Settings")
        lines.append(("Desktop notifications: " + ("on" if view.get("desktop", config.notifications.desktop) else "off")))
        lines.append("Sound: " + ("on" if view.get("sound", config.notifications.sound) else "off"))
        lines.append("j/k select  space toggle  esc close")
    elif overlay == "picker":
        lines.append("Watch scope")
        lines.append("Mode: " + view.get("picker_mode", config.watch.mode))
        known = sorted({pr.get("key", "").split("#", 1)[0] for pr in state.get("pull_requests", {}).values()} | set(config.watch.repositories))
        for repo in known:
            mark = "[x]" if repo.casefold() in {item.casefold() for item in config.watch.repositories} else "[ ]"
            lines.append(f"{mark} {repo}")
        lines.append("Focus")
        for relation in ("authored", "assigned", "review requested", "team review"):
            actual = {"review requested": "review-requested", "team review": "team-review-requested"}.get(relation, relation)
            mark = "[x]" if actual in config.focus.relations else "[ ]"
            lines.append(f"{mark} {relation}")
        if not known:
            lines.append("Add: owner/name")
    elif overlay in {"activity_detail", "pr_detail"}:
        if overlay == "activity_detail":
            records = activities
            if not records:
                lines.append("no activity")
            else:
                record = records[min(view.get("activity_index", 0), len(records) - 1)]
                pr = state.get("pull_requests", {}).get(record.get("pull_request")) or {}
                number = str(pr.get("key", "")).split("#")[-1]
                repo_name = str(pr.get("key", "")).split("/")[-1].split("#")[0]
                lines.append(f"Activity {repo_name}#{number}")
                lines.append(f"Activity · {pr.get('key')}")
                lines.append(f"Comment by @{record.get('actor')}" if record.get("kind") == "comment" else record.get("summary", ""))
                if record_approximate(record):
                    lines.append(f"When: ~{format_when(record.get('observed_at'))} (fetch time)")
                else:
                    lines.append(f"When: {_when(record)}")
                    if record.get("occurred_at") and record.get("observed_at"):
                        lines.append(f"Fetched: {format_when(record.get('observed_at'))}")
                lines.append(f"Pull request {pr.get('key')}")
                lines.append(f"Status: {pr.get('state')}   CI: {pr.get('ci')}")
                lines.append(f"Creator: @{pr.get('creator')}")
                if pr.get("assignees"):
                    lines.append("Assignee: " + ", ".join("@" + login for login in pr["assignees"]))
                lines.append("unread" if not record.get("read") else "read")
                stack = pr.get("stack") or {}
                if stack:
                    lines.append(f"stack {stack.get('number')} {stack.get('position')}/{stack.get('size')} base {stack.get('base')}")
                body = markdown_lines(record.get("body") or "")
                start = view.get("detail_scroll", 0)
                window = body[start:start + 10]
                lines.extend(window)
                if view.get("scroll_end"):
                    lines.extend(body[-10:])
                lines.append("Enter open in browser")
        else:
            prs_now = prs
            if prs_now:
                pr = prs_now[min(view.get("pr_index", 0), len(prs_now) - 1)]
                lines.append(f"Pull request {pr.get('key')}")
                lines.append(f"Status: {pr.get('state')}   CI: {pr.get('ci')}")
                lines.append(f"Creator: @{pr.get('creator')}")
                if pr.get("assignees"):
                    lines.append("Assignee: " + ", ".join("@" + login for login in pr["assignees"]))
                lines.append("Description:")
                body = markdown_lines(pr.get("description") or "")
                start = view.get("detail_scroll", 0)
                lines.extend(body[start:start + 10])
                if view.get("scroll_end"):
                    lines.extend(body[-10:])
                lines.append("Enter open in browser")
    return lines


def render_from_paths(config: AppConfig, state_path, view: dict[str, Any]) -> list[str]:
    return render(config, load_state(state_path), view)
