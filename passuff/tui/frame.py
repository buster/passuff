"""Styled text frame renderer for the terminal inbox."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from typing import Any

from ..config import AppConfig
from ..inbox import ActivityInboxService, format_when, record_approximate
from ..state import load_state
from ..status import WatchStatusReader


@dataclass(frozen=True)
class Span:
    text: str
    role: str = "bright"


@dataclass
class StyledLine:
    spans: list[Span]

    @property
    def text(self) -> str:
        return "".join(span.text for span in self.spans)


PALETTE: dict[str, tuple[int | None, bool, bool]] = {
    "accent": (81, False, False),
    "accent_bold": (81, True, False),
    "bright": (15, False, False),
    "bright_bold": (15, True, False),
    "body": (252, False, False),
    "gold": (214, False, False),
    "gold_bold": (214, True, False),
    "green": (114, False, False),
    "green_bold": (114, True, False),
    "rule": (240, False, False),
    "dim": (None, False, True),
}


def _line(text: str, role: str = "bright") -> StyledLine:
    return StyledLine([Span(text, role)])


def _parts(*parts: tuple[str, str]) -> StyledLine:
    return StyledLine([Span(text, role) for text, role in parts])


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


def markdown_lines(text: str) -> list[str]:
    return [line for line, _role in markdown_styled(text)]


def markdown_styled(text: str) -> list[tuple[str, str]]:
    lines: list[tuple[str, str]] = []
    in_code = False
    for raw in str(text or "").splitlines():
        line = raw.rstrip()
        stripped = line.strip()
        if stripped.startswith("```"):
            in_code = not in_code
            continue
        if in_code:
            lines.append(("  " + line, "dim"))
            continue
        heading = bool(re.match(r"^#{1,6}\s+", stripped))
        line = re.sub(r"<!--.*?-->", "", line)
        line = re.sub(r"^#{1,6}\s+", "", line)
        line = line.replace("**", "").replace("__", "").replace("`", "")
        line = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", line)
        line = re.sub(r"^\s*[-*]\s+", "• ", line)
        line = re.sub(r"^\s*\d+\.\s+", "• ", line)
        line = re.sub(r"^\s*---+\s*$", "", line)
        lines.append((line, "green_bold" if heading else "body"))
    return lines


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


def _box(lines: list[StyledLine], title: str, title_role: str, width: int) -> list[StyledLine]:
    top = _parts(("┌─ ", "dim"), (title, title_role), (" ", "dim"), ("─" * max(1, width - len(title) - 6), "dim"), ("┐", "dim"))
    body = []
    for line in lines:
        text = line.text
        if len(text) > width - 4:
            text = text[: width - 4]
            line = _line(text, line.spans[0].role if line.spans else "bright")
        pad = " " * max(0, width - 4 - len(text))
        body.append(StyledLine([Span("│ ", "dim")] + line.spans + [Span(pad, "dim"), Span(" │", "dim")]))
    bottom = _line("└" + "─" * max(1, width - 2) + "┘", "dim")
    return [top] + body + [bottom]


def render_styled(config: AppConfig, state: dict[str, Any], view: dict[str, Any]) -> list[StyledLine]:
    reader = WatchStatusReader(config, state)
    service = ActivityInboxService(config, state)
    prs = _sort_prs(reader.watched(), view.get("sort", "updated"), view.get("sort_direction", 1))
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
    focus = view.get("focus", "activity")
    width = max(40, int(view.get("width", 100)))
    lines: list[StyledLine] = []

    header: list[Span] = []
    for index, (label, login) in enumerate(reader.accounts()):
        if index > 0:
            header.append(Span("   |   ", "dim"))
        header.append(Span(label, "bright"))
        header.append(Span(" ", "bright"))
        header.append(Span(f"@{login}" if login else "not resolved", "accent"))
    header.append(Span("   |   ", "dim"))
    header.append(Span("watch scope: ", "dim"))
    header.append(Span(reader.scope_line().removeprefix("watch scope: "), "bright"))
    lines.append(StyledLine(header))

    filtered = bool(view.get("filter") or view.get("unread_only"))
    if filtered:
        filtered_unread = sum(1 for record in activities if not record.get("read"))
        count = filtered_unread
        visible = len(activities)
    else:
        count = unread
        visible = total_activity
    counts = [
        Span(str(count), "gold_bold"),
        Span(" unread of ", "dim"),
        Span(str(visible), "bright"),
    ]
    if filtered:
        counts.append(Span(f"   (filtered from {total_activity})", "dim"))
    counts.extend([
        Span("   |   ", "dim"),
        Span("Focus: ", "dim"),
        Span("PRs" if focus == "prs" else "activity", "accent_bold"),
    ])
    lines.append(StyledLine(counts))

    if config.watch.mode == "selected" and not config.watch.repositories:
        lines.append(_line("watching nothing", "dim"))

    watched_focus = focus == "prs"
    watched_border = "green" if watched_focus else "dim"
    watched_title = "green_bold" if watched_focus else "dim"
    direction = "↑" if view.get("sort_direction", 1) > 0 else "↓"
    mode = "all" if view.get("show_ignored") else "filtered"
    lines.append(_parts(
        ("── ", watched_border),
        ("Watched pull requests", watched_title),
        (f" · {mode} · sort: {view.get('sort', 'updated')} {direction} ", "dim"),
        ("──", watched_border),
    ))
    has_stack = any(pr.get("stack") for pr in prs)
    watched_headers = "Pull request   State  CI  " + ("☰  " if has_stack else "") + "☺  Title"
    lines.append(_line(watched_headers, "accent_bold"))
    if not prs:
        lines.append(_line("no watched pull requests", "bright"))
    for index, pr in enumerate(prs):
        selected = index == view.get("pr_index", 0)
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
        marker = "▶ " if selected else "  "
        row_role = "dim" if pr.get("key", "").casefold() in ignored else ("gold_bold" if selected else "bright")
        lines.append(_parts(
            (marker, "gold_bold" if selected else "dim"),
            (f"{key}  {pr.get('state')}  CI {pr.get('ci')}{stack_text}{people}{ignored_mark}", row_role),
        ))
        if stack:
            lines.append(_line(f"    stack {stack.get('number')} base {stack.get('base')}", "dim"))
    if prs:
        selected = prs[min(view.get("pr_index", 0), len(prs) - 1)]
        lines.append(_line(f"Pull request {selected.get('key')}", "bright"))
        lines.append(_line(f"Selected pull request: {selected.get('key')}", "dim"))

    activity_focus = focus != "prs"
    activity_border = "green" if activity_focus else "dim"
    activity_title = "green_bold" if activity_focus else "dim"
    title_suffix = ""
    if view.get("filter"):
        title_suffix = f" · {str(view['filter'])}"
    elif view.get("unread_only"):
        title_suffix = " · unread"
    lines.append(_parts(
        ("── ", activity_border),
        ("Activity", activity_title),
        (title_suffix + " ", "dim"),
        ("──", activity_border),
    ))
    lines.append(_line("When        Event                                      Stack  Read", "accent_bold"))
    if not activities:
        lines.append(_line("no activity yet", "bright"))
    for index, record in enumerate(activities[:40]):
        selected = index == view.get("activity_index", 0)
        marker = "▶ " if selected else "  "
        role = "gold_bold" if selected else "bright"
        state_text = "read" if record.get("read") else "unread"
        lines.append(_parts(
            (marker, "gold_bold" if selected else "dim"),
            (f"{_activity_type(record)}  {_when(record)}  {state_text}", role),
        ))
        if record.get("kind") == "comment":
            lines.append(_line(f"    Comment by @{record.get('actor')}", "dim" if not selected else "gold_bold"))
        excerpt = " ".join(str(record.get("body") or "").split())[:80]
        if excerpt:
            lines.append(_line(f"    {excerpt}", "body"))
        if record.get("summary") and record.get("summary") != _activity_type(record):
            lines.append(_line(f"    {record.get('summary')}", "dim"))
    if view.get("filter"):
        if view.get("filter_stack"):
            selected_pr = next((pr for pr in prs if pr.get("key") == view["filter"]), None)
            stack = (selected_pr or {}).get("stack") or {}
            lines.append(_line(f"Activity · {str(view['filter']).split('#')[0]} stack {stack.get('number')}", "dim"))
        else:
            lines.append(_line(f"Activity · {view['filter']}", "dim"))
    elif view.get("unread_only"):
        lines.append(_line("Activity · unread", "dim"))
    if activities:
        record = activities[min(view.get("activity_index", 0), len(activities) - 1)]
        lines.append(_line(f"Selected activity: {record.get('summary')}", "dim"))

    lines.append(_line("Controls", "accent"))
    lines.append(_line("mark visible read", "dim"))
    ignored_selected = False
    if prs:
        current = prs[min(view.get("pr_index", 0), len(prs) - 1)]
        ignored_selected = current.get("key", "").casefold() in ignored
    footer = [
        Span("x", "accent_bold"),
        Span(" unignore" if ignored_selected else " ignore", "dim"),
        Span("   ", "dim"),
        Span("tab", "accent_bold"),
        Span(" switch", "dim"),
        Span("   ", "dim"),
        Span("h", "accent_bold"),
        Span(" help", "dim"),
        Span("   ", "dim"),
        Span("q", "accent_bold"),
        Span(" quit", "dim"),
    ]
    lines.append(StyledLine(footer))
    if view.get("status"):
        lines.append(_line(str(view["status"]), "gold"))

    overlay = view.get("overlay")
    if overlay == "help":
        pane = "PRs" if focus == "prs" else "activity"
        heading = "Pull request pane" if pane == "PRs" else "Activity pane"
        help_lines = [
            _line(heading, "accent_bold"),
            _line("Key reference", "accent_bold"),
            _parts(("mark", "accent_bold"), (" visible read  ", "dim"), ("enter", "accent_bold"), (" open  ", "dim"), ("r", "accent_bold"), (" mark read  ", "dim"), ("a", "accent_bold"), (" mark visible read", "dim")),
            _parts(("tab", "accent_bold"), (" switch  ", "dim"), ("j/k", "accent_bold"), (" move  ", "dim"), ("s", "accent_bold"), (" sort  ", "dim"), ("f", "accent_bold"), (" filter  ", "dim"), ("u", "accent_bold"), (" unread  ", "dim"), ("x", "accent_bold"), (" ignore  ", "dim"), (",", "accent_bold"), (" settings  ", "dim"), ("p", "accent_bold"), (" watch scope  ", "dim"), ("q", "accent_bold"), (" close", "dim")),
        ]
        lines.extend(_box(help_lines, "Help", "accent_bold", width - 4))
    elif overlay == "settings":
        settings_lines: list[StyledLine] = []
        for index, (label, actual) in enumerate((
            ("Desktop notifications", "desktop"),
            ("Sound", "sound"),
        )):
            enabled = bool(view.get(actual, config.notifications.__getattribute__(actual)))
            selected = int(view.get("settings_index", 0)) == index
            marker = "▶ " if selected else "  "
            settings_lines.append(_parts(
                (marker, "gold_bold" if selected else "dim"),
                (label + ": ", "gold_bold" if selected else "bright"),
                ("on" if enabled else "off", "green" if enabled else "dim"),
            ))
        settings_lines.append(_parts(("j/k", "accent_bold"), (" select  ", "dim"), ("space", "accent_bold"), (" toggle  ", "dim"), ("esc", "accent_bold"), (" close", "dim")))
        lines.extend(_box(settings_lines, "Settings", "accent_bold", width - 4))
    elif overlay == "picker":
        picker_lines: list[StyledLine] = [
            _parts(("Mode: ", "dim"), (str(view.get("picker_mode", config.watch.mode)), "accent_bold")),
        ]
        known = sorted({pr.get("key", "").split("#", 1)[0] for pr in state.get("pull_requests", {}).values()} | set(config.watch.repositories))
        for index, repo in enumerate(known):
            mark = "[x]" if repo.casefold() in {item.casefold() for item in config.watch.repositories} else "[ ]"
            role = "gold_bold" if index == int(view.get("picker_cursor", 0)) else "bright"
            picker_lines.append(_line(f"{mark} {repo}", role))
        picker_lines.append(_line("Focus", "dim"))
        for relation in ("authored", "assigned", "review requested", "team review"):
            actual = {"review requested": "review-requested", "team review": "team-review-requested"}.get(relation, relation)
            mark = "[x]" if actual in config.focus.relations else "[ ]"
            picker_lines.append(_line(f"{mark} {relation}", "green" if mark == "[x]" else "dim"))
        if not known:
            picker_lines.append(_line("Add: owner/name", "dim"))
        picker_lines.append(_parts(("space", "accent_bold"), (" toggle   ", "dim"), ("esc", "accent_bold"), (" close", "dim")))
        picker_block = _box(picker_lines, "passuff — Watch scope", "bright_bold", width - 4)
        picker_block[0] = _parts(("┌─ ", "dim"), ("passuff ", "bright_bold"), ("— Watch scope ", "dim"), ("─" * max(1, width - 30), "dim"), ("┐", "dim"))
        lines.extend(picker_block)
    elif overlay in {"activity_detail", "pr_detail"}:
        if overlay == "activity_detail":
            records = activities
            if not records:
                lines.extend(_box([_line("no activity", "dim")], "Activity", "accent_bold", width - 4))
            else:
                record = records[min(view.get("activity_index", 0), len(records) - 1)]
                pr = state.get("pull_requests", {}).get(record.get("pull_request")) or {}
                number = str(pr.get("key", "")).split("#")[-1]
                repo_name = str(pr.get("key", "")).split("/")[-1].split("#")[0]
                body = markdown_styled(record.get("body") or "")
                start = int(view.get("detail_scroll", 0))
                window = body[start:start + 10]
                if view.get("scroll_end"):
                    window = body[-10:]
                detail_lines: list[StyledLine] = [
                    _line(f"Activity · {pr.get('key')}", "dim"),
                    _line(f"Comment by @{record.get('actor')}" if record.get("kind") == "comment" else str(record.get("summary", "")), "bright"),
                ]
                if record_approximate(record):
                    detail_lines.append(_parts(("When: ", "accent"), (f"~{format_when(record.get('observed_at'))} (fetch time)", "dim")))
                else:
                    detail_lines.append(_parts(("When: ", "accent"), (_when(record), "dim")))
                    if record.get("occurred_at") and record.get("observed_at"):
                        detail_lines.append(_parts(("Fetched: ", "accent"), (format_when(record.get("observed_at")), "dim")))
                detail_lines.extend([
                    _parts(("Pull request ", "accent"), (str(pr.get("key")), "dim")),
                    _parts(("Status: ", "accent"), (f"{pr.get('state')}   CI: {pr.get('ci')}", "dim")),
                    _parts(("Creator: ", "accent"), (f"@{pr.get('creator')}", "dim")),
                ])
                if pr.get("assignees"):
                    detail_lines.append(_parts(("Assignee: ", "accent"), (", ".join("@" + login for login in pr["assignees"]), "dim")))
                stack = pr.get("stack") or {}
                if stack:
                    detail_lines.append(_parts(("stack ", "accent"), (f"{stack.get('number')} {stack.get('position')}/{stack.get('size')} base {stack.get('base')}", "dim")))
                detail_lines.append(_line("─" * max(1, width - 6), "rule"))
                detail_lines.extend(_line(text, role) for text, role in window)
                detail_lines.append(_parts(("Enter", "accent_bold"), (" open in browser   ", "dim"), ("Esc/q", "accent_bold"), (" close", "dim")))
                lines.extend(_box(detail_lines, f"Activity {repo_name}#{number}", "accent_bold", width - 4))
        else:
            prs_now = prs
            if prs_now:
                pr = prs_now[min(view.get("pr_index", 0), len(prs_now) - 1)]
                body = markdown_styled(pr.get("description") or "")
                start = int(view.get("detail_scroll", 0))
                window = body[start:start + 10]
                if view.get("scroll_end"):
                    window = body[-10:]
                detail_lines = [
                    _parts(("Status: ", "accent"), (f"{pr.get('state')}   CI: {pr.get('ci')}", "dim")),
                    _parts(("Creator: ", "accent"), (f"@{pr.get('creator')}", "dim")),
                ]
                if pr.get("assignees"):
                    detail_lines.append(_parts(("Assignee: ", "accent"), (", ".join("@" + login for login in pr["assignees"]), "dim")))
                detail_lines.append(_line("Description:", "accent_bold"))
                detail_lines.extend(_line(text, role) for text, role in window)
                detail_lines.append(_parts(("Enter", "accent_bold"), (" open in browser   ", "dim"), ("Esc/q", "accent_bold"), (" close", "dim")))
                lines.extend(_box(detail_lines, f"PR - {pr.get('key')}", "accent_bold", width - 4))
            else:
                lines.extend(_box([_line("no pull request", "dim")], "PR", "accent_bold", width - 4))

    if view.get("save_prompt"):
        prompt_lines = [
            _line("Save sort and filter settings?", "bright"),
            _parts(("y", "accent_bold"), (" yes  ", "dim"), ("n", "accent_bold"), (" no  ", "dim"), ("Esc", "accent_bold"), (" cancel", "dim")),
        ]
        lines.extend(_box(prompt_lines, "Leave", "accent", width - 4))

    top = _parts(("┌─ ", "dim"), ("passuff ", "bright_bold"), ("— Inbox ", "dim"), ("─" * max(1, width - 16), "dim"), ("┐", "dim"))
    body_lines: list[StyledLine] = []
    for line in lines:
        text = line.text
        if len(text) > width - 4:
            text = text[: width - 4]
            line = _line(text, line.spans[0].role if line.spans else "bright")
        pad = " " * max(0, width - 4 - len(text))
        body_lines.append(StyledLine([Span("│ ", "dim")] + line.spans + [Span(pad, "dim"), Span(" │", "dim")]))
    bottom = _line("└" + "─" * max(1, width - 2) + "┘", "dim")
    return [top] + body_lines + [bottom]


def render(config: AppConfig, state: dict[str, Any], view: dict[str, Any]) -> list[str]:
    return [line.text for line in render_styled(config, state, view)]


def render_ansi(config: AppConfig, state: dict[str, Any], view: dict[str, Any]) -> str:
    result = []
    for line in render_styled(config, state, view):
        chunks = []
        for span in line.spans:
            color, bold, dim = PALETTE.get(span.role, PALETTE["bright"])
            codes = []
            if color is not None:
                codes.append(f"38;5;{color}")
            if bold:
                codes.append("1")
            if dim:
                codes.append("2")
            chunks.append("\x1b[" + (";".join(codes) if codes else "0") + "m" + span.text + "\x1b[0m")
        result.append("".join(chunks))
    return "\n".join(result)


def render_roles(config: AppConfig, state: dict[str, Any], view: dict[str, Any]) -> str:
    payload = [
        [{"text": span.text, "role": span.role} for span in line.spans]
        for line in render_styled(config, state, view)
    ]
    return json.dumps(payload)


def render_from_paths(config: AppConfig, state_path, view: dict[str, Any]) -> list[str]:
    return render(config, load_state(state_path), view)
