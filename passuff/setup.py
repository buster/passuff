"""Interactive first-run configuration."""

from __future__ import annotations

import stat
from pathlib import Path

from .config import write_initial_config


class SetupError(Exception):
    pass


def _ask(stdin, stdout, prompt: str, default: str = "") -> str:
    stdout.write(prompt + " ")
    stdout.flush()
    line = stdin.readline()
    if not line:
        raise SetupError("input ended")
    value = line.rstrip("\n")
    return value if value else default


def run_setup(config_path: Path, stdin, stdout) -> bool:
    if config_path.exists():
        answer = _ask(stdin, stdout, "Configuration exists. Replace? [y/N]", "n").strip().lower()
        if answer not in {"y", "yes"}:
            stdout.write("configuration unchanged\n")
            return False
    accounts: dict[str, dict[str, str]] = {}
    while True:
        label = _ask(stdin, stdout, "Account label:", "").strip()
        if not label:
            raise SetupError("account label must not be empty")
        auth = _ask(stdin, stdout, "Auth [gh_oauth]:", "gh_oauth").strip() or "gh_oauth"
        if auth not in {"gh_oauth", "token_env", "token_command", "token_file"}:
            raise SetupError("unknown auth mode")
        login = _ask(stdin, stdout, "Expected GitHub login:", "").strip()
        if not login:
            raise SetupError("requires github_user")
        if auth == "gh_oauth":
            account = {"auth": "gh_oauth", "github_user": login}
        elif auth == "token_env":
            variable = _ask(stdin, stdout, "Environment variable:", "").strip()
            if not variable:
                raise SetupError("environment variable must not be empty")
            account = {"auth": "github_pat", "github_user": login, "token_env": variable}
        elif auth == "token_command":
            command = _ask(stdin, stdout, "Command:", "").strip()
            if not command:
                raise SetupError("command must not be empty")
            account = {"auth": "github_pat", "github_user": login, "token_command": command}
        else:
            token_file = _ask(stdin, stdout, "Token file:", "").strip()
            if not token_file:
                raise SetupError("token file must not be empty")
            if not (token_file.startswith("/") or token_file.startswith("~")):
                raise SetupError("token file must be absolute or start with ~")
            path = Path(token_file).expanduser()
            if path.exists():
                mode = stat.S_IMODE(path.stat().st_mode)
                if mode & 0o077:
                    raise SetupError("insecure token file")
            account = {"auth": "github_pat", "github_user": login, "token_file": token_file}
        if label in accounts:
            raise SetupError("duplicate account label")
        accounts[label] = account
        more = _ask(stdin, stdout, "Add another account? [y/N]:", "n").strip().lower()
        if more not in {"y", "yes"}:
            break
    mode = _ask(stdin, stdout, "Watch mode [all]:", "all").strip() or "all"
    if mode not in {"all", "selected"}:
        raise SetupError("watch mode must be all or selected")
    repositories: list[str] = []
    if mode == "selected":
        while True:
            raw = _ask(stdin, stdout, "Repositories (comma-separated):", "").strip()
            repositories = [item.strip() for item in raw.split(",") if item.strip()]
            if all(item.count("/") == 1 and not any(ch.isspace() for ch in item) for item in repositories):
                break
            stdout.write("must be owner/repo\n")
    try:
        write_initial_config(config_path, accounts, mode, repositories)
    except OSError as exc:
        raise SetupError(f"cannot write configuration: {exc}") from exc
    stdout.write("configuration created\n")
    return True
