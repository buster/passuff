#!/usr/bin/env bash
# Prove the frozen black-box package is sensitive to seeded defects.
#
# Each defect is applied to a scratch copy of the tree. The package must build,
# the suite must fail, the defect is reverted, and the suite must pass again.
# The real tree is only read: the content hash of its tracked and untracked
# files is compared before and after, including when the script fails. The
# ignored local record store under givn/history is excluded because Givn writes
# it on every command and it is not repository content.
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
  setup-drops-accounts
  credentials-token-file-mode
  read-argument-guard
  migration-not-persisted
  quarantine-not-moved
  ci-recovery-unseen
  own-comment-not-read
  dedupe-replays-comment
  scope-filter-dropped
  narrow-size-threshold
  tab-does-not-switch
  watch-interval-capped
  key-parser-unbinds-quit
  inbox-paging-skips-selected-page
)

# The scenario each seeded defect must fail. A defect may fail more scenarios;
# the named one must be among them.
declare -A expected_scenario=(
  [narrow-size-blind]="The inbox row is visible at the narrow terminal size"
  [status-unread-count]="A first check baselines the account and status reports the scope"
  [delivery-silent-ok]="A notification is delivered as one JSON line"
  [observe-empty-payload]="The activity detail opens at the wide terminal size and quits cleanly"
  [server-error-success]="A server error fails the check"
  [transport-failure-skipped]="An unreachable endpoint fails with a diagnostic"
  [setup-drops-accounts]="Setup writes a usable configuration"
  [credentials-token-file-mode]="A token file readable by other users is refused"
  [read-argument-guard]="Read with no id and with an id and --all exits 2"
  [migration-not-persisted]="A version-1 state file migrates on the next status"
  [quarantine-not-moved]="A corrupt state file is quarantined and replaced"
  [ci-recovery-unseen]="A failing check notifies and a recovery notifies"
  [own-comment-not-read]="The viewer's own comment is recorded read and does not notify"
  [dedupe-replays-comment]="The same activity delivered twice produces one record"
  [scope-filter-dropped]="Two accounts report only the watched scope"
  [narrow-size-threshold]="A terminal too short prints the minimum size"
  [tab-does-not-switch]="Tab and back-tab switch the active pane"
  [watch-interval-capped]="The default command watches and delivers one notification"
  [key-parser-unbinds-quit]="The inbox row is visible at the narrow terminal size"
  [inbox-paging-skips-selected-page]="The activity detail opens at the wide terminal size and quits cleanly"
)

hash_dir() {
  local dir="$1"
  (
    cd "$dir" &&
      find . -type f \
        -not -path './.git/*' \
        -not -path './target/*' \
        -not -path './crates/passuff-e2e/target/*' \
        -not -path './coverage/*' \
        -not -path './givn/history/*' \
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
  --exclude=./crates/passuff-e2e/target \
  --exclude=./coverage \
  -cf - . | tar -C "$scratch" -xf -
scratch_before="$(hash_dir "$scratch")"

build_suite() {
  (
    cd "$scratch" &&
      cargo build --locked -p passuff --bins &&
      cargo test --locked --manifest-path crates/passuff-e2e/Cargo.toml \
        --test e2e_runner --no-run
  )
}

run_suite() {
  (
    cd "$scratch" &&
      cargo build --locked -p passuff --bins &&
      PASSUFF_BIN_DIR="$scratch/target/debug" \
        cargo test --locked --manifest-path crates/passuff-e2e/Cargo.toml \
          --test e2e_runner
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

# Every scenario whose body has a failing step, from the cucumber transcript.
failing_scenarios() {
  awk '/^  Scenario: / { name = substr($0, 13) } /✘/ && name != "" { print name }' \
    "$1" | LC_ALL=C sort -u
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
    scenarios="$(failing_scenarios "$logs/$name.suite.log")"
    expected="${expected_scenario[$name]:-}"
    if [[ -z "$expected" ]]; then
      echo "qualify: no expected scenario for $name" >&2
      status=1
      break
    fi
    if grep -Fxq -- "$expected" <<<"$scenarios"; then
      echo "detected: $name [\"$expected\"]"
      detected=$((detected + 1))
    else
      echo "DEFECT NOT DETECTED IN ITS SCENARIO: $name" >&2
      echo "  expected: $expected" >&2
      echo "  failed: ${scenarios:-none}" >&2
      tail -n 40 "$logs/$name.suite.log" >&2
      status=1
    fi
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
