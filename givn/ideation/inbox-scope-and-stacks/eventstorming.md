# Event Storming: inbox-scope-and-stacks

## Scope of this canvas

Two chains share one observation cycle:

- **Watch scope editing:** the developer selects or deselects a repository from
  the terminal UI; the inbox hides out-of-scope records; reselecting records
  catch-up activity silently.
- **Stack grouping:** GitHub reports stack membership during observation; the
  inbox groups watched pull requests and their activity by stack.

Decisions applied: Q1 Hide, Q2 metadata only, Q3 global scope, Q4 catch-up
silently, Q5 known plus manual entry.

## Event chain

| # | Event | Emitted by | Trigger: actor → command | Policy reaction | Read model |
|---|---|---|---|---|---|
| 1 | Repository recorded | Repository policy (or the manual-entry command) | GitHub reports an authored pull request, or the developer enters owner/name → Observe authored pull requests / Enter repository owner/name | Add the repository to the picker if it is not known. | Repository picker |
| 2 | Repository selected | Watch scope (configuration) | Developer → Select repository | Mark catch-up pending for the repository; the next cycle observes it. | Repository picker, Inbox |
| 3 | Repository deselected | Watch scope (configuration) | Developer → Deselect repository | Stop observing the repository; hide its records from the inbox and status; keep them in state. | Inbox, Watch status |
| 4 | Pull request observed | Watched Pull Request (aggregate) | Check cycle → Observe authored pull requests | Classify new activity; update the observation and baseline. | Inbox |
| 5 | Stack membership recorded | Watched Pull Request (aggregate) | GitHub reports stack metadata → Observe authored pull requests | Record stack number, position, size, and stack base on the watched pull request; group it in the inbox. | Inbox |
| 6 | Stack membership cleared | Watched Pull Request (aggregate) | GitHub reports no stack → Observe authored pull requests | Clear the recorded stack membership; show the pull request as standalone. | Inbox |
| 7 | Activity recorded | Watched Pull Request (aggregate) | Classification of a new event → Observe authored pull requests | Append one activity record with a stable id and an unread flag. | Inbox |
| 8 | Notification produced | Watched Pull Request (aggregate) | Classification of a new event → Observe authored pull requests | Produce one grouped notification per pull request unless a catch-up cycle suppresses it. | Notification report |
| 9 | Gap activity recorded | Catch-up policy | Catch-up pending + first observation after reselect | Record the events as activity without producing a notification or sound. | Inbox |
| 10 | Catch-up completed | Catch-up policy | First observation after reselect | Clear the catch-up pending marker; normal notification resumes on the next cycle. | Repository picker, Inbox |

## Reverse narrative

**Why can activity be recorded without a notification?**
Because the developer reselected a repository, and catch-up completed only after
the first observation recorded the gap. Before that, the developer selected the
repository, and before that, the repository was deselected and hidden — which
requires a known repository and a watch scope.

**Why does the inbox group rows?**
Because stack membership was recorded during observation, which requires the
GitHub stack metadata and a watched pull request.

**Why is a repository in the picker?**
Because it was recorded from an authored pull request or entered manually, or
because it is already in the watch scope.

## Canvas

