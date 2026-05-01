# Feature Specification: Unify Route Summaries And Config Projection

**Feature Branch**: `024-unify-route-summaries`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Unify route summaries and config projection across native, workflow, review, governance, and compatibility surfaces while preserving explicit route ownership."

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

### User Story 1 - Read One Coherent Follow-Up Story (Priority: P1)

An operator can use `status`, `next`, `inspect`, and workflow follow-up commands to read one coherent bounded summary vocabulary across session-native, workflow, review, governance, and explicit compatibility runs without losing which route currently owns the work.

**Why this priority**: Route ownership is already explicit, but the operator-facing summaries still feel fragmented. Aligning the summary model is the highest-value next step because it reduces ambiguity on every existing route without introducing a new surface.

**Independent Test**: Can be fully tested by driving representative native, workflow, review/governance, and explicit compatibility scenarios, then verifying that the follow-up commands expose aligned summary fields and wording while still naming the owning route and authority.

**Acceptance Scenarios**:

1. **Given** a workspace with an active session-native run, **When** the operator calls `status`, `next`, and `inspect`, **Then** the returned summary uses the same bounded route, authority, execution-condition, and next-step vocabulary that workflow-aware surfaces use for the same state.
2. **Given** a workspace whose latest authoritative follow-up state comes from an explicit compatibility trace rather than an active session, **When** the operator calls `status`, `next`, or `inspect`, **Then** Synod aligns the summary wording with the native route surfaces while still making explicit that compatibility owns the current follow-up story.
3. **Given** a workflow run that is paused or blocked in review or governance, **When** the operator reads the follow-up surfaces, **Then** Synod uses the same summary vocabulary as other routes for condition, authority, and recommended next action without implying that a different route owns the work.

---

### User Story 2 - Surface Routing And Config Inputs Explicitly (Priority: P2)

An operator can see the routing and configuration inputs that explain why Synod chose a native, workflow, review/governance, or compatibility follow-up path, instead of inferring that behavior from scattered or route-specific output.

**Why this priority**: Summary convergence is only trustworthy if the configuration and routing inputs that shaped the result are visible at the same time. Without that projection, unified wording risks hiding why a route was chosen.

**Independent Test**: Can be fully tested by configuring representative workspace and global defaults, running mixed-route scenarios, and verifying that the follow-up surfaces project the active route, routing source, relevant config defaults, and any explicit workflow or compatibility choice in the same operator-facing summary family.

**Acceptance Scenarios**:

1. **Given** a workspace with routing defaults, workflow metadata, and governance settings that affect follow-up guidance, **When** the operator checks `status` or `inspect`, **Then** Synod surfaces the relevant config and routing projections alongside the unified summary without requiring the operator to inspect multiple commands manually.
2. **Given** a direct explicit compatibility run overrides available session-native or workflow defaults, **When** follow-up commands are rendered, **Then** Synod shows the override and preserves explicit compatibility ownership instead of presenting the result as if it came from config-driven native routing alone.
3. **Given** a workspace mixes session-native state with review/governance pauses or workflow metadata, **When** the operator reads the summary surfaces, **Then** Synod projects only the configuration and routing facts that materially affect follow-up while excluding irrelevant or stale settings.

---

### User Story 3 - Ship The Unified Story As One Release (Priority: P3)

A maintainer can ship one `0.24.0` release where runtime behavior, docs, assistant guidance, version metadata, and changelog all describe the same unified route-summary and config-projection story.

**Why this priority**: This slice changes how operators interpret multiple existing routes. The release is incomplete if the runtime and its docs diverge or if validation, coverage, and release metadata are not refreshed together.

**Independent Test**: Can be fully tested by following the shipped docs and assistant guidance on a mixed-route workspace, then verifying that runtime output, versioned documentation, and release metadata all describe the same route-summary and config-projection behavior.

**Acceptance Scenarios**:

1. **Given** the `0.24.0` release artifacts, **When** a maintainer follows the updated docs for mixed-route follow-up, **Then** the observed CLI output matches the documented summary vocabulary, ownership rules, and config-projection behavior.
2. **Given** changed Rust sources and summary logic for this slice, **When** maintainers run the project validation suite, **Then** formatting, clippy, targeted coverage, and the required tests pass without introducing undocumented output regressions.

