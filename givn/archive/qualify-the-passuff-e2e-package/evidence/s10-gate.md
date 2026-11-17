# S10 gate result

Phase `S10` of `givn/ideation/ise-rebuildability/plan.md`. Gate `G9` reads: the
Passuff `e2e` package declares a production dependency, or a seeded defect
passes, stops the pilot. Recorded 2026-11-17 on branch `ise/trial`.

The 29 acceptance scenarios live in the seven capability feature files under
`givn/specs`; the frozen package selects `@e2e` from that corpus and keeps the
step definitions and harness.

| Condition | Command | Observed | Verdict |
|---|---|---|---|
| No production dependency | `cargo tree --manifest-path crates/passuff-e2e/Cargo.toml -e normal,build,dev --prefix none` | 404 lines; no `passuff`, `passuff-core`, or `passuff-config` package line | PASS |
| Every seeded defect fails its named scenario | `./scripts/qualify-e2e.sh` | `20/20 defects detected; clean tree green`, exit 0 | PASS |
| Clean tree after every revert | the same run | every `detected:` line is followed by a clean suite pass; the content hash is unchanged | PASS |
| Frozen suite green | `./scripts/verify.sh --e2e` | 6 features, 29 scenarios, 240 steps, 0 failed, 0 skipped; `GIVN_RESULT_FILE` has `{"failed":0,"passed":29,"skipped":0,"total":29}` | PASS |
| Interaction inventory clean | `givn lint` | 7 files checked; no blocking finding, advisories only | PASS |
| Component scope green | `./scripts/verify.sh` | 215 scenarios, 1731 steps, 0 failed | PASS |
| Two viewport transcripts | `passuff-tui` on a real pty | `transcripts/80x24.txt`, `transcripts/120x40.txt` | PASS |
| Key-parser defect fails | `key-parser-unbinds-quit.patch` | `The inbox row is visible at the narrow terminal size`, failing step `Then the TUI exits with code 0` | PASS |
| Inbox-paging defect fails | `inbox-paging-skips-selected-page.patch` | `The activity detail opens at the wide terminal size and quits cleanly`, failing step `Then the screen shows "comment by @reviewer"` | PASS |
| 80x24 layout defect fails | `narrow-size-blind.patch` | `The inbox row is visible at the narrow terminal size`, failing step `Then the screen shows "1 unread of 1"` | PASS |

Gate verdict: PASS. `G9` does not stop the pilot.

## Exit criteria

| Criterion | Result |
|---|---|
| No production dependency | Met: the package links cucumber, portable-pty, serde_json, tempfile, tokio, vt100, and wiremock only. |
| Clean pass | Met: 29/29 scenarios, 240/240 steps, result file written. |
| Seeded defects | Met: 20/20, each in its named scenario. The plan names three (key parser, 80x24 layout, inbox paging); all three fail. |
| Two viewport transcripts | Met: one parsed screen per size. |
| Reclassification reason | Not applicable: the 29 scenarios moved from `crates/passuff-e2e/features/boundary.feature` into the capability feature files under `givn/specs` and stayed `@e2e`; no scenario changed classification. |

## Evidence files

| File | Content |
|---|---|
| `cargo-tree-passuff-e2e.txt` | The package graph |
| `qualify-20-of-20.txt` | The qualification run, every defect line, and the summary |
| `verify-e2e-clean.txt` | The 29-scenario pass |
| `verify-e2e-result.json` | The four counts the runner wrote |
| `defect-key-parser.txt` | The key-parser failure block |
| `defect-inbox-paging.txt` | The inbox-paging failure block |
| `defect-narrow-size.txt` | The 80x24 layout failure block |
| `transcripts/80x24.txt` | The narrow frame |
| `transcripts/120x40.txt` | The wide frame |

The open limit from `S14` still holds: 29 scenarios qualify the boundary, not
the whole product (`docs/arc42/11-risks-and-technical-debt.md`, TD-007). The
E2E coverage scope measures no production code yet (TD-008). Eight declared
consumer actions have no acceptance scenario; TD-009 records each action and
the scenario it needs.
