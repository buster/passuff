"""Shared helpers for the step bindings."""

from __future__ import annotations

import json
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


def frame_roles(context):
    world = w(context)
    if getattr(world, "frame_roles", None) is None:
        world.capture_roles()
    return world.frame_roles or []


def _spans(context):
    return [span for line in frame_roles(context) for span in line]


def assert_role(context, text: str, role: str) -> None:
    spans = _spans(context)
    assert any(span.get("role") == role and text in str(span.get("text", "")) for span in spans), (text, role, spans)


def assert_any_role(context, role: str) -> None:
    spans = _spans(context)
    assert any(span.get("role") == role for span in spans), (role, spans)


def assert_role_member(context, role: str, texts: set[str]) -> None:
    spans = _spans(context)
    assert any(span.get("role") == role and str(span.get("text", "")).strip() in texts for span in spans), (role, texts, spans)


def assert_metadata_lines(context) -> None:
    lines = frame_roles(context)
    for line in lines:
        text = "".join(str(span.get("text", "")) for span in line)
        if "Status:" in text or "When:" in text:
            assert any(span.get("role") == "accent" for span in line), line
            assert any(span.get("role") == "dim" for span in line), line
            return
    raise AssertionError(("no metadata line", lines))


def saved_ui(context) -> dict:
    path = w(context).state_path.with_name("ui.json")
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else {}


def assert_role_digits(context, role: str) -> None:
    spans = _spans(context)
    assert any(span.get("role") == role and str(span.get("text", "")).strip().isdigit() for span in spans), (role, spans)


def _line_text(line) -> str:
    return "".join(str(span.get("text", "")) for span in line)


def assert_line_role(context, needle: str, role: str) -> None:
    lines = frame_roles(context)
    for line in lines:
        if needle in _line_text(line):
            assert any(span.get("role") == role for span in line), (needle, role, line)
            return
    raise AssertionError((needle, role, lines))


def assert_line_role_any(context, needles, role: str) -> None:
    lines = frame_roles(context)
    for line in lines:
        text = _line_text(line)
        if any(needle in text for needle in needles) and any(span.get("role") == role for span in line):
            return
    raise AssertionError((needles, role, lines))


def assert_role_contains(context, role: str, texts) -> None:
    spans = _spans(context)
    assert any(span.get("role") == role and any(text in str(span.get("text", "")) for text in texts) for span in spans), (role, texts, spans)


def assert_metadata_line_role(context, role: str) -> None:
    lines = frame_roles(context)
    for line in lines:
        text = _line_text(line)
        if "Status:" in text or "When:" in text:
            assert any(span.get("role") == role for span in line), (role, line)
            return
    raise AssertionError((role, lines))
