# ADR Template

This is the template for an individual Architecture Decision Record (ADR). Copy
this file to `docs/arc42/adr/ADR-NNNN-short-slug.md` and tailor it to the
decision. It is self-contained: frontmatter, the decision rationale, the real
alternatives, and the consequences. The concise qualification is recorded once
in the change assessment (`arc42.md`); this file does not repeat it as another
worksheet or structured verdict.

Before writing or modifying an ADR, read the resolved Arc42 addon instruction
(`givn instructions arc42-docs`) and the concise assessment in the change's
`arc42.md`. A candidate whose architectural commitment, real alternatives,
durable consequence, or existing-decision search does not qualify is not an ADR.

<!-- ========================================================================
     Copy everything below this banner into the new ADR file, then replace
     every {{...}} value and remove the instructional comments.
     ======================================================================== -->

---

status: proposed
date: {YYYY-MM-DD}
decision-makers: {list everyone involved}
consulted: {list subject-matter experts consulted}
informed: {list everyone kept up-to-date}
---

# {Short title representative of solved problem and found solution}

## Context and Problem Statement

{Describe the context and problem in 2-4 sentences. Make the scope explicit by
pointing to structural elements such as components and connectors.}

## Decision Drivers

- desired software quality, constraint, or concern
- ...

## Considered Options

- Option title 1
- Option title 2

List real, distinct alternatives, not strawmen.

## Decision Outcome

Chosen option: "Option title N", because {justification, giving the chosen
option plus its single most important reason}.

### Consequences

- Good, because project-specific positive consequence
- Bad, because project-specific negative consequence

Keep both consequences. A routine tradeoff is not automatically risk debt; an
actionable unresolved downside also gets a line in Arc42 chapter 11 with its
location, consequence, and remediation or deferral reason.

### Confirmation

Name the concrete test type, fitness function, linting rule, or review
checkpoint that confirms implementation and compliance. Do not write merely "we
will review it".

## Pros and Cons of the Options

### Option title 1

- Good, because argument
- Neutral, because argument
- Bad, because argument

### Option title 2

- Good, because argument
- Bad, because argument

## More Information

Additional evidence, team agreement, links to related decisions, or resources.
