"""Shared step bindings for the passuff corpus."""

from __future__ import annotations

import json
import os
import shlex
import stat
import time
from pathlib import Path

import tomlkit
from behave import given, step, then, use_step_matcher

from givn.steps.support import _assert, _config, _parse_key, assert_line_span_role_any, assert_metadata_line_role, assert_metadata_value_role, assert_role, transcript, w
from tests.support.pty_driver import run_pty


use_step_matcher("parse")

@step('a configured account "{label}" with login "{login}"')
def account_login(context, label, login):
    w(context).account(label, login)

@step('a configured account "{label}" with no credential source')
def account_no_source(context, label):
    w(context).accounts[label] = {"auth": "github_pat", "graphql_url": w(context).fake.url()}
    w(context).dirty = True

use_step_matcher("re")

@step(r'an authored pull request "(?P<key>[^"]+)" by "(?P<author>[^"]+)"')
def authored_pr(context, key, author):
    w(context).pr(key, author)

use_step_matcher("parse")

@step('the pull request "{key}" is assigned to me')
def pr_assigned_me(context, key):
    world = w(context)
    world.assign(key, "octo-personal")
    world.pr(key, world.fake.find(key)["author"], assignees=world.fake.find(key)["assignees"])

use_step_matcher("re")

@step(r'the pull request "(?P<key>[^"]+)" is "(?P<state>[^"]+)"')
def pr_state(context, key, state):
    w(context).set_state(key, state)

use_step_matcher("parse")

@step('a team review is requested on "{key}"')
def team_review(context, key):
    w(context).request_team(key, "acme-team")

@step('the CI result on "{key}" becomes "{state}"')
def ci_becomes(context, key, state):
    w(context).set_ci(key, state)

use_step_matcher("re")

@step(r'"(?P<actor>[^"]+)" comments "(?P<body>[^"]+)" on "(?P<key>[^"]+)"')
def comment(context, actor, body, key):
    w(context).comment(key, actor, body)

use_step_matcher("parse")

@step('"{actor}" comments "{body}" on "{key}" at "{at}"')
def comment_at(context, actor, body, key, at):
    w(context).comment(key, actor, body, at)

use_step_matcher("re")

@step(r'the watcher has already observed "(?P<key>[^"]+)"')
def observed(context, key):
    w(context).observe([key])

use_step_matcher("parse")

@step('the watcher has already observed "{first}" and "{second}"')
def observed_two(context, first, second):
    w(context).observe([first, second])

@step('the watcher has already observed "{key}" with a failing CI result')
def observed_failing(context, key):
    world = w(context)
    world.set_ci(key, "failing")
    world.observe([key])

@step('no state file')
def no_state_file(context):
    world = w(context)
    if world.state_path.exists() and world.state_path.is_file():
        world.state_path.unlink()

@step('no configuration file exists')
def no_config(context):
    world = w(context)
    world.suppress_config_write = True
    if world.config_path.exists():
        world.config_path.unlink()

@step('the watch scope is limited to "{repo}"')
def scope_limited(context, repo):
    world = w(context)
    world.watch_mode = "selected"
    world.repositories = [repo]
    world.dirty = True

@given('the focus relations are "{relations}"')
def focus_relations(context, relations):
    world = w(context)
    world.focus = [item.strip() for item in relations.split(",") if item.strip()]
    world.dirty = True

@step('the state directory cannot be written')
def state_dir_readonly(context):
    world = w(context)
    world.write_config(force=True)
    os.chmod(world.tmp, 0o500)

@step('the state file version is {version:d}')
def state_version_step(context, version):
    assert w(context).state_version() == version

@step('the activity "{summary}" is unread')
def activity_unread(context, summary):
    world = w(context)
    record = world.state_activity(summary)
    if record is None:
        raise AssertionError("activity not found: " + summary)
    assert not record.get("read")

@step('the activity "{summary}" is read')
def activity_read(context, summary):
    world = w(context)
    record = world.state_activity(summary)
    if record is None:
        raise AssertionError("activity not found: " + summary)
    assert record.get("read")

@step('passuff runs one check')
def run_check(context):
    w(context).run(["check"])

@step('the developer asks for status')
def ask_status(context):
    w(context).run(["status"])

@step('the developer lists the inbox')
def list_inbox(context):
    w(context).run(["inbox"])

@step('the developer lists all activity')
def list_all(context):
    w(context).run(["inbox", "--all"])

@step('the developer marks all activity read')
def mark_all(context):
    w(context).run(["read", "--all"])

@step('the developer marks the newest activity read')
def mark_newest(context):
    world = w(context)
    records = world.load_state().get("activity", [])
    if records:
        world.run(["read", records[-1]["id"]])
    else:
        world.run(["read", "--all"])

@step('the transcript reports "{text}"')
def transcript_reports(context, text):
    _assert(context, text)

@step('the transcript does not report "{text}"')
def transcript_not_reports(context, text):
    _assert(context, text, False)

@step('the transcript reports no notifications')
def transcript_no_notifications(context):
    assert "no notifications" in transcript(context)

@step('the exit status is {code}')
def exit_status(context, code):
    world = w(context)
    actual = world.last.code if world.last else None
    assert actual == int(code), f"expected exit {code}, got {actual}\n{transcript(context)}"

@step('the developer opens the inbox TUI')
def open_tui(context):
    w(context).open_tui(["--snapshot"])

@step('the developer opens the inbox TUI with keys "{keys}"')
def open_tui_keys(context, keys):
    w(context).open_tui(["--snapshot", "--keys", keys])

@step('the developer presses "{key}"')
def press(context, key):
    w(context).press(key)

@step('the developer narrows the focus to "{relations}"')
def narrow_focus(context, relations):
    w(context).run_tui(["--snapshot", "--picker", "--focus-set", relations])

use_step_matcher("re")

@step(r'the TUI shows "(?P<text>[^"]+)"')
def tui_shows(context, text):
    _assert(context, text)

use_step_matcher("parse")

@step('the TUI does not show "{text}"')
def tui_not_shows(context, text):
    _assert(context, text, False)

@step('the TUI reports "{text}"')
def tui_reports(context, text):
    _assert(context, text)

@step('the desktop notification surface is unavailable')
def desktop_unavailable(context):
    w(context).desktop_unavailable = True

@step('GitHub reports stack {number:d} for "{key}" at position {position:d} of {size:d} with base "{base}"')
def stack_report(context, number, key, position, size, base):
    w(context).stack(key, number, position, size, base)

@step('the inbox shows "{summary}" for "{key}" as unread')
def inbox_record_unread(context, summary, key):
    record = w(context).state_activity(summary)
    assert record and not record.get("read"), record


@step('the selected row is gold and bold')
def selected_row_gold(context):
    assert_role(context, "▶", "gold_bold")


@step('the watched table headers are cyan-blue and bold')
def watched_headers_role(context):
    assert_role(context, "Pull request", "accent_bold")
    assert_role(context, "Title", "accent_bold")


@step('the activity table headers are cyan-blue and bold')
def activity_headers_role(context):
    assert_role(context, "When", "accent_bold")
    assert_role(context, "Event", "accent_bold")


@step('the focused pane title is green and bold')
def focused_title_role(context):
    assert_line_span_role_any(context, ("Activity", "Watched pull requests"), ("Activity", "Watched pull requests"), "green_bold")


@step('the metadata labels are cyan-blue')
def metadata_labels_role(context):
    assert_metadata_line_role(context, "accent")


@step('the metadata values are dim')
def metadata_values_role(context):
    assert_metadata_value_role(context)
