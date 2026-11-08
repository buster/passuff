"""Step bindings for activity-inbox."""

from __future__ import annotations

import json
import os
import shlex
import stat
import time
from pathlib import Path

import tomlkit
from behave import given, step, then, use_step_matcher

from givn.steps.support import _assert, _config, _parse_key, assert_any_role, assert_line_role, assert_role, assert_role_digits, assert_role_member, transcript, w
from tests.support.pty_driver import run_pty


use_step_matcher("parse")

@step('the state path is a directory')
def state_path_dir(context):
    world = w(context)
    world.state_path.mkdir(exist_ok=True)

@step('a version-1 state file with one watched pull request')
def version_one_state(context):
    world = w(context)
    state = world.empty_state(1)
    state["accounts"] = {"personal": {"login": "octo-personal"}}
    state["pull_requests"] = {
        "acme/api#42": {
            "key": "acme/api#42",
            "title": "PR 42",
            "creator": "octo-personal",
            "state": "open",
            "ci": "passing",
            "relations": ["authored"],
        }
    }
    world.write_state(state)

@step('a state with one activity record')
def state_one_activity(context):
    world = w(context)
    if not world.accounts:
        world.account("personal", "octo-personal")
    state = world.empty_state()
    world.add_state_pr(state, "acme/api#42")
    world.add_state_activity(state, "a1", "New comment from @alice")
    state["next_activity_id"] = 2
    world.write_state(state)

@step('a state with 500 activity records')
def state_500(context):
    world = w(context)
    if not world.accounts:
        world.account("personal", "octo-personal")
    state = world.empty_state()
    world.add_state_pr(state, "acme/api#42")
    for index in range(1, 501):
        world.add_state_activity(state, f"a{index}", f"New comment {index:03d}")
    state["next_activity_id"] = 501
    world.write_state(state)

@step('the state retains 500 activity records')
def state_retains_500(context):
    assert len(w(context).load_state().get("activity", [])) == 500

@step('the oldest activity record is gone')
def oldest_gone(context):
    summaries = [item.get("summary") for item in w(context).load_state().get("activity", [])]
    assert "New comment 001" not in summaries

@step('the developer marks activity "{identifier}" read')
def mark_activity(context, identifier):
    w(context).run(["read", identifier])

@step('the developer runs read with no target')
def read_no_target(context):
    w(context).run(["read"])

@step('the developer starts the inbox TUI without a snapshot')
def start_tui_no_snapshot(context):
    w(context).run_tui([])

@step('the developer starts the inbox TUI binary')
def start_tui_binary(context):
    world = w(context)
    world.write_config()
    cmd = world._command(True)
    output, code = run_pty(cmd, world._base_env(), "q")
    from tests.support.world import Result

    world.last = Result(output, "", code)

@step('the browser will fail to open')
def browser_fail(context):
    w(context).browser_fail = True

@step('the inbox shows "{first}" before "{second}" for "{key}"')
def inbox_order(context, first, second, key):
    body = transcript(context)
    assert first in body and second in body and body.index(first) < body.index(second), body

@step('the inbox shows "{summary}" for "{key}" as read')
def inbox_record_read(context, summary, key):
    record = w(context).state_activity(summary)
    assert record and record.get("read"), record

@step('every inbox record is unread')
def every_unread(context):
    world = w(context)
    state = world.load_state()
    for record in state.get("activity", []):
        record["read"] = False
    world.write_state(state)

@step('no state file was created')
def no_state_created(context):
    assert not w(context).state_path.exists()

@step('the developer reopens passuff')
def reopens(context):
    world = w(context)
    assert world.state_path.exists()
    world.last = None

@step('the inbox shows an activity id and occurrence time')
def inbox_id_time(context):
    import re as _re
    body = transcript(context)
    assert _re.search(r"\[a\d+\]", body), body
    assert _re.search(r"\d\d-\d\d \d\d:\d\d", body), body

@step('the inbox shows the occurrence time "{occurred}" for "{key}"')
def inbox_time_for(context, occurred, key):
    from passuff.inbox import format_when
    text = format_when(occurred)
    _assert(context, text)

@step('the inbox marks "{summary}" on "{key}" as approximate')
def inbox_approx(context, summary, key):
    body = transcript(context)
    assert "~" in body and summary in body, body

@step('the pull request "{key}" is ignored')
def pr_ignored(context, key):
    world = w(context)
    if key not in world.ignored:
        world.ignored.append(key)
    world.dirty = True
    world.write_config(force=True)

@step('the selected activity is "{summary}"')
def assert_selected_activity(context, summary):
    _assert(context, f"Selected activity: {summary}")

@step('the terminal UI is closed')
def tui_closed(context):
    world = w(context)
    assert world.last is not None and world.last.code == 0, (world.last.code if world.last else None)


@step('the frame title is bright and bold')
def frame_title_role(context):
    assert_role(context, "passuff", "bright_bold")


@step('the frame title suffix is dim')
def frame_suffix_role(context):
    assert_role(context, "— Inbox", "dim")


@step('the outer frame border is dim')
def outer_border_role(context):
    assert_line_role(context, "passuff", "dim")


@step('the account login is cyan-blue')
def account_login_role(context):
    assert_role(context, "@", "accent")


@step('the unread count is gold and bold')
def unread_count_role(context):
    assert_role_digits(context, "gold_bold")


@step('the focus label is dim')
def focus_label_role(context):
    assert_role(context, "Focus:", "dim")


@step('the focused pane name is cyan-blue and bold')
def focused_name_role(context):
    assert_role_member(context, "accent_bold", {"activity", "PRs"})


@step('the activity body is bright gray')
def activity_body_role(context):
    assert_any_role(context, "body")


@step('the controls bar title is cyan-blue')
def controls_title_role(context):
    assert_role(context, "Controls", "accent")


@step('the key hints are cyan-blue and bold')
def key_hints_role(context):
    assert_role_member(context, "accent_bold", {"x", "tab", "h", "q"})


@step('the ignored row is dim')
def ignored_row_role(context):
    assert_role(context, "⊘", "dim")


@step('the watched table shows the people column')
def people_column_role(context):
    assert_role(context, "☺", "accent_bold")


@step('the empty state text is bright')
def empty_state_role(context):
    assert_role(context, "no watched pull requests", "bright")


@step('the detail overlay border is dim')
def detail_border_role(context):
    assert_line_role(context, "Activity api", "dim")


@step('the detail overlay title is cyan-blue and bold')
def detail_title_role(context):
    assert_line_role(context, "Activity api", "accent_bold")


@step('the detail separator uses the rule color')
def detail_separator_role(context):
    assert_role(context, "─", "rule")


@step('the developer opens the inbox TUI observing the account')
def open_tui_observe(context):
    w(context).open_tui(["--snapshot", "--observe"])
