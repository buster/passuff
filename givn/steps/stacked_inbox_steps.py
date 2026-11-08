"""Step bindings for stacked-inbox."""

from __future__ import annotations

import json
import os
import shlex
import stat
import time
from pathlib import Path

import tomlkit
from behave import given, step, then, use_step_matcher

from givn.steps.support import _assert, _config, _parse_key, assert_role, transcript, w
from tests.support.pty_driver import run_pty


use_step_matcher("parse")

@step('the inbox has asked GitHub for the stack fields "{count}" times')
def count_stack(context, count):
    assert w(context).fake.stack_attempts == int(count), w(context).fake.stack_attempts

@step('the TUI shows "{first}" before "{second}"')
def tui_order(context, first, second):
    body = transcript(context)
    assert first in body and second in body and body.index(first) < body.index(second), body

@step('the GitHub stack fields are rejected for "{repo}"')
def stack_rejected(context, repo):
    w(context).fake.stack_rejected = True


@step('the stack column header is cyan-blue and bold')
def stack_header_role(context):
    assert_role(context, "☰", "accent_bold")


@step('the stack positions are bright')
def stack_positions_role(context):
    assert_role(context, "2/2", "bright")
