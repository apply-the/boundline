# Feature Specification: Plan Analysis Contract

**Feature Branch**: `069-plan-analysis-contract`

**Created**: 2026-06-04

**Status**: Draft

**Input**: User description: "Define Plan Analysis Contract as a read-only Boundline planning-coherence gate based on the end-to-end framing across goal, plan, backlog, validation strategy, risks, constraints, execution readiness, and governed Canon evidence, rather than a backlog-only validator." More input defined in ./feat-plan-analysis-contract.md

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Block Incoherent Execution Handoff (Priority: P1)

An operator finishes planning and wants Boundline to offer execution only when the full planning picture is mutually consistent across the active goal, plan projection, backlog packet, validation strategy, risks, constraints, and governed evidence.

**Why this priority**: This is the feature's core value. If the analysis cannot stop incoherent execution handoff, the gate does not protect the runtime.

**Independent Test**: Can be fully tested by evaluating one planning session with a critical cross-artifact inconsistency and confirming that execution is withheld while the analysis stays read-only.

**Acceptance Scenarios**:

1. **Given** plan quality and backlog quality are already ready, **When** planning analysis finds a critical inconsistency such as an uncovered success criterion or missing required governed evidence, **Then** Boundline marks planning analysis as blocked and does not offer execution handoff.
2. **Given** planning analysis is blocked, **When** the operator inspects the active session, **Then** Boundline reports the blocking findings with enough source context to repair the planning state without mutating the plan or Canon packet.

---

### User Story 2 - Inspect Planning Coherence (Priority: P2)

An operator wants status, inspect, and orchestration output to explain whether planning is coherent, what findings were detected, and which planning artifacts caused them.

**Why this priority**: A blocking gate without an inspectable explanation creates dead ends and slows repair work.

**Independent Test**: Can be fully tested by evaluating clean, warning-only, and blocked planning sessions and confirming that every supported runtime surface exposes the same additive planning-analysis projection.

**Acceptance Scenarios**:

1. **Given** planning analysis finishes with no blocking issues, **When** the operator checks status or inspect output, **Then** Boundline surfaces a clean planning-analysis state and a coverage summary showing the planning slice is execution-ready.
2. **Given** planning analysis finishes with non-blocking findings, **When** the operator checks runtime output, **Then** Boundline reports the findings, affected sources, and coverage metrics without upgrading the session to a blocked state.

---

### User Story 3 - Preserve Assistant-Safe Continuation (Priority: P3)

An assistant host or orchestration client wants planning analysis to behave as a real runtime gate so that plan, run, status, inspect, and phase-request flows remain symmetric and do not invent execution continuation when coherence is missing.

**Why this priority**: Assistant assets are a projection over the runtime contract. If they drift from the gate semantics, they will mislead users about what can run next.

**Independent Test**: Can be fully tested by validating assistant-facing plan and run flows against a blocked planning-analysis session and confirming that the host stays on the planning repair path.

**Acceptance Scenarios**:

1. **Given** planning analysis is blocked, **When** an assistant host requests the next step, **Then** Boundline emits the planning continuation or phase-request path rather than a direct execution continuation.
2. **Given** planning analysis has not run for an older compatible session snapshot, **When** a host reads status or inspect output, **Then** the additive planning-analysis fields are omitted without breaking backward compatibility.

### Edge Cases

