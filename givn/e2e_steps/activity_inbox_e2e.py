"""E2E bindings for activity-inbox."""

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


from givn.steps import activity_inbox_steps  # noqa: F401

from givn.steps import activity_inbox_steps  # noqa: F401

use_step_matcher("parse")

@step('the developer opens the inbox TUI observing the account')
def open_tui_observe(context):
    w(context).open_tui(["--snapshot", "--observe"])

@step('the browser opened "{url}"')
def browser_opened_then(context, url):
    assert any(url in line for line in w(context).browser_lines()), w(context).browser_lines()
