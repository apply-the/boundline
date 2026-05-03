# Research: Goal Negotiation And Constraint Modeling

**Feature**: 026-goal-constraint-modeling  
**Date**: 2026-05-01

## R1: Generate the negotiation packet automatically during capture

**Decision**: Derive one negotiated delivery packet as part of the existing
session-native capture step instead of adding a new standalone `negotiate`
command.

**Rationale**: The roadmap and spec both position negotiation as the boundary
between captured intent and planning, not as a separate operator workflow.
Running negotiation during capture keeps the slice small, preserves the current
`start -> capture -> plan` story, and ensures that plan gating can rely on one
authoritative negotiation result.

**Alternatives Considered**:
- Add a new explicit `negotiate` command: rejected because it teaches a second
  workflow step before the feature proves its value.
- Delay negotiation until planning: rejected because it keeps the most
  important acceptance and constraint decisions hidden inside planner logic.

## R2: Persist negotiation state in the existing session and task-context surfaces

**Decision**: Store the negotiated delivery packet as optional session-owned
state, project it into task context when a task is planned, and reference it in
 traces/events instead of creating a new negotiation persistence file.

**Rationale**: `.boundline/session.json`, task context, and traces already define
the local state model for the current operator story. Reusing them keeps one
authoritative source of truth and makes negotiation visible to later commands
without adding reconciliation complexity.

**Alternatives Considered**:
- Add a dedicated `.boundline/negotiation.json`: rejected because it would create a
  second local authority to keep in sync with sessions and traces.
- Recompute negotiation state on demand from the goal and briefs only: rejected
  because follow-up surfaces need a stable, inspectable decision record.

## R3: Reuse authored brief, clarification, and governance intent as negotiation evidence

**Decision**: Build the negotiation packet from the existing captured goal,
authored brief bundle, clarification model, governance intent, workspace
signals, and current run limits instead of introducing a broader questionnaire
system.

**Rationale**: The repository already normalizes authored inputs and records
clarifications when requests are ambiguous or unbounded. Reusing those signals
lets the slice deliver explicit negotiation value now without expanding into a
new interactive product surface.

**Alternatives Considered**:
- Add a new multi-question interview loop: rejected because it expands the
  operator surface beyond the minimal valuable slice.
- Model only execution limits and ignore goal/acceptance constraints: rejected
  because the roadmap explicitly prioritizes acceptance boundaries and tradeoff
  visibility before execution begins.

## R4: Gate planning on a credible negotiation result and carry it through follow-up surfaces

**Decision**: Require planning to use a credible negotiated packet, preserve a
summary of active constraints and tradeoff rationale in the goal-plan/session
projection, and surface the binding story through `plan`, `run`, `status`,
`next`, and `inspect`.

**Rationale**: The feature is only valuable if the operator can see which
constraints were honored and which tradeoff was chosen after capture. Gating the
planner on the negotiation result fixes the hidden-heuristic gap at the root of
the roadmap item.

**Alternatives Considered**:
- Keep negotiation visible only in capture output: rejected because the story
  would disappear when the operator needs it most during follow-up.
- Add a separate reporting surface for negotiation only: rejected because it
  would drift from the aligned session-native summaries operators already use.

## R5: Keep compatibility and cluster authority explicit rather than implicit

**Decision**: Treat the negotiated packet as primary-session state, make
explicit compatibility follow-up state say when it lacks authoritative
negotiation ownership, and project cluster negotiation from the primary
workspace when a clustered session is active.

**Rationale**: Recent features made route ownership and clustered authority
explicit. Negotiation must follow the same rule so follow-up output never
suggests a hidden session-native authority on an explicit compatibility route.

**Alternatives Considered**:
- Silently synthesize session-native negotiation authority for compatibility
  traces: rejected because it violates explicit ownership.
- Duplicate packet ownership into every cluster member workspace: rejected
  because clustered authority already lives in the primary workspace session.

## R6: Close the slice as 0.26.0 with release-aligned validation

**Decision**: Include version bump, impacted docs, assistant guidance,
changelog, touched-Rust coverage refresh, clippy cleanup, and repository
formatting as first-class implementation tasks for the feature.

**Rationale**: The operator story changes as soon as capture and planning become
explicitly negotiated. The release must ship one coherent runtime and
documentation story.

**Alternatives Considered**:
- Defer release closeout until after runtime work lands: rejected because it
  risks shipping a partially explained operator workflow.
- Stop at tests and skip coverage refresh for touched Rust files: rejected
  because the requested release discipline explicitly includes coverage, clippy,
  and formatting.