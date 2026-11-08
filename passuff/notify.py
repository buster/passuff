"""Platform notification delivery."""

from __future__ import annotations

import json
import os
import subprocess
import sys


class DeliveryError(Exception):
    pass


class DesktopDelivery:
    def __init__(self, *, enabled: bool = True, urgency: str = "normal") -> None:
        self.enabled = enabled
        self.urgency = urgency

    def send(self, summary: str, body: str) -> None:
        if not self.enabled:
            return
        if os.environ.get("PASSUFF_DESKTOP_UNAVAILABLE") == "1":
            raise DeliveryError("desktop notification failed: unavailable")
        if sys.platform == "darwin":
            self._send_macos(summary, body)
        else:
            self._send_linux(summary, body)

    def _send_linux(self, summary: str, body: str) -> None:
        try:
            from jeepney import DBusAddress, new_method_call
            from jeepney.io.blocking import open_dbus_connection
        except ImportError as exc:
            raise DeliveryError("desktop notification failed: jeepney is unavailable") from exc
        address = DBusAddress(
            "/org/freedesktop/Notifications",
            bus_name="org.freedesktop.Notifications",
            interface="org.freedesktop.Notifications",
        )
        urgency = {"low": 0, "normal": 1, "critical": 2}.get(self.urgency, 1)
        hints = {"urgency": ("y", urgency)}
        try:
            connection = open_dbus_connection(bus="SESSION")
            message = new_method_call(
                address,
                "Notify",
                "susssasa{sv}i",
                ("passuff", 0, "", summary, body, [], hints, 5000),
            )
            connection.send_and_get_reply(message)
            connection.close()
        except Exception as exc:  # jeepney raises several transport errors
            raise DeliveryError(f"desktop notification failed: {exc}") from exc

    def _send_macos(self, summary: str, body: str) -> None:
        script = f'display notification {json.dumps(body)} with title {json.dumps(summary)}'
        try:
            result = subprocess.run(["osascript", "-e", script], capture_output=True, text=True, timeout=20)
        except (OSError, subprocess.SubprocessError) as exc:
            raise DeliveryError(f"desktop notification failed: {exc}") from exc
        if result.returncode != 0:
            raise DeliveryError("desktop notification failed: " + (result.stderr.strip() or "osascript failed"))
