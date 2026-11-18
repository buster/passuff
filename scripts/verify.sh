#!/usr/bin/env bash
# Component verification (default) and frozen black-box acceptance (--e2e).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
    echo "usage: scripts/verify.sh [--e2e [--package <PATH>]]" >&2
}

scope="change"
package="crates/passuff-e2e"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --e2e)
            scope="e2e"
            shift
            ;;
        --package)
            package="${2:-}"
            if [[ -z "$package" ]]; then
                usage
                exit 2
            fi
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            usage
            exit 2
            ;;
    esac
done

if [[ "$scope" == "change" ]]; then
    if [[ -n "${GIVN_FEATURES:-}" ]]; then
        echo "component corpus selection: $GIVN_FEATURES" >&2
    fi
    cargo fmt --all -- --check
    cargo clippy --offline --all-targets -p passuff -p passuff-core -p passuff-config -- -D warnings
    cargo test --offline -p passuff -p passuff-core -p passuff-config
    exit 0
fi

cargo build --offline --locked -p passuff --bins
bin_dir="$ROOT/target/debug"
if [[ ! -x "$bin_dir/passuff" || ! -x "$bin_dir/passuff-tui" ]]; then
    echo "verify: $bin_dir does not hold both binaries" >&2
    exit 2
fi
if [[ ! -f "$package/Cargo.toml" ]]; then
    echo "verify: the frozen acceptance package is missing at $package" >&2
    exit 2
fi

result_file="${GIVN_RESULT_FILE:-$ROOT/target/givn-verify-result.json}"
mkdir -p "$(dirname "$result_file")"
rm -f "$result_file"
export GIVN_RESULT_FILE="$result_file"

PASSUFF_BIN_DIR="$bin_dir" cargo test --offline --locked \
    --manifest-path "$package/Cargo.toml" --test e2e_runner
