#!/usr/bin/env bash
# Prove the frozen black-box package is sensitive to seeded defects.
#
# Each defect is applied to a scratch copy of the tree. The package must build,
# the suite must fail, the defect is reverted, and the suite must pass again.
# The real tree is only read: its content hash is compared before and after,
# including when the script fails.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
defects="$root/scripts/qualify"
defect_names=(
  narrow-size-blind
  status-unread-count
  delivery-silent-ok
  observe-empty-payload
  server-error-success
  transport-failure-skipped
)

hash_dir() {
  local dir="$1"
  (
    cd "$dir" &&
      find . -type f \
        -not -path './.git/*' \
        -not -path './target/*' \
        -not -path './coverage/*' \
        -print0 |
      LC_ALL=C sort -z |
      xargs -0 sha256sum |
      sha256sum |
      cut -d' ' -f1
  )
}

before="$(hash_dir "$root")"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/passuff-e2e-qualify.XXXXXX")"
logs="$(mktemp -d "${TMPDIR:-/tmp}/passuff-e2e-qualify-logs.XXXXXX")"

cleanup() {
  local code=$?
  trap - EXIT
  rm -rf "$scratch" "$logs"
  local after
  after="$(hash_dir "$root")"
  if [[ "$before" != "$after" ]]; then
    echo "qualify: the working tree changed during qualification" >&2
    code=1
  fi
  exit "$code"
}
trap cleanup EXIT

tar -C "$root" \
  --exclude=./.git \
  --exclude=./target \
  --exclude=./coverage \
  -cf - . | tar -C "$scratch" -xf -
scratch_before="$(hash_dir "$scratch")"

build_suite() {
  (
    cd "$scratch" &&
      cargo build --locked -p passuff --bins &&
      cargo test --locked -p passuff-e2e --test e2e_runner --no-run
  )
}

run_suite() {
  (
    cd "$scratch" &&
      cargo build --locked -p passuff --bins &&
      PASSUFF_BIN_DIR="$scratch/target/debug" \
        cargo test --locked -p passuff-e2e --test e2e_runner
  )
}

apply_defect() {
  local patch_file="$1"
  patch -p1 --forward --batch --silent --no-backup-if-mismatch \
    --dry-run -d "$scratch" <"$patch_file"
  patch -p1 --forward --batch --silent --no-backup-if-mismatch \
    -d "$scratch" <"$patch_file"
}

revert_defect() {
  local patch_file="$1"
  patch -R -p1 --forward --batch --silent --no-backup-if-mismatch \
    -d "$scratch" <"$patch_file"
}

# Name the scenario whose step failed, from the cucumber transcript.
failing_scenario() {
  awk '/^  Scenario: / { name = substr($0, 13) } /✘/ { print name; exit }' "$1"
}

echo "qualify: building the clean scratch tree"
build_suite >"$logs/build-clean.log" 2>&1 || {
  echo "qualify: the clean scratch tree does not build" >&2
  tail -n 40 "$logs/build-clean.log" >&2
  exit 1
}

status=0
detected=0
for name in "${defect_names[@]}"; do
  patch_file="$defects/$name.patch"
  if ! apply_defect "$patch_file"; then
    echo "qualify: cannot apply $name" >&2
    status=1
    break
  fi
  if ! build_suite >"$logs/$name.build.log" 2>&1; then
    echo "qualify: $name does not compile; that is not a detection" >&2
    tail -n 40 "$logs/$name.build.log" >&2
    status=1
    break
  fi
  if run_suite >"$logs/$name.suite.log" 2>&1; then
    echo "DEFECT NOT DETECTED: $name" >&2
    tail -n 40 "$logs/$name.suite.log" >&2
    status=1
  else
    echo "detected: $name [$(failing_scenario "$logs/$name.suite.log")]"
    detected=$((detected + 1))
  fi
  revert_defect "$patch_file"
  if [[ "$(hash_dir "$scratch")" != "$scratch_before" ]]; then
    echo "qualify: $name did not revert cleanly" >&2
    status=1
    break
  fi
  if ! run_suite >"$logs/$name.clean.log" 2>&1; then
    echo "CLEAN TREE FAILED after reverting $name" >&2
    tail -n 40 "$logs/$name.clean.log" >&2
    status=1
  fi
done

if [[ $status -eq 0 ]]; then
  echo "qualification: ${detected}/${#defect_names[@]} defects detected; clean tree green"
fi
exit "$status"
