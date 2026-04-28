# Feature Specification: Multi-Workspace Orchestration

**Feature Branch**: `012-multi-workspace`  
**Created**: 2026-04-28  
**Status**: Draft  
**Input**: User description: "Bounded multi-workspace orchestration for Synod: add cluster-aware session tracking, cluster-level configuration precedence, cross-workspace trace inspection, and targeted execution that can move between member repositories while preserving bounded state and inspectability."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Establish a Clustered Delivery Context (Priority: P1)

As an operator working across two or more repositories for one delivery, I want
Synod to register those repositories as one bounded cluster so that I can start
from a single entry point instead of manually stitching together separate
workspace sessions.

**Why this priority**: Without a credible clustered entry point, multi-workspace
delivery remains a manual convention rather than a real Synod capability.

**Independent Test**: Register two repositories into one cluster, start or
capture clustered work once, and verify that Synod records a shared cluster
context without mutating unrelated single-workspace flows.

**Acceptance Scenarios**:

1. **Given** two valid Synod workspaces, **When** the operator initializes a
  cluster and names both members, **Then** Synod records the cluster membership
  and reports one bounded cluster context the operator can reuse.
2. **Given** an existing cluster, **When** the operator starts clustered work,
  **Then** Synod records the active cluster identity and member list in the
  session state instead of creating unrelated per-workspace sessions.
3. **Given** one requested member path is missing, duplicated, or outside the
  allowed workspace boundary, **When** the operator tries to initialize the
  cluster, **Then** Synod stops without partial registration and explains the
  invalid member.

---

### User Story 2 - Inspect Cluster Status and Trace Context (Priority: P2)

As an operator diagnosing or reviewing clustered work, I want a unified status
and inspection view for the member workspaces so that I can see which workspace
is active, blocked, or stale without manually opening each repository.

**Why this priority**: Multi-workspace orchestration is not trustworthy if the
operator cannot inspect which member workspace moved last or where execution is
blocked.

**Independent Test**: With one clustered session and at least one recorded
trace, run cluster-aware status and inspection commands and verify that Synod
surfaces the latest member state, trace references, and blocking context.

**Acceptance Scenarios**:

1. **Given** a cluster with member workspaces in different session states,
  **When** the operator requests cluster status, **Then** Synod lists each
  member workspace, its current activity summary, and whether it matches the
  shared cluster context.
2. **Given** clustered work has produced traces in more than one member
  workspace, **When** the operator inspects the cluster, **Then** Synod shows a
  unified view that points to the latest relevant trace for each member.
3. **Given** one member workspace has no active session or latest trace,
  **When** the operator requests cluster inspection, **Then** Synod reports the
  gap explicitly rather than implying healthy state.

---

### User Story 3 - Apply Cluster Defaults Without Losing Local Control (Priority: P3)

As an operator managing related repositories, I want to save cluster-wide
defaults that member workspaces can inherit so that I can keep shared routing
and governance intent aligned while still allowing workspace-specific overrides.

**Why this priority**: Shared delivery intent becomes expensive and error-prone
 if each workspace has to duplicate the same defaults by hand.

**Independent Test**: Save one cluster-level default, keep a different local
override in one member workspace, inspect effective resolution, and verify the
reported source for each value.

**Acceptance Scenarios**:

1. **Given** a cluster-level default and no local override, **When** Synod
  resolves a supported setting for a member workspace, **Then** it uses the
  cluster-level value and reports that source.
2. **Given** a member workspace sets its own local override, **When** Synod
  resolves the same setting, **Then** the workspace-local value wins over the
  cluster-level default and the source is visible.
3. **Given** the cluster configuration is malformed or conflicts with member
  identity, **When** Synod loads the cluster defaults, **Then** it blocks the
  operation with actionable guidance instead of silently ignoring the problem.

---

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- A cluster member path may point to a valid directory that is not a Synod
  workspace; Synod must reject that member explicitly.
- A member workspace may already contain an active session that belongs to a
  different cluster identity; Synod must stop and require reconciliation.
- A cluster may contain duplicate canonical paths written in different lexical
  forms; Synod must normalize them before validation.
- Cluster inspection may find traces in some workspaces but not others; Synod
  must surface missing traces as an explicit gap, not as success.
