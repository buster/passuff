#!/usr/bin/env bash
# Givn runner for passuff. Executes the permanent corpus and the change delta.
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PY="$ROOT/.venv/bin/python"
[ -x "$PY" ] || PY="python3"

SCOPE="${GIVN_RESULT_SCOPE:-regular}"
E2E=0
NAME=""
STRICT=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --e2e) E2E=1; SCOPE=e2e ;;
    --name) NAME="${2:-}"; shift ;;
    --strictness) STRICT=1 ;;
    *) echo "verify: unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

if [ "$(id -u)" = 0 ] && [ "${PASSUFF_ALLOW_ROOT:-0}" != 1 ]; then
  echo "verify: permission-failure fixtures require a non-root user" >&2
  exit 2
fi

if [ "$STRICT" = 1 ]; then
  chmod +x "$ROOT/scripts/verify.sh" 2>/dev/null || true
  fail=0
  echo "probe: undefined step"
  "$PY" -m behave -f progress tests/strictness/undefined.feature
  [ "$?" -eq 0 ] && fail=1
  echo "probe: skipped scenario"
  "$PY" -m behave -f json -o /tmp/passuff-strictness-skip.json tests/strictness/skipped.feature >/dev/null 2>&1
  "$PY" "$ROOT/scripts/behave_counts.py" /tmp/passuff-strictness-skip.json
  [ "$?" -eq 0 ] && fail=1
  echo "probe: pass-only step body"
  "$PY" "$ROOT/scripts/check_steps.py" --path tests/strictness/pass_only.py
  [ "$?" -eq 0 ] && fail=1
  exit "$fail"
fi

"$PY" "$ROOT/scripts/check_steps.py" --path "$ROOT/givn/steps" --path "$ROOT/givn/e2e_steps" || exit 1

shopt -s nullglob
ACTIVE=("$ROOT"/givn/changes/*/)
if [ -n "${GIVN_CHANGE_ID:-}" ]; then
  export GIVN_CHANGE_ID
elif [ ${#ACTIVE[@]} -eq 1 ]; then
  export GIVN_CHANGE_ID="$(basename "${ACTIVE[0]%/}")"
else
  export GIVN_CHANGE_ID=""
fi
PERM=("$ROOT"/givn/specs/*/*.feature)
DELTA=("$ROOT"/givn/changes/*/specs/*/*.feature)
EXTRA=()
if [ -n "${GIVN_FEATURES:-}" ]; then
  for f in $(printf '%s' "$GIVN_FEATURES" | tr ':\n' '  '); do
    [ -f "$f" ] && EXTRA+=("$f")
  done
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
JSONS=()
RUN=0
NAME_ARGS=()
[ -n "$NAME" ] && NAME_ARGS=(-n "$NAME")

run_behave() {
  local out="$TMP/run$RUN.json"
  RUN=$((RUN + 1))
  JSONS+=("$out")
  "$PY" -m behave -f json -o "$out" --no-summary "${NAME_ARGS[@]}" "$@" || true
}

# E2E scope: permanent and delta @e2e scenarios through the staged tree.
if [ "$E2E" = 1 ] || [ "$SCOPE" = "e2e" ]; then
  [ ${#PERM[@]} -gt 0 ] && run_behave --stage e2e --tags="@e2e" "${PERM[@]}"
  [ ${#DELTA[@]} -gt 0 ] && run_behave --stage e2e --tags="@e2e" "${DELTA[@]}"
  [ ${#EXTRA[@]} -gt 0 ] && run_behave --stage e2e --tags="@e2e" "${EXTRA[@]}"
elif [ "$SCOPE" = "regular" ]; then
  [ ${#PERM[@]} -gt 0 ] && run_behave --tags="not @e2e" "${PERM[@]}"
  [ ${#EXTRA[@]} -gt 0 ] && run_behave --tags="not @e2e" "${EXTRA[@]}"
else
  # change: every delta feature first, then the permanent non-E2E corpus.
  [ ${#DELTA[@]} -gt 0 ] && run_behave "${DELTA[@]}"
  [ ${#EXTRA[@]} -gt 0 ] && run_behave "${EXTRA[@]}"
  [ ${#PERM[@]} -gt 0 ] && run_behave --tags="not @e2e" "${PERM[@]}"
fi

if [ ${#JSONS[@]} -eq 0 ]; then
  echo "verify: no feature files selected" >&2
  exit 2
fi

set +e
COUNTS="$("$PY" "$ROOT/scripts/behave_counts.py" "${JSONS[@]}")"
RC=$?
set -e
printf '%s\n' "$COUNTS"
if [ -n "${GIVN_RESULT_FILE:-}" ]; then
  eval "$(printf '%s\n' "$COUNTS" | grep -E '^(total|passed|failed|skipped)=')"
  givn test-result --total "$total" --passed "$passed" --failed "$failed" --skipped "$skipped" || true
fi
exit "$RC"
