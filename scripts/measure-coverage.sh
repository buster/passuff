#!/usr/bin/env bash
# Measure regular and E2E coverage as distinct instrumentation runs.
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
PY="$ROOT/.venv/bin/python"
[ -x "$PY" ] || PY="python3"

mkdir -p "$ROOT/coverage/non-e2e" "$ROOT/coverage/e2e"

export COVERAGE_PROCESS_START="$ROOT/.coveragerc"
export PASSUFF_CLI="$PY -m coverage run --parallel-mode -m passuff"
export PASSUFF_TUI="$PY -m coverage run --parallel-mode -m passuff.tui"

export COVERAGE_FILE="$ROOT/coverage/non-e2e/.coverage"
GIVN_RESULT_SCOPE=regular "$ROOT/scripts/verify.sh" >/dev/null || exit 1
"$PY" -m coverage combine "$ROOT/coverage/non-e2e" >/dev/null 2>&1 || true
"$PY" -m coverage xml -o "${GIVN_COVERAGE_NON_E2E_PATH:-$ROOT/coverage/non-e2e-cobertura.xml}" || exit 1

export COVERAGE_FILE="$ROOT/coverage/e2e/.coverage"
GIVN_RESULT_SCOPE=e2e "$ROOT/scripts/verify.sh" --e2e >/dev/null || exit 1
"$PY" -m coverage combine "$ROOT/coverage/e2e" >/dev/null 2>&1 || true
"$PY" -m coverage xml -o "${GIVN_COVERAGE_E2E_PATH:-$ROOT/coverage/e2e-cobertura.xml}" || exit 1