---

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when the latest authoritative state comes from a compatibility trace but a stale active session record is still present in the workspace?
- What happens when workflow, review, or governance metadata exists but the current follow-up state is already terminal and no resumable action remains?
- What happens when workspace-local and user-global routing defaults disagree with an explicit command-line route choice?
- What happens when a unified summary surface cannot project a config field because the underlying route never produced that state?
- What happens when the same execution condition name appears across routes but the owning route still needs different corrective guidance?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST project a shared operator-facing summary model for follow-up state across session-native, workflow, review/governance, and explicit compatibility routes.
- **FR-002**: System MUST preserve explicit route ownership, continuity authority, and execution-condition semantics even when summary wording converges across routes.
- **FR-003**: System MUST align `run`, `status`, `next`, `inspect`, and workflow follow-up wording around one bounded vocabulary for current state, latest authority, and recommended next action.
- **FR-004**: System MUST migrate more review/governance and compatibility follow-up state onto the session-native summary model instead of keeping route-specific summary gaps where equivalent bounded meaning already exists.
- **FR-005**: System MUST project routing and configuration inputs that materially explain the current follow-up story, including explicit route overrides and relevant workspace or global defaults when they affect interpretation.
- **FR-006**: System MUST omit irrelevant or stale config projections so unified summaries do not imply that inactive settings control the current route.
- **FR-007**: System MUST keep explicit compatibility ownership visible when the latest authoritative follow-up state comes from compatibility rather than an active session, even if the summary vocabulary matches native surfaces more closely.
- **FR-008**: System MUST keep workflow, review, and governance paused, blocked, failed, and completed states inspectable through the same summary family without implying hidden promotion into a different orchestration route.
- **FR-009**: System MUST handle mixed-route and non-success states explicitly, including terminal, blocked, paused, failed, exhausted, or inspect-only follow-up outcomes.
- **FR-010**: System MUST update runtime projections, tests, release version metadata, docs, assistant guidance, and changelog together for the `0.24.0` release.
- **FR-011**: System MUST keep the slice bounded to summary and projection convergence, without adding a new orchestration engine, provider gateway, hidden compatibility promotion, or distributed execution surface.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: shared follow-up summary projection across native, workflow, review/governance, and compatibility routes; explicit projection of routing/config facts that affect interpretation; preservation of route ownership while vocabulary converges; release-aligned version bump, impacted docs, assistant guidance, changelog, coverage, clippy cleanup, and formatting.
- **Out of Scope**: new execution engines or background services; hidden promotion of compatibility into the primary native route; provider-agnostic model routing; cross-repository orchestration expansion; UI or dashboard work; Canon-owned control flow beyond existing governed-stage boundaries.

### Key Entities *(include if feature involves data)*

- **Unified Route Summary**: The bounded operator-facing projection that describes current route, authority, execution condition, next action, and relevant route evidence using one shared vocabulary.
- **Route Ownership Projection**: The explicit statement of which route currently owns follow-up state, such as session-native, workflow-owned state, review/governance pause, or compatibility trace authority.
- **Config Projection**: The subset of workspace or global routing and governance defaults, workflow metadata, and explicit route overrides that materially explain the current follow-up story.
- **Follow-Up Authority State**: The persisted state that determines whether the operator should resume a session, inspect a compatibility trace, wait on governance, or treat the route as terminal.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative native, workflow, review/governance, and compatibility scenarios, operators can identify the current route owner, follow-up authority, execution condition, and recommended next action from one summary surface in under 2 minutes.
- **SC-002**: 100% of supported mixed-route follow-up scenarios stop in an explicit completed, paused, blocked, failed, exhausted, or inspect-only state without requiring operators to infer route ownership from missing fields.
- **SC-003**: Maintainers can verify the `0.24.0` unified summary story by running the documented commands and observing matching runtime and documentation behavior in under 15 minutes.
- **SC-004**: Changed Rust sources for this slice pass formatting, clippy, required tests, and refreshed coverage checks before release closeout.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Session-native orchestration remains the primary route for new work, while compatibility continues as an explicit route with its own ownership even when summaries become more aligned.
- Existing route and follow-up concepts such as `continuity_authority`, `execution_condition`, review/governance pauses, and inspect-only compatibility guidance remain valid and should be converged rather than replaced.
- The slice closes as `0.24.0`, so version bump, impacted docs, assistant guidance, changelog, coverage refresh for modified Rust files, clippy cleanup, and formatting are required deliverables rather than optional polish.
- The repo should reuse existing session, trace, workflow, and configuration persistence surfaces instead of adding a new persistence file for summary projection alone.