- A cluster-scoped default may exist for a member workspace that has no local
  config file; Synod must still resolve the value deterministically.
- Single-workspace commands must continue to work unchanged when no cluster
  context is present.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: Synod MUST provide a way to register a named cluster that contains
  two or more member workspaces under one bounded delivery context.
- **FR-002**: Synod MUST validate every requested cluster member before saving
  cluster state and MUST refuse partial registration when any member is invalid.
- **FR-003**: Synod MUST persist enough cluster state for later commands to
  identify the cluster, its member workspaces, and its primary workspace.
- **FR-004**: Synod MUST let clustered start or capture flows reuse one active
  cluster identity instead of creating unrelated per-workspace session state.
- **FR-005**: Synod MUST preserve existing single-workspace session behavior
  when no cluster context is requested.
- **FR-006**: Synod MUST surface cluster status that identifies every member
  workspace and its current session summary or missing-state condition.
- **FR-007**: Synod MUST surface cluster inspection output that identifies the
  latest relevant trace reference for each member workspace or the absence of
  one.
- **FR-008**: Synod MUST make cluster inspection output explicit about which
  member workspace is blocked, stale, mismatched, or healthy.
- **FR-009**: Synod MUST support saving cluster-scoped defaults separately from
  workspace-local and user-global defaults.
- **FR-010**: Synod MUST resolve effective settings using this precedence:
  explicit CLI input, workspace-local config, cluster-level config,
  user-global config, then built-in defaults.
- **FR-011**: Synod MUST expose the source of each resolved effective value when
  cluster-aware configuration is inspected.
- **FR-012**: Synod MUST block cluster-scoped operations when the cluster
  configuration is malformed, references an unknown member, or conflicts with
  the active member identity.
- **FR-013**: Synod MUST preserve inspectable bounded state for cluster-aware
  commands so operators can understand what happened without opening internal
  files manually.
- **FR-014**: Synod MUST document the clustered workflow, precedence model, and
  operator expectations alongside the existing single-workspace workflow.
- **FR-015**: Synod MUST keep the first slice bounded to cluster registration,
  cluster-aware session/status/inspection, and inherited defaults, deferring
  fully automatic cross-repository execution planning to later slices.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: bounded cluster registration, cluster-aware session identity,
  cluster status and inspection, cluster-scoped configuration defaults,
  deterministic precedence, and documentation for operators.
- **Out of Scope**: automatic multi-repository plan generation, distributed
  execution across machines, cluster-wide voting councils beyond existing
  review behavior, provider-auth abstractions, and non-CLI user interfaces.

### Key Entities *(include if feature involves data)*

- **Workspace Cluster**: A named bounded delivery context that groups member
  workspaces, identifies a primary workspace, and carries shared intent across
  related repositories.
- **Cluster Member**: One canonical workspace path within a cluster together
  with its derived session and trace summary.
- **Cluster Session Projection**: The cluster-aware view of active session state
  that links the cluster identity to member workspace status.
- **Cluster Configuration**: Shared defaults saved at cluster scope and used
  during effective-resolution when no higher-precedence value exists.
- **Cluster Trace Summary**: The inspectable pointer to the latest relevant
  trace or explicit missing-trace state for a cluster member.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative two-workspace and three-workspace scenarios,
  operators can register a cluster and reach a reusable cluster context in under
  5 minutes without hand-editing internal files.
- **SC-002**: In representative clustered session scenarios, 100% of status and
  inspection runs identify each member workspace as healthy, missing, blocked,
  or mismatched instead of returning ambiguous state.
- **SC-003**: In representative configuration scenarios, operators can explain
  the effective source of a cluster-aware setting for any member workspace in
  under 60 seconds.
- **SC-004**: Existing single-workspace flows remain behaviorally unchanged in
  all regression coverage relevant to session, inspection, and configuration
  commands.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Operators can provide absolute or workspace-relative paths for member
  workspaces, and Synod can canonicalize them before saving cluster state.
- Each member workspace continues to own its local `.synod/` files even when it
  participates in a cluster.
- The first slice may designate one member as the primary workspace for shared
  cluster metadata and inspection entry points.
- Existing runtime, review, and governance capabilities continue to resolve
  through current single-workspace logic unless a cluster-aware default or
  projection is explicitly requested.
