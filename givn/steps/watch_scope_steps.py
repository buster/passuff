"""Step bindings for watch-scope."""

from __future__ import annotations

import json
import os
import shlex
import stat
import time
from pathlib import Path

import tomlkit
from behave import given, step, then, use_step_matcher

from givn.steps.support import _assert, _config, _parse_key, assert_role, assert_role_member, transcript, w
from tests.support.pty_driver import run_pty


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

@step('the configuration file is corrupted')
def corrupt_config(context):
    w(context).config_path.write_text("this is not toml [[[\n")

@step('the configuration directory cannot be written')
def config_dir_readonly(context):
    world = w(context)
    world.write_config(force=True)
    os.chmod(world.tmp, 0o500)

@step('the state retains "{key}"')
def state_retains(context, key):
    assert key in w(context).load_state().get("pull_requests", {})

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

@step('the developer opens the scope picker')
def open_picker(context):
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

@step('the picker shows "{text}"')
def picker_shows(context, text):
    _assert(context, text)

@step('the picker does not show "{text}"')
def picker_not_shows(context, text):
    _assert(context, text, False)

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


@step('the picker border is dim')
def picker_border_role(context):
    assert_role(context, "┌", "dim")


@step('the picker brand is bright and bold')
def picker_brand_role(context):
    assert_role(context, "passuff", "bright_bold")


@step('the picker title suffix is dim')
def picker_suffix_role(context):
    assert_role(context, "— Watch scope", "dim")


@step('the picker mode is cyan-blue and bold')
def picker_mode_role(context):
    assert_role_member(context, "accent_bold", {"all", "selected"})


@step('the selected repository is gold')
def picker_selected_role(context):
    assert_role(context, "[x]", "gold_bold")
