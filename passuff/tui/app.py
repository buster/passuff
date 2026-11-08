"""Terminal inbox adapter."""

from __future__ import annotations

import argparse
import os
import sys
import threading
import time
from pathlib import Path
from typing import Any

from ..browser import BrowserError, SystemBrowserOpener
from ..config import ConfigError, RELATIONS, load_config, update_ignored
from ..core import PassuffCore
from ..inbox import ActivityInboxService, WatchStatusReader, parse_time
from ..notify import DeliveryError, DesktopDelivery
from ..render import digest
from ..scope import ScopeEditor
from ..setup import SetupError, run_setup
from ..sound import SoundError, SystemSoundPlayer
from ..state import StateError, load_state, load_ui, save_state, save_ui
from .frame import render

KEYS = {
    "comma": ",",
    "space": " ",
    "tab": "tab",
    "backtab": "backtab",
    "enter": "enter",
    "esc": "esc",
    "left": "left",
    "right": "right",
    "pageup": "pageup",
    "pagedown": "pagedown",
    "home": "home",
    "end": "end",
    "down": "down",
    "up": "up",
}


def _view(config, ui: dict[str, Any]) -> dict[str, Any]:
    return {
        "focus": "activity",
        "pr_index": 0,
        "activity_index": 0,
        "sort": ui.get("sort", "updated"),
        "sort_direction": int(ui.get("sort_direction", 1)),
        "filter": ui.get("filter"),
        "filter_stack": False,
        "unread_only": bool(ui.get("unread_only", False)),
        "show_ignored": bool(ui.get("show_ignored", False)),
        "desktop": bool(ui.get("desktop", config.notifications.desktop)),
        "sound": bool(ui.get("sound", config.notifications.sound)),
        "overlay": None,
        "detail_scroll": 0,
        "scroll_end": False,
        "picker_mode": config.watch.mode,
        "picker_cursor": 0,
        "settings_index": 0,
        "status": None,
    }


def _filtered_activity(config, state, view) -> list[dict[str, Any]]:
    service = ActivityInboxService(config, state)
    reader = WatchStatusReader(config, state)
    prs = reader.watched()
    activities = service.view(include_read=True)
    ignored = {item.casefold() for item in config.watch.ignored_pull_requests}
    if view.get("filter"):
        key = view["filter"]
        if view.get("filter_stack"):
            stack = next((pr.get("stack") for pr in prs if pr.get("key") == key), None)
            activities = [
                record for record in activities
                if (state.get("pull_requests", {}).get(record.get("pull_request")) or {}).get("stack", {})
                and (state.get("pull_requests", {}).get(record.get("pull_request")) or {})["stack"].get("number") == (stack or {}).get("number")
            ]
        else:
            activities = [record for record in activities if record.get("pull_request") == key]
    if view.get("unread_only"):
        activities = [record for record in activities if not record.get("read")]
    if not view.get("show_ignored"):
        activities = [record for record in activities if record.get("pull_request", "").casefold() not in ignored]
    return activities


def _prs(config, state, view) -> list[dict[str, Any]]:
    reader = WatchStatusReader(config, state)
    prs = reader.watched()
    return prs


def _selected_key(config, state, view) -> str | None:
    prs = _prs(config, state, view)
    if not prs:
        return None
    return prs[min(view.get("pr_index", 0), len(prs) - 1)].get("key")


def _browser_open(url: str, view: dict[str, Any]) -> None:
    try:
        SystemBrowserOpener().open(url)
        view["status"] = None
        view["overlay"] = None
    except BrowserError as exc:
        view["status"] = str(exc)


