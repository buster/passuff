# Proposal: passuff-tui-live

## Release Note

The terminal inbox observes the developer's accounts itself and shows new activity within about a second of it appearing, without a separate check process or restart.

## Problem / Opportunity

The terminal inbox is a passive view. It re-reads the persisted watch state on a fixed timer, but only an external `passuff check` or `passuff` watch process writes that state. With no external process running, the inbox never shows new activity. With one, the developer waits for that process's configured interval, and two observation processes compete for the same account allowance.

The developer watching the inbox is the moment freshness matters most. That is exactly the moment the current design depends on another process.

GitHub offers no push channel to a desktop application, so freshness is bounded by how often passuff may ask. Measured on the current account, one full observation costs 57 of the 5,000 points GitHub allows per hour; asking for a full observation every second would exhaust the allowance in about four minutes. A lightweight change probe costs 1 point. Fast updates are therefore affordable only when the common case is cheap.

## Proposed Solution

While the terminal inbox is open, it observes the configured accounts itself instead of waiting for another process:

- It asks GitHub for a cheap change signal about once per second and performs the full observation only when that signal reports a change.
- When the full observation completes, the screen reflects it immediately; the developer does not press a key, run another command, or restart.
- Focus and the selected pull request survive every automatic update.
- The observation paces itself against the account's remaining API allowance: as the allowance falls, observation slows; it never spends the allowance faster than it is replenished.
- A failed observation keeps the last screen and is retried; it does not blank the inbox or end the program.
- Activity observed while the inbox is open is recorded and notified exactly as the watch command records and notifies it, so closing the inbox never loses an alert.

## Out of Scope

- GitHub webhooks or any server-side push; a desktop application has no inbound endpoint.
- Changing the background watch command's cadence or notification behaviour.
- Changing the watch scope, the stored state format, or the read/notification replay rules.
- Coordinating multiple observation processes against one state file; the inbox observes while it is open, and running the watch command at the same time remains redundant but permitted.
- Reducing the cost of the full observation query; this change makes the common case cheap instead.

## Open Questions

- D1: How aggressive may observation be before it harms the account's allowance?
- D2: What happens when the developer runs the watch command while the inbox is open?
- D3: Does the configured poll interval govern the open inbox?

## Use-Case Context

- Use case: `review-activity-inbox` (`givn/specs/review-activity-inbox/usecase.md`).
- Capability: `inbox-navigation`.
- Source topic: none; this change does not originate from an ideation seed.
- Personas: `multi-account-developer`.
- Review kind: self-review.
- Interface: terminal.

## Capability Routing

| Capability | Decision | Rationale |
|---|---|---|
| activity-inbox | Modify the existing "Developer opens the inbox TUI" scenario so opening observes the account | Opening the inbox now performs the first observation; the scenario no longer needs an external check to populate the view. |
| inbox-navigation | Add the live update, the cheap-probe behaviour, the allowance pacing, and the failure fallback to this capability | These behaviours keep the terminal inbox's panes current after opening; they advance the same developer goal as navigation and need no second capability. |
