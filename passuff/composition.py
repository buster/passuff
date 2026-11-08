"""Composition root for the CLI and the terminal inbox.

The adapters are named here, not inside the core. Kept intentionally small:
the CLI and TUI import this module for the default platform adapters.
"""

from __future__ import annotations

from .browser import SystemBrowserOpener
from .notify import DesktopDelivery
from .sound import SystemSoundPlayer


def openers():
    return SystemBrowserOpener(), DesktopDelivery(), SystemSoundPlayer()
