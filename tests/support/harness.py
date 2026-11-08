"""Behave hooks: one private bus, one fake world per scenario."""

from __future__ import annotations

import os
import signal
import subprocess
import time

from .world import World

_DBUS_PID = None


def before_all(context) -> None:
    global _DBUS_PID
    try:
        output = subprocess.check_output(
            ["dbus-daemon", "--session", "--fork", "--print-address=1", "--print-pid=1"],
            text=True,
        ).splitlines()
        if len(output) >= 2:
            os.environ["DBUS_SESSION_BUS_ADDRESS"] = output[0].strip()
            _DBUS_PID = int(output[1].strip())
    except (OSError, subprocess.SubprocessError, ValueError):
        _DBUS_PID = None


def after_all(context) -> None:
    global _DBUS_PID
    if _DBUS_PID:
        try:
            os.kill(_DBUS_PID, signal.SIGTERM)
        except OSError:
            pass
    _DBUS_PID = None


def before_scenario(context, scenario) -> None:
    context.world = World(e2e="e2e" in scenario.tags)
    context.e2e = "e2e" in scenario.tags


def after_scenario(context, scenario) -> None:
    world = getattr(context, "world", None)
    if world:
        if "e2e" in scenario.tags and world.last is not None:
            import re
            from pathlib import Path

            from .world import REPO

            slug = re.sub(r"[^a-z0-9]+", "-", scenario.name.lower()).strip("-")
            path = REPO / "givn" / "changes" / "python-rebuild" / "evidence" / "visual" / slug / "transcript.txt"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(world.last.text, encoding="utf-8")
        world.cleanup()
    time.sleep(0.1)


def before_step(context, step) -> None:
    pass


def after_step(context, step) -> None:
    pass
