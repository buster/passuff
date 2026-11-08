"""Watch status and activity inbox read models."""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from .config import AppConfig
from .state import mark_all_read as state_mark_all, mark_read as state_mark_read


def parse_time(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def format_when(value: str | None) -> str:
    moment = parse_time(value)
    if moment is None:
        return "??-?? ??:??"
    moment = moment.astimezone(timezone.utc)
    return moment.strftime("%m-%d %H:%M")


def effective_time(record: dict[str, Any]) -> datetime:
    return parse_time(record.get("occurred_at")) or parse_time(record.get("observed_at")) or datetime.min.replace(tzinfo=timezone.utc)


def record_approximate(record: dict[str, Any]) -> bool:
    return not record.get("occurred_at")


def visible(pr: dict[str, Any], relations: list[str]) -> bool:
    stored = pr.get("relations") or []
    return not stored or bool(set(stored) & set(relations))


def in_scope(config: AppConfig, key: str) -> bool:
    if config.watch.mode == "all":
        return True
    repo = key.split("#", 1)[0]
    return repo.casefold() in {item.casefold() for item in config.watch.repositories}


def stack_sort_key(pr: dict[str, Any]) -> tuple:
    stack = pr.get("stack") or {}
    return (stack.get("number", 0), stack.get("position", 0))


class WatchStatusReader:
    def __init__(self, config: AppConfig, state: dict[str, Any]) -> None:
        self.config = config
        self.state = state

    def accounts(self) -> list[tuple[str, str | None]]:
        rows = []
        for label in self.config.accounts:
            login = (self.state.get("accounts", {}).get(label) or {}).get("login")
            rows.append((label, login))
        return rows

    def scope_line(self) -> str:
        if self.config.watch.mode == "selected":
            return "watch scope: " + (", ".join(self.config.watch.repositories) if self.config.watch.repositories else "none")
        return "watch scope: all authored pull requests"

    def watched(self) -> list[dict[str, Any]]:
        rows = []
        for key, pr in self.state.get("pull_requests", {}).items():
            if not in_scope(self.config, key):
                continue
            if not visible(pr, self.config.focus.relations):
                continue
            rows.append(pr)
        def key_order(pr):
            key = pr.get("key", "")
            repo, _, number = key.rpartition("#")
            try:
                return (repo, int(number))
            except ValueError:
                return (repo, 0)

        rows.sort(key=lambda pr: (stack_sort_key(pr), key_order(pr)))
        return rows

    def activity(self, include_read: bool = True) -> list[dict[str, Any]]:
        rows = []
        for record in self.state.get("activity", []):
            key = record.get("pull_request", "")
            if not in_scope(self.config, key):
                continue
            pr = self.state.get("pull_requests", {}).get(key) or {}
            if not visible(pr, self.config.focus.relations):
                continue
            if not include_read and record.get("read"):
                continue
            rows.append(record)
        rows.sort(key=lambda record: (effective_time(record), record.get("id", "")), reverse=True)
        return rows

    def unread(self) -> int:
        return sum(1 for record in self.activity() if not record.get("read"))

    def last_check(self) -> str | None:
        times = [
            value.get("last_check")
            for value in self.state.get("accounts", {}).values()
            if value.get("last_check")
        ]
        return max(times) if times else None

    def view(self) -> dict[str, Any]:
        return {
            "accounts": self.accounts(),
            "scope": self.scope_line(),
            "watched": self.watched(),
            "unread": self.unread(),
            "total": len(self.activity()),
            "last_check": self.last_check(),
            "watching_nothing": self.config.watch.mode == "selected" and not self.config.watch.repositories,
        }


class ActivityInboxService:
    def __init__(self, config: AppConfig, state: dict[str, Any]) -> None:
        self.config = config
        self.state = state

    def view(self, include_read: bool = False) -> list[dict[str, Any]]:
        reader = WatchStatusReader(self.config, self.state)
        return reader.activity(include_read=include_read)

    def unread_count(self) -> int:
        return WatchStatusReader(self.config, self.state).unread()

    def mark_read(self, identifier: str) -> bool:
        return state_mark_read(self.state, identifier)

    def mark_all(self) -> int:
        return state_mark_all(self.state)
