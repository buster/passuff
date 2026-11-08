"""Recording delivery double for adapter tests and the fake notification service."""

from __future__ import annotations

import json
from pathlib import Path


class RecordingDelivery:
    def __init__(self, path: Path) -> None:
        self.path = Path(path)

    def record(self, summary: str, body: str) -> None:
        with open(self.path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps({"summary": summary, "body": body}) + "\n")

    def send(self, summary: str, body: str) -> None:
        self.record(summary, body)
