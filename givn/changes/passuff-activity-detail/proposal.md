# Proposal: passuff-activity-detail

## Release Note

Pressing Enter on an activity in the terminal inbox opens a detail popup that describes the event and can open the pull request in the browser.

## Problem / Opportunity

The terminal inbox shows each activity as a single summary line: an id, an observation time, the pull request, its state, and the event summary. The developer cannot see the rest of the recorded context (the pull request title, CI result, stack position) and cannot reach the pull request itself from the inbox. Reviewing an alert means switching to a browser and finding the pull request by hand.

## Proposed Solution

- With the activity pane focused, Enter opens a popup for the selected activity.
- The popup shows the recorded activity context: the activity id, the event summary, the observation time, the read state, and the pull request's key, title, state, CI result, and stack position. Comment and review bodies are not shown because passuff does not persist them.
- Escape closes the popup and returns to the inbox unchanged.
- Enter in the popup opens the pull request's URL in the system browser and closes the popup.
- A browser launch failure keeps the popup open and reports the failure in the inbox footer.
- Enter outside the activity pane, or with no activity selected, changes nothing.

## Out of Scope

- Fetching and displaying comment or review bodies; passuff records none.
- Marking read, navigating lists, or any other action inside the popup.
- A detail popup for a watched pull request without an activity.
- Mouse input, copying text, or choosing a browser.

## Open Questions

- D1: Does a successful browser launch close the popup?
- D2: What does Enter do outside the activity pane?
- D3: How is the browser selected, and what happens when it is unavailable?

## Use-Case Context

- Use case: `review-activity-inbox` (`givn/specs/review-activity-inbox/usecase.md`).
- Capability: `activity-inbox`.
- Source topic: none; no ideation seed.
- Personas: `multi-account-developer`.
- Review kind: self-review.
- Interface: terminal.

## Capability Routing

| Proposed capability | Route's recommendation | Decision | Rationale (if you deviated from route) |
|---|---|---|---|
| activity-inbox | no signal (tie) | `EXTEND activity-inbox` | The detail popup and the browser launch belong to the existing activity-review goal; no second capability is warranted. |
