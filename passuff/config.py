"""Configuration schema, validation, and atomic writes."""

from __future__ import annotations

import os
import stat
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

import tomlkit

RELATIONS = ("authored", "assigned", "review-requested", "team-review-requested")
EVENT_FAMILIES = ("comments", "reviews", "ci_failure", "ci_recovery", "state_changes", "bot_comments")
AUTH_MODES = ("gh_oauth", "github_pat")


class ConfigError(Exception):
    """A configuration error exits 2."""


@dataclass
class AccountConfig:
    label: str
    auth: str = "github_pat"
    github_user: str | None = None
    token_env: str | None = None
    token_command: str | None = None
    token_file: str | None = None
    graphql_url: str = "https://api.github.com/graphql"


@dataclass
class WatchConfig:
    mode: str = "all"
    repositories: list[str] = field(default_factory=list)
    ignored_pull_requests: list[str] = field(default_factory=list)


@dataclass
class FocusConfig:
    relations: list[str] = field(default_factory=lambda: list(RELATIONS))


@dataclass
class EventsConfig:
    comments: bool = True
    reviews: bool = True
    ci_failure: bool = True
    ci_recovery: bool = True
    state_changes: bool = True
    bot_comments: bool = False


@dataclass
class NotificationsConfig:
    desktop: bool = True
    sound: bool = True
    sound_file: str = "default"
    urgency: str = "normal"


@dataclass
class AppConfig:
    poll_interval_secs: int = 60
    reconciliation_interval_secs: int = 3600
    watch: WatchConfig = field(default_factory=WatchConfig)
    focus: FocusConfig = field(default_factory=FocusConfig)
    events: EventsConfig = field(default_factory=EventsConfig)
    notifications: NotificationsConfig = field(default_factory=NotificationsConfig)
    accounts: dict[str, AccountConfig] = field(default_factory=dict)
    path: Path | None = None


def _unknown_keys(table: dict[str, Any], allowed: set[str], where: str) -> None:
    for key in table:
        if key not in allowed:
            raise ConfigError(f"unknown configuration key: {where}{key}")


def _is_loopback(url: str) -> bool:
    parsed = urlparse(url)
    host = (parsed.hostname or "").lower()
    return host in {"localhost", "127.0.0.1", "::1"}


def _validate_token_file(path_text: str) -> None:
    if not (path_text.startswith("/") or path_text.startswith("~")):
        raise ConfigError("token file must be absolute or start with ~")
    path = Path(path_text).expanduser()
    if not path.exists():
        return
    if not path.is_file():
        raise ConfigError("token file is not a regular file")
    mode = stat.S_IMODE(path.stat().st_mode)
    if mode & 0o077:
        raise ConfigError("token_file is not private")


