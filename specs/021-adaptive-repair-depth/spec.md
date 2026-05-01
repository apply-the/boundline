# Feature Specification: Adaptive Repair Depth

**Feature Branch**: `021-adaptive-repair-depth`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Broaden adaptive repair depth so Synod can use validation failure evidence to choose more credible bounded repair attempts, preserve explicit workspace-slice and attempt-lineage evidence, and keep the same bounded orchestration story without introducing Canon-owned control."

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

### User Story 1 - Replan Adaptive Repairs From Validation Evidence (Priority: P1)

A developer running bounded adaptive delivery can have Synod choose the next repair candidate from the latest validation failure evidence instead of exhausting only deterministic syntax flips in a fixed order.

**Why this priority**: The current adaptive path already scores a bounded workspace slice and avoids repeated signatures, but it still relies on deterministic local patterns that ignore why validation failed. This is the smallest credible step toward deeper adaptive repair.

**Independent Test**: Can be fully tested by running an adaptive compatibility execution profile whose first candidate fails validation with actionable failure output, then verifying that Synod selects a materially different next candidate because of that failure evidence and either succeeds or stops explicitly within the configured limits.

**Acceptance Scenarios**:

1. **Given** an adaptive compatibility run whose first validation failure contains actionable error terms or location hints, **When** Synod replans the next bounded attempt, **Then** it prioritizes a new candidate using that failure evidence instead of falling back to the same deterministic candidate order.
2. **Given** multiple bounded candidates remain available after a failed validation, **When** one candidate aligns better with the latest failure evidence, **Then** Synod selects that candidate, records why it was preferred, and leaves previously rejected candidates visible in attempt lineage rather than silently forgetting them.
3. **Given** a failed validation produces no credible new bounded repair path, **When** Synod evaluates the next adaptive attempt, **Then** it stops in an explicit failed or exhausted terminal state with visible evidence that no materially different bounded candidate remained.

---

### User Story 2 - Inspect Adaptive Selection And Route Boundaries Clearly (Priority: P2)

An operator can understand why an adaptive attempt changed, which workspace slice remains in play, and how that adaptive run still relates to the explicit compatibility route even when the workspace also uses session-native workflows, review, or governance.

**Why this priority**: Stronger heuristics are only useful if the selection rationale is inspectable. Adaptive behavior that changes silently would violate Synod's bounded, explicit execution model.

**Independent Test**: Can be fully tested by running an adaptive compatibility scenario that replans after failed validation, then verifying that `run`, `status`, `next`, and `inspect` surface the validation-guided rationale, updated workspace slice or candidate ordering, and explicit compatibility routing without implying workflow-owned or Canon-owned orchestration.

**Acceptance Scenarios**:

1. **Given** an adaptive run that replans after failed validation, **When** the developer checks `status`, `next`, or `inspect`, **Then** Synod exposes the latest validation-guided selection reason, bounded workspace slice, attempt lineage, and terminal or recovery condition.
2. **Given** a workspace that also defines named workflows or bounded review or governance configuration, **When** adaptive compatibility execution is inspected, **Then** Synod keeps the routing story explicit and does not imply that workflows, review councils, or Canon own the adaptive control flow.

---

### User Story 3 - Ship Bounded Adaptive Repair Guidance Cleanly (Priority: P3)

A maintainer can configure and release the deeper adaptive repair slice without mistaking it for open-ended autonomous code generation or a new orchestration mode.

**Why this priority**: This slice changes how adaptive candidates are chosen, so operator and maintainer guidance must stay explicit about what became smarter and what remains bounded.

**Independent Test**: Can be fully tested by following the shipped docs to author a representative adaptive execution profile, confirm the new validation-guided behavior, and verify that unsupported expectations such as session-native adaptive ownership or unbounded candidate search remain explicitly out of scope.

**Acceptance Scenarios**:

