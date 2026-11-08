"""Notification sound through a system player."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

PLAYERS = ("paplay", "pw-play", "aplay", "canberra-gtk-play")
MACOS_PLAYERS = ("afplay",)


class SoundError(Exception):
    pass


def default_sound() -> Path:
    return Path(__file__).resolve().parent.parent / "assets" / "notification.wav"


class SystemSoundPlayer:
    def __init__(self, *, enabled: bool = True) -> None:
        self.enabled = enabled

    def play(self, sound_file: str = "default") -> None:
        if not self.enabled:
            return
        path = default_sound() if sound_file == "default" else Path(sound_file).expanduser()
        player = None
        candidates = MACOS_PLAYERS if sys.platform == "darwin" else PLAYERS
        for name in candidates:
            if shutil.which(name):
                player = name
                break
        if player is None:
            raise SoundError("sound unavailable")
        try:
            result = subprocess.run([player, str(path)], capture_output=True, text=True, timeout=20)
        except (OSError, subprocess.SubprocessError) as exc:
            raise SoundError(f"sound playback failed: {exc}") from exc
        if result.returncode != 0:
            raise SoundError("sound playback failed")