def load_config(path: str | os.PathLike[str] | None = None) -> AppConfig:
    path = Path(path).expanduser() if path else Path.home() / ".config" / "passuff" / "config.toml"
    if not path.exists():
        raise ConfigError("configuration file not found: " + str(path))
    try:
        raw = tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        raise ConfigError(f"invalid TOML: {exc}") from exc
    except OSError as exc:
        raise ConfigError(f"configuration file cannot be read: {exc}") from exc

    _unknown_keys(
        raw,
        {"poll_interval_secs", "reconciliation_interval_secs", "watch", "focus", "events", "notifications", "accounts"},
        "",
    )
    config = AppConfig(path=path)

    poll = raw.get("poll_interval_secs", 60)
    reconcile = raw.get("reconciliation_interval_secs", 3600)
    if not isinstance(poll, int) or poll < 1:
        raise ConfigError("poll_interval_secs must be an integer of at least 1")
    if not isinstance(reconcile, int) or reconcile < 1:
        raise ConfigError("reconciliation_interval_secs must be an integer of at least 1")
    config.poll_interval_secs = poll
    config.reconciliation_interval_secs = reconcile

    watch = raw.get("watch", {})
    if not isinstance(watch, dict):
        raise ConfigError("watch must be a table")
    _unknown_keys(watch, {"mode", "repositories", "ignored_pull_requests"}, "watch.")
    mode = watch.get("mode")
    repos = watch.get("repositories", [])
    if not isinstance(repos, list):
        raise ConfigError("watch.repositories must be an array")
    seen = set()
    for repo in repos:
        if not isinstance(repo, str) or repo.count("/") != 1 or any(ch.isspace() for ch in repo):
            raise ConfigError("repository must be owner/repo")
        lowered = repo.casefold()
        if lowered in seen:
            raise ConfigError("duplicate repository: " + repo)
        seen.add(lowered)
    if mode is None:
        mode = "selected" if repos else "all"
    if mode not in {"all", "selected"}:
        raise ConfigError("watch.mode must be all or selected")
    ignored = watch.get("ignored_pull_requests", [])
    if not isinstance(ignored, list):
        raise ConfigError("watch.ignored_pull_requests must be an array")
    seen_ignored = set()
    for key in ignored:
        if not isinstance(key, str) or "#" not in key:
            raise ConfigError("ignored pull request must be owner/repo#number")
        repo, _, number = key.rpartition("#")
        if repo.count("/") != 1 or not number.isdigit() or int(number) < 1:
            raise ConfigError("ignored pull request must be owner/repo#number")
        lowered = key.casefold()
        if lowered in seen_ignored:
            raise ConfigError("duplicate ignored pull request: " + key)
        seen_ignored.add(lowered)
    config.watch = WatchConfig(mode=mode, repositories=list(repos), ignored_pull_requests=list(ignored))

    focus = raw.get("focus", {})
    if not isinstance(focus, dict):
        raise ConfigError("focus must be a table")
    _unknown_keys(focus, {"relations"}, "focus.")
    relations = focus.get("relations", list(RELATIONS))
    if not isinstance(relations, list) or any(rel not in RELATIONS for rel in relations):
        raise ConfigError("focus.relations contains an invalid relation")
    config.focus = FocusConfig(relations=list(relations))

    events = raw.get("events", {})
    if not isinstance(events, dict):
        raise ConfigError("events must be a table")
    _unknown_keys(events, set(EVENT_FAMILIES), "events.")
    config.events = EventsConfig(**{name: bool(events.get(name, getattr(EventsConfig(), name))) for name in EVENT_FAMILIES})

    notifications = raw.get("notifications", {})
    if not isinstance(notifications, dict):
        raise ConfigError("notifications must be a table")
    _unknown_keys(notifications, {"desktop", "sound", "sound_file", "urgency"}, "notifications.")
    sound_file = notifications.get("sound_file", "default")
    if sound_file != "default":
        sound_path = Path(str(sound_file)).expanduser()
        if not sound_path.is_file() or not os.access(sound_path, os.R_OK):
            raise ConfigError("sound_file is not a readable file")
    urgency = notifications.get("urgency", "normal")
    if urgency not in {"low", "normal", "critical"}:
        raise ConfigError("notifications.urgency must be low, normal, or critical")
    config.notifications = NotificationsConfig(
        desktop=bool(notifications.get("desktop", True)),
        sound=bool(notifications.get("sound", True)),
        sound_file=str(sound_file),
        urgency=urgency,
    )

    accounts = raw.get("accounts")
    if not isinstance(accounts, dict) or not accounts:
        raise ConfigError("no accounts are configured")
    for label, values in accounts.items():
        if not isinstance(label, str) or not label:
            raise ConfigError("account label must not be empty")
        if not isinstance(values, dict):
            raise ConfigError(f"account {label} must be a table")
        if "token" in values:
            raise ConfigError("account must not contain a literal token")
        _unknown_keys(
            values,
            {"auth", "github_user", "token_env", "token_command", "token_file", "graphql_url"},
            f"accounts.{label}.",
        )
        auth = values.get("auth", "github_pat")
        if auth not in AUTH_MODES:
            raise ConfigError("unknown auth mode")
        login = values.get("github_user")
        if login is not None and (not isinstance(login, str) or any(ch.isspace() for ch in login)):
            raise ConfigError("github_user must not contain whitespace")
        sources = [name for name in ("token_env", "token_command", "token_file") if values.get(name) is not None]
        if auth == "gh_oauth":
            if not login:
                raise ConfigError("gh_oauth requires github_user")
            if sources:
                raise ConfigError("gh_oauth must not set a personal access token source")
        else:
            if len(sources) != 1:
                raise ConfigError("account must select exactly one credential source")
            if values.get("token_file") is not None:
                _validate_token_file(str(values["token_file"]))
        url = str(values.get("graphql_url", "https://api.github.com/graphql"))
        parsed = urlparse(url)
        if parsed.scheme not in {"https", "http"}:
            raise ConfigError("graphql_url must use https")
        if parsed.scheme == "http" and not _is_loopback(url):
            raise ConfigError("graphql_url must not use plain HTTP outside loopback")
        config.accounts[label] = AccountConfig(
            label=label,
            auth=auth,
            github_user=login,
            token_env=values.get("token_env"),
            token_command=values.get("token_command"),
            token_file=values.get("token_file"),
            graphql_url=url,
        )
    return config


