"""E2E bindings for inbox-navigation."""

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


from givn.steps import inbox_navigation_steps  # noqa: F401

from givn.steps import inbox_navigation_steps  # noqa: F401

use_step_matcher("parse")

@step('"{actor}" comments "{body}" on "{key}" after the first observation')
def comment_after(context, actor, body, key):
    world = w(context)

    def action():
        world.comment(key, actor, body)

    world.after_first_observe = action

@step('the developer opens the inbox TUI and lets it observe for "{seconds}" seconds')
def open_tui_observe_for(context, seconds):
    w(context).open_tui(["--snapshot", "--observe-for", str(int(seconds) * 1000)])
