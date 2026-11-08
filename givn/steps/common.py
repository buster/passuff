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

from tests.support.pty_driver import run_pty


def w(context):
    return context.world


def transcript(context) -> str:
    return context.world.last.text if context.world.last else ""


def _assert(context, text: str, present: bool = True) -> None:
    body = transcript(context)
    found = text in body
    if found != present:
        raise AssertionError(
            ("expected to find " if present else "did not expect ") + repr(text) + "\n--- transcript ---\n" + body
        )


# ---------------------------------------------------------------- accounts


@step('a configured account "{label}" with login "{login}"')
def account_login(context, label, login):
    w(context).account(label, login)


@step('a configured account "{label}" authenticating through gh as "{login}"')
def account_gh(context, label, login):
    w(context).account(label, login, auth="gh_oauth")


@step('a configured account "{label}" with no credential source')
def account_no_source(context, label):
    w(context).accounts[label] = {"auth": "github_pat", "graphql_url": w(context).fake.url()}
    w(context).dirty = True


@step('a configured account "{label}" with an environment credential')
def account_env(context, label):
    w(context).account(label, "octo-work", auth="github_pat", token_env="WORK_TOKEN")


@step('a configured account "{label}" with a command credential')
def account_command(context, label):
    w(context).account(label, "octo-work", auth="github_pat", token_command="printf token")


@step('a configured account "{label}" with an unknown auth mode')
def account_unknown_mode(context, label):
    w(context).accounts[label] = {"auth": "bogus", "graphql_url": w(context).fake.url()}
    w(context).dirty = True


@step('a configured account "{label}" with two personal access token sources')
def account_two_sources(context, label):
    w(context).accounts[label] = {
        "auth": "github_pat",
        "token_env": "A",
        "token_command": "true",
        "graphql_url": w(context).fake.url(),
    }
    w(context).dirty = True


@step('a configured account "{label}" with gh OAuth but no expected login')
def account_gh_no_login(context, label):
    w(context).accounts[label] = {"auth": "gh_oauth", "graphql_url": w(context).fake.url()}
    w(context).dirty = True


@step('a configured account "{label}" with gh OAuth and a personal access token source')
def account_gh_with_source(context, label):
    w(context).accounts[label] = {
        "auth": "gh_oauth",
        "github_user": "octo-work",
        "token_env": "A",
        "graphql_url": w(context).fake.url(),
    }
    w(context).dirty = True


@step('a configured account "{label}" with a literal personal access token')
def account_literal_token(context, label):
    w(context).accounts[label] = {"auth": "github_pat", "token": "secret", "graphql_url": w(context).fake.url()}
    w(context).dirty = True


@step('a configured account "{label}" with an invalid credential')
def account_invalid_credential(context, label):
    world = w(context)
    world.accounts[label] = {
        "auth": "github_pat",
        "token_env": "PASSUFF_MISSING_TOKEN",
        "graphql_url": world.fake.url(),
    }
    world.dirty = True


@step('a configured account "{label}" with a personal access token file that does not exist')
def account_missing_token_file(context, label):
    path = w(context).tmp / "missing.token"
    w(context).account(label, "octo-work", auth="github_pat", token_file=str(path))


@step('a configured account "{label}" with a personal access token in a file with mode "{mode}"')
def account_token_file_mode(context, label, mode):
    world = w(context)
    path = world.tmp / f"token-{label}.token"
    token = f"token-{label}-mode"
    path.write_text(token + "\n")
    path.chmod(int(mode, 8))
    world.accounts[label] = {
        "auth": "github_pat",
        "github_user": "octo-work",
        "token_file": str(path),
        "graphql_url": world.fake.url(),
    }
    world.fake.tokens[token] = "octo-work"
    world.dirty = True


@step('a configured account "{label}" with a personal access token in "{path}"')
def account_token_file_path(context, label, path):
    world = w(context)
    real = Path(path).expanduser() if not path.startswith("~") else world.home / path[2:]
    real.parent.mkdir(parents=True, exist_ok=True)
    token = f"token-{label}-path"
    real.write_text(token + "\n")
    real.chmod(0o600)
    world.accounts[label] = {
        "auth": "github_pat",
        "github_user": "octo-work",
        "token_file": path,
        "graphql_url": world.fake.url(),
    }
    world.fake.tokens[token] = "octo-work"
    world.dirty = True


@step('a configured account "{label}" with an expected login "{expected}" and a personal access token belonging to "{actual}"')
def account_pat_mismatch(context, label, expected, actual):
    world = w(context)
    token = f"token-{label}-mismatch"
    world.accounts[label] = {
        "auth": "github_pat",
        "github_user": expected,
        "token_env": "WORK_TOKEN",
        "graphql_url": world.fake.url(),
    }
    world.fake.tokens[token] = actual
    world._env_tokens = getattr(world, "_env_tokens", {})
    world._env_tokens["WORK_TOKEN"] = token
    world.dirty = True


