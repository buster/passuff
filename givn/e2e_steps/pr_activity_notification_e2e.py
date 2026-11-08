"""E2E bindings for pr-activity-notification."""

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


from givn.steps import pr_activity_notification_steps  # noqa: F401

from givn.steps import pr_activity_notification_steps  # noqa: F401

use_step_matcher("parse")

@step('passuff starts watching')
def start_watching(context):
    world = w(context)
    world.start_watch(interactive=getattr(world, "terminal_interactive", False))
    if getattr(world, "terminal_interactive", False):
        world.wait_for("passuff: watching", timeout=5)
    else:
        world.wait_for("Fresh baseline", timeout=5)

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
