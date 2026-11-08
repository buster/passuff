#!/usr/bin/env python3
"""Merge behave JSON outputs into the four givn counts."""

import json
import sys

COUNTS = {"total": 0, "passed": 0, "failed": 0, "skipped": 0}


def walk(feature):
    for element in feature.get("elements", []):
        if element.get("type") != "scenario":
            continue
        COUNTS["total"] += 1
        status = element.get("status", "failed")
        if status == "passed":
            COUNTS["passed"] += 1
        elif status == "skipped":
            COUNTS["skipped"] += 1
        else:
            COUNTS["failed"] += 1


for path in sys.argv[1:]:
    try:
        with open(path, encoding="utf-8") as handle:
            data = json.load(handle)
    except (OSError, json.JSONDecodeError):
        continue
    if isinstance(data, list):
        for feature in data:
            walk(feature)
    elif isinstance(data, dict):
        walk(data)

for key, value in COUNTS.items():
    print(f"{key}={value}")

if COUNTS["total"] == 0 or COUNTS["failed"] or COUNTS["skipped"]:
    sys.exit(1)
sys.exit(0)
