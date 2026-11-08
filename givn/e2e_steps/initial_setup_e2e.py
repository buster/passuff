"""E2E bindings for initial-setup."""

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


from givn.steps import initial_setup_steps  # noqa: F401

from givn.steps import initial_setup_steps  # noqa: F401

use_step_matcher("parse")

@step('the configuration contains account "{label}"')
def config_account(context, label):
    assert label in _config(context).get("accounts", {})

@step('the configuration contains auth "{auth}"')
def config_auth(context, auth):
    accounts = _config(context).get("accounts", {})
    assert any(value.get("auth") == auth for value in accounts.values()), accounts

@step('the configuration is created')
def config_created(context):
    assert w(context).config_path.exists()
