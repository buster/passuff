#!/usr/bin/env bash
# Produce the non-E2E and E2E Cobertura reports when cargo-llvm-cov is present.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "measure-coverage: cargo-llvm-cov and the llvm-tools component are required" >&2
    exit 2
fi

mkdir -p coverage
cargo llvm-cov --offline --workspace --ignore-filename-regex 'crates/passuff-e2e' \
    --cobertura --output-path coverage/non-e2e-cobertura.xml
if [[ -f crates/passuff-e2e/Cargo.toml ]]; then
    cargo llvm-cov --offline --manifest-path crates/passuff-e2e/Cargo.toml \
        --cobertura --output-path coverage/e2e-cobertura.xml
fi
