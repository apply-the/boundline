# Feature Specification: Multi-Agent Review & Voting

**Feature Branch**: `007-multi-agent-review`  
**Created**: 2026-04-26  
**Status**: Draft  
**Input**: User description: "Add Spec 007 — Multi-Agent Review & Voting so Boundline can validate execution output through explicit multi-agent councils before considering a delivery task complete."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Review A Delivery Result (Priority: P1)

As a developer using Boundline to deliver code, I want Boundline to run a bounded review phase after execution so that multiple reviewers can independently assess the produced output before Boundline treats the task as complete.

**Why this priority**: Once Boundline writes code automatically, a single execution result is not enough. The immediate value is a visible quality-control step that can accept, reject, or escalate the result before the developer relies on it.

**Independent Test**: Complete a delivery run that produces reviewable output, trigger the review phase, and verify that Boundline records the participating reviewers, their findings, the vote result, and one explicit terminal review outcome.

**Acceptance Scenarios**:

1. **Given** an execution run with a completed output and review enabled, **When** Boundline enters the review phase, **Then** it launches a bounded set of independent reviewers, captures their structured findings, and records the result in the same task lifecycle.
2. **Given** a review phase where the reviewers approve the output, **When** the vote resolves in favor of acceptance, **Then** Boundline records an accepted review outcome and exposes the participating reviewers, findings summary, vote result, and final decision.
3. **Given** a review phase where blocking findings are raised, **When** the vote resolves against acceptance, **Then** Boundline records a rejected or escalated review outcome and preserves the blocking evidence for inspection.

---

### User Story 2 - Resolve Reviewer Disagreement (Priority: P2)

As a developer receiving conflicting reviewer feedback, I want Boundline to apply explicit voting and bounded adjudication so that disagreement ends in one visible outcome instead of an ambiguous review state.

**Why this priority**: Multi-agent review is only credible if disagreement is handled explicitly. Otherwise the system introduces more noise without improving delivery decisions.

**Independent Test**: Trigger a review scenario with conflicting findings, verify that Boundline applies the configured vote rule, optionally runs adjudication when required, and stops in an explicit accepted, rejected, or escalated review state within defined limits.

**Acceptance Scenarios**:

1. **Given** a review phase with mixed reviewer findings, **When** Boundline evaluates the vote, **Then** it applies a visible decision rule such as majority or weighted voting and records how the result was computed.
2. **Given** a review phase where the initial vote is insufficient to produce a credible decision, **When** adjudication is permitted, **Then** Boundline performs one bounded adjudication step and records the final decision path.
3. **Given** a review phase that cannot reach a credible decision within the configured limits, **When** review exhaustion occurs, **Then** Boundline stops in an explicit escalated or failed review outcome and preserves the disagreement evidence.

---

### User Story 3 - Inspect Review Evidence (Priority: P3)

As a developer checking whether Boundline's output is safe to keep, I want status and inspection surfaces to show the review participants, findings, votes, and final adjudication so that I can understand the quality decision quickly.

**Why this priority**: Review only improves trust if its output is inspectable. A hidden or opaque vote would not materially improve the developer's ability to judge the result.

**Independent Test**: Complete one accepted review and one rejected or escalated review, then verify that Boundline status and inspect output expose the reviewers, review trigger, findings summary, vote method, vote outcome, and terminal review decision.

**Acceptance Scenarios**:

1. **Given** a completed review phase, **When** the user asks for status or inspect output, **Then** Boundline shows who reviewed the task, what categories of findings were produced, how the vote resolved, and what decision was taken.
2. **Given** a review phase triggered by a risky or failed execution result, **When** the developer inspects the trace, **Then** Boundline makes the review trigger, disagreement history, and final review outcome visible without requiring manual reconstruction.

---

### Edge Cases

