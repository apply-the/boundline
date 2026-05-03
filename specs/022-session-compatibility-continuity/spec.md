# Feature Specification: Session And Compatibility Continuity

**Feature Branch**: `022-session-compatibility-continuity`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Tighten session and compatibility continuity so explicit compatibility runs reconnect cleanly to status, next, and inspect follow-up surfaces with explicit routing ownership and shared adaptive, review, and governance summaries"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.
  When both a session-native workflow and a compatibility workflow exist, the spec MUST name which path is primary and keep compatibility behavior explicit rather than implicit.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Continue After An Explicit Compatibility Run (Priority: P1)

A developer who intentionally used the explicit compatibility path can run follow-up commands and immediately see whether Boundline expects session-native continuation, trace inspection only, or an explicit non-resumable terminal outcome.

**Why this priority**: This is the main operator gap left after `0.21.0`. Compatibility runs already persist usable traces, but the next command and state ownership story are still too implicit.

**Independent Test**: Can be fully tested by running a compatibility execution profile in a workspace that also has an active native session, then verifying that `status`, `next`, and `inspect` consistently explain which persisted state is authoritative and what the operator can do next.

**Acceptance Scenarios**:

1. **Given** a workspace with an active session-native plan and a later explicit compatibility `run`, **When** the developer executes `status`, **Then** Boundline preserves the native session state and also surfaces the latest compatibility follow-up continuity without implying the compatibility run replaced the active session.
2. **Given** a compatibility run that produced a latest workspace trace but no resumable compatibility session, **When** the developer executes `next`, **Then** Boundline reports an inspect-oriented next action instead of implying that `step` or `run` will resume hidden compatibility work.
3. **Given** a compatibility run that ended in an explicit non-success terminal state, **When** the developer executes `inspect`, **Then** Boundline resolves the latest compatibility trace and explains the terminal reason plus the route ownership clearly.

---

### User Story 2 - Reuse One Summary Vocabulary Across Routes (Priority: P2)

An operator can compare native and compatibility traces without relearning different wording for adaptive, review, governance, routing, and follow-up summaries.

**Why this priority**: The route model only becomes trustworthy if overlapping concepts are named consistently. Otherwise continuity remains technically present but still confusing in practice.

**Independent Test**: Can be fully tested by executing representative native and compatibility runs that include adaptive, review, or governance evidence, then verifying that `status`, `next`, and `inspect` expose aligned summary fields while still naming the route that actually ran.

**Acceptance Scenarios**:

1. **Given** native and compatibility traces that both include adaptive or governance evidence, **When** the developer inspects them, **Then** Boundline uses the same headline and summary vocabulary where the concepts overlap while still preserving explicit route attribution.
2. **Given** a compatibility run in a workspace with workflow or governance configuration, **When** the developer checks follow-up output, **Then** Boundline shows the shared summary fields without implying workflow-owned or Canon-owned orchestration.

---

### User Story 3 - Release One Coherent Operator Story (Priority: P3)

A maintainer can ship the continuity slice with docs, assistant guidance, and release notes that explain exactly how explicit compatibility runs relate to session-native follow-up.

**Why this priority**: This slice changes operator expectations. Without release guidance, the runtime behavior may improve while assistant prompts, docs, and changelog still teach the older ambiguous story.

**Independent Test**: Can be fully tested by following the updated docs and assistant guidance in a workspace that uses both routes, then verifying that the documented commands and follow-up expectations match the actual CLI behavior.

**Acceptance Scenarios**:

1. **Given** a maintainer reads the shipped docs for `0.22.0`, **When** they run a compatibility flow followed by `status`, `next`, and `inspect`, **Then** the documented continuity story matches the observed routing and follow-up behavior.
2. **Given** an assistant or operator expects compatibility execution to become a hidden session-native continuation, **When** they consult the shipped guidance, **Then** that expectation is explicitly rejected.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when a workspace has both an active native session and a newer compatibility trace with a different terminal outcome?
- What happens when a compatibility run fails before producing a resumable session state but still persists a trace?
- What happens when `status` or `next` are invoked in a workspace with no active session but a latest compatibility trace is present?
- What happens when adaptive, review, or governance summaries are present on one route but not the other?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST define an explicit continuity model for follow-up after an explicit compatibility `run`, including whether the authoritative follow-up state is an active session, a latest workspace trace, or an explicit no-session result.
- **FR-002**: System MUST preserve session-native state when a later explicit compatibility run occurs in the same workspace and MUST NOT silently replace the active native plan or decision history.
- **FR-003**: System MUST make `status`, `next`, and `inspect` report the continuity model consistently for compatibility follow-up instead of leaving resumability to operator inference.
- **FR-004**: System MUST keep route ownership explicit in all follow-up surfaces so compatibility execution never appears to have silently become session-native, workflow-owned, or Canon-owned.
- **FR-005**: System MUST expose the latest compatibility follow-up evidence through bounded summaries that include routing, execution path, terminal or recovery condition, and any available adaptive, review, or governance evidence.
- **FR-006**: System MUST preserve explicit non-success outcomes when no resumable compatibility continuation exists and MUST NOT invent hidden background progression.
- **FR-007**: System MUST reuse the same summary vocabulary across native and compatibility traces where adaptive, review, governance, and terminal concepts overlap.
- **FR-008**: System MUST keep compatibility follow-up bounded to persisted workspace state and traces already produced by Boundline rather than introducing open-ended trace search or external reconciliation services.
- **FR-009**: System MUST ship release-aligned maintainer and operator guidance for the continuity slice, including version bump, changed route expectations, and updated follow-up examples.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: explicit continuity rules after compatibility runs; bounded follow-up behavior for `status`, `next`, and `inspect`; shared summary wording across native and compatibility traces; release-aligned docs and assistant guidance for the updated route story.
- **Out of Scope**: broadening adaptive mutation families; promoting compatibility execution to the primary session-native route; generic workflow-engine behavior; Canon-owned orchestration decisions; distributed trace reconciliation; provider-routing or UI work.

### Key Entities *(include if feature involves data)*

- **Compatibility Follow-Up State**: The bounded persisted state that tells later commands whether a compatibility run left an inspectable trace, a resumable continuation, or only a terminal outcome.
- **Route Continuity Summary**: The operator-facing explanation that names the active route, the authoritative persisted state, and the next bounded follow-up action.
- **Shared Summary Surface**: The aligned set of routing, adaptive, review, governance, and terminal fields reused across native and compatibility outputs when the concepts overlap.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative workspaces that use both native and compatibility routes, developers can determine the correct follow-up command after a compatibility run in under 2 minutes without reading raw trace JSON.
- **SC-002**: 100% of compatibility follow-up flows stop in an explicit inspectable or resumable state rather than leaving route ownership ambiguous.
- **SC-003**: Native and compatibility trace summaries use the same operator-facing wording for overlapping adaptive, review, governance, and terminal concepts in all representative validation scenarios.
- **SC-004**: Maintainers can follow the shipped `0.22.0` docs to reproduce the continuity story and confirm the route boundaries in under 15 minutes.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Compatibility execution remains an explicit manifest-backed route in `0.22.0`, even when the same workspace also has session-native state.
- Existing workspace traces under `.boundline/traces/` remain the authoritative persisted evidence for compatibility follow-up; the slice does not introduce a new trace store.
- The highest-value improvement is continuity and inspectability across existing surfaces, not adding new execution power.
- The slice closes as `0.22.0` and therefore includes version bump, impacted docs, changelog updates, clippy cleanup, formatting, and coverage refresh for modified Rust files.
