#!/usr/bin/env bash
# Gherkin runner for the givn verify contract.
#
#   scripts/verify.sh          formatting, clippy, library tests, component tier
#   scripts/verify.sh --e2e    frozen black-box package against the built binaries
#
# givn passes GIVN_FEATURES and GIVN_RESULT_SCOPE; both are honoured in the
# component scope. The e2e scope always runs every `@e2e` scenario in
# `givn/specs` through the frozen `passuff-e2e` package: it is the acceptance
# authority and must not be narrowed to a change's feature files. The component
# runner is the in-process tier; its feature path is made absolute because cargo
# runs the test binary from the crate directory. Static gates run in the
# component scope only, so the e2e invocation stays a single test pass.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0

scope="${GIVN_RESULT_SCOPE:-change}"
package_dir="crates/passuff-e2e"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --e2e)
      scope="e2e"
      ;;
    --package)
      package_dir="${2:?--package needs a project-relative package directory}"
      shift
      ;;
    *)
      echo "verify: unknown argument: $1" >&2
      exit 2
      ;;
  esac
  shift
done
export GIVN_RESULT_SCOPE="$scope"
package_manifest="$root/$package_dir/Cargo.toml"

result_file="${GIVN_RESULT_FILE:-$root/target/givn-verify-result.json}"
mkdir -p "$(dirname "$result_file")"
rm -f "$result_file"
export GIVN_RESULT_FILE="$result_file"

if [[ "$scope" == "e2e" ]]; then
  cargo build --locked -p passuff --bins
  # The frozen package declares its own workspace so its tree builds without
  # the production crates; it reads the scenarios from the Specification corpus.
  # It is addressed by manifest path, not as a workspace member. The harness
  # refuses to run without its binary directory; there is no in-process
  # fallback because the package declares no product dependency.
  PASSUFF_BIN_DIR="${PASSUFF_BIN_DIR:-$root/target/debug}" \
    cargo test --locked --manifest-path "$package_manifest" \
      --test e2e_runner
  exit 0
fi

# The black-box package must not reach the product as a library. The pattern
# matches the production crates only; the root line `passuff-e2e v...` does not
# match because a space must follow the crate name.
tree="$(cargo tree --manifest-path "$package_manifest" \
  -e normal,build,dev --prefix none)"
if grep -Eq '^passuff(-config|-core)? v' <<<"$tree"; then
  echo "passuff-e2e must not depend on production crates" >&2
  exit 1
fi

features="${GIVN_FEATURES:-}"
if [[ -z "$features" ]]; then
  if compgen -G "$root/givn/changes/*/specs" >/dev/null; then
    features="$root/givn/changes"
  else
    features="$root/givn/specs"
  fi
fi
case "$features" in
  /*) ;;
  *) features="$root/$features" ;;
esac
export GIVN_FEATURES="$features"

cargo fmt --all -- --check
cargo fmt --manifest-path "$package_manifest" -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo clippy --locked --manifest-path "$package_manifest" \
  --all-targets -- -D warnings
cargo clippy --locked -p passuff --all-targets --features test-support -- -D warnings
cargo test --locked --workspace --lib

cargo test --locked -p passuff --features test-support --test features_runner
