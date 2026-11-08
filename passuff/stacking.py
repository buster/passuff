"""Stack grouping for the terminal inbox."""

from __future__ import annotations

from typing import Any


class StackGrouper:
    def group(self, pull_requests: list[dict[str, Any]]) -> list[dict[str, Any]]:
        groups: dict[tuple[str, int], list[dict[str, Any]]] = {}
        standalone: list[dict[str, Any]] = []
        for pr in pull_requests:
            stack = pr.get("stack")
            if not stack:
                standalone.append(pr)
                continue
            repo = pr.get("key", "").split("#", 1)[0]
            groups.setdefault((repo, int(stack.get("number", 0))), []).append(pr)
        result = []
        for (repo, number), members in groups.items():
            members.sort(key=lambda pr: int((pr.get("stack") or {}).get("position", 0)))
            result.append({"kind": "stack", "repo": repo, "number": number, "members": members})
        for pr in standalone:
            result.append({"kind": "standalone", "pr": pr})
        result.sort(key=lambda item: (item.get("repo") or item["pr"].get("key", "")))
        return result
