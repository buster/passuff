---
status: accepted
date: 2026-11-02
decision-makers: passuff maintainer (operator)
consulted: none
informed: none
---

# Deliver through local desktop notifications and a bundled sound

## Context and Problem Statement

The developer wants a desktop notification with an audible alert on macOS and
Linux, from a self-contained watcher. The delivery mechanism determines whether
the tool needs a third-party account, a hosted push service, or an audio
library. The alternatives are the local operating-system notification surface
with a bundled sound, a hosted push channel, and email.

## Decision Drivers

- Local, self-contained operation with no third-party notification account.
- Native desktop presentation on macOS and Linux.
- An audible alert that can be disabled independently.
- No ALSA or CoreAudio development dependency.
- Deterministic testability without a live desktop session.
- No inbound network surface.

## Considered Options

- Local operating-system notifications plus a bundled WAV played by a system
  player.
- A hosted push channel such as ntfy or Telegram.
- Email delivery.

## Decision Outcome

Chosen option: "Local operating-system notifications plus a bundled WAV played
by a system player", because it keeps delivery local and native without adding
a hosted account or an audio build dependency.

### Consequences

- Good, because no third-party service, account, or credential is involved in
  delivery.
- Good, because the notification appears in the operating system's notification
  centre.
- Good, because the recording delivery adapter and fake player make the delivery
  decision testable without a live desktop.
- Bad, because Linux requires a reachable notification daemon and a sound
  player; the XDG hint fallback may be ignored by some daemons.
- Bad, because macOS needs the `Passuff.app` bundle for stable notification
  attribution.
- Bad, because there is no remote or mobile delivery.

### Confirmation

The E2E scenarios assert the recorded notification content and the fake sound
player invocation; on Linux the real platform notification adapter runs against a
private D-Bus fake. Unit tests assert the notification title/body and platform
player selection. A recorded manual smoke test on macOS confirms that the real
desktop notification and sound are delivered.

## Pros and Cons of the Options

### Local operating-system notifications plus a bundled WAV

- Good, because it is native and local.
- Good, because it has no third-party credential or account.
- Good, because the sound can be disabled or replaced.
- Neutral, because the platform surface differs between Linux and macOS.
- Bad, because it depends on a notification daemon and a sound player.
- Bad, because macOS notification attribution needs an app bundle.

### A hosted push channel

- Good, because it can reach a phone or another machine.
- Good, because it can deliver when the desktop is unavailable.
- Bad, because it requires a third-party account and token.
- Bad, because it adds an outbound dependency and a second credential store.
- Bad, because it is no longer self-contained.

### Email delivery

- Good, because it is universally available.
- Good, because it leaves a durable record.
- Bad, because it requires SMTP credentials and configuration.
- Bad, because it is not an immediate desktop alert.
- Bad, because it adds a second delivery failure domain and typically more
  latency than a local notification.

## More Information

- [Use case: watch-pull-requests](../../../givn/specs/watch-pull-requests/usecase.md)
- [Context and scope](../03-context-and-scope.md)
- [ADR register](../09-architecture-decisions.md)
