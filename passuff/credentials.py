"""Credential resolution through gh, environment, command, or token file."""

from __future__ import annotations

import os
import shlex
import stat
import subprocess
from pathlib import Path

from .config import AccountConfig


class CredentialError(Exception):
    """An account failure."""


_LOGIN_ATTEMPTS: set[str] = set()


def _run_gh(args: list[str], user: str | None = None) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    if user:
        env["PASSUFF_GH_LOGIN_USER"] = user
    try:
        return subprocess.run(["gh", *args], capture_output=True, text=True, timeout=30, env=env)
    except (OSError, subprocess.SubprocessError) as exc:
        raise CredentialError(f"gh is unavailable: {exc}") from exc


def _gh_token(account: AccountConfig) -> str | None:
    result = _run_gh(["auth", "token", "--user", account.github_user or ""])
    if result.returncode == 0 and result.stdout.strip():
        return result.stdout.strip()
    return None


def _gh_login(account: AccountConfig) -> str | None:
    login = account.github_user or account.label
    if login in _LOGIN_ATTEMPTS:
        raise CredentialError("gh auth login failed")
    _LOGIN_ATTEMPTS.add(login)
    result = _run_gh(["auth", "login", "--web", "--git-protocol", "https"], user=login)
    if result.returncode == 0:
        return _gh_token(account)
    return None


def resolve_credential(account: AccountConfig, interactive: bool) -> str:
    if account.auth == "gh_oauth":
        token = _gh_token(account)
        if token:
            return token
        if not interactive:
            raise CredentialError("gh session is missing for " + (account.github_user or account.label))
        token = _gh_login(account)
        if token:
            return token
        raise CredentialError("gh auth login failed")
    if account.token_env:
        value = os.environ.get(account.token_env)
        if not value:
            raise CredentialError("credential value is missing from " + account.token_env)
        return value.strip()
    if account.token_command:
        try:
            result = subprocess.run(
                account.token_command,
                shell=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
        except (OSError, subprocess.SubprocessError) as exc:
            raise CredentialError(f"credential command failed: {exc}") from exc
        if result.returncode != 0 or not result.stdout.strip():
            raise CredentialError("credential command failed")
        return result.stdout.strip()
    if account.token_file:
        path = Path(account.token_file).expanduser()
        if not path.exists():
            raise CredentialError("token file is missing: " + str(path))
        if not path.is_file():
            raise CredentialError("token file is not a regular file")
        mode = stat.S_IMODE(path.stat().st_mode)
        if mode & 0o077:
            raise CredentialError("token_file is not private")
        value = path.read_text(encoding="utf-8").strip()
        if not value:
            raise CredentialError("token file is empty")
        return value
    raise CredentialError("must select exactly one credential source")
