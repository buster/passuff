# Proposal: passuff-auth-modes

## Release note

Configure each GitHub account with an explicit credential mode: a gh-managed OAuth session that can open a browser login when needed, or a personal access token read from the environment, a command, or a permission-checked file.

## Use-Case Context

- Use-case ID: `watch-pull-requests`
- Ideation topic: none
- Seed path: none
- Confirmed Personas: `multi-account-developer` — confirmed by the operator's brief on 2026-11-02 and by the instruction to proceed with this change.

## Review Context

- Review kind: independent
- Reason: this change alters the public configuration contract, credential handling, and authorization identity checks — a security- and authorization-relevant major change.

## Problem / Opportunity

The archived watcher accepts only an environment variable or a helper command for each account. An operator who already authenticated with `gh` must hand-wire `gh auth token --user <login>` per account, gets no recovery when that session is missing, and receives an opaque failure. A token file can only be read through a shell command, with no check that other local users cannot read it. Nothing verifies that a token belongs to the account the operator intended, so a wrong token silently watches another login's pull requests instead of reporting the mistake. The configuration also has no explicit notion of credential mode, so the intended source and its failure modes are not visible.

## Proposed Solution

Each configured account declares one credential mode.

- `gh_oauth`: the operator names the expected GitHub login. passuff uses the operator's existing gh session for that login. When an interactive continuous watch starts and no usable token is available, passuff opens gh's browser login before observing, so the operator can authorize the account. The one-shot check and any non-interactive run never open a browser; they report the account failure instead.
- `github_pat`: the token comes from exactly one of three sources: an environment variable, a helper command, or a token file. A token file must not be readable or writable by group or other users, and a leading `~` names the operator's home directory. A literal token inside the configuration is rejected.
- When an account names its expected GitHub login, a credential that belongs to a different login is an account failure that names both logins. No pull requests are processed for that account until the credential matches.
- An existing configuration that declares only an environment variable or a helper command remains valid and is treated as a personal-access-token account.

The observable outcome: the operator can add a second account by naming its login and credential mode; a missing gh session produces a browser login in an interactive watch; a wrong or loosely protected credential produces a named account failure instead of silent or incorrect watching.

## Capability Routing

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| pr-activity-notification | EXTEND pr-activity-notification (use case `watch-pull-requests`, score 16.00, signal) | `EXTEND pr-activity-notification` | None; the use case already owns credential-failure and configuration rules (R19, extension 1a), and the new modes change how step 1 resolves an account, not the notification outcome. |

## Out of Scope

- A passuff-owned OAuth application or an embedded device flow. The future TUI needs it, but it requires a registered application and a separate change.
- Token refresh beyond what the gh session and the operating system already provide.
- Keyring storage as a first-class mode; a keyring lookup remains expressible as a helper command.
- Per-account notification or event-family configuration.
- Windows support.
- Automatic re-login after an identity mismatch within the same process.
- GitHub Enterprise Server login differences beyond deriving the gh hostname from the account's GraphQL endpoint.

## Open Questions

### D1 — Where may a personal access token be stored?

- Question: For a `github_pat` account, which token sources must be supported, and may a literal token be stored in the configuration file?
- Options:
  1. **Indirection only** — the token comes from an environment variable or a helper command. Gain: no secret ever lives in the configuration; the archived no-tokens-in-configuration rule stays intact. Cost: the operator needs an environment variable or a helper such as `pass`, `op`, or `secret-tool`. In practice: `token_command = "pass show work/github-pat"`.
  2. **Literal token field** — the token is written in the configuration. Gain: one file and copy-paste setup. Cost: the configuration becomes a secret to protect, sync, and exclude from backups; the credential can leak through a dotfiles repository. In practice: `token = "ghp_..."` in `config.toml`.
  3. **Permission-checked token file** — the token lives in a separate file that passuff opens only when group and other permissions are absent. Gain: configuration stays secret-free, no helper process is required, and the file can come from a secret manager or manual placement. Cost: one more file per account and a permission check that can reject an otherwise readable token. In practice: `token_file = "~/.config/passuff/work.token"` with mode `0600`, or `token_command = "cat $XDG_CONFIG_HOME/passuff/work.token"` as the unsafeguarded alternative.
- Disposition: answered 2026-11-02 (operator) — combine options 1 and 3: support environment, command, and a permission-checked token file; reject a literal token. The operator noted that a command can already read the file (`cat $XDG_CONFIG_HOME/passuff/work.token`); the token-file mode adds the permission guarantee.

### D2 — Is an expected login optional for a personal access token?

- Question: Must a `github_pat` account name its expected GitHub login, or is identity validation optional?
- Options:
  1. **Optional** — the token identifies the login, and validation runs only when the operator names one. Gain: the existing environment/command configurations keep working unchanged. Cost: a wrong token is still not detected for accounts that omit the login. In practice: an existing `token_env` account keeps working; a new account may add `github_user` for the check.
  2. **Required** — every account names its expected login. Gain: every credential is checked against an intended identity. Cost: every existing configuration must add a field, and a legitimate token for the right account is rejected until the operator declares the login. In practice: an archived configuration fails validation after upgrade until amended.
- Disposition: carried to `design.md` (owner: design author) — the design decides the required/optional rule and states the migration consequence; the proposal keeps the existing environment/command form valid.