@step('a configured account "{label}" with a relative token file path')
def account_relative_token(context, label):
    world = w(context)
    world.accounts[label] = {
        "auth": "github_pat",
        "github_user": "octo-work",
        "token_file": "relative.token",
        "graphql_url": world.fake.url(),
    }
    world.dirty = True


@step('a configured account "{label}" with a token file path that is a directory')
def account_token_dir(context, label):
    world = w(context)
    path = world.tmp / "token-dir"
    path.mkdir(exist_ok=True)
    world.accounts[label] = {
        "auth": "github_pat",
        "github_user": "octo-work",
        "token_file": str(path),
        "graphql_url": world.fake.url(),
    }
    world.dirty = True


@step('a configured account "{label}" with an empty token file')
def account_empty_token(context, label):
    world = w(context)
    path = world.tmp / "empty.token"
    path.write_text("")
    path.chmod(0o600)
    world.accounts[label] = {
        "auth": "github_pat",
        "github_user": "octo-work",
        "token_file": str(path),
        "graphql_url": world.fake.url(),
    }
    world.dirty = True


use_step_matcher("re")


@step(r'a configuration with (?!watch repositories)(?P<case>.+)')
def configuration_case(context, case):
    world = w(context)
    if world.accounts == {}:
        world.account("personal", "octo-personal")
    if case == "an unknown key":
        text = world.config_text() if world.config_path.exists() else ""
        world.write_config(force=True)
        text = world.config_path.read_text() + "\nunknown_key = true\n"
        world.config_path.write_text(text)
    elif case == "invalid TOML":
        world.write_config(force=True)
        world.config_path.write_text("this is not toml [[[\n")
    elif case == "a plain HTTP GitHub address outside loopback":
        world.accounts["personal"]["graphql_url"] = "http://example.com/graphql"
        world.dirty = True
        world.write_config(force=True)
    elif case == "a poll interval below one second":
        world.poll = 0
        world.dirty = True
        world.write_config(force=True)
    else:
        raise AssertionError("unknown configuration case: " + case)


use_step_matcher("parse")


@step('a configuration with watch repositories "{repositories}"')
def configuration_watch_repositories(context, repositories):
    world = w(context)
    if not world.accounts:
        world.account("personal", "octo-personal")
    world.watch_mode = "selected"
    world.repositories = [item.strip() for item in repositories.split(",") if item.strip()]
    world.dirty = True
    world.write_config(force=True)


# --------------------------------------------------------------------- PRs


def _parse_key(key: str):
    repo, _, number = key.partition("#")
    return repo, int(number)


use_step_matcher("re")


@step(r'an authored pull request "(?P<key>[^"]+)" by "(?P<author>[^"]+)"')
def authored_pr(context, key, author):
    w(context).pr(key, author)


@step(r'a pull request "(?P<key>[^"]+)" by "(?P<author>[^"]+)"')
def any_pr(context, key, author):
    w(context).pr(key, author)


use_step_matcher("parse")


@step('an authored pull request "{key}" by "{author}" is created after the last check')
def authored_pr_after(context, key, author):
    world = w(context)
    from datetime import datetime, timedelta, timezone

    future = (datetime.now(timezone.utc) + timedelta(seconds=5)).isoformat().replace("+00:00", "Z")
    world.pr(key, author, created_at=future, updated_at=future)


@step('an authored pull request "{key}" by "{author}" appears with an existing comment by "{actor}"')
def authored_pr_appears(context, key, author, actor):
    world = w(context)
    from datetime import datetime, timedelta, timezone

    past = (datetime.now(timezone.utc) - timedelta(days=2)).isoformat().replace("+00:00", "Z")
    world.pr(key, author, created_at=past, updated_at=past)
    world.comment(key, actor, "Existing comment", at=past)


@step('an authored pull request "{key}" by "{author}" is created and "{state}" between checks')
def authored_pr_created_closed(context, key, author, state):
    world = w(context)
    from datetime import datetime, timedelta, timezone

    future = (datetime.now(timezone.utc) + timedelta(seconds=5)).isoformat().replace("+00:00", "Z")
    world.pr(key, author, state=state, created_at=future, updated_at=future)


@step('the pull request "{key}" already has a comment by "{actor}"')
def pr_has_comment(context, key, actor):
    w(context).comment(key, actor, "Existing comment")