def _atomic_write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + ".tmp")
    with open(tmp, "w", encoding="utf-8") as handle:
        handle.write(text)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, path)


def write_initial_config(path: Path, accounts: dict[str, dict[str, Any]], mode: str, repositories: list[str]) -> None:
    doc = tomlkit.document()
    doc.add("poll_interval_secs", 60)
    doc.add("reconciliation_interval_secs", 3600)
    watch = tomlkit.table()
    watch.add("mode", mode)
    watch.add("repositories", repositories)
    doc.add("watch", watch)
    accounts_table = tomlkit.table()
    for label, values in accounts.items():
        table = tomlkit.table()
        for key, value in values.items():
            table.add(key, value)
        accounts_table.add(label, table)
    doc.add("accounts", accounts_table)
    _atomic_write(path, tomlkit.dumps(doc))


def update_scope(path: Path, mode: str, repositories: list[str]) -> None:
    try:
        text = path.read_text(encoding="utf-8")
        doc = tomlkit.parse(text)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ConfigError(f"configuration file cannot be read: {exc}") from exc
    watch = doc.get("watch")
    if watch is None:
        watch = tomlkit.table()
        doc.add("watch", watch)
    watch["mode"] = mode
    watch["repositories"] = list(repositories)
    try:
        _atomic_write(path, tomlkit.dumps(doc))
    except OSError as exc:
        raise ConfigError(f"cannot write configuration: {exc}") from exc


def update_focus(path: Path, relations: list[str]) -> None:
    try:
        doc = tomlkit.parse(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ConfigError(f"configuration file cannot be read: {exc}") from exc
    focus = doc.get("focus")
    if focus is None:
        focus = tomlkit.table()
        doc.add("focus", focus)
    focus["relations"] = list(relations)
    try:
        _atomic_write(path, tomlkit.dumps(doc))
    except OSError as exc:
        raise ConfigError(f"cannot write configuration: {exc}") from exc


def update_ignored(path: Path, ignored: list[str]) -> None:
    try:
        doc = tomlkit.parse(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ConfigError(f"configuration file cannot be read: {exc}") from exc
    watch = doc.get("watch")
    if watch is None:
        watch = tomlkit.table()
        doc.add("watch", watch)
    watch["ignored_pull_requests"] = list(ignored)
    try:
        _atomic_write(path, tomlkit.dumps(doc))
    except OSError as exc:
        raise ConfigError(f"cannot write configuration: {exc}") from exc
