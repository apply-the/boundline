# Feature Specification: Broaden Bounded Adaptive Repair

**Feature Branch**: `023-broaden-bounded-adaptive-repair`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Broaden bounded adaptive repair so explicit compatibility execution can use richer deterministic mutation families, clearer credibility and exhaustion reporting, and continuity-aware follow-up across status, next, and inspect"

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

### User Story 1 - Repair More Credible Bounded Failures (Priority: P1)

A developer using the explicit compatibility route can have Synod try materially different bounded repair strategies after failed validation instead of exhausting only arithmetic, comparison, or boolean flips.

**Why this priority**: The adaptive path is already useful, but it is too narrow to recover many representative failures. Expanding bounded mutation families is the largest delivery gain available without changing route ownership.

**Independent Test**: Can be fully tested by running representative adaptive compatibility profiles whose first attempt fails for non-arithmetic reasons, then verifying that Synod selects a different bounded candidate family, updates the trace and follow-up surfaces, and either succeeds or stops explicitly within configured limits.

**Acceptance Scenarios**:

1. **Given** an adaptive compatibility run whose first validation failure points to a bounded textual replacement, missing branch, or wrong literal rather than an arithmetic mismatch, **When** Synod replans, **Then** it can choose a different supported bounded mutation family instead of exhausting immediately.
2. **Given** several bounded mutation families remain credible for the selected workspace slice, **When** the latest validation evidence aligns better with one family than the others, **Then** Synod prefers that family and records why it was selected.
3. **Given** a candidate family has already produced a materially identical failed result in the same run, **When** Synod considers the next bounded attempt, **Then** it avoids repeating that failed candidate unless new bounded evidence justifies it.

---

### User Story 2 - Explain Credibility And Exhaustion Clearly (Priority: P2)

An operator can tell why one adaptive candidate was chosen, why another candidate was skipped, and when the bounded compatibility path is truly exhausted instead of merely failing without explanation.

**Why this priority**: Broader mutation families are only trustworthy if the selection logic remains inspectable. Synod cannot become more adaptive by becoming less explicit.

**Independent Test**: Can be fully tested by running adaptive compatibility scenarios that replan multiple times, then verifying that `run`, `status`, `next`, and `inspect` expose candidate credibility, rejection reasons, exhaustion reasons, and explicit compatibility ownership.

**Acceptance Scenarios**:

1. **Given** an adaptive run that chooses one bounded candidate over several alternatives, **When** the developer checks `status` or `inspect`, **Then** Synod explains the latest credibility reason, selection headline, attempt lineage, and why the rejected alternatives were not chosen.
2. **Given** an adaptive run reaches a state where no remaining bounded candidate is credible, **When** the run stops, **Then** Synod reports an explicit exhausted or failed terminal outcome with enough evidence to show why bounded recovery ended.
3. **Given** a workspace also has session-native state, named workflows, review configuration, or governance configuration, **When** adaptive compatibility execution is inspected, **Then** Synod preserves the explicit compatibility route story and does not imply session-native, workflow-owned, or Canon-owned adaptive control.

---

### User Story 3 - Ship One Complete Adaptive Operator Story (Priority: P3)

A maintainer can configure, validate, and release the deeper adaptive slice with docs and assistant guidance that make the new mutation depth and bounded limits unambiguous.

**Why this priority**: This release changes what adaptive compatibility execution can do. If docs and assistant prompts remain shallow or ambiguous, the runtime change will be harder to use correctly.

**Independent Test**: Can be fully tested by following the shipped docs and assistant guidance to run a representative bounded adaptive profile, observe richer mutation selection, and confirm that unsupported expectations such as open-ended code generation or hidden session-native ownership are still rejected.

**Acceptance Scenarios**:

1. **Given** a maintainer follows the shipped `0.23.0` guidance for adaptive execution, **When** they run representative compatibility scenarios, **Then** the documented mutation breadth, credibility story, and exhaustion behavior match the CLI outputs.
2. **Given** an assistant or operator expects open-ended repository search, hidden resumability, or Canon-owned adaptive planning, **When** they consult the shipped guidance, **Then** those expectations remain explicitly unsupported.