def apply_key(key: str, config, state_path, config_path, view: dict[str, Any]) -> None:
    key = KEYS.get(key, key)
    state = load_state(state_path)
    prs = _prs(config, state, view)
    records = _filtered_activity(config, state, view)
    if key in {"tab", "backtab"}:
        view["focus"] = "activity" if view.get("focus") == "prs" else "prs"
        return
    if key in {"j", "down"}:
        if view.get("overlay") in {"activity_detail", "pr_detail"}:
            view["detail_scroll"] = int(view.get("detail_scroll", 0)) + 1
            view["scroll_end"] = False
        elif view.get("overlay") == "picker":
            view["picker_cursor"] = int(view.get("picker_cursor", 0)) + 1
        elif view.get("overlay") == "settings":
            view["settings_index"] = min(1, int(view.get("settings_index", 0)) + 1)
        elif view.get("focus") == "prs":
            view["pr_index"] = min(len(prs) - 1, int(view.get("pr_index", 0)) + 1) if prs else 0
        else:
            view["activity_index"] = min(len(records) - 1, int(view.get("activity_index", 0)) + 1) if records else 0
        return
    if key in {"k", "up"}:
        if view.get("overlay") in {"activity_detail", "pr_detail"}:
            view["detail_scroll"] = max(0, int(view.get("detail_scroll", 0)) - 1)
            view["scroll_end"] = False
        elif view.get("overlay") == "picker":
            view["picker_cursor"] = max(0, int(view.get("picker_cursor", 0)) - 1)
        elif view.get("focus") == "prs":
            view["pr_index"] = max(0, int(view.get("pr_index", 0)) - 1)
        else:
            view["activity_index"] = max(0, int(view.get("activity_index", 0)) - 1)
        return
    if key == "pageup":
        view["detail_scroll"] = max(0, int(view.get("detail_scroll", 0)) - 10)
        view["scroll_end"] = False
        return
    if key == "pagedown":
        view["detail_scroll"] = int(view.get("detail_scroll", 0)) + 10
        view["scroll_end"] = False
        return
    if key == "home":
        if view.get("overlay") in {"activity_detail", "pr_detail"}:
            view["detail_scroll"] = 0
            view["scroll_end"] = False
        elif view.get("focus") == "prs":
            view["pr_index"] = 0
        else:
            view["activity_index"] = 0
        return
    if key == "end":
        if view.get("overlay") in {"activity_detail", "pr_detail"}:
            view["scroll_end"] = True
            view["detail_scroll"] = 0
        elif view.get("focus") == "prs":
            view["pr_index"] = max(0, len(prs) - 1)
        else:
            view["activity_index"] = max(0, len(records) - 1)
        return
    if key == "enter":
        overlay = view.get("overlay")
        if overlay == "activity_detail":
            record = records[min(view.get("activity_index", 0), len(records) - 1)] if records else None
            if record:
                _browser_open("https://github.com/" + record.get("pull_request", ""), view)
            return
        if overlay == "pr_detail":
            keyed = _selected_key(config, state, view)
            if keyed:
                _browser_open("https://github.com/" + keyed, view)
            return
        if view.get("focus") == "prs" and prs:
            view["overlay"] = "pr_detail"
            view["detail_scroll"] = 0
            view["scroll_end"] = False
        elif records:
            view["overlay"] = "activity_detail"
            view["detail_scroll"] = 0
            view["scroll_end"] = False
        return
    if key in {"esc", "q"}:
        if view.get("overlay"):
            view["overlay"] = None
            view["status"] = None
            return
        if key == "q":
            view["quit"] = True
        return
    if key == "r":
        if records:
            record = records[min(view.get("activity_index", 0), len(records) - 1)]
            ActivityInboxService(config, state).mark_read(record["id"])
            save_state(state_path, state)
        return
    if key == "a":
        service = ActivityInboxService(config, state)
        for record in records:
            if not record.get("read"):
                service.mark_read(record["id"])
        save_state(state_path, state)
        return
    if key == "x":
        keyed = _selected_key(config, state, view)
        if keyed:
            ignored = list(config.watch.ignored_pull_requests)
            if keyed.casefold() in {item.casefold() for item in ignored}:
                ignored = [item for item in ignored if item.casefold() != keyed.casefold()]
            else:
                ignored.append(keyed)
            try:
                update_ignored(config_path, ignored)
                config.watch.ignored_pull_requests = ignored
            except ConfigError as exc:
                view["status"] = str(exc)
        return
    if key in {"s", "S"}:
        modes = ["updated", "name", "state", "age"]
        if key == "S":
            view["sort_direction"] = -int(view.get("sort_direction", -1))
        else:
            current = view.get("sort", "updated")
            view["sort"] = modes[(modes.index(current) + 1) % len(modes)] if current in modes else "updated"
            view["sort_direction"] = 1
        return
    if key in {"f", "F"}:
        keyed = _selected_key(config, state, view)
        if keyed:
            view["filter"] = keyed
            view["filter_stack"] = key == "F"
        return
    if key == "u":
        view["unread_only"] = not view.get("unread_only", False)
        return
    if key == "c":
        view["filter"] = None
        view["filter_stack"] = False
        view["unread_only"] = False
        return
    if key == "C":
        view.update({"filter": None, "filter_stack": False, "unread_only": False, "sort": "updated", "sort_direction": 1, "show_ignored": False})
        return
    if key == ",":
        view["overlay"] = "settings"
        return
    if key == "p":
        view["overlay"] = "picker"
        view["picker_mode"] = config.watch.mode
        return
    if key in {"h", "H", "F1"}:
        view["overlay"] = "help"
        return
    if key == "left":
        view["show_ignored"] = False
        return
    if key == "right":
        view["show_ignored"] = True
        return
    if key in {" ", "space"}:
        if view.get("overlay") == "settings":
            if view.get("settings_index", 0) == 0:
                view["desktop"] = not view.get("desktop", True)
            else:
                view["sound"] = not view.get("sound", True)
            save_ui(Path(config_path).with_name("ui.json"), {
                "sort": view.get("sort", "updated"),
                "sort_direction": view.get("sort_direction", 1),
                "filter": view.get("filter"),
                "unread_only": view.get("unread_only", False),
                "show_ignored": view.get("show_ignored", False),
                "desktop": view.get("desktop", True),
                "sound": view.get("sound", True),
            })
        elif view.get("overlay") == "picker":
            editor = ScopeEditor(config, config_path)
            known = sorted({pr.get("key", "").split("#", 1)[0] for pr in state.get("pull_requests", {}).values()} | set(config.watch.repositories))
            cursor = int(view.get("picker_cursor", 0))
            if cursor < len(known):
                repo = known[cursor]
                try:
                    if repo.casefold() in {item.casefold() for item in config.watch.repositories}:
                        editor.deselect(repo)
                    else:
                        editor.select(repo)
                except ConfigError as exc:
                    view["status"] = str(exc)
            else:
                index = cursor - len(known)
                if index < len(RELATIONS):
                    relation = RELATIONS[index]
                    relations = list(config.focus.relations)
                    if relation in relations:
                        relations.remove(relation)
                    else:
                        relations.append(relation)
                    try:
                        editor.set_focus(relations)
                    except ConfigError as exc:
                        view["status"] = str(exc)
        return
    if key == "m":
        if view.get("overlay") == "picker":
            try:
                mode = "selected" if config.watch.mode == "all" else "all"
                ScopeEditor(config, config_path).set_mode(mode)
                view["picker_mode"] = mode
            except ConfigError as exc:
                view["status"] = str(exc)
        return


