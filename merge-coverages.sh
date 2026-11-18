#!/usr/bin/env bash
# Merge the non-E2E and E2E Cobertura reports into coverage/cobertura-coverage.xml.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "merge-coverages: cargo-llvm-cov is required" >&2
    exit 2
fi

mkdir -p coverage
cargo llvm-cov report --offline --cobertura \
    --output-path coverage/cobertura-coverage.xml
