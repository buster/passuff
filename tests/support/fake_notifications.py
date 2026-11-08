"""Fake org.freedesktop.Notifications service for the test session bus."""

from __future__ import annotations

import asyncio
import threading
from pathlib import Path

from .recording_delivery import RecordingDelivery


class FakeNotifications:
    def __init__(self, log_path: Path) -> None:
        self.log_path = Path(log_path)
        self.thread = None
        self.loop = None
        self.bus = None
        self.ready = threading.Event()

    def start(self) -> None:
        self.thread = threading.Thread(target=self._run, daemon=True)
        self.thread.start()
        self.ready.wait(timeout=5)

    def _run(self) -> None:
        from dbus_next.aio import MessageBus
        from dbus_next.service import ServiceInterface, method

        log_path = self.log_path
        delivery = RecordingDelivery(log_path)

        class Notifications(ServiceInterface):
            def __init__(self) -> None:
                super().__init__("org.freedesktop.Notifications")

            @method()
            def Notify(self, app_name: "s", replaces_id: "u", app_icon: "s", summary: "s", body: "s", actions: "as", hints: "a{sv}", expire_timeout: "i") -> "u":
                delivery.record(summary, body)
                return 1

            @method()
            def CloseNotification(self, identifier: "u") -> "":
                return None

            @method()
            def GetCapabilities(self) -> "as":
                return ["body"]

            @method()
            def GetServerInformation(self) -> "ssss":
                return ("passuff-fake", "passuff", "1.0", "1.2")

        async def run() -> None:
            from dbus_next.constants import RequestNameReply

            self.bus = await MessageBus().connect()
            self.bus.export("/org/freedesktop/Notifications", Notifications())
            reply = await self.bus.request_name("org.freedesktop.Notifications")
            while reply != RequestNameReply.PRIMARY_OWNER:
                await asyncio.sleep(0.1)
                reply = await self.bus.request_name("org.freedesktop.Notifications")
            self.ready.set()
            await asyncio.get_running_loop().create_future()

        try:
            self.loop = asyncio.new_event_loop()
            asyncio.set_event_loop(self.loop)
            self.loop.run_until_complete(run())
        except Exception:
            self.ready.set()

    def stop(self) -> None:
        if self.loop and self.bus:
            try:
                self.loop.call_soon_threadsafe(self.bus.release_name, "org.freedesktop.Notifications")
            except Exception:
                pass
            self.loop.call_soon_threadsafe(self.bus.disconnect)
        if self.loop:
            self.loop.call_soon_threadsafe(self.loop.stop)
        if self.thread:
            self.thread.join(timeout=2)
        import time as _time
        _time.sleep(0.5)