def _deliver(config, report) -> None:
    payload = digest(report)
    if payload is None:
        return
    summary, body = payload
    if config.notifications.desktop:
        try:
            DesktopDelivery(enabled=True, urgency=config.notifications.urgency).send(summary, body)
        except DeliveryError as exc:
            print(str(exc), file=sys.stderr)
    if config.notifications.sound:
        try:
            SystemSoundPlayer(enabled=True).play(config.notifications.sound_file)
        except SoundError as exc:
            print(str(exc), file=sys.stderr)


def _observe(config, state_path, view, duration_ms: int | None = None) -> None:
    state = load_state(state_path)
    core = PassuffCore(config, state, state_path, interactive=False)
    deadline = time.monotonic() + (duration_ms or 0) / 1000.0
    while True:
        report = core.check_cycle()
        _deliver(config, report)
        if duration_ms is None:
            return
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return
        time.sleep(min(config.poll_interval_secs, remaining))


def _interactive(config, state_path, config_path, view) -> int:
    import curses

    stop = threading.Event()
    lock = threading.Lock()

    def worker():
        state = load_state(state_path)
        core = PassuffCore(config, state, state_path, interactive=True)
        while not stop.is_set():
            try:
                report = core.check_cycle()
                _deliver(config, report)
                delay = max(config.poll_interval_secs, report.next_delay)
            except Exception:
                delay = config.poll_interval_secs
            stop.wait(delay)

    thread = threading.Thread(target=worker, daemon=True)
    thread.start()

    def loop(stdscr):
        curses.curs_set(0)
        stdscr.nodelay(True)
        while not view.get("quit"):
            state = load_state(state_path)
            lines = render(config, state, view)
            height, width = stdscr.getmaxyx()
            stdscr.erase()
            for row, line in enumerate(lines[: max(0, height - 1)]):
                try:
                    stdscr.addnstr(row, 0, line, max(1, width - 1))
                except curses.error:
                    pass
            stdscr.refresh()
            ch = stdscr.getch()
            if ch == -1:
                time.sleep(0.05)
                continue
            try:
                key = chr(ch) if 0 <= ch < 256 else str(ch)
            except ValueError:
                key = ""
            apply_key(key, config, state_path, config_path, view)
        stop.set()

    try:
        curses.wrapper(loop)
    except KeyboardInterrupt:
        stop.set()
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="passuff-tui", description="Terminal inbox for passuff")
    parser.add_argument("--config", default=None)
    parser.add_argument("--state", default=None)
    parser.add_argument("--snapshot", action="store_true")
    parser.add_argument("--observe", action="store_true")
    parser.add_argument("--observe-for", type=int, default=None, dest="observe_for")
    parser.add_argument("--scope-select", action="append", default=[], dest="scope_select")
    parser.add_argument("--scope-deselect", action="append", default=[], dest="scope_deselect")
    parser.add_argument("--scope-mode", default=None, dest="scope_mode")
    parser.add_argument("--focus-set", default=None, dest="focus_set")
    parser.add_argument("--picker", action="store_true")
    parser.add_argument("--keys", default=None)
    parser.add_argument("--no-desktop", action="store_true")
    parser.add_argument("--no-sound", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    config_path = Path(args.config).expanduser() if args.config else Path(os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config")) / "passuff" / "config.toml"
    state_path = Path(args.state).expanduser() if args.state else Path(os.environ.get("XDG_STATE_HOME", Path.home() / ".local" / "state")) / "passuff" / "state.json"
    if not config_path.exists():
        if args.snapshot or not (sys.stdin.isatty() and sys.stdout.isatty()):
            print("passuff setup")
            return 2
        try:
            run_setup(config_path, sys.stdin, sys.stdout)
        except SetupError as exc:
            print(str(exc))
            return 2
    try:
        config = load_config(config_path)
    except ConfigError as exc:
        print(f"configuration error: {exc}", file=sys.stderr)
        return 2
    try:
        load_state(state_path)
    except StateError as exc:
        print(f"state error: {exc}", file=sys.stderr)
        return 3
    editor = ScopeEditor(config, config_path)
    try:
        for repo in args.scope_select:
            editor.select(repo)
        for repo in args.scope_deselect:
            editor.deselect(repo)
        if args.scope_mode:
            editor.set_mode(args.scope_mode)
        if args.focus_set:
            editor.set_focus([item.strip() for item in args.focus_set.split(",") if item.strip()])
    except ConfigError as exc:
        print(str(exc), file=sys.stderr)
        return 2
    ui = load_ui(state_path.with_name("ui.json"))
    if args.no_desktop:
        ui["desktop"] = False
    if args.no_sound:
        ui["sound"] = False
    view = _view(config, ui)
    if args.picker:
        view["overlay"] = "picker"
        view["picker_mode"] = config.watch.mode
    if args.observe or args.observe_for is not None:
        _observe(config, state_path, view, args.observe_for)
    if args.keys:
        valid = {"tab", "backtab", "j", "down", "k", "up", "r", "a", "q", "enter", "esc", "left", "right", "pageup", "pagedown", "home", "end", "x", "comma", "space", "s", "S", "f", "F", "u", "c", "C", "h", "H", "F1", "p", "m"}
        tokens = [item.strip() for item in args.keys.split(",")]
        for token in tokens:
            if token not in valid:
                print("invalid key", file=sys.stderr)
                return 2
        for key in tokens:
            apply_key(key, config, state_path, config_path, view)
    if args.snapshot:
        state = load_state(state_path)
        print("\n".join(render(config, state, view)))
        return 0
    if not (sys.stdin.isatty() and sys.stdout.isatty()):
        print("requires a terminal")
        return 3
    save_ui(state_path.with_name("ui.json"), {
        "sort": view.get("sort", "updated"),
        "sort_direction": view.get("sort_direction", 1),
        "filter": view.get("filter"),
        "unread_only": view.get("unread_only", False),
        "show_ignored": view.get("show_ignored", False),
        "desktop": view.get("desktop", True),
        "sound": view.get("sound", True),
    })
    return _interactive(config, state_path, config_path, view)


if __name__ == "__main__":
    raise SystemExit(main())
