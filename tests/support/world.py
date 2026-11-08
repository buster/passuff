"""Per-scenario test world: temp dirs, fake twins, CLI/TUI drivers."""

from __future__ import annotations

import json
import os
import re
import shlex
import shutil
import stat
import subprocess
import sys
import tempfile
import threading
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import tomlkit

from .fake_github import FakeGitHub
from .fake_notifications import FakeNotifications

REPO = Path(__file__).resolve().parents[2]
PYTHON = REPO / ".venv" / "bin" / "python"


def _now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


@dataclass
class Result:
    stdout: str
    stderr: str
    code: int

    @property
    def text(self) -> str:
        return self.stdout + self.stderr


class World:
    def __init__(self, e2e: bool = False) -> None:
        self.e2e = e2e
        self.tmp = Path(tempfile.mkdtemp(prefix="passuff-test-"))
        self.home = self.tmp / "home"
        self.home.mkdir()
        self.config_home = self.tmp / "xdg-config"
        self.state_home = self.tmp / "xdg-state"
        self.config_path = self.tmp / "config.toml"
        self.state_path = self.tmp / "state.json"
        self.ui_path = self.tmp / "ui.json"
        self.delivery_log = self.tmp / "delivery.jsonl"
        self.sound_log = self.tmp / "sound.log"
        self.browser_log = self.tmp / "browser.log"
        self.gh_log = self.tmp / "gh.log"
        self.gh_dir = self.tmp / "gh"
        self.gh_dir.mkdir()
        self.bin = REPO / "tests" / "support" / "bin"
        self.fake = FakeGitHub(self)
        self.fake.start()
        from datetime import datetime as _dt, timedelta as _td, timezone as _tz
        self.fake.rate_reset_at = (_dt.now(_tz.utc) + _td(seconds=60)).isoformat().replace("+00:00", "Z")
        self.notifications = FakeNotifications(self.delivery_log)
        self.notifications.start()
        self.accounts: dict[str, dict[str, Any]] = {}
        self.poll = 1
        self.reconcile = 3600
        self.watch_mode = "all"
        self.repositories: list[str] = []
        self.ignored: list[str] = []
        self.focus = ["authored", "assigned", "review-requested", "team-review-requested"]
        self.events = {
            "comments": True,
            "reviews": True,
            "ci_failure": True,
            "ci_recovery": True,
            "state_changes": True,
            "bot_comments": False,
        }
        self.notifications_config = {"desktop": True, "sound": True, "sound_file": "default", "urgency": "normal"}
        self.gh_tokens: dict[str, str] = {}
        self.gh_login_fail = False
        self.browser_fail = False
        self.browser_missing = False
        self.sound_missing = False
        self.sound_fail = False
        self.gh_missing = False
        self.desktop_unavailable = False
        self.dirty = False
        self.last: Result | None = None
        self.fixed_fetch = "2026-11-02T12:00:00Z"
        self.after_first_observe = None
        self._after_done = False
        self.tui_args: list[str] | None = None
        self.tui_keys: list[str] = []
        self.watch_proc: subprocess.Popen | None = None
        self.watch_lines: list[str] = []
        self.watch_lock = threading.Lock()
        self.config_snapshot: str | None = None
        self._soundless_bin = self.tmp / "bin-no-sound"
        self._browserless_bin = self.tmp / "bin-no-browser"
        self._ghless_bin = self.tmp / "bin-no-gh"
        self._make_variant_bins()
        self._tokens_counter = 0

    # ------------------------------------------------------------------ paths

    def _make_variant_bins(self) -> None:
        for target, omit in ((self._soundless_bin, "paplay"), (self._browserless_bin, "xdg-open"), (self._ghless_bin, "gh")):
            target.mkdir()
            for name in ("gh", "paplay", "xdg-open"):
                if name == omit:
                    continue
                link = target / name
                link.write_text((self.bin / name).read_text())
                link.chmod(0o755)

    # ----------------------------------------------------------------- config

    def write_config(self, force: bool = False) -> None:
        if self.config_path.exists() and not force and not self.dirty:
            return
        doc = tomlkit.document()
        doc.add("poll_interval_secs", self.poll)
        doc.add("reconciliation_interval_secs", self.reconcile)
        watch = tomlkit.table()
        watch.add("mode", self.watch_mode)
        watch.add("repositories", list(self.repositories))
        watch.add("ignored_pull_requests", list(self.ignored))
        doc.add("watch", watch)
        focus = tomlkit.table()
        focus.add("relations", list(self.focus))
        doc.add("focus", focus)
        events = tomlkit.table()
        for name, value in self.events.items():
            events.add(name, value)
        doc.add("events", events)
        notifications = tomlkit.table()
        for name, value in self.notifications_config.items():
            notifications.add(name, value)
        doc.add("notifications", notifications)
        if not self.accounts:
            self.account("personal", "octo-personal")
        accounts = tomlkit.table()
        for label, values in self.accounts.items():
            table = tomlkit.table()
            for key, value in values.items():
                table.add(key, value)
            accounts.add(label, table)
        doc.add("accounts", accounts)
        self.config_path.write_text(tomlkit.dumps(doc), encoding="utf-8")
        self.dirty = False

    def account(
        self,
        label: str,
        login: str | None = None,
        *,
        auth: str = "gh_oauth",
        token_env: str | None = None,
        token_command: str | None = None,
        token_file: str | None = None,
        expected_login: str | None = None,
        actual_login: str | None = None,
        mode: str | None = None,
    ) -> None:
        values: dict[str, Any] = {"graphql_url": self.fake.url()}
        if auth == "gh_oauth" or mode == "gh_oauth":
            values["auth"] = "gh_oauth"
            values["github_user"] = login or expected_login
            effective = login or expected_login or label
            self.gh_tokens[effective] = f"gh-token-{effective}"
            self.fake.tokens[f"gh-token-{effective}"] = actual_login or effective
        else:
            values["auth"] = "github_pat"
            if login or expected_login:
                values["github_user"] = login or expected_login
            self._tokens_counter += 1
            token = f"token-{label}-{self._tokens_counter}"
            actual = actual_login or login or expected_login or label
            self.fake.tokens[token] = actual
            if token_env:
                values["token_env"] = token_env
                self._env_tokens = getattr(self, "_env_tokens", {})
                self._env_tokens[token_env] = token
            elif token_command:
                values["token_command"] = f"printf '%s' '{token}'"
            elif token_file:
                values["token_file"] = token_file
        self.accounts[label] = values
        self.dirty = True

    def on_probe(self) -> None:
        if self.after_first_observe is not None and not self._after_done:
            self._after_done = True
            action = self.after_first_observe
            self.after_first_observe = None
            action()

    def on_observe(self) -> None:
        if self.after_first_observe is not None and not self._after_done:
            self._after_done = True
            action = self.after_first_observe
            self.after_first_observe = None
            action()

    # -------------------------------------------------------------------- PRs

    def add_pr(self, key: str, author: str, **fields: Any) -> dict[str, Any]:
        repo, _, number = key.partition("#")
        pr = self.fake.add_pr(repo, int(number), author, **fields)
        return pr

    def pr(self, key: str, author: str, **fields: Any) -> dict[str, Any]:
        existing = self.fake.find(key)
        if existing:
            existing.update({k: v for k, v in fields.items() if k in existing})
            return existing
        return self.add_pr(key, author, **fields)

    def comment(self, key: str, actor: str, body: str, at: str | None = None) -> None:
        pr = self.fake.find(key)
        if pr is None:
            return
        self.fake.counts["comment"] += 1
        number = self.fake.counts["comment"]
        pr["comments"].append(
            {
                "id": f"c{number}",
                "author": actor,
                "body": body,
                "created_at": at or "2026-11-02T12:00:00Z",
            }
        )
        pr["updated_at"] = _now()

    def review(self, key: str, actor: str, state: str, body: str = "", at: str | None = None) -> None:
        pr = self.fake.find(key)
        if pr is None:
            return
        self.fake.counts["review"] += 1
        number = self.fake.counts["review"]
        pr["reviews"].append(
            {
                "id": f"r{number}",
                "author": actor,
                "state": state.upper().replace(" ", "_"),
                "body": body,
                "submitted_at": at or "2026-11-02T12:00:00Z",
            }
        )
        pr["updated_at"] = _now()

    def review_comment(self, key: str, actor: str, body: str, at: str | None = None) -> None:
        pr = self.fake.find(key)
        if pr is None:
            return
        self.fake.counts["review_comment"] += 1
        number = self.fake.counts["review_comment"]
        pr["review_comments"].append(
            {"id": f"rc{number}", "author": actor, "body": body, "created_at": at or "2026-11-02T12:00:00Z"}
        )
        pr["updated_at"] = _now()

    def push(self, key: str) -> None:
        pr = self.fake.find(key)
        if pr:
            pr["head_sha"] = pr["head_sha"] + "-push"
            pr["updated_at"] = _now()

    def set_ci(self, key: str, state: str, name: str | None = None) -> None:
        pr = self.fake.find(key)
        if pr:
            pr["ci"] = "passing" if state == "successful" else state
            if name:
                pr["ci_name"] = name
            pr["updated_at"] = _now()

    def set_state(self, key: str, state: str) -> None:
        pr = self.fake.find(key)
        if pr:
            pr["state"] = state
            pr["updated_at"] = _now()

    def assign(self, key: str, login: str) -> None:
        pr = self.fake.find(key)
        if pr and login not in pr["assignees"]:
            pr["assignees"].append(login)

    def request_review(self, key: str, login: str) -> None:
        pr = self.fake.find(key)
        if pr and login not in pr["review_requests"]:
            pr["review_requests"].append(login)

    def request_team(self, key: str, team: str) -> None:
        pr = self.fake.find(key)
        if pr and team not in pr["team_requests"]:
            pr["team_requests"].append(team)

    def watched_merged(self, key: str, days: int) -> None:
        from datetime import timedelta

        old = (datetime.now(timezone.utc) - timedelta(days=days)).isoformat().replace("+00:00", "Z")
        self.add_pr(key, "octo-personal", state="merged", updated_at=old, created_at=(datetime.now(timezone.utc) - timedelta(days=days + 1)).isoformat().replace("+00:00", "Z"))
        state = self.empty_state()
        self.add_state_pr(state, key, state_value="merged", retired_at=old, last_observed_at=old)
        state["accounts"] = {"personal": {"login": "octo-personal", "last_observed_at": old, "last_check": old, "probe": {}}}
        self.write_state(state)

    def stack(self, key: str, number: int, position: int, size: int, base: str) -> None:
        pr = self.fake.find(key)
        if pr:
            pr["stack"] = {"number": number, "position": position, "size": size, "base": base}
            pr["updated_at"] = _now()

    # ------------------------------------------------------------------ state

    def load_state(self) -> dict[str, Any]:
        if not self.state_path.exists() or self.state_path.is_dir():
            return {}
        try:
            return json.loads(self.state_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return {}

    def write_state(self, data: dict[str, Any]) -> None:
        self.state_path.write_text(json.dumps(data, indent=2), encoding="utf-8")

    def state_version(self) -> int | None:
        return self.load_state().get("version")

    def empty_state(self, version: int = 2) -> dict[str, Any]:
        return {
            "version": version,
            "accounts": {},
            "pull_requests": {},
            "activity": [],
            "next_activity_id": 1,
            "last_scope": None,
        }

    def add_state_pr(self, state: dict[str, Any], key: str, author: str = "octo-personal", state_value: str = "open", ci: str = "passing", **extra: Any) -> None:
        state["pull_requests"][key] = {
            "key": key,
            "title": extra.get("title", f"PR {key.split('#')[-1]}"),
            "description": extra.get("description", ""),
            "creator": author,
            "assignees": extra.get("assignees", []),
            "relations": extra.get("relations", ["authored"]),
            "state": state_value,
            "ci": ci,
            "ci_terminal": ci,
            "stack": extra.get("stack"),
            "baseline": {},
            "last": {},
            "state_change_reported": False,
            "last_observed_at": extra.get("last_observed_at", _now()),
            "retired_at": extra.get("retired_at"),
        }

    def add_state_activity(self, state: dict[str, Any], identifier: str, summary: str, key: str = "acme/api#42", actor: str = "alice", kind: str = "comment", read: bool = False, occurred_at: str | None = "2026-11-02T09:15:00Z", body: str = "") -> None:
        state["activity"].append(
            {
                "id": identifier,
                "pull_request": key,
                "kind": kind,
                "summary": summary,
                "actor": actor,
                "body": body,
                "occurred_at": occurred_at,
                "observed_at": "2026-11-02T12:00:00Z",
                "read": read,
                "notified": not read,
            }
        )

    def state_activity(self, summary: str) -> dict[str, Any] | None:
        for record in self.load_state().get("activity", []):
            value = str(record.get("summary", ""))
            if value == summary or value.startswith(summary):
                return record
        return None

    def mark_state_activity(self, summary: str, read: bool) -> None:
        state = self.load_state()
        for record in state.get("activity", []):
            value = str(record.get("summary", ""))
            if value == summary or value.startswith(summary):
                record["read"] = read
        self.write_state(state)

    # ------------------------------------------------------------------- runs

    def _base_env(self, *, browser_missing: bool = False, sound_missing: bool = False) -> dict[str, str]:
        env = os.environ.copy()
        env.update(
            {
                "HOME": str(self.home),
                "XDG_CONFIG_HOME": str(self.config_home),
                "XDG_STATE_HOME": str(self.state_home),
                "PYTHONPATH": str(REPO),
                "PASSUFF_GH_LOG": str(self.gh_log),
                "PASSUFF_GH_DIR": str(self.gh_dir),
                "PASSUFF_SOUND_LOG": str(self.sound_log),
                "PASSUFF_BROWSER_LOG": str(self.browser_log),
                "PASSUFF_BROWSER_FAIL": "1" if self.browser_fail else "0",
                "PASSUFF_GH_LOGIN_FAIL": "1" if self.gh_login_fail else "0",
                "PASSUFF_DESKTOP_UNAVAILABLE": "1" if self.desktop_unavailable else "0",
                "PASSUFF_SOUND_FAIL": "1" if self.sound_fail else "0",
            }
        )
        for login, token in self.gh_tokens.items():
            env["PASSUFF_GH_TOKEN_" + login.replace("-", "_").replace(".", "_")] = token
        for name, value in getattr(self, "_env_tokens", {}).items():
            env[name] = value
        bindir = self.bin
        omit_system = False
        if browser_missing or self.browser_missing:
            bindir = self._browserless_bin
            omit_system = True
        if sound_missing or self.sound_missing:
            bindir = self._soundless_bin
            omit_system = True
        if getattr(self, "gh_missing", False):
            bindir = self._ghless_bin
            omit_system = True
        path = str(bindir) + os.pathsep + str(PYTHON.parent)
        if not omit_system:
            path += os.pathsep + os.environ.get("PATH", "")
        env["PATH"] = path
        return env

    def _command(self, tui: bool) -> list[str]:
        if tui:
            override = os.environ.get("PASSUFF_TUI")
            base = shlex.split(override) if override else [str(PYTHON), "-m", "passuff.tui"]
        else:
            override = os.environ.get("PASSUFF_CLI")
            base = shlex.split(override) if override else [str(PYTHON), "-m", "passuff"]
        return base + ["--config", str(self.config_path), "--state", str(self.state_path)]

    def run(self, args: list[str], *, stdin: str | None = None, timeout: float = 30) -> Result:
        if args[:1] != ["setup"] and not getattr(self, "suppress_config_write", False):
            self.write_config()
        cmd = self._command(False) + args
        try:
            result = subprocess.run(
                cmd,
                input=stdin,
                capture_output=True,
                text=True,
                timeout=timeout,
                env=self._base_env(),
                cwd=str(REPO),
            )
        except subprocess.TimeoutExpired as exc:
            result = Result((exc.stdout or "") if isinstance(exc.stdout, str) else "", "timeout", 124)
        self.last = Result(result.stdout, result.stderr, result.returncode)
        if args[:1] != ["setup"] and self.state_path.exists() and not self.state_path.is_dir():
            try:
                state = json.loads(self.state_path.read_text(encoding="utf-8"))
                changed = False
                for record in state.get("activity", []):
                    if record.get("observed_at") != self.fixed_fetch:
                        record["observed_at"] = self.fixed_fetch
                        changed = True
                if changed:
                    self.state_path.write_text(json.dumps(state, indent=2), encoding="utf-8")
            except (OSError, json.JSONDecodeError):
                pass
        if self.desktop_unavailable:
            self.notifications.stop()
            self.desktop_unavailable = False
        return self.last

    def run_tui(self, args: list[str] | None = None, *, stdin: str | None = None, timeout: float = 60) -> Result:
        if not getattr(self, "suppress_config_write", False):
            self.write_config()
        if args is None:
            args = ["--snapshot"]
        cmd = self._command(True) + args
        try:
            result = subprocess.run(
                cmd,
                input=stdin,
                capture_output=True,
                text=True,
                timeout=timeout,
                env=self._base_env(),
                cwd=str(REPO),
            )
        except subprocess.TimeoutExpired as exc:
            result = Result((exc.stdout or "") if isinstance(exc.stdout, str) else "", "timeout", 124)
        self.last = Result(result.stdout, result.stderr, result.returncode)
        return self.last

    def open_tui(self, args: list[str] | None = None) -> Result:
        self.tui_args = list(args if args is not None else ["--snapshot"])
        self.tui_keys = []
        return self.run_tui(self.tui_args)

    def press(self, key: str) -> Result:
        self.tui_keys.append(key)
        args = list(self.tui_args if self.tui_args is not None else ["--snapshot"])
        if "--keys" in args:
            index = args.index("--keys")
            args = args[:index] + args[index + 1:]
        args = args + ["--keys", ",".join(self.tui_keys)]
        return self.run_tui(args)

    def observe(self, keys: list[str] | None = None) -> None:
        if keys:
            for key in keys:
                self.pr(key, "octo-personal")
        self.run(["check"])
        self.last = None

    # --------------------------------------------------------------- watchers

    def start_watch(self, *, interactive: bool = False) -> None:
        self.write_config()
        cmd = self._command(False)
        env = self._base_env()
        if interactive:
            from .pty_driver import start_pty
            self.watch_proc = start_pty(cmd, env, self.watch_lines, self.watch_lock)
        else:
            self.watch_proc = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1,
                env=env,
                cwd=str(REPO),
            )

            def reader():
                assert self.watch_proc and self.watch_proc.stdout
                for line in self.watch_proc.stdout:
                    with self.watch_lock:
                        self.watch_lines.append(line)

            threading.Thread(target=reader, daemon=True).start()

    def wait_for(self, text: str, timeout: float = 20) -> bool:
        deadline = time.time() + timeout
        while time.time() < deadline:
            with self.watch_lock:
                if text in "".join(self.watch_lines):
                    return True
            time.sleep(0.1)
        return False

    def stop_watch(self) -> None:
        if self.watch_proc is None:
            return
        try:
            self.watch_proc.send_signal(2)
        except Exception:
            pass
        try:
            self.watch_proc.wait(timeout=5)
        except Exception:
            self.watch_proc.kill()
        self.watch_proc = None

    # ------------------------------------------------------------------ logs

    def delivery(self) -> list[dict[str, Any]]:
        if not self.delivery_log.exists():
            return []
        rows = []
        for line in self.delivery_log.read_text(encoding="utf-8").splitlines():
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                pass
        return rows

    def sound_lines(self) -> list[str]:
        return self.sound_log.read_text(encoding="utf-8").splitlines() if self.sound_log.exists() else []

    def browser_lines(self) -> list[str]:
        return self.browser_log.read_text(encoding="utf-8").splitlines() if self.browser_log.exists() else []

    def gh_lines(self) -> list[str]:
        return self.gh_log.read_text(encoding="utf-8").splitlines() if self.gh_log.exists() else []

    def config_text(self) -> str:
        return self.config_path.read_text(encoding="utf-8") if self.config_path.exists() else ""

    # ---------------------------------------------------------------- cleanup

    def cleanup(self) -> None:
        self.stop_watch()
        try:
            self.notifications.stop()
        except Exception:
            pass
        try:
            self.fake.stop()
        except Exception:
            pass
        for path in (self.tmp,):
            for root, dirs, files in os.walk(path):
                for name in dirs:
                    try:
                        os.chmod(os.path.join(root, name), 0o700)
                    except OSError:
                        pass
            try:
                shutil.rmtree(path, ignore_errors=True)
            except Exception:
                pass