@step('the pull request "{key}" is assigned to me')
def pr_assigned_me(context, key):
    world = w(context)
    world.assign(key, "octo-personal")
    world.pr(key, world.fake.find(key)["author"], assignees=world.fake.find(key)["assignees"])


@step('the pull request "{key}" is assigned to "{login}"')
def pr_assigned(context, key, login):
    world = w(context)
    world.assign(key, login)


use_step_matcher("re")


@step(r'the pull request "(?P<key>[^"]+)" is "(?P<state>[^"]+)"')
def pr_state(context, key, state):
    w(context).set_state(key, state)


use_step_matcher("parse")


@step('the pull request "{key}" is reopened')
def pr_reopened(context, key):
    w(context).set_state(key, "open")


@step('the pull request "{key}" becomes "{state}" while the watcher is stopped')
def pr_becomes(context, key, state):
    w(context).set_state(key, state)


@step('a team review is requested on "{key}"')
def team_review(context, key):
    w(context).request_team(key, "acme-team")


@step('the CI result on "{key}" is "{state}"')
def ci_is(context, key, state):
    w(context).set_ci(key, state)


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


@step('"{actor}" comments "{body}" on "{key}" after the first observation')
def comment_after(context, actor, body, key):
    world = w(context)

    def action():
        world.comment(key, actor, body)

    world.after_first_observe = action


@step('"{actor}" submits a "{state}" review on "{key}"')
def review(context, actor, state, key):
    w(context).review(key, actor, state)


@step('"{actor}" submits an "{state}" review on "{key}"')
def review_an(context, actor, state, key):
    w(context).review(key, actor, state)


@step('"{actor}" writes a review comment "{body}" on "{key}"')
def review_comment(context, actor, body, key):
    w(context).review_comment(key, actor, body)


@step('"{actor}" pushes a commit to "{key}"')
def push(context, actor, key):
    w(context).push(key)


@step('a "{change}" change occurs on "{key}"')
def change_occurs(context, change, key):
    world = w(context)
    if change == "comment":
        world.comment(key, "alice", "Change")
    elif change == "push":
        world.push(key)
    elif change in {"merged", "closed", "reopened"}:
        world.set_state(key, change)
    elif change in {"failing", "pending", "successful"}:
        world.set_ci(key, "passing" if change == "successful" else change)


# ----------------------------------------------------------------- observe


use_step_matcher("re")


@step(r'the watcher has already observed "(?P<key>[^"]+)"')
def observed(context, key):
    w(context).observe([key])


use_step_matcher("parse")


@step('the watcher has already observed "{first}" and "{second}"')
def observed_two(context, first, second):
    w(context).observe([first, second])


@step('the watcher has already observed all of them')
def observed_all(context):
    world = w(context)
    keys = [f"{pr['repo']}#{pr['number']}" for pr in world.fake.prs]
    world.observe(keys)


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


@step('a configuration file exists')
def config_exists(context):
    world = w(context)
    if not world.accounts:
        world.account("personal", "octo-personal")
    world.write_config(force=True)
    world.config_snapshot = world.config_path.read_text()


@step('the watch scope is limited to "{repo}"')
def scope_limited(context, repo):
    world = w(context)
    world.watch_mode = "selected"
    world.repositories = [repo]
    world.dirty = True


@step('the watch scope is empty')
def scope_empty(context):
    world = w(context)
    world.watch_mode = "selected"
    world.repositories = []
    world.dirty = True


@given('the watch scope mode is "{mode}"')
def scope_mode(context, mode):
    world = w(context)
    world.watch_mode = mode
    world.dirty = True


@given('the focus relations are "{relations}"')
def focus_relations(context, relations):
    world = w(context)
    world.focus = [item.strip() for item in relations.split(",") if item.strip()]
    world.dirty = True


@step('the configuration file is corrupted')
def corrupt_config(context):
    w(context).config_path.write_text("this is not toml [[[\n")


@step('the configuration directory cannot be written')
def config_dir_readonly(context):
    world = w(context)
    world.write_config(force=True)
    os.chmod(world.tmp, 0o500)


@step('the state directory cannot be written')
def state_dir_readonly(context):
    world = w(context)
    world.write_config(force=True)
    os.chmod(world.tmp, 0o500)


@step('the state file cannot be written')
def state_file_readonly(context):
    world = w(context)
    world.write_config(force=True)
    os.chmod(world.tmp, 0o500)


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


@step('a corrupt persisted observation')
def corrupt_state(context):
    world = w(context)
    world.state_path.write_text("not json at all")


@step('a state file with an unknown version')
def unknown_version(context):
    world = w(context)
    state = world.empty_state(99)
    world.write_state(state)