- What happens when Canon-governed evidence is referenced by the planning flow but the producer packet omits a field that only Canon can author?
- How does the system handle a planning session where the backlog is ready but the validation strategy contradicts the goal's success criteria?
- What happens when planning analysis can compute findings only from local Boundline artifacts because Canon is optional or absent?
- How does the system handle duplicate or overlapping findings that originate from multiple planning artifacts but describe the same blocking coherence gap?
- What happens when a compatibility session snapshot predates planning-analysis persistence and a later command needs to decide whether execution may continue?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST run planning analysis only after goal quality, plan quality, and backlog quality have produced a non-blocking planning state.
- **FR-002**: The system MUST treat planning analysis as a read-only Boundline-owned gate that never mutates source files, plan tasks, backlog artifacts, Canon packets, or governed evidence.
- **FR-003**: The system MUST evaluate end-to-end planning coherence across the active goal, success criteria, plan projection, backlog packet, validation strategy, risks, constraints, execution readiness, and governed Canon evidence when those inputs are present.
- **FR-004**: The system MUST detect critical inconsistencies that make execution unsafe, including at minimum uncovered success criteria, missing validation coverage for required outcomes, contradictions between planning artifacts, missing required execution inputs, and governed evidence gaps that invalidate execution readiness.
- **FR-005**: The system MUST distinguish blocked planning-analysis outcomes from non-blocking findings so that execution stops only for critical coherence failures.
- **FR-006**: The system MUST persist additive planning-analysis runtime fields that include the analysis state, findings, and coverage-oriented metrics when analysis has run.
- **FR-007**: The system MUST omit planning-analysis fields from status-compatible outputs when analysis has not run, so that older snapshots remain readable without synthetic defaults.
- **FR-008**: The system MUST expose planning-analysis findings through runtime surfaces with enough source attribution for an operator to identify which planning artifacts need repair.
- **FR-009**: The system MUST prevent execution handoff whenever planning analysis is blocked, even if earlier planning gates are ready.
- **FR-010**: The system MUST preserve the existing planning repair path when analysis blocks execution, including plan-stage continuation and phase-request style follow-up where those surfaces are already used.
- **FR-011**: The system MUST treat missing producer-owned Canon fields as explicit producer contract gaps rather than inventing heuristics or inferred Canon data.
- **FR-012**: The system MUST allow planning analysis to complete using only Boundline-owned planning artifacts when Canon is optional or not part of the active planning route.
- **FR-013**: The system MUST summarize planning-analysis coverage in a way that shows whether requirements, success criteria, backlog slices, and validation evidence remain aligned enough for execution readiness.
- **FR-014**: The system MUST deduplicate materially identical planning-analysis findings so that one coherence problem is not reported as multiple independent blockers without a meaningful distinction.
- **FR-015**: Assistant-facing plan, run, status, inspect, and orchestration surfaces MUST preserve the planning-analysis state and MUST NOT present direct execution continuation while the analysis is blocked.

### Key Entities *(include if feature involves data)*

- **Planning Analysis Assessment**: The persisted result of the read-only coherence gate, including overall state, findings, and coverage-oriented summary data.
- **Planning Analysis Finding**: A single coherence issue detected during analysis, including severity, affected planning source, and operator-readable repair context.
- **Planning Analysis Coverage Summary**: An additive projection that explains how well goal outcomes, plan tasks, backlog slices, validation strategy, risks, and governed evidence align.
- **Producer Contract Gap**: A specific finding class used when execution readiness depends on Canon-authored data that is absent from the governed packet and therefore cannot be safely inferred by Boundline.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In blocked planning-analysis scenarios, Boundline withholds execution handoff in 100% of validated regression cases.
- **SC-002**: In clean planning-analysis scenarios, operators can identify that execution is ready from status or inspect output in under 30 seconds without opening raw packet files.
- **SC-003**: In regression fixtures with a deliberately uncovered success criterion, planning analysis reports at least one source-attributed finding before execution can continue.
- **SC-004**: Backward-compatible session snapshots that predate planning-analysis persistence continue to render successfully in all validated runtime surfaces with no synthetic blocked state introduced.

## Assumptions

- The next feature remains Boundline-owned and does not introduce a Canon-side packet schema change as part of this slice.
- Existing goal-quality, plan-quality, and backlog-quality gates remain the upstream admission stages for planning analysis.
- The first release of this feature focuses on coherence detection and runtime projection, not automated repair or standalone analysis commands.
- When Canon data is missing and only Canon can author it, the correct behavior is to block with a producer contract gap finding rather than guess.