- If the required reviewer set cannot be assembled, Boundline must stop the review phase explicitly instead of silently skipping quality control.
- If reviewers return malformed or incomplete findings, Boundline must treat that as visible review failure evidence rather than as implicit approval.
- If reviewers disagree but adjudication is not allowed or no adjudication budget remains, Boundline must terminate in an explicit escalated or failed review state.
- If execution already ended in a non-reviewable terminal state, Boundline must not pretend a valid review occurred.
- If a review trigger fires more than once for the same task stage, Boundline must record the duplicate trigger visibly and ignore the later trigger in the initial slice rather than starting an unbounded second review phase.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST support a bounded review phase for execution output within the existing task and session lifecycle.
- **FR-002**: Boundline MUST represent the review phase as explicit task state rather than as hidden background behavior.
- **FR-003**: Boundline MUST support a review council composed of multiple explicit reviewers for one bounded review run.
- **FR-004**: Boundline MUST capture structured findings from each reviewer, including whether the finding is blocking, non-blocking, or approving.
- **FR-005**: Boundline MUST record which reviewers participated in the review and whether each reviewer completed, failed, or was omitted.
- **FR-006**: Boundline MUST support at least one majority-based vote rule and one weighted vote rule for resolving reviewer input.
- **FR-007**: Boundline MUST make the applied vote rule and resulting tally inspectable.
- **FR-008**: Boundline MUST support one bounded adjudication path when the review configuration requires adjudication and reviewer disagreement prevents an immediate credible decision.
- **FR-009**: Boundline MUST stop every review phase in one explicit terminal outcome such as accepted, rejected, escalated, or failed.
- **FR-010**: Boundline MUST support explicit review triggers for at least risky changes, failed validation, and PR-readiness checks.
- **FR-011**: Boundline MUST preserve review evidence in task state and trace output so later status and inspect commands can surface it.
- **FR-012**: Boundline MUST expose review participants, findings summaries, vote details, trigger reason, and final review outcome through user-visible inspection surfaces.
- **FR-013**: Boundline MUST keep review behavior bounded by explicit reviewer, step, retry, or adjudication limits.
- **FR-014**: Boundline MUST treat malformed reviewer output, unavailable reviewers, and unresolved disagreement as visible review failures or escalations.
- **FR-015**: Boundline MUST keep the review surface provider-agnostic from the user perspective even when reviewers come from different review sources.
- **FR-016**: Boundline MUST evaluate review triggers at most once per task stage in the initial slice and MUST preserve duplicate trigger evidence when later triggers are ignored.

### Scope Boundaries *(mandatory)*

- **In Scope**: bounded post-execution review phases, multiple reviewers, structured findings, majority voting, weighted voting, bounded adjudication, explicit review triggers, and status or inspect visibility for review outcomes.
- **Out of Scope**: open-ended debate simulation, governance workflows delegated to Canon, distributed review systems, long-term memory outside task scope, UI or UX work, deployment pipelines, and unconstrained provider-routing frameworks.

### Key Entities *(include if feature involves data)*

- **Review Council**: The bounded group of reviewers assigned to assess one delivery result, including the active participant list and any participation limits.
- **Reviewer Finding**: The structured review output produced by one reviewer, including severity, rationale, and approval or rejection signal.
- **Vote Resolution**: The inspectable record of how reviewer signals were tallied, which rule was used, and what preliminary result was produced.
- **Adjudication Outcome**: The bounded follow-up decision used when disagreement remains unresolved after the initial vote.
- **Review Trigger**: The explicit reason the review phase started, such as risky change detection, failed validation, or PR-readiness evaluation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative delivery scenarios that require review, Boundline can complete a bounded review phase with multiple reviewers and one explicit terminal review decision without manual reconstruction.
- **SC-002**: 100% of review phases stop in an explicit accepted, rejected, escalated, or failed terminal state within configured limits.
- **SC-003**: Developers can identify the participating reviewers, highest-severity findings, vote method, and final review decision from status or inspect output in under 30 seconds.
- **SC-004**: In representative disagreement scenarios, 100% of review runs either resolve through the configured vote or bounded adjudication path, or terminate explicitly as escalated or failed.
- **SC-005**: No review scenario silently converts malformed reviewer output, reviewer unavailability, or unresolved disagreement into implicit approval.

## Assumptions

- Review occurs only after Boundline has produced a reviewable terminal delivery result for the current run rather than after every intermediate attempt.
- The initial release needs only one bounded review phase per delivery run rather than nested or repeated councils.
- Developers primarily consume review outcomes through the existing CLI status, next, run, and inspect surfaces.
- The first slice may rely on a predefined set of reviewer roles and trigger types rather than open-ended review workflow composition.