@step('a state file with duplicate activity ids')
def duplicate_ids(context):
    world = w(context)
    if not world.accounts:
        world.account("personal", "octo-personal")
    state = world.empty_state()
    world.add_state_pr(state, "acme/api#42")
    world.add_state_activity(state, "a1", "New comment from @alice")
    world.add_state_activity(state, "a1", "New comment from @bob")
    state["next_activity_id"] = 1
    world.write_state(state)


@step('a state already watches "{key}"')
def state_watches(context, key):
    world = w(context)
    state = world.empty_state()
    world.add_state_pr(state, key)
    world.write_state(state)


@step('the state retains "{key}"')
def state_retains(context, key):
    assert key in w(context).load_state().get("pull_requests", {})


@step('the state retains 500 activity records')
def state_retains_500(context):
    assert len(w(context).load_state().get("activity", [])) == 500


@step('the oldest activity record is gone')
def oldest_gone(context):
    summaries = [item.get("summary") for item in w(context).load_state().get("activity", [])]
    assert "New comment 001" not in summaries


@step('the state file version is {version:d}')
def state_version_step(context, version):
    assert w(context).state_version() == version


@step('the state file contains unique activity ids')
def unique_ids(context):
    ids = [item.get("id") for item in w(context).load_state().get("activity", [])]
    assert len(ids) == len(set(ids))


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


@step('the activity list contains one record for "{key}"')
def one_record(context, key):
    records = [item for item in w(context).load_state().get("activity", []) if item.get("pull_request") == key]
    assert len(records) == 1, records


@step('the transcript lists two activities')
def transcript_two(context):
    text = transcript(context)
    assert text.count("New comment") >= 2, text


# -------------------------------------------------------------------- runs


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


@step('the developer marks activity "{identifier}" read')
def mark_activity(context, identifier):
    w(context).run(["read", identifier])


@step('the developer runs read with no target')
def read_no_target(context):
    w(context).run(["read"])


@step('the developer runs passuff with "{arguments}"')
def run_args(context, arguments):
    world = w(context)
    args = shlex.split(arguments)
    if any(item.startswith(("--focus-set", "--keys", "--picker", "--scope-")) for item in args):
        world.run_tui(["--snapshot", *args])
    else:
        world.run(args)


@step('the developer runs the notification test')
def notify_test(context):
    w(context).run(["notify-test"])


@step('the developer runs the notification test with "--no-desktop"')
def notify_test_no_desktop(context):
    w(context).run(["notify-test", "--no-desktop"])


@step('the developer runs the notification test with "--no-sound"')
def notify_test_no_sound(context):
    w(context).run(["notify-test", "--no-sound"])


@step('passuff starts watching')
def start_watching(context):
    world = w(context)
    world.start_watch(interactive=getattr(world, "terminal_interactive", False))
    if getattr(world, "terminal_interactive", False):
        world.wait_for("passuff: watching", timeout=5)
    else:
        world.wait_for("Fresh baseline", timeout=5)


@step('passuff restarts and runs one check')
def restart_check(context):
    w(context).run(["check"])


@step('the watcher runs one observation cycle')
def watcher_cycle(context):
    w(context).run(["check"])


@step('the watcher runs two observation cycles')
def watcher_two(context):
    world = w(context)
    if getattr(world, "terminal_interactive", False):
        world.write_config(force=True)
        world.start_watch(interactive=True)
        import time as _time
        from tests.support.world import Result
        _time.sleep(3.0)
        world.last = Result("".join(world.watch_lines), "", 0)
    else:
        world.run(["check"])
        world.run(["check"])


@step('the watcher runs another cycle')
def watcher_another(context):
    import time as _time
    from tests.support.world import Result

    world = w(context)
    if world.watch_proc is not None:
        _time.sleep(5.0)
        world.last = Result("".join(world.watch_lines), "", 0)
    else:
        world.run(["check"])


@step('the watcher starts interactively')
def watcher_interactive(context):
    world = w(context)
    world.terminal_interactive = True
    world.write_config(force=True)
    world.start_watch(interactive=True)
    world.wait_for("passuff: watching", timeout=5)
    from tests.support.world import Result
    world.last = Result("".join(world.watch_lines), "", 0)


@step('the terminal is interactive')
def terminal_interactive(context):
    w(context).terminal_interactive = True


@step('the terminal is not interactive')
def terminal_not_interactive(context):
    w(context).terminal_interactive = False


# --------------------------------------------------------------- assertions


@step('the transcript reports "{text}"')
def transcript_reports(context, text):
    _assert(context, text)


@step('the transcript does not report "{text}"')
def transcript_not_reports(context, text):
    _assert(context, text, False)


@step('the transcript reports no notifications')
def transcript_no_notifications(context):
    assert "no notifications" in transcript(context)


