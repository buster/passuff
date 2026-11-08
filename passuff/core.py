"""The UI-agnostic check cycle."""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any

from .classify import classify, snapshot
from .config import AppConfig, ConfigError
from .credentials import CredentialError, resolve_credential
from .github import GitHubApiAdapter, GitHubError, PullRequest
from .state import StateError, append_activity, save_state


def _now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def _parse(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def _seconds_until(value: str | None) -> float:
    moment = _parse(value)
    if moment is None:
        return 3600.0
    return max(0.0, (moment - datetime.now(timezone.utc)).total_seconds())


def plan_delay(remaining: int, reset_at: str | None, poll: int) -> int:
    seconds = _seconds_until(reset_at)
    if remaining < 100:
        return max(poll, int(math.ceil(seconds)))
    share = max(1, remaining // 2)
    return max(poll, int(math.ceil(seconds / share)))


@dataclass
class Notification:
    key: str
    events: list[dict[str, Any]]


@dataclass
class RefreshReport:
    notifications: list[Notification] = field(default_factory=list)
    failures: list[tuple[str, str]] = field(default_factory=list)
    baselines: list[str] = field(default_factory=list)
    next_delay: int = 60

    @property
    def activity_count(self) -> int:
        return sum(len(item.events) for item in self.notifications)

    @property
    def ok(self) -> bool:
        return not self.failures


class PassuffCore:
    def __init__(
        self,
        config: AppConfig,
        state: dict[str, Any],
        state_path: Any = None,
        *,
        interactive: bool = False,
        persist: bool = True,
    ) -> None:
        self.config = config
        self.state = state
        self.state_path = state_path
        self.interactive = interactive
        self.persist = persist

    def _in_scope(self, key: str) -> bool:
        repo = key.split("#", 1)[0]
        if self.config.watch.mode == "selected":
            return repo in {item.casefold() for item in self.config.watch.repositories}
        return True

    def _ignored(self, key: str) -> bool:
        return key.casefold() in {item.casefold() for item in self.config.watch.ignored_pull_requests}

    def _resolve(self) -> tuple[list[tuple[str, Any, GitHubApiAdapter, str]], list[tuple[str, str]]]:
        resolved = []
        failures = []
        for label, account in self.config.accounts.items():
            try:
                token = resolve_credential(account, self.interactive)
                adapter = GitHubApiAdapter(account.graphql_url, token)
                login, rate = adapter.viewer()
                if account.github_user and login.casefold() != account.github_user.casefold():
                    raise CredentialError(
                        f"credential belongs to @{login}, expected @{account.github_user}"
                    )
                resolved.append((label, account, adapter, login))
                self.state["accounts"].setdefault(label, {})["login"] = login
                self.state["accounts"][label]["rate"] = {"remaining": rate.remaining, "reset_at": rate.reset_at}
            except (CredentialError, GitHubError) as exc:
                failures.append((label, str(exc)))
        return resolved, failures

    def _snapshot_for(self, pr: PullRequest, previous: dict[str, Any] | None, previous_ci_terminal: str | None) -> dict[str, Any]:
        data = snapshot(pr)
        if pr.ci in {"failing", "passing"}:
            data["ci_terminal"] = pr.ci
        elif previous_ci_terminal:
            data["ci_terminal"] = previous_ci_terminal
        elif previous and previous.get("ci_terminal"):
            data["ci_terminal"] = previous["ci_terminal"]
        else:
            data["ci_terminal"] = None
        return data

    def _observe_account(
        self,
        label: str,
        account: Any,
        adapter: GitHubApiAdapter,
        login: str,
        login_set: set[str],
        report: RefreshReport,
    ) -> None:
        account_state = self.state["accounts"].setdefault(label, {})
        last_observed = account_state.get("last_observed_at")
        last_check = account_state.get("last_check")
        probe_map = dict(account_state.get("probe") or {})
        now = _now()

        summaries, rate = adapter.probe()
        account_state["rate"] = {"remaining": rate.remaining, "reset_at": rate.reset_at}
        report.next_delay = plan_delay(rate.remaining, rate.reset_at, self.config.poll_interval_secs)

        summaries = [
            item for item in summaries
            if item.get("id") and self._in_scope(item.get("key", "")) and not self._ignored(item.get("key", ""))
        ]
        from datetime import timedelta

        def retired_too_long(item: dict[str, Any]) -> bool:
            stored = self.state["pull_requests"].get(item.get("key", ""))
            if not stored or stored.get("state") not in {"merged", "closed"}:
                return False
            retired = _parse(stored.get("retired_at"))
            return bool(retired and datetime.now(timezone.utc) - retired > timedelta(days=30))

        summaries = [item for item in summaries if not retired_too_long(item)]
        authored = [item for item in summaries if item.get("author") in login_set]
        discovered = []
        try:
            for item in adapter.discover():
                if not item.get("id") or not self._in_scope(item.get("key", "")) or self._ignored(item.get("key", "")):
                    continue
                if retired_too_long(item):
                    continue
                if item.get("author") in login_set:
                    continue
                if login in (item.get("assignees") or []) or set(item.get("relations") or []) & {"review-requested", "team-review-requested"}:
                    discovered.append(item)
        except GitHubError:
            discovered = []

        reconcile_due = False
        if last_check:
            elapsed = (datetime.now(timezone.utc) - (_parse(last_check) or datetime.now(timezone.utc))).total_seconds()
            reconcile_due = elapsed >= self.config.reconciliation_interval_secs

        targets: dict[str, dict[str, Any]] = {}
        if last_observed is None:
            for item in authored + discovered:
                targets[item["key"]] = item
        else:
            for item in authored + discovered:
                key = item.get("key")
                known = probe_map.get(key)
                state = item.get("state")
                if known is None:
                    if state == "open":
                        targets[key] = item
                    elif state in {"merged", "closed"}:
                        created = _parse(item.get("created_at"))
                        boundary = _parse(last_observed)
                        if created and boundary and created > boundary:
                            targets[key] = item
                else:
                    fingerprint = (item.get("updated_at"), item.get("head_sha"), state, item.get("ci"))
                    known_fingerprint = (known.get("updated_at"), known.get("head_sha"), known.get("state"), known.get("ci"))
                    if fingerprint != known_fingerprint:
                        targets[key] = item
            if reconcile_due:
                for key, stored in self.state["pull_requests"].items():
                    if stored.get("state") == "open" and self._in_scope(key) and not self._ignored(key):
                        summary = next((item for item in authored + discovered if item.get("key") == key), None)
                        if summary:
                            targets.setdefault(key, summary)

        observed = adapter.observe([item["id"] for item in targets.values()])
        last_scope = self.state.get("last_scope") or {}

        def was_out(key: str) -> bool:
            if last_scope.get("mode") != "selected":
                return False
            repo = key.split("#", 1)[0]
            return repo.casefold() not in {item.casefold() for item in last_scope.get("repositories", [])} and self._in_scope(key)

        for pr in observed:
            relations = set(pr.relations)
            if pr.author in login_set:
                relations.add("authored")
            if set(pr.assignees) & login_set:
                relations.add("assigned")
            pr.relations = sorted(relations)
        observed_by_key = {pr.key: pr for pr in observed}

        for key, summary in targets.items():
            pr = observed_by_key.get(key)
            if pr is None:
                continue
            stored = self.state["pull_requests"].get(key)
            previous = None
            previous_ci_terminal = None
            state_change_reported = False
            if stored:
                previous = stored.get("last")
                previous_ci_terminal = (previous or {}).get("ci_terminal") or stored.get("ci_terminal")
                state_change_reported = bool(stored.get("state_change_reported"))
            events = classify(
                previous,
                pr,
                login_set=login_set,
                focus=self.config.focus,
                events=self.config.events,
                state_change_reported=state_change_reported,
                last_observed_at=last_observed,
            )
            if stored and was_out(key):
                for item in events:
                    item["notify"] = False
            if previous is None:
                report.baselines.append(key)
            for item in events:
                append_activity(
                    self.state,
                    {
                        "pull_request": key,
                        "kind": item["kind"],
                        "summary": item["summary"],
                        "actor": item.get("actor", ""),
                        "body": item.get("body", ""),
                        "occurred_at": item.get("occurred_at"),
                        "read": bool(item.get("read")),
                        "notified": bool(item.get("notify")),
                    },
                )
            notify_events = [item for item in events if item.get("notify")]
            if notify_events:
                report.notifications.append(Notification(key=key, events=notify_events))

            new_snapshot = self._snapshot_for(pr, previous, previous_ci_terminal)
            state_change_reported = bool(notify_events) and any(item["kind"] == "state_change" for item in notify_events)
            if previous and previous.get("state") in {"merged", "closed"} and pr.state == "open":
                state_change_reported = False
            self.state["pull_requests"][key] = {
                "key": key,
                "title": pr.title,
                "description": pr.body,
                "creator": pr.author,
                "assignees": list(pr.assignees),
                "relations": list(pr.relations),
                "state": pr.state,
                "ci": pr.ci,
                "ci_terminal": new_snapshot.get("ci_terminal"),
                "stack": pr.stack,
                "baseline": (stored or {}).get("baseline") or new_snapshot,
                "last": new_snapshot,
                "state_change_reported": state_change_reported,
                "last_observed_at": now,
                "retired_at": now if pr.state in {"merged", "closed"} else None,
            }

        for item in summaries:
            key = item.get("key")
            if key:
                probe_map[key] = {
                    "updated_at": item.get("updated_at"),
                    "head_sha": item.get("head_sha"),
                    "state": item.get("state"),
                    "ci": item.get("ci"),
                }
        account_state["probe"] = probe_map
        account_state["last_observed_at"] = now
        account_state["last_check"] = now

        delay = plan_delay(rate.remaining, rate.reset_at, self.config.poll_interval_secs)
        account_state["next_delay"] = delay
        report.next_delay = max(report.next_delay, delay)

    def check_cycle(self) -> RefreshReport:
        report = RefreshReport(next_delay=self.config.poll_interval_secs)
        resolved, failures = self._resolve()
        report.failures.extend(failures)
        login_set = {login for _, _, _, login in resolved}
        for label, account, adapter, login in resolved:
            try:
                self._observe_account(label, account, adapter, login, login_set, report)
            except (GitHubError, CredentialError) as exc:
                report.failures.append((label, str(exc)))
            finally:
                adapter.close()
        self.state["last_scope"] = {
            "mode": self.config.watch.mode,
            "repositories": list(self.config.watch.repositories),
        }
        if self.persist and self.state_path is not None:
            save_state(self.state_path, self.state)
        return report
