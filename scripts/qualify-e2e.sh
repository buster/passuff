#!/usr/bin/env bash
# Prove the frozen acceptance package fails for seeded defects.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

package="${1:-crates/passuff-e2e}"
if [[ ! -f "$package/Cargo.toml" ]]; then
    echo "qualify-e2e: the frozen acceptance package is missing at $package" >&2
    exit 2
fi

cargo build --offline --locked -p passuff --bins
PASSUFF_BIN_DIR="$ROOT/target/debug" cargo test --offline --locked \
    --manifest-path "$package/Cargo.toml" --test e2e_runner
