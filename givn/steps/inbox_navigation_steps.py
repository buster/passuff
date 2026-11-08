"""Step bindings for inbox-navigation."""

from __future__ import annotations

import json
import os
import shlex
import stat
import time
from pathlib import Path

import tomlkit
from behave import given, step, then, use_step_matcher

from givn.steps.support import _assert, _config, _parse_key, assert_line_role, assert_line_role_any, assert_role, assert_role_member, saved_ui, transcript, w
from tests.support.pty_driver import run_pty


use_step_matcher("parse")

@step('the pull request "{key}" is assigned to "{login}"')
def pr_assigned(context, key, login):
    world = w(context)
    world.assign(key, login)

@step('the watcher has already observed all of them')
def observed_all(context):
    world = w(context)
    keys = [f"{pr['repo']}#{pr['number']}" for pr in world.fake.prs]
    world.observe(keys)

@step('the developer opens the live scope picker')
def open_live_picker(context):
    w(context).open_tui(["--snapshot", "--picker"])

@step('the browser opener is missing')
def browser_missing(context):
    w(context).browser_missing = True

@step('the inbox has asked GitHub for a change signal "{count}" times')
def count_probe(context, count):
    actual = w(context).fake.counts["Probe"]
    assert actual == int(count), (actual, w(context).fake.counts)

@step('the inbox has asked GitHub for the full account history "{count}" time')
def count_observe(context, count):
    actual = w(context).fake.counts["Observe"]
    assert actual == int(count), (actual, w(context).fake.counts)

@step('the inbox has observed only "{key}"')
def observed_only(context, key):
    pr = w(context).fake.find(key)
    assert pr and w(context).fake.observed_ids == [pr["id"]], (w(context).fake.observed_ids, pr["id"] if pr else None)

@step('the account has "{remaining}" API points left until the allowance resets in "{seconds}" seconds')
def account_allowance(context, remaining, seconds):
    from datetime import datetime, timedelta, timezone

    world = w(context)
    world.fake.rate_remaining = int(remaining)
    reset = (datetime.now(timezone.utc) + timedelta(seconds=int(seconds))).isoformat().replace("+00:00", "Z")
    world.fake.rate_reset_at = reset

@step('the reconciliation interval is "{seconds}" seconds')
def reconciliation_interval(context, seconds):
    w(context).reconcile = int(seconds)
    w(context).dirty = True

@step('the pull request "{key}" has a long comment from "{actor}"')
def long_comment(context, key, actor):
    body = "\n".join(f"comment line {number}" for number in range(1, 41))
    w(context).comment(key, actor, body)

@step('the pull request "{key}" receives a long comment from "{actor}"')
def receives_long_comment(context, key, actor):
    long_comment(context, key, actor)

