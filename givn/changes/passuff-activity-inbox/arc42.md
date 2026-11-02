# Arc42 Impact Assessment: passuff-activity-inbox

Adds a bounded activity history to the persisted state, a core read model, three
CLI read commands, and a global repository watch scope. The core/UI boundary is
unchanged; the TUI is the next change.

| # | Chapter | Affected? | Reason / summary |
|---|---|---|---|
| 1 | 01-introduction-and-goals | Yes | Requirements and the authoritative use-case list now include status and activity review; chapter 01 names all three use cases. |
| 2 | 02-architecture-constraints | No | No new platform, legal, or deployment constraint. |
| 3 | 03-context-and-scope | Yes | The CLI gaind read commands; status/inbox/read do not contact GitHub. |
| 4 | 04-solution-strategy | Yes | A bounded read model over the existing state store and a global watch scope. |
| 5 | 05-building-block-view | Yes | `Inbox` service in the core; state version 2; CLI renderer. |
| 6 | 06-runtime-view | Yes | Activity recording, status/inbox load, and read write-back flows. |
| 7 | 07-deployment-view | No | Same workstation binary; no new service or file location. |
| 8 | 08-crosscutting-concepts | Yes | History retention, state migration, read-state persistence, watch-scope filtering. |
| 9 | 09-architecture-decisions | Yes | ADR-0005 records the version-2 state migration; ADR-0001/0003 still own observation and the core/UI boundary. |
| 10 | 10-quality-requirements | Yes | Quality scenarios for bounded history, read persistence, and scope filtering. |
| 11 | 11-risks-and-technical-debt | Yes | State growth, write-on-read, global scope limitation. |
| 12 | 12-glossary | Yes | Activity record, inbox, read state, watch scope. |

## ADR qualification

### Candidate: migrate the persisted state to version 2

- Architectural commitment: yes. It fixes a persisted-data compatibility
  boundary and the migration rule future changes must follow.
- Real alternatives: yes. Keep version 1 with defaulted fields, a version-2
  rewrite, or a second history file.
- Durable consequence: yes. Reversing the choice changes the on-disk format and
  the upgrade path; version-1 files must remain loadable.
- Existing decisions: none. ADR-0001 covers observation, ADR-0003 the core/UI
  boundary, ADR-0004 credentials.
- Route: `NEW_ADR` at
  `docs/arc42/adr/ADR-0005-state-version-2-migration.md`.

No other candidate qualifies: the 500-record cap, read state, and watch scope
are behavior owned by the use cases and design.

## Status

STATUS: DONE
