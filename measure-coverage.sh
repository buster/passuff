#!/usr/bin/env bash
# Measure two coverage runs: regular (non-@e2e) and @e2e.
#
# givn supplies GIVN_COVERAGE_NON_E2E_PATH, GIVN_COVERAGE_E2E_PATH and
# GIVN_FEATURES. The regular scope runs the component corpus. The @e2e
# scenarios bind to the frozen package's step definitions, so the component
# runner cannot execute them: the E2E scope selects no scenario and records the
# zero-line measurement as TD-008, unless GIVN_COVERAGE_E2E_FEATURES names a
# component-tier corpus explicitly. Both reports are written from distinct
# instrumentation runs.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$root"

export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0

toolchain="${COVERAGE_TOOLCHAIN:-nightly}"

non_e2e="${GIVN_COVERAGE_NON_E2E_PATH:-$root/coverage/non-e2e-cobertura.xml}"
e2e="${GIVN_COVERAGE_E2E_PATH:-$root/coverage/e2e-cobertura.xml}"
mkdir -p "$(dirname "$non_e2e")" "$(dirname "$e2e")"

features="${GIVN_FEATURES:-givn/changes}"
case "$features" in
  /*) ;;
  *) features="$root/$features" ;;
esac

run_scope() {
  local scope="$1" output="$2"
  local scope_features="$features"
  local placeholder=""
  if [[ "$scope" == "e2e" ]]; then
    if [[ -n "${GIVN_COVERAGE_E2E_FEATURES:-}" ]]; then
      scope_features="$GIVN_COVERAGE_E2E_FEATURES"
    else
      placeholder="$(mktemp -d "${TMPDIR:-/tmp}/passuff-coverage-e2e.XXXXXX")"
      cat >"$placeholder/placeholder.feature" <<'FEATURE'
Feature: E2E coverage placeholder
  Scenario: The component tier runs no acceptance scenario
    Given the coverage scope excludes e2e
FEATURE
      scope_features="$placeholder"
    fi
  fi
  cargo "+$toolchain" llvm-cov clean --profraw-only >/dev/null
  if [[ "$scope" == "regular" ]]; then
    cargo "+$toolchain" llvm-cov --no-report --branch \
      -p passuff-config -p passuff-core -p passuff \
      --features test-support --lib
  fi
  GIVN_FEATURES="$scope_features" \
  GIVN_RESULT_SCOPE="$scope" \
  GIVN_RESULT_FILE="$root/target/coverage-$scope-result.json" \
    cargo "+$toolchain" llvm-cov --no-report --branch \
      -p passuff --features test-support --test features_runner
  local json="$root/target/coverage-$scope.json"
  cargo "+$toolchain" llvm-cov report --json --output-path "$json"
  python3 "$root/scripts/json_to_cobertura.py" "$json" "$output"
  if [[ -n "$placeholder" ]]; then
    rm -rf "$placeholder"
  fi
}

run_scope regular "$non_e2e"
run_scope e2e "$e2e"

if [[ -z "${KEEP_COVERAGE_TARGET:-}" ]]; then
  cargo "+$toolchain" llvm-cov clean --workspace >/dev/null
fi
