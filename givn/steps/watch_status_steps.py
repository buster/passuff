"""Step bindings for watch-status."""

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

@step('the CI result on "{key}" is "{state}"')
def ci_is(context, key, state):
    w(context).set_ci(key, state)