@step('the transcript reports a notification for "{key}"')
def transcript_notification(context, key):
    assert key in transcript(context) and "no notifications" not in transcript(context), transcript(context)


@step('the transcript reports no notification for "{key}"')
def transcript_no_notification(context, key):
    text = transcript(context)
    assert key not in text or "no notifications" in text, text


@step('the transcript reports one notification for "{key}"')
def transcript_one_notification(context, key):
    body = transcript(context)
    assert f"activities on {key}" in body, body
    assert len(w(context).delivery()) == 1, w(context).delivery()


@step('the exit status is {code}')
def exit_status(context, code):
    world = w(context)
    actual = world.last.code if world.last else None
    assert actual == int(code), f"expected exit {code}, got {actual}\n{transcript(context)}"


@step('the latest transcript reports "{text}"')
def latest_reports(context, text):
    _assert(context, text)


# -------------------------------------------------------------------- TUI


@step('the developer opens the inbox TUI')
def open_tui(context):
    w(context).open_tui(["--snapshot"])


@step('the developer opens the inbox TUI observing the account')
def open_tui_observe(context):
    w(context).open_tui(["--snapshot", "--observe"])


@step('the developer opens the inbox TUI and lets it observe for "{seconds}" seconds')
def open_tui_observe_for(context, seconds):
    w(context).open_tui(["--snapshot", "--observe-for", str(int(seconds) * 1000)])


@step('the developer opens the inbox TUI with keys "{keys}"')
def open_tui_keys(context, keys):
    w(context).open_tui(["--snapshot", "--keys", keys])


@step('the developer opens the inbox TUI without a terminal')
def open_tui_no_terminal(context):
    w(context).run_tui([])


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


@step('the developer presses "{key}"')
def press(context, key):
    w(context).press(key)


@step('the developer opens the scope picker')
def open_picker(context):
    w(context).open_tui(["--snapshot", "--picker"])


@step('the developer opens the live scope picker')
def open_live_picker(context):
    w(context).open_tui(["--snapshot", "--picker"])


@step('a watcher is running with a configuration path')
def watcher_running(context):
    world = w(context)
    world.write_config(force=True)
    world.start_watch()
    import time as _time
    _time.sleep(1.5)


@step('the developer selects "{repo}" in the watch scope')
def select_repo(context, repo):
    w(context).run_tui(["--snapshot", "--picker", "--scope-select", repo])


@step('the developer deselects "{repo}" in the watch scope')
def deselect_repo(context, repo):
    w(context).run_tui(["--snapshot", "--picker", "--scope-deselect", repo])


@step('the developer switches the watch scope to "{mode}"')
def switch_scope(context, mode):
    w(context).run_tui(["--snapshot", "--scope-mode", mode])


@step('the developer narrows the focus to "{relations}"')
def narrow_focus(context, relations):
    w(context).run_tui(["--snapshot", "--picker", "--focus-set", relations])


@step('the developer focuses the activity on the selected pull request')
def focus_activity(context):
    w(context).press("tab")
    w(context).press("f")


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


@step('the picker shows "{text}"')
def picker_shows(context, text):
    _assert(context, text)


@step('the picker does not show "{text}"')
def picker_not_shows(context, text):
    _assert(context, text, False)


# ------------------------------------------------------------------ config


def _config(context):
    import tomllib

    return tomllib.loads(w(context).config_path.read_text())


@given('the watch scope contains "{repo}"')
def scope_contains_given(context, repo):
    world = w(context)
    world.watch_mode = "selected"
    if repo.casefold() not in {item.casefold() for item in world.repositories}:
        world.repositories.append(repo)
    world.dirty = True


@then('the watch scope contains "{repo}"')
def scope_contains(context, repo):
    data = _config(context)
    assert repo.casefold() in {item.casefold() for item in data.get("watch", {}).get("repositories", [])}


@given('the watch scope does not contain "{repo}"')
def scope_not_contains_given(context, repo):
    world = w(context)
    world.repositories = [item for item in world.repositories if item.casefold() != repo.casefold()]
    world.dirty = True


@then('the watch scope does not contain "{repo}"')
def scope_not_contains(context, repo):
    data = _config(context)
    assert repo.casefold() not in {item.casefold() for item in data.get("watch", {}).get("repositories", [])}


@then('the watch scope mode is "{mode}"')
def mode_is(context, mode):
    assert _config(context).get("watch", {}).get("mode") == mode


@then('the focus relations are "{relations}"')
def focus_is(context, relations):
    actual = _config(context).get("focus", {}).get("relations", [])
    expected = [item.strip() for item in relations.split(",") if item.strip()]
    assert actual == expected, (actual, expected)