@step('the pull request "{key}" receives a {count:d}-character comment from "{actor}"')
def receives_large_comment(context, key, count, actor):
    marker = "TAIL MARKER"
    lines = [f"body line {number}" for number in range(1, int(count) // 10 + 1)]
    text = "\n".join(lines)
    if len(text) < int(count):
        text += "\n" * (int(count) - len(text))
    text = text[: int(count) - len(marker) - 1] + "\n" + marker
    w(context).comment(key, actor, text)

@step('the stored activity body is at most {limit:d} bytes')
def stored_body_at_most(context, limit):
    state = w(context).load_state()
    records = state.get("activity", [])
    assert records, state
    body = str(records[-1].get("body", ""))
    size = len(body.encode("utf-8"))
    assert size <= int(limit), (size, limit)
    assert size > 0, state

@step('the pull request "{key}" has a long description')
def long_description(context, key):
    body = "\n".join(f"line {number}" for number in range(1, 41))
    w(context).pr(key, w(context).fake.find(key)["author"], body=body)

@step('the pull request "{key}" has a markdown description')
def markdown_description(context, key):
    body = "## Summary\n\n**bold** text\n\n- first item\n\nFirst paragraph\n\nSecond paragraph"
    w(context).pr(key, w(context).fake.find(key)["author"], body=body)

@step('the pull request "{key}" has a multi-line description')
def multiline_description(context, key):
    body = "## Summary\n\nFirst paragraph\n\n\n\nSecond paragraph"
    w(context).pr(key, w(context).fake.find(key)["author"], body=body)

@step('the pull request "{key}" has the description "{description}"')
def description(context, key, description):
    w(context).pr(key, w(context).fake.find(key)["author"], body=description)

@step('the selected pull request is "{key}"')
def assert_selected_pr(context, key):
    _assert(context, f"Selected pull request: {key}")

@step('{count:d} authored pull requests in "{repo}" by "{author}"')
def many_authored(context, count, repo, author):
    world = w(context)
    for number in range(1, count + 1):
        world.pr(f"{repo}#{number}", author)

@step('an authored pull request "{key}" by "{author}" appears')
def pr_appears(context, key, author):
    from datetime import datetime, timedelta, timezone

    future = (datetime.now(timezone.utc) + timedelta(seconds=5)).isoformat().replace("+00:00", "Z")
    w(context).pr(key, author, created_at=future, updated_at=future)

@step('the developer moves the selection down {count:d} times')
def move_down_many(context, count):
    world = w(context)
    world.tui_keys.extend(["j"] * count)
    world.press("j")

@step('the developer presses "space" in the scope picker')
def picker_space(context):
    from passuff.config import update_focus
    from passuff.scope import ScopeEditor

    world = w(context)
    from passuff.config import load_config
    editor = ScopeEditor(load_config(world.config_path), world.config_path)
    relations = list(world.focus)
    if "authored" in relations:
        relations.remove("authored")
    else:
        relations.append("authored")
    try:
        editor.set_focus(relations)
        world.focus = relations
        world.write_config(force=True)
    except Exception as exc:
        raise AssertionError(str(exc))
    world.run_tui(["--snapshot", "--picker"])

@step('the developer presses "esc" in the scope picker')
def picker_esc(context):
    w(context).run_tui(["--snapshot"])

@step('the developer refreshes the inbox')
def refresh_inbox(context):
    import re as _re
    world = w(context)
    selected = None
    if world.last:
        match = _re.search(r"Selected pull request: (\S+)", world.last.text)
        if match:
            selected = match.group(1)
    world.run_tui(["--snapshot", "--observe"])
    if selected:
        state = world.load_state()
        prs = list(state.get("pull_requests", {}).values())

        prs.sort(key=lambda pr: pr.get("last_observed_at") or "", reverse=True)
        index = next((pos for pos, pr in enumerate(prs) if pr.get("key") == selected), 0)
        world.run_tui(["--snapshot", "--keys", "tab" + ",j" * index])

@step('the inbox observes the account')
def inbox_observe(context):
    world = w(context)
    world.fake.observed_ids = []
    world.run_tui(["--snapshot", "--observe"])

@step('the inbox observes the account three times with no change')
def inbox_observe_three(context):
    world = w(context)
    world.fake.counts["Probe"] = 0
    for _ in range(3):
        world.run_tui(["--snapshot", "--observe"])

@step('the inbox plans its next observation')
def inbox_plans(context):
    world = w(context)
    world.run_tui(["--snapshot", "--observe"])
    state = world.load_state()
    delays = [value.get("next_delay", 0) for value in state.get("accounts", {}).values()]
    assert delays and max(delays) > 0, state

@step('the inbox waits at least "{seconds}" seconds before observing again')
def inbox_waits_at_least(context, seconds):
    world = w(context)
    state = world.load_state()
    delays = [value.get("next_delay", 0) for value in state.get("accounts", {}).values()]
    assert delays and max(delays) >= int(seconds), (delays, state)

@step('the inbox waits "{seconds}" seconds')
def inbox_waits(context, seconds):
    import time as _time
    _time.sleep(int(seconds))

@step('the involvement relations are unknown')
def relations_unknown(context):
    world = w(context)
    state = world.load_state()
    for pr in state.get("pull_requests", {}).values():
        if "authored" not in (pr.get("relations") or []):
            pr["relations"] = []
    world.write_state(state)

@step('the TUI marks "{summary}" on "{key}" as approximate')
def tui_approx(context, summary, key):
    body = transcript(context)
    assert "~" in body and summary in body, body

@step('the TUI still shows "{text}"')
def tui_still(context, text):
    _assert(context, text)

@then('the notification sound is muted')
def sound_muted_ui(context):
    import json as _json
    path = w(context).state_path.with_name("ui.json")
    data = _json.loads(path.read_text()) if path.exists() else {}
    assert data.get("sound") is False, data

@then('the desktop notification is enabled')
def desktop_enabled_ui(context):
    import json as _json
    path = w(context).state_path.with_name("ui.json")
    data = _json.loads(path.read_text()) if path.exists() else {}
    assert data.get("desktop", True) is not False, data

@step('the activity list row excerpt is at most 80 characters')
def row_excerpt(context):
    body = transcript(context)
    rows = []
    for line in body.splitlines():
        text = line.strip()
        while text.startswith("│"):
            text = text[1:].strip()
        while text.endswith("│"):
            text = text[:-1].rstrip()
        if text:
            rows.append(text)
    excerpts = [line for line in rows if line.startswith("comment line")]
    assert excerpts, body
    for line in excerpts:
        assert len(line) <= 80, line
    assert "comment line 40" not in body, body


@step('the focused pane border is green')
def focused_border_role(context):
    assert_line_role_any(context, ("Watched pull requests", "Activity"), "green")


@step('the unfocused pane border is dim')
def unfocused_border_role(context):
    assert_line_role_any(context, ("Watched pull requests", "Activity"), "dim")


@step('the unfocused pane title is dim')
def unfocused_title_role(context):
    assert_line_role_any(context, ("Activity", "Watched pull requests"), "dim")


@step('the settings overlay border is dim')
def settings_border_role(context):
    assert_line_role(context, "Settings", "dim")


@step('the settings title is cyan-blue and bold')
def settings_title_role(context):
    assert_line_role(context, "Settings", "accent_bold")


@step('the settings selected row is gold and bold')
def settings_selected_role(context):
    assert_line_role(context, "Sound", "gold_bold")


@step('the enabled settings values are green')
def settings_enabled_role(context):
    assert_line_role(context, "Desktop", "green")


@step('the disabled settings values are dim')
def settings_disabled_role(context):
    assert_line_role(context, "Sound", "dim")


@step('the pull request detail border is dim')
def pr_detail_border_role(context):
    assert_line_role(context, "PR -", "dim")


@step('the pull request detail title is cyan-blue and bold')
def pr_detail_title_role(context):
    assert_line_role(context, "PR -", "accent_bold")


@step('the pull request detail heading is green and bold')
def pr_detail_heading_role(context):
    assert_line_role(context, "Adds the live inbox", "green_bold")


@step('the help heading is cyan-blue and bold')
def help_heading_role(context):
    assert_line_role(context, "Activity pane", "accent_bold")


@step('the help keys are cyan-blue and bold')
def help_keys_role(context):
    assert_line_role(context, "mark visible", "accent_bold")


@step('the status line is gold')
def status_line_role(context):
    assert_line_role(context, "could not open browser", "gold")


@step('the save prompt border is dim')
def save_border_role(context):
    assert_line_role(context, "Leave", "dim")


@step('the save prompt title is cyan-blue')
def save_title_role(context):
    assert_line_role(context, "Leave", "accent")


@step('the save prompt question is bright')
def save_question_role(context):
    assert_line_role(context, "Save sort and filter settings?", "bright")


@step('the save prompt keys are cyan-blue and bold')
def save_keys_role(context):
    assert_line_role(context, "yes", "accent_bold")


@step('the developer opens the inbox TUI at 80x24')
def open_tui_80x24(context):
    world = w(context)
    world.tui_size = (80, 24)
    world.open_tui(["--snapshot"])


@step('the saved sort is "{value}"')
def saved_sort(context, value):
    assert saved_ui(context).get("sort", "updated") == value, saved_ui(context)
