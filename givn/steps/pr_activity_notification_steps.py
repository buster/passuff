"""Step bindings for pr-activity-notification."""

from __future__ import annotations

import json
import os
import shlex
import stat
import time
from pathlib import Path

import tomlkit
from behave import given, step, then, use_step_matcher

from givn.steps.support import _assert, _config, _parse_key, transcript, w
from tests.support.pty_driver import run_pty


use_step_matcher("parse")

@step('a configured account "{label}" authenticating through gh as "{login}"')
def account_gh(context, label, login):
    w(context).account(label, login, auth="gh_oauth")

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

@step('the pull request "{key}" is reopened')
def pr_reopened(context, key):
    w(context).set_state(key, "open")

@step('the pull request "{key}" becomes "{state}" while the watcher is stopped')
def pr_becomes(context, key, state):
    w(context).set_state(key, state)

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

@step('the state file cannot be written')
def state_file_readonly(context):
    world = w(context)
    world.write_config(force=True)
    os.chmod(world.tmp, 0o500)

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

@step('the state file contains unique activity ids')
def unique_ids(context):
    ids = [item.get("id") for item in w(context).load_state().get("activity", [])]
    assert len(ids) == len(set(ids))

@step('the activity list contains one record for "{key}"')
def one_record(context, key):
    records = [item for item in w(context).load_state().get("activity", []) if item.get("pull_request") == key]
    assert len(records) == 1, records

@step('the transcript lists two activities')
def transcript_two(context):
    text = transcript(context)
    assert text.count("New comment") >= 2, text

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

@step('the latest transcript reports "{text}"')
def latest_reports(context, text):
    _assert(context, text)

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

@step('the sound is configured to "{path}"')
def sound_configured(context, path):
    world = w(context)
    world.notifications_config["sound_file"] = str(Path(path).resolve())
    world.dirty = True

@step('the sound player is unavailable')
def sound_unavailable(context):
    w(context).sound_missing = True

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

@step('the account has no authored pull requests')
def no_authored(context):
    assert not w(context).fake.prs

@step('the GitHub response repeats the pull request node for "{key}"')
def repeat_node(context, key):
    w(context).fake.repeat_node = True

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

@step('the pull request "{key}" becomes "{state}"')
def pr_becomes_state(context, key, state):
    w(context).set_state(key, state)

@step('a watched pull request "{key}" merged {days:d} days ago')
def watched_merged(context, key, days):
    w(context).watched_merged(key, days)

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