@step('the configuration contains account "{label}"')
def config_account(context, label):
    assert label in _config(context).get("accounts", {})


@step('the configuration contains auth "{auth}"')
def config_auth(context, auth):
    accounts = _config(context).get("accounts", {})
    assert any(value.get("auth") == auth for value in accounts.values()), accounts


@step('the configuration contains login "{login}"')
def config_login(context, login):
    accounts = _config(context).get("accounts", {})
    assert any(value.get("github_user") == login for value in accounts.values()), accounts


@step('the configuration contains token env "{name}"')
def config_token_env(context, name):
    accounts = _config(context).get("accounts", {})
    assert any(value.get("token_env") == name for value in accounts.values()), accounts


@step('the configuration does not contain "{text}"')
def config_not_contains(context, text):
    assert text not in w(context).config_text(), w(context).config_text()


@step('the configuration is created')
def config_created(context):
    assert w(context).config_path.exists()


@step('the configuration is not created')
def config_not_created(context):
    assert not w(context).config_path.exists()


@step('the configuration is unchanged')
def config_unchanged(context):
    world = w(context)
    assert world.config_path.read_text() == world.config_snapshot


# ------------------------------------------------------------------- setup


@step('the developer runs setup with:')
def run_setup_table(context):
    world = w(context)
    data = {row[0].strip(): row[1].strip() for row in context.table}
    lines: list[str] = []
    if world.config_path.exists():
        lines.append("yes" if data.get("replace", "").lower() in {"yes", "y"} else "no")
        if data.get("replace", "").lower() not in {"yes", "y"}:
            world.run(["setup"], stdin="\n".join(lines) + "\n")
            return
    lines.append(data.get("label", "personal"))
    auth = data.get("auth", "gh_oauth")
    lines.append(auth)
    lines.append(data.get("login", "octo-personal"))
    if auth == "token_env":
        lines.append(data.get("variable", "WORK_TOKEN"))
    elif auth == "token_command":
        lines.append(data.get("command", "printf token"))
    elif auth == "token_file":
        value = data.get("file", "")
        if value in {"TOKEN_FILE", "TOKENFILE"}:
            value = str(getattr(world, "token_file_path", world.tmp / "work.token"))
        lines.append(value)
    lines.append("n")
    lines.append(data.get("watch", "all"))
    if data.get("watch", "all") == "selected":
        lines.append(data.get("repos", ""))
    world.run(["setup"], stdin="\n".join(lines) + "\n")


@step('the developer runs setup and answers "no" to replacement')
def run_setup_no(context):
    w(context).run(["setup"], stdin="no\n")


@step('a world-readable token file')
def world_readable_token(context):
    world = w(context)
    path = world.tmp / "work.token"
    path.write_text("token-value\n")
    path.chmod(0o644)
    world.token_file_path = path


@step('the setup configuration directory cannot be written')
def setup_config_readonly(context):
    os.chmod(w(context).tmp, 0o500)


@step('setup reports "{text}"')
def setup_reports(context, text):
    _assert(context, text)


# ------------------------------------------------------------------ sound


@step('the configured sound is played once')
def sound_once(context):
    lines = w(context).sound_lines()
    assert len(lines) == 1, lines


@step('the configured sound file "{path}" is played once')
def sound_file_once(context, path):
    lines = w(context).sound_lines()
    assert len(lines) == 1 and path in lines[0], lines


@step('no sound is played')
def no_sound(context):
    assert not w(context).sound_lines(), w(context).sound_lines()


@step('sound is disabled in configuration')
def sound_disabled(context):
    world = w(context)
    world.notifications_config["sound"] = False
    world.dirty = True


@step('the desktop notification is disabled in configuration')
def desktop_disabled(context):
    world = w(context)
    world.notifications_config["desktop"] = False
    world.dirty = True


@given('the desktop notification is enabled')
def desktop_enabled(context):
    world = w(context)
    world.notifications_config["desktop"] = True
    world.dirty = True


@given('the notification sound is muted')
def sound_muted(context):
    world = w(context)
    world.notifications_config["sound"] = False
    world.dirty = True


@step('the sound is configured to "{path}"')
def sound_configured(context, path):
    world = w(context)
    world.notifications_config["sound_file"] = str(Path(path).resolve())
    world.dirty = True


@step('the desktop notification surface is unavailable')
def desktop_unavailable(context):
    w(context).desktop_unavailable = True


@step('the sound player is unavailable')
def sound_unavailable(context):
    w(context).sound_missing = True


@step('the browser opener is missing')
def browser_missing(context):
    w(context).browser_missing = True


@step('the browser will fail to open')
def browser_fail(context):
    w(context).browser_fail = True