```mermaid
flowchart LR
    subgraph legend["Legend"]
        legendActor(["Actor"])
        legendCommand[["Command"]]
        legendEvent("Event")
        legendAggregate{{"Aggregate"}}
        legendConfig[/"Configuration"/]
        legendRecord[("Record")]
        legendPolicy{"Policy"}
        legendRead[["Read model"]]
    end

    subgraph actors["Actors and external systems"]
        dev(["Developer"])
        github["GitHub"]
    end

    subgraph review["Activity Review"]
        cmdSelect[["Select repository"]]
        cmdDeselect[["Deselect repository"]]
        cmdEnter[["Enter repository owner/name"]]
        scope[/"Watch scope<br/>configuration"/]
        evSelected("Repository selected")
        evDeselected("Repository deselected")
        evRepoRecorded("Repository recorded")
        repoIndex[("Known repository index<br/>record")]
        pending[("Catch-up pending<br/>record")]
        policyScope{"Scope policy"}
        readPicker[["Repository picker<br/>read model"]]
        readInbox[["Inbox<br/>read model"]]
        readStatus[["Watch status<br/>read model"]]
    end

    subgraph watch["Pull Request Watching"]
        policyCycle{"Check cycle"}
        cmdObserve[["Observe authored pull requests"]]
        aggPR{{"Watched Pull Request<br/>aggregate"}}
        evObserved("Pull request observed")
        evStack("Stack membership recorded")
        evStackCleared("Stack membership cleared")
        evActivity("Activity recorded")
        evGap("Gap activity recorded")
        evCatchup("Catch-up completed")
        evNotification("Notification produced")
        aggActivity{{"Activity Record<br/>aggregate"}}
        policyCatchup{"Catch-up policy"}
        policyRepo{"Repository policy"}
        policyStack{"Stack policy"}
        notification[/"Notification<br/>report"/]
    end

    dev -->|selects| cmdSelect --> scope
    dev -->|deselects| cmdDeselect --> scope
    dev -->|enters| cmdEnter --> evRepoRecorded
    scope -->|emits| evSelected
    scope -->|emits| evDeselected
    evSelected -->|marks| pending
    evDeselected --> policyScope
    policyScope -->|stops observation and hides records| readInbox
    policyScope -->|hides records| readStatus
    scope -.->|filters observation and display| readInbox
    scope -.->|filters display| readStatus

    policyCycle --> cmdObserve
    github -->|authored pull requests and stack metadata| cmdObserve
    cmdObserve --> aggPR
    aggPR --> evObserved
    aggPR --> evStack
    aggPR --> evStackCleared
    aggPR -.->|watched pull requests| readStatus
    evObserved --> policyRepo
    policyRepo --> evRepoRecorded
    evRepoRecorded --> repoIndex
    repoIndex --> readPicker
    scope -->|selected flags| readPicker
    evStack --> policyStack
    evStackCleared --> policyStack
    policyStack --> readInbox
    aggPR --> readInbox

    aggPR --> evActivity
    evActivity --> aggActivity
    aggActivity --> readInbox
    aggPR --> evNotification
    evNotification --> notification

    pending --> policyCatchup
    evObserved --> policyCatchup
    policyCatchup -->|records without notification| evGap
    evGap --> aggActivity
    policyCatchup -.->|suppresses| notification
    policyCatchup --> evCatchup
    evCatchup -->|clears| pending

    class legendActor,dev actor;
    class legendCommand,cmdSelect,cmdDeselect,cmdEnter,cmdObserve command;
    class legendEvent,evSelected,evDeselected,evRepoRecorded,evObserved,evStack,evStackCleared,evActivity,evGap,evCatchup,evNotification event;
    class legendAggregate,aggPR,aggActivity aggregate;
    class legendConfig,scope config;
    class legendRecord,repoIndex,pending record;
    class legendPolicy,policyScope,policyCatchup,policyRepo,policyStack,policyCycle policy;
    class legendRead,readPicker,readInbox,readStatus readmodel;
    class github,notification external;

    classDef actor fill:#f3f4f6,stroke:#374151,stroke-width:2px,color:#111827;
    classDef command fill:#dbeafe,stroke:#1e40af,stroke-width:2px,color:#172554;
    classDef event fill:#ffedd5,stroke:#c2410c,stroke-width:2px,color:#7c2d12;
    classDef aggregate fill:#e9d5ff,stroke:#6b21a8,stroke-width:3px,color:#3b0764;
    classDef config fill:#fef9c3,stroke:#a16207,stroke-width:2px,color:#713f12;
    classDef record fill:#ecfdf5,stroke:#047857,stroke-width:2px,color:#064e3b;
    classDef policy fill:#fce7f3,stroke:#9d174d,stroke-width:2px,color:#831843;
    classDef readmodel fill:#f0fdf4,stroke:#166534,stroke-width:2px,color:#14532d;
    classDef external fill:#f3f4f6,stroke:#374151,stroke-width:2px,stroke-dasharray:6 4,color:#111827;
```

## Elements and kinds

| Kind | Elements |
|---|---|
| Actor | Developer |
| External system | GitHub |
| Commands | Select repository, Deselect repository, Enter repository owner/name, Observe authored pull requests |
| Events | Repository recorded, Repository selected, Repository deselected, Pull request observed, Stack membership recorded, Stack membership cleared, Activity recorded, Notification produced, Gap activity recorded, Catch-up completed |
| Aggregates | Watched Pull Request (baseline, seen identities, CI, state, stack membership; at most one notification per event), Activity Record (stable id, observation time, read flag; history bounded to 500) |
| Configuration | Watch scope (global repository allowlist) |
| Records | Known repository index, Catch-up pending |
| Policies | Check cycle, Repository policy, Scope policy, Stack policy, Catch-up policy |
| Read models | Repository picker, Inbox, Watch status |
| Reports | Notification |

No record, configuration, external fact, or report is drawn as an aggregate:

- **Watch scope** is configuration. It receives the select/deselect commands and
  emits the selection events; it is not an aggregate.
- **Known repository index** is a record built from observation and manual entry.
- **Catch-up pending** is a per-repository marker in watcher state.
- **Pull request stack** is a GitHub fact recorded on the Watched Pull Request;
  passuff issues no stack commands, so it is not an aggregate.
- **Notification** is a grouped report, not an aggregate.

## Model consequences for the existing implementation

- `cycle.rs:212` currently prunes watched pull requests outside the watch scope.
  Q1 Hide requires keeping them and filtering the read model instead.
- The observation currently skips out-of-scope pull requests before
  classification. The known repository index needs the repository name from
  those pruned observations without recording the pull request.
- Reselect requires a per-repository catch-up marker and a suppression path in
  the notification policy; the first cycle after reselect records activity but
  produces no notification.
- `WatchedPullRequest` and the GraphQL query need stack number, position, size,
  and stack base; the query currently fetches only `baseRefName`.
- A GraphQL error for the preview-gated stack field must degrade to standalone,
  not fail the account observation.
- The inbox read model needs stack groups and a hidden-repository filter; the
  status read model needs the same filter.

## Open questions

None. Q1–Q5 are closed and the terms are confirmed.

## Phase review (Event Storming)

- Canvas rendered with `mmdc` (Chrome at `/usr/bin/google-chrome`); all ten
  events, four commands, both aggregates, both actors, five policies, two
  records, the configuration, three read models, and the notification report
  appear in the picture.
- Cause and effect are symmetric: select/deselect, stack membership
  recorded/cleared, catch-up pending/completed, activity recorded/notification
  produced with a suppression path.
- No record, configuration, external fact, or report is drawn as an aggregate.
- Every confirmed term records a collision check; known repository and
  catch-up were confirmed before the canvas relied on them.
- Corrected during review: repository-recorded and activity-recorded emission
  now match the canvas paths, and manual entry now reaches the same
  repository-recorded event.
