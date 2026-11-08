"""Step bindings for initial-setup."""

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

@step('a configuration file exists')
def config_exists(context):
    world = w(context)
    if not world.accounts:
        world.account("personal", "octo-personal")
    world.write_config(force=True)
    world.config_snapshot = world.config_path.read_text()

@step('the developer opens the inbox TUI without a terminal')
def open_tui_no_terminal(context):
    w(context).run_tui([])

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

@step('the configuration is not created')
def config_not_created(context):
    assert not w(context).config_path.exists()

@step('the configuration is unchanged')
def config_unchanged(context):
    world = w(context)
    assert world.config_path.read_text() == world.config_snapshot

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
