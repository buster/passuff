"""Watch-status read model over the persisted watcher state."""

from __future__ import annotations

from typing import Any

from .config import AppConfig
from .inbox import activity_rows, in_scope, stack_sort_key, visible


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
        return activity_rows(self.config, self.state, include_read=include_read)

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
