"""Behave hooks: one private bus, one fake world per scenario."""

from __future__ import annotations

import json
import os
import signal
import subprocess
import time
from pathlib import Path

from .world import REPO, World

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
        if "e2e" in scenario.tags and (world.last is not None or world.watch_lines):
            import re

            change_id = os.environ.get("GIVN_CHANGE_ID")
            if not change_id:
                roots = [item for item in (REPO / "givn" / "changes").glob("*") if (item / "specs").is_dir()]
                change_id = roots[0].name if len(roots) == 1 else None
            if change_id:
                slug = re.sub(r"[^a-z0-9]+", "-", scenario.name.lower()).strip("-")
                path = REPO / "givn" / "changes" / change_id / "evidence" / "visual" / slug
                path.mkdir(parents=True, exist_ok=True)
                if world.tui_args is not None and world.last is not None:
                    world.capture_ansi()
                text = world.last.text if world.last is not None else "".join(world.watch_lines)
                (path / "transcript.txt").write_text(text, encoding="utf-8")
                if world.last_ansi is not None:
                    (path / "transcript.ansi").write_text(world.last_ansi, encoding="utf-8")
                if world.frame_roles is not None:
                    (path / "roles.json").write_text(json.dumps(world.frame_roles, indent=2), encoding="utf-8")
        world.cleanup()
    time.sleep(0.1)


def before_step(context, step) -> None:
    pass


def after_step(context, step) -> None:
    pass
