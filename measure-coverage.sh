#!/usr/bin/env bash
# Measure two coverage runs: regular (non-@e2e) and @e2e.
#
# givn supplies GIVN_COVERAGE_NON_E2E_PATH, GIVN_COVERAGE_E2E_PATH and
# GIVN_FEATURES. Both reports are written from distinct instrumentation runs.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$root"

export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0

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
  cargo llvm-cov clean --workspace >/dev/null
  GIVN_FEATURES="$features" \
  GIVN_RESULT_SCOPE="$scope" \
  GIVN_RESULT_FILE="$root/target/coverage-$scope-result.json" \
    cargo llvm-cov --no-report \
      -p passuff-cli --features test-support --test features_runner
  cargo llvm-cov report --format cobertura --output-path "$output"
}

run_scope regular "$non_e2e"
run_scope e2e "$e2e"