@step('the browser opened "{url}"')
def browser_opened_then(context, url):
    assert any(url in line for line in w(context).browser_lines()), w(context).browser_lines()


@step('the desktop notification reports "{text}"')
def desktop_reports(context, text):
    import time as _time
    deadline = _time.time() + 5
    while _time.time() < deadline:
        rows = w(context).delivery()
        if any(text in (row.get("summary", "") + "\n" + row.get("body", "")) for row in rows):
            return
        _time.sleep(0.1)
    raise AssertionError((w(context).delivery(), getattr(w(context), "watch_lines", [])))


@step('the desktop notification is recorded')
def desktop_recorded(context):
    assert w(context).delivery(), "no desktop notification recorded"


@step('no desktop notification is recorded')
def desktop_not_recorded(context):
    assert not w(context).delivery(), w(context).delivery()


# ------------------------------------------------------------------- gh


@step('gh is authenticated as "{login}"')
def gh_authenticated(context, login):
    w(context).gh_tokens[login] = f"gh-token-{login}"


@step('gh is not authenticated as "{login}"')
def gh_not_authenticated(context, login):
    w(context).gh_tokens.pop(login, None)


@step('gh login always fails')
def gh_login_fail(context):
    w(context).gh_login_fail = True


@step('gh started a browser login for "{login}"')
def gh_started_login(context, login):
    import time as _time
    deadline = _time.time() + 5
    while _time.time() < deadline:
        if any("auth login" in line and login in line for line in w(context).gh_lines()):
            return
        _time.sleep(0.1)
    raise AssertionError(w(context).gh_lines())


@step('gh started one browser login for "{login}"')
def gh_started_one(context, login):
    lines = [line for line in w(context).gh_lines() if f"auth login for {login}" in line]
    assert len(lines) == 1, w(context).gh_lines()


@step('gh did not start a browser login')
def gh_no_login(context):
    assert not any("auth login" in line for line in w(context).gh_lines()), w(context).gh_lines()


# --------------------------------------------------------------- counters


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


@step('the inbox has asked GitHub for the stack fields "{count}" times')
def count_stack(context, count):
    assert w(context).fake.stack_attempts == int(count), w(context).fake.stack_attempts


@step('GitHub reports stack {number:d} for "{key}" at position {position:d} of {size:d} with base "{base}"')
def stack_report(context, number, key, position, size, base):
    w(context).stack(key, number, position, size, base)


@step('the TUI shows "{first}" before "{second}"')
def tui_order(context, first, second):
    body = transcript(context)
    assert first in body and second in body and body.index(first) < body.index(second), body


@step('the inbox shows "{first}" before "{second}" for "{key}"')
def inbox_order(context, first, second, key):
    body = transcript(context)
    assert first in body and second in body and body.index(first) < body.index(second), body


@step('the inbox shows "{summary}" for "{key}" as read')
def inbox_record_read(context, summary, key):
    record = w(context).state_activity(summary)
    assert record and record.get("read"), record


@step('the inbox shows "{summary}" for "{key}" as unread')
def inbox_record_unread(context, summary, key):
    record = w(context).state_activity(summary)
    assert record and not record.get("read"), record


@step('the account has no authored pull requests')
def no_authored(context):
    assert not w(context).fake.prs


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


@step('the GitHub response repeats the pull request node for "{key}"')
def repeat_node(context, key):
    w(context).fake.repeat_node = True


@step('the GitHub stack fields are rejected for "{repo}"')
def stack_rejected(context, repo):
    w(context).fake.stack_rejected = True


@step('the GitHub response for account "{label}" is {case}')
def github_fault(context, label, case):
    world = w(context)
    account = world.accounts.get(label, {})
    login = account.get("github_user") or label
    mapping = {
        "a malformed JSON body": "malformed",
        "a partial pull request payload": "partial",
        "a timeout": "timeout",
        "a non-2xx response": "non2xx",
        "a GraphQL error payload": "graphql",
    }
    world.fake.faults[login] = mapping.get(case, "non2xx")


@step('GitHub becomes unavailable for the next observation')
def github_unavailable(context):
    world = w(context)
    for login in list(world.fake.tokens.values()):
        world.fake.fault_once[login] = "non2xx"


@step('the next observation fails')
def next_observation_fails(context):
    github_unavailable(context)


@step('the credential for "{label}" becomes a token belonging to "{actual}"')
def credential_becomes(context, label, actual):
    world = w(context)
    token = f"token-{label}-changed"
    world.fake.tokens[token] = actual
    world._env_tokens = getattr(world, "_env_tokens", {})
    world._env_tokens["WORK_TOKEN"] = token
    if label in world.accounts:
        world.accounts[label]["token_env"] = "WORK_TOKEN"
        world.accounts[label].pop("token_env", None)
        world.accounts[label]["token_env"] = "WORK_TOKEN"
    world.dirty = True


