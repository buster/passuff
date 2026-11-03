# Domain terms: inbox-scope-and-stacks

| Term | Status | Definition | Contrast | First use | Collision check |
|---|---|---|---|---|---|
| Repository | confirmed 2026-11-03 | A GitHub resource identified by an owner/name pair that contains pull requests. | Repository, not project — "project" also names GitHub Projects boards and local work directories. | frame.md | `grep -rin "repository" docs/glossary.md \| wc -l` → 2 (definitions only, no row); `grep -rin "stack" givn/specs/ \| wc -l` → 0. |
| Pull request stack | confirmed 2026-11-03 | An ordered chain of pull requests in one repository, identified by a repository-scoped number and a total size, where each member has a 1-based position and the chain shares one ultimate base. On a watched member, passuff records the number, size, position, and base as `PullRequestStack`. | Pull request stack, not a stack of commits — the members are pull requests, not commits. | frame.md | `grep -rin "stack" docs/ crates/ \| wc -l` → 0; `grep -rin "stack" givn/specs/ \| wc -l` → 0. |
| Stack position | confirmed 2026-11-03 | The 1-based ordinal of a pull request within its stack, counting from the bottom pull request closest to the ultimate base. | Position, not pull request number — the number is repository-scoped identity, the position is stack order. | frame.md | `grep -rn "position" crates/passuff-core/src \| wc -l` → 0. |
| Stack base | confirmed 2026-11-03 | The branch that the whole stack ultimately targets. | Stack base, not pull request base — a pull request's base is its direct parent branch. | frame.md | `grep -rin "stack" docs/ crates/ \| wc -l` → 0; the observation stores no base or head ref. |
| Known repository | confirmed 2026-11-03 | A repository that passuff has observed in an authored pull request or that appears in the watch scope. | Known repository, not every readable repository — the full catalog is a superset passuff does not hold. | questions.md Q5 | `grep -rin "known repository" docs/ crates/ \| wc -l` → 0. |
| Catch-up | confirmed 2026-11-03 | The recording of activity that happened while a repository was outside the watch scope, delivered to the inbox without a notification. | Catch-up, not a notification backlog — catch-up is silent; a backlog is delivered. | questions.md Q4 | `grep -rin "catch-up" docs/ crates/ \| wc -l` → 0; `grep -rin "catch up" docs/ crates/ \| wc -l` → 0. |

Verified names: 2026-11-03 by the operator ("Lgtm", confirm all) for the four
confirmed terms; "Confirm" for known repository and catch-up.

## Concrete domain sentences

- Repository: "Selecting `acme/api` adds the repository to the watch scope; the
  personal account no longer observes pull requests outside it."
- Pull request stack: "The bottom pull request of the stack targets `main`; each
  later pull request targets the branch directly below it."
- Stack position: "The second pull request from the bottom has stack position 2,
  regardless of its pull request number."
- Stack base: "The stack base is `main` for every member of the stack, even
  though only the bottom pull request directly targets it."
- Known repository: "`acme/api` appears in the picker because passuff saw an
  authored pull request there, even though the repository is currently
  deselected."
- Catch-up: "Reselecting `acme/api` records the week's comments in the inbox
  without a desktop notification or sound."
