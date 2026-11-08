"""Live probe and reconcile schedule for the terminal inbox."""

from __future__ import annotations

import sys
import threading
import time

from ..core import PassuffCore
from ..notify import DeliveryError, DesktopDelivery
from ..render import digest
from ..sound import SoundError, SystemSoundPlayer
from ..state import load_state


class ObservationSession:
    def __init__(self, config, state_path, *, interactive: bool) -> None:
        self.config = config
        self.state_path = state_path
        self.interactive = interactive
        state = load_state(state_path)
        self.core = PassuffCore(config, state, state_path, interactive=interactive)

    def deliver(self, report) -> None:
        payload = digest(report)
        if payload is None:
            return
        summary, body = payload
        if self.config.notifications.desktop:
            try:
                DesktopDelivery(enabled=True, urgency=self.config.notifications.urgency).send(summary, body)
            except DeliveryError as exc:
                print(str(exc), file=sys.stderr)
        if self.config.notifications.sound:
            try:
                SystemSoundPlayer(enabled=True).play(self.config.notifications.sound_file)
            except SoundError as exc:
                print(str(exc), file=sys.stderr)

    def cycle(self):
        report = self.core.check_cycle()
        self.deliver(report)
        return report

    def run_once(self) -> None:
        self.cycle()

    def run_for(self, duration_ms: int | None = None) -> None:
        if duration_ms is None:
            self.run_once()
            return
        deadline = time.monotonic() + duration_ms / 1000.0
        while True:
            self.cycle()
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return
            time.sleep(min(self.config.poll_interval_secs, remaining))

    def start(self, stop: threading.Event, on_report=None) -> threading.Thread:
        thread = threading.Thread(target=self._worker, args=(stop, on_report), daemon=True)
        thread.start()
        return thread

    def _worker(self, stop: threading.Event, on_report) -> None:
        while not stop.is_set():
            try:
                report = self.cycle()
                delay = max(self.config.poll_interval_secs, report.next_delay)
                if on_report is not None:
                    on_report(report)
            except Exception:
                delay = self.config.poll_interval_secs
            stop.wait(delay)
