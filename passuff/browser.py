"""Open a pull request URL in the platform browser."""

from __future__ import annotations

import subprocess
import sys


class BrowserError(Exception):
    pass


class SystemBrowserOpener:
    def open(self, url: str) -> None:
        command = ["open", url] if sys.platform == "darwin" else ["xdg-open", url]
        try:
            result = subprocess.run(command, capture_output=True, text=True, timeout=20)
        except (OSError, subprocess.SubprocessError) as exc:
            raise BrowserError("could not open browser: " + str(exc)) from exc
        if result.returncode != 0:
            raise BrowserError("could not open browser: " + (result.stderr.strip() or "opener failed"))
