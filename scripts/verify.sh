#!/usr/bin/env bash
# Gherkin runner for the givn verify contract.
#
#   scripts/verify.sh          formatting, clippy, library tests, regular scope
#   scripts/verify.sh --e2e    only @e2e scenarios
#
# givn passes GIVN_FEATURES and GIVN_RESULT_SCOPE; both are honoured. The
# feature path is made absolute because cargo runs the test binary from the
# crate directory. The static gates run in the regular scope only, so the e2e
# invocation stays a single test pass.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0

scope="${GIVN_RESULT_SCOPE:-change}"
if [[ "${1:-}" == "--e2e" ]]; then
  scope="e2e"
fi
export GIVN_RESULT_SCOPE="$scope"

features="${GIVN_FEATURES:-}"
if [[ -z "$features" ]]; then
  if [[ "$scope" == "e2e" ]]; then
    features="$root/givn/specs"
  elif compgen -G "$root/givn/changes/*/specs" >/dev/null; then
    features="$root/givn/changes"
  else
    features="$root/givn/specs"
  fi
fi
case "$features" in
  /*) ;;
  *) features="$root/$features" ;;
esac
if [[ "$scope" == "e2e" ]] && ! grep -Rqs --include='*.feature' '@e2e' "$features"; then
  features="$root/givn/specs"
fi
export GIVN_FEATURES="$features"

result_file="${GIVN_RESULT_FILE:-$root/target/givn-verify-result.json}"
mkdir -p "$(dirname "$result_file")"
rm -f "$result_file"
export GIVN_RESULT_FILE="$result_file"

if [[ "$scope" != "e2e" ]]; then
  cargo fmt --all -- --check
  cargo clippy --locked --all-targets -- -D warnings
  cargo clippy --locked -p passuff --all-targets --features test-support -- -D warnings
  cargo test --locked --workspace --lib
fi

cargo test -p passuff --features test-support --test features_runner