---

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when validation output points to multiple plausible bounded mutation families with different credibility levels?
- What happens when the latest failure evidence points back to a candidate signature that already failed earlier in the same adaptive run?
- What happens when no validation output is available even though the compatibility validation command failed?
- What happens when all remaining mutation families are technically allowed but none is credible enough to justify another bounded attempt?
- What happens when adaptive compatibility execution is inspected in a workspace that also has an active native session or named workflow state?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST keep deeper adaptive repair on the explicit compatibility route rather than silently promoting it into the primary session-native or workflow-owned route.
- **FR-002**: System MUST broaden bounded adaptive mutation support beyond arithmetic, comparison, and boolean flips so representative compatibility repair scenarios can try materially different bounded candidates.
- **FR-003**: System MUST keep mutation selection deterministic and bounded to manifest-declared scope, built-in supported change families, and explicit execution limits.
- **FR-004**: System MUST use the latest validation evidence to rank, prefer, or reject bounded mutation families and candidates when replanning is required.
- **FR-005**: System MUST preserve candidate signatures and avoid repeating a materially identical failed adaptive candidate unless new bounded evidence changes its credibility.
- **FR-006**: System MUST persist and surface explicit credibility evidence for the selected candidate, including why it was chosen and why other bounded candidates were not chosen when that information is available.
- **FR-007**: System MUST stop every deeper adaptive run in an explicit succeeded, failed, or exhausted terminal state when configured limits are hit or no credible bounded candidate remains.
- **FR-008**: System MUST expose richer adaptive evidence through `run`, `status`, `next`, and `inspect`, including selection headline, bounded workspace slice, attempt lineage, latest credibility reason, and explicit exhaustion or rejection rationale.
- **FR-009**: System MUST preserve the continuity-aware follow-up story from `0.22.0`, including explicit compatibility ownership and inspect-oriented follow-up when no resumable session exists.
- **FR-010**: System MUST reject or stop explicitly when validation evidence is absent, ambiguous, or insufficient to justify another materially different bounded adaptive candidate.
- **FR-011**: System MUST ship release-aligned maintainer, operator, and assistant guidance for the deeper adaptive slice, including version bump, impacted docs, and changelog updates.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: broader deterministic bounded mutation families on the compatibility route; validation-guided credibility and exhaustion handling; richer adaptive selection and rejection evidence on `run`, `status`, `next`, and `inspect`; release-aligned docs, assistant guidance, and changelog updates.
- **Out of Scope**: open-ended autonomous code generation; promotion of adaptive repair into hidden session-native ownership; background daemons or generic retry engines; Canon-owned adaptive planning; UI, deployment, or provider-routing work.

### Key Entities *(include if feature involves data)*

- **Adaptive Mutation Family**: A bounded built-in change category that can generate deterministic repair candidates for the selected workspace slice.
- **Adaptive Candidate Credibility**: The inspectable explanation of why one bounded candidate is more plausible than the remaining candidates after considering validation evidence and prior failures.
- **Adaptive Exhaustion State**: The explicit terminal explanation that no remaining bounded candidate is credible enough or allowed enough to continue the compatibility run.
- **Adaptive Selection Evidence**: The persisted summary of the chosen candidate, rejected alternatives, validation hints, and attempt-lineage relationship for the latest adaptive replan.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative adaptive compatibility scenarios that are not solvable by arithmetic, comparison, or boolean flips alone, Synod chooses a materially different bounded second candidate instead of exhausting immediately.
- **SC-002**: 100% of deeper adaptive runs stop in an explicit succeeded, failed, or exhausted terminal state within configured execution limits.
- **SC-003**: Developers can identify the latest adaptive selection reason, rejection or exhaustion rationale, and bounded workspace slice from `status` or `inspect` in under 2 minutes.
- **SC-004**: Maintainers can follow the shipped `0.23.0` docs to configure and validate a representative broader adaptive profile in under 15 minutes without depending on undocumented behavior.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Adaptive execution remains manifest-backed through `<workspace>/.synod/execution.json` for this slice, even when the same workspace also uses session-native routes for other work.
- The deeper adaptive slice should prefer additional bounded mutation families and better credibility reasoning over any new top-level execution surface.
- Existing trace, session, review, governance, and assistant-facing surfaces remain the required observability layers; this slice should deepen their summaries rather than add a new dashboard or persistence file.
- The feature closes as `0.23.0` and therefore includes version bump, impacted docs, assistant guidance, changelog updates, coverage refresh for modified Rust files, clippy cleanup, and formatting.
