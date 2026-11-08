#!/usr/bin/env bash
# Merge the two coverage data sets into the output Cobertura report.
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
PY="$ROOT/.venv/bin/python"
[ -x "$PY" ] || PY="python3"

mkdir -p "$ROOT/coverage/merged"
cp "$ROOT/coverage/non-e2e/.coverage" "$ROOT/coverage/merged/.coverage.non" 2>/dev/null || true
cp "$ROOT/coverage/e2e/.coverage" "$ROOT/coverage/merged/.coverage.e2e" 2>/dev/null || true

export COVERAGE_FILE="$ROOT/coverage/merged/.coverage"
"$PY" -m coverage combine "$ROOT/coverage/merged" >/dev/null 2>&1 || true
"$PY" -m coverage xml -o "${GIVN_COVERAGE_OUTPUT_PATH:-$ROOT/coverage/cobertura-coverage.xml}" || exit 1