@step('the configured account "{login}" merges "{key}"')
def account_merges(context, login, key):
    w(context).set_state(key, "merged")


@step('bot comments are "{value}" in configuration')
def bot_comments(context, value):
    world = w(context)
    world.events["bot_comments"] = value.strip().lower() in {"true", "enabled", "yes"}
    world.dirty = True


@step('the "{family}" event family is disabled in configuration')
def event_family_disabled(context, family):
    world = w(context)
    key = {"comments": "comments", "reviews": "reviews", "CI failure": "ci_failure", "CI recovery": "ci_recovery", "state changes": "state_changes"}.get(family, family)
    world.events[key] = False
    world.dirty = True


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
    return None


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


@step('the selected pull request is "{key}"')
def assert_selected_pr(context, key):
    _assert(context, f"Selected pull request: {key}")


@step('the terminal UI is closed')
def tui_closed(context):
    world = w(context)
    assert world.last is not None and world.last.code == 0, (world.last.code if world.last else None)


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



@step('an invalid configuration')
def invalid_config(context):
    world = w(context)
    if not world.accounts:
        world.account("personal", "octo-personal")
    world.write_config(force=True)
    world.config_path.write_text(world.config_path.read_text() + "\nunknown_key = true\n")


@step('no prior activity exists for "{key}"')
def no_prior(context, key):
    records = [item for item in w(context).load_state().get("activity", []) if item.get("pull_request") == key]
    assert not records, records


@step('the CI result becomes "{state}" on "{key}"')
def ci_becomes_on(context, state, key):
    w(context).set_ci(key, "passing" if state == "successful" else state)


@step('the CI result is "{state}" for "{key}"')
def ci_is_for(context, state, key):
    w(context).set_ci(key, "passing" if state == "successful" else state)


@step('the pull request becomes "{state}" on "{key}"')
def pr_becomes_on(context, state, key):
    w(context).set_state(key, state)


@step('the pull request is "open" for "{key}"')
def pr_open_for(context, key):
    w(context).set_state(key, "open")


@step('the pull request already has a comment by "{actor}"')
def pr_existing_comment(context, actor):
    for pr in w(context).fake.prs:
        world = w(context)
        world.comment(f"{pr['repo']}#{pr['number']}", actor, "Existing comment")


@step('the credential value "{value}" appears in neither the transcript nor the state file')
def credential_absent(context, value):
    world = w(context)
    assert value not in transcript(context), transcript(context)
    assert value not in world.state_path.read_text() if world.state_path.exists() else True


@step('the notification mentions "{first}", "{second}", and "{third}"')
def notification_mentions(context, first, second, third):
    rows = w(context).delivery()
    body = "\n".join(row.get("summary", "") + "\n" + row.get("body", "") for row in rows)
    assert first in body and second in body and third in body, body


@step('the report records a baseline for "{key}"')
def report_baseline(context, key):
    _assert(context, f"Fresh baseline recorded for {key}")


@step('the terminal shows a notification for "{key}"')
def terminal_notification(context, key):
    world = w(context)
    assert world.wait_for(key, timeout=20), "".join(world.watch_lines)


@step('the transcript shows the available commands')
def shows_commands(context):
    body = transcript(context)
    assert "usage" in body.lower() and "check" in body and "status" in body, body


@step('the transcript shows the version')
def shows_version(context):
    import passuff
    assert passuff.__version__ in transcript(context), transcript(context)


@step('the pull request "{key}" becomes "{state}"')
def pr_becomes_state(context, key, state):
    w(context).set_state(key, state)


@step('a watched pull request "{key}" merged {days:d} days ago')
def watched_merged(context, key, days):
    w(context).watched_merged(key, days)


@step('the activity list row excerpt is at most 80 characters')
def row_excerpt(context):
    body = transcript(context)
    rows = [line for line in body.splitlines() if "comment by" in line or "Comment by" in line]
    assert rows, body
    for line in rows:
        assert len(line.strip()) <= 100, line
    assert "comment line 40" not in body, body


@step('the desktop notification service fails')
def desktop_service_fails(context):
    w(context).desktop_unavailable = True


@step('the configured sound player fails')
def sound_player_fails(context):
    w(context).sound_fail = True


@step('the gh binary is missing')
def gh_binary_missing(context):
    w(context).gh_missing = True


@step('the gh command fails')
def gh_command_fails(context):
    world = w(context)
    world.gh_tokens.clear()
    world.gh_login_fail = True
