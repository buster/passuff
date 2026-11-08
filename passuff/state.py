"""Versioned JSON watch state and UI preferences."""

from __future__ import annotations

import json
import os
import shutil
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

STATE_VERSION = 2


class StateError(Exception):
    """A state failure exits 3."""


def empty_state() -> dict[str, Any]:
    return {
        "version": STATE_VERSION,
        "accounts": {},
        "pull_requests": {},
        "activity": [],
        "next_activity_id": 1,
        "last_scope": None,
    }


def _now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def _quarantine(path: Path) -> None:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    target = path.with_name(path.name + f".corrupt-{stamp}")
    try:
        os.replace(path, target)
    except OSError as exc:
        raise StateError(f"state quarantine failed: {exc}") from exc


def _migrate_v1(data: dict[str, Any]) -> dict[str, Any]:
    state = empty_state()
    if isinstance(data.get("accounts"), dict):
        state["accounts"] = data["accounts"]
    if isinstance(data.get("pull_requests"), dict):
        state["pull_requests"] = data["pull_requests"]
    state["last_scope"] = data.get("last_scope")
    return state


def normalize(state: dict[str, Any]) -> dict[str, Any]:
    state.setdefault("accounts", {})
    state.setdefault("pull_requests", {})
    state.setdefault("activity", [])
    state.setdefault("last_scope", None)
    maximum = 0
    seen: set[str] = set()
    for record in state["activity"]:
        raw = str(record.get("id", "a0"))
        try:
            maximum = max(maximum, int(raw.lstrip("a")))
        except ValueError:
            pass
        if raw in seen:
            maximum += 1
            record["id"] = activity_id(maximum)
            raw = record["id"]
        seen.add(raw)
    state["next_activity_id"] = max(int(state.get("next_activity_id", 1)), maximum + 1)
    return state


def load_state(path: Path) -> dict[str, Any]:
    if not path.exists():
        return empty_state()
    try:
        text = path.read_text(encoding="utf-8")
        data = json.loads(text)
    except OSError as exc:
        raise StateError(f"state cannot be read: {exc}") from exc
    except json.JSONDecodeError:
        _quarantine(path)
        return empty_state()
    if not isinstance(data, dict):
        _quarantine(path)
        return empty_state()
    version = data.get("version")
    if version == 1:
        migrated = normalize(_migrate_v1(data))
        save_state(path, migrated)
        return migrated
    if version != STATE_VERSION:
        _quarantine(path)
        return empty_state()
    before = [record.get("id") for record in data.get("activity", [])]
    data = normalize(data)
    after = [record.get("id") for record in data.get("activity", [])]
    if before != after:
        save_state(path, data)
    return data


def save_state(path: Path, state: dict[str, Any]) -> None:
    if not path.parent.exists():
        try:
            path.parent.mkdir(parents=True, exist_ok=True)
        except OSError as exc:
            raise StateError(f"state directory cannot be created: {exc}") from exc
    tmp = path.with_name(path.name + ".tmp")
    try:
        with open(tmp, "w", encoding="utf-8") as handle:
            json.dump(state, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(tmp, path)
    except OSError as exc:
        try:
            if tmp.exists():
                tmp.unlink()
        except OSError:
            pass
        raise StateError(f"state cannot be written: {exc}") from exc


def activity_id(number: int) -> str:
    return f"a{number}"


def append_activity(state: dict[str, Any], record: dict[str, Any]) -> dict[str, Any]:
    number = int(state.get("next_activity_id", 1))
    record = dict(record)
    record["id"] = activity_id(number)
    record.setdefault("read", False)
    record.setdefault("observed_at", _now())
    state["activity"].append(record)
    state["next_activity_id"] = number + 1
    if len(state["activity"]) > 500:
        state["activity"] = state["activity"][-500:]
    return record


def find_activity(state: dict[str, Any], identifier: str) -> dict[str, Any] | None:
    for record in state.get("activity", []):
        if record.get("id") == identifier:
            return record
    return None


def mark_read(state: dict[str, Any], identifier: str) -> bool:
    record = find_activity(state, identifier)
    if record is None:
        return False
    record["read"] = True
    return True


def mark_all_read(state: dict[str, Any]) -> int:
    changed = 0
    for record in state.get("activity", []):
        if not record.get("read"):
            record["read"] = True
            changed += 1
    return changed


UI_DEFAULTS = {
    "sort": "updated",
    "sort_direction": -1,
    "filter": None,
    "unread_only": False,
    "show_ignored": False,
    "desktop": True,
    "sound": True,
}


def load_ui(path: Path) -> dict[str, Any]:
    data = dict(UI_DEFAULTS)
    try:
        if path.exists():
            loaded = json.loads(path.read_text(encoding="utf-8"))
            if isinstance(loaded, dict):
                data.update(loaded)
    except (OSError, json.JSONDecodeError):
        pass
    return data


def save_ui(path: Path, ui: dict[str, Any]) -> None:
    try:
        tmp = path.with_name(path.name + ".tmp")
        tmp.write_text(json.dumps(ui, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        os.replace(tmp, path)
    except OSError:
        pass