1. **Given** a maintainer configures an adaptive profile with bounded read targets and validation-guided change kinds, **When** they follow the shipped examples, **Then** Synod accepts the supported profile and documents the resulting route and inspection story coherently.
2. **Given** a maintainer expects open-ended repository exploration, implicit workflow ownership, or Canon-owned adaptive control, **When** they consult the shipped docs or run the feature, **Then** those expectations remain explicitly unsupported.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when validation stderr or stdout contains multiple plausible hints that point to different bounded repair paths?
- What happens when the latest failure evidence points back to a candidate signature that Synod already tried and rejected earlier in the same run?
- What happens when adaptive execution runs in a workspace that also exposes named workflows, review state, or governance configuration but the operator is still on the explicit compatibility route?
- What happens when no validation output is available even though the command failed?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST preserve adaptive execution as an explicit bounded compatibility path rather than silently promoting it into hidden session-native or workflow-owned orchestration.
- **FR-002**: System MUST use the latest validation failure evidence from the active adaptive run to rank or filter the next bounded repair candidates when a replan is required.
- **FR-003**: System MUST preserve bounded workspace-slice selection, candidate signatures, attempt lineage, and configured execution limits while applying validation-guided adaptive repair.
- **FR-004**: System MUST record explicit selection evidence for validation-guided adaptive replans, including the failure hints or rationale that made the selected candidate credible.
- **FR-005**: System MUST avoid repeating a materially identical failed adaptive candidate unless new bounded evidence makes that retry credible.
- **FR-006**: System MUST stop every validation-guided adaptive run in an explicit succeeded, failed, or exhausted terminal state when no credible new bounded candidate remains.
- **FR-007**: System MUST expose validation-guided adaptive selection reasons, workspace-slice evidence, attempt lineage, validation outcomes, and terminal or recovery conditions through `run`, `status`, `next`, and `inspect`.
- **FR-008**: System MUST keep the routing relationship explicit when adaptive compatibility execution appears in a workspace that also supports session-native workflows, review, or governance, and MUST NOT imply that those surfaces own adaptive control flow.
- **FR-009**: System MUST reject or stop explicitly when validation evidence is absent, ambiguous, or insufficient to justify a materially different bounded adaptive candidate.
- **FR-010**: System MUST ship release-aligned maintainer and operator guidance for the deeper adaptive repair slice, including version bump, updated examples, and changed release notes.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: validation-guided adaptive candidate selection and replanning on the existing compatibility execution path; explicit adaptive selection evidence and attempt-lineage reporting; bounded route guidance when workflows, review, or governance also exist in the workspace; release-aligned docs for the delivered slice.
- **Out of Scope**: moving adaptive execution onto the primary session-native route; open-ended repository exploration; provider-driven code generation heuristics; Canon-owned orchestration; concurrency; generic workflow-engine behavior; UI or deployment work.

### Key Entities *(include if feature involves data)*

- **Validation Guidance**: The bounded subset of the latest validation failure evidence that can credibly influence adaptive replanning, such as error terms, file or line hints, and explicit mismatch text.
- **Adaptive Candidate Ranking**: The ordered set of bounded repair candidates considered for the next adaptive attempt after combining workspace-slice scoring, candidate signatures, and validation guidance.
- **Adaptive Selection Evidence**: The inspectable explanation of why one bounded repair candidate was selected after failed validation and why other candidates were not.
- **Attempt Lineage**: The explicit relationship between adaptive attempts, including whether the next attempt narrowed, replaced, or exhausted the prior path and what validation evidence justified that change.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative adaptive failure scenarios with actionable validation output, Synod chooses a materially different second bounded candidate based on that output instead of only following the original deterministic candidate order.
- **SC-002**: 100% of validation-guided adaptive runs stop in an explicit succeeded, failed, or exhausted terminal state within configured execution limits.
- **SC-003**: Developers can identify the latest adaptive selection reason, bounded workspace slice, and attempt lineage from `status` or `inspect` in under 2 minutes.
- **SC-004**: Maintainers can configure and validate a representative validation-guided adaptive profile, plus find the changed route and release guidance, in under 15 minutes without relying on undocumented behavior.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Adaptive execution remains manifest-backed through `<workspace>/.synod/execution.json` for this slice, even when the same workspace also uses session-native workflows for other delivery paths.
- The first delivery gain comes from improving candidate ranking and selection with local validation evidence, not from introducing new open-ended change generators.
- Existing trace, session, review, and governance surfaces remain the operator-facing observability layers; this slice should deepen those summaries rather than introduce a new adaptive dashboard.
- The slice closes as the next versioned increment and therefore includes version bump, impacted docs, changelog updates, coverage refresh for modified Rust files, clippy cleanup, and formatting.
