"""Shared helpers for the step bindings."""

from __future__ import annotations

import tomllib


def w(context):
    return context.world


def transcript(context) -> str:
    return context.world.last.text if context.world.last else ""


def _assert(context, text: str, present: bool = True) -> None:
    body = transcript(context)
    found = text in body
    if found != present:
        raise AssertionError(
            ("expected to find " if present else "did not expect ") + repr(text) + "\n--- transcript ---\n" + body
        )


def _parse_key(key: str):
    repo, _, number = key.partition("#")
    return repo, int(number)


def _config(context):
    return tomllib.loads(w(context).config_path.read_text())
