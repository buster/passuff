"""Command-line adapter and composition root."""

from __future__ import annotations

import argparse
import os
import sys
import time
from pathlib import Path
from typing import Any

from .browser import BrowserError, SystemBrowserOpener
from .config import AppConfig, ConfigError, load_config
from .core import PassuffCore
from .inbox import ActivityInboxService, WatchStatusReader
from .notify import DeliveryError, DesktopDelivery
from .render import digest, render_check, render_inbox, render_status
from .setup import SetupError, run_setup
from .sound import SoundError, SystemSoundPlayer
from .state import StateError, load_state, save_state

VERSION = "0.2.0"


def default_config_path() -> Path:
    base = os.environ.get("XDG_CONFIG_HOME")
    return (Path(base) if base else Path.home() / ".config") / "passuff" / "config.toml"


def default_state_path() -> Path:
    base = os.environ.get("XDG_STATE_HOME")
    return (Path(base) if base else Path.home() / ".local" / "state") / "passuff" / "state.json"


def _deliver(config: AppConfig, report, no_desktop: bool, no_sound: bool) -> None:
    payload = digest(report)
    if payload is None:
        return
    summary, body = payload
    if config.notifications.desktop and not no_desktop:
        try:
            DesktopDelivery(enabled=True, urgency=config.notifications.urgency).send(summary, body)
        except DeliveryError as exc:
            print(str(exc), file=sys.stderr)
    if config.notifications.sound and not no_sound:
        try:
            SystemSoundPlayer(enabled=True).play(config.notifications.sound_file)
        except SoundError as exc:
            print(str(exc), file=sys.stderr)


def _run_check(config: AppConfig, state_path: Path, no_desktop: bool, no_sound: bool, verbose: bool) -> int:
    state = load_state(state_path)
    core = PassuffCore(config, state, state_path, interactive=False)
    report = core.check_cycle()
    for line in render_check(report):
        print(line)
    _deliver(config, report, no_desktop, no_sound)
    return 1 if report.failures else 0


def _watch(config: AppConfig, state_path: Path, no_desktop: bool, no_sound: bool, verbose: bool, config_path: Path | None = None) -> int:
    print("passuff: watching", flush=True)
    state = load_state(state_path)
    interactive = sys.stdin.isatty()
    core = PassuffCore(config, state, state_path, interactive=interactive)
    try:
        while True:
            if config_path is not None:
                try:
                    core.config = load_config(config_path)
                except ConfigError as exc:
                    print(f"configuration error: {exc}", file=sys.stderr, flush=True)
            report = core.check_cycle()
            for line in render_check(report):
                print(line, flush=True)
            _deliver(config, report, no_desktop, no_sound)
            time.sleep(max(config.poll_interval_secs, report.next_delay))
    except KeyboardInterrupt:
        return 0


def _status(config: AppConfig, state_path: Path) -> int:
    state = load_state(state_path)
    for line in render_status(WatchStatusReader(config, state).view()):
        print(line)
    return 0


def _inbox(config: AppConfig, state_path: Path, include_read: bool) -> int:
    state = load_state(state_path)
    service = ActivityInboxService(config, state)
    reader = WatchStatusReader(config, state)
    rows = service.view(include_read=include_read)
    for line in render_inbox(rows, reader.unread(), len(reader.activity()), include_read):
        print(line)
    return 0


def _read(config: AppConfig, state_path: Path, identifier: str | None, all_records: bool) -> int:
    if all_records == bool(identifier):
        print("read requires exactly one activity id or --all")
        return 2
    state = load_state(state_path)
    service = ActivityInboxService(config, state)
    if all_records:
        changed = service.mark_all()
        save_state(state_path, state)
        print(f"{changed} activity {'record' if changed == 1 else 'records'} read")
        print(f"{service.unread_count()} unread of {len(service.view(include_read=True))}")
        return 0
    if not service.mark_read(str(identifier)):
        print(f"no activity with id {identifier}")
        return 2
    save_state(state_path, state)
    print("1 activity record read")
    print(f"{service.unread_count()} unread of {len(service.view(include_read=True))}")
    return 0


def _notify_test(config: AppConfig, no_desktop: bool, no_sound: bool) -> int:
    desktop = config.notifications.desktop and not no_desktop
    sound = config.notifications.sound and not no_sound
    if not desktop and not sound:
        print("desktop notifications are disabled")
        print("sound is disabled")
        print("nothing to test")
        return 2
    failed = False
    if desktop:
        try:
            DesktopDelivery(enabled=True, urgency=config.notifications.urgency).send(
                "If you can read this, passuff notifications are working.", ""
            )
            print("desktop notification sent")
        except DeliveryError as exc:
            print(str(exc))
            failed = True
    else:
        print("desktop notifications are disabled")
    if sound:
        try:
            SystemSoundPlayer(enabled=True).play(config.notifications.sound_file)
            print("sound played")
        except SoundError as exc:
            print(str(exc))
            failed = True
    else:
        print("sound is disabled")
    return 1 if failed else 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="passuff", description="Multi-account GitHub pull request inbox and notifier")
    parser.add_argument("--config", default=str(default_config_path()))
    parser.add_argument("--state", default=str(default_state_path()))
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--no-desktop", action="store_true")
    parser.add_argument("--no-sound", action="store_true")
    parser.add_argument("--version", action="version", version=f"passuff {VERSION}")
    sub = parser.add_subparsers(dest="command")
    def flags(p):
        p.add_argument("--no-desktop", action="store_true", default=argparse.SUPPRESS)
        p.add_argument("--no-sound", action="store_true", default=argparse.SUPPRESS)
        return p

    flags(sub.add_parser("check", help="run one observation pass"))
    flags(sub.add_parser("status", help="show accounts, scope, watched pull requests, and unread count"))
    inbox = flags(sub.add_parser("inbox", help="list unread activity"))
    inbox.add_argument("--all", action="store_true", dest="include_read")
    read = flags(sub.add_parser("read", help="mark activity read"))
    read.add_argument("id", nargs="?")
    read.add_argument("--all", action="store_true", dest="all_records")
    flags(sub.add_parser("setup", help="create the configuration interactively"))
    flags(sub.add_parser("notify-test", help="send a test notification and play the configured sound"))
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    config_path = Path(args.config).expanduser()
    state_path = Path(args.state).expanduser()
    try:
        if args.command == "setup":
            try:
                run_setup(config_path, sys.stdin, sys.stdout)
                return 0
            except SetupError as exc:
                print(str(exc))
                return 3 if "cannot write configuration" in str(exc) else 2
        config = load_config(config_path)
        if args.command == "status":
            return _status(config, state_path)
        if args.command == "inbox":
            return _inbox(config, state_path, args.include_read)
        if args.command == "read":
            return _read(config, state_path, args.id, args.all_records)
        if args.command == "notify-test":
            return _notify_test(config, args.no_desktop, args.no_sound)
        if args.command == "check":
            return _run_check(config, state_path, args.no_desktop, args.no_sound, args.verbose)
        return _watch(config, state_path, args.no_desktop, args.no_sound, args.verbose, config_path)
    except ConfigError as exc:
        print(f"configuration error: {exc}", file=sys.stderr)
        return 2
    except StateError as exc:
        print(f"state error: {exc}", file=sys.stderr)
        return 3


if __name__ == "__main__":
    raise SystemExit(main())
