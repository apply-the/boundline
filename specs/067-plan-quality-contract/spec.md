# Feature Specification: Plan Quality Contract

**Feature Branch**: `067-plan-quality-contract`

**Created**: 2026-06-02

**Status**: Released in Boundline `0.67.0`

**Input**: User description from `roadmap/features/03-plan-quality-contract.md`, promoted as the next Boundline planning-readiness feature with release, documentation, clippy, and changed-file patch-coverage closure requirements.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Block Unsafe Execution Handoff (Priority: P1)

As a repository operator, I can rely on Boundline to stop before execution when the active plan lacks an explicit validation strategy, so incomplete planning does not silently become implementation work.

**Why this priority**: A plan without a validation strategy cannot demonstrate working-code delivery. Blocking that handoff is the smallest independently valuable planning-readiness improvement.

**Independent Test**: Can be fully tested in an isolated temporary workspace by capturing a goal, requesting a plan that omits its validation strategy, and verifying that Boundline keeps planning active, emits one actionable clarification request, records the block in traces, and does not offer execution.

**Acceptance Scenarios**:

1. **Given** an active session with goal quality satisfied and a plan that includes an explicit validation strategy, **When** the operator requests the next delivery step, **Then** Boundline marks plan quality as ready and may offer execution handoff.
2. **Given** an active session with goal quality satisfied and a plan that omits its validation strategy, **When** the operator requests the next delivery step, **Then** Boundline keeps the plan non-terminal, marks plan quality as requiring clarification, emits exactly one actionable `phase_request`, and does not offer execution handoff.
3. **Given** a blocked planning session with a missing validation strategy, **When** the operator supplies an adequate validation strategy and resumes planning, **Then** Boundline re-evaluates plan quality, records the recovered assessment, and may offer execution handoff when no blocking finding remains.

---

### User Story 2 - Inspect Plan Readiness (Priority: P2)

As a repository operator, I can inspect whether a plan is ready, which quality findings still block it, and which low-impact assumptions were accepted, so I can understand the runtime decision without reconstructing it from chat history.

**Why this priority**: A blocking gate is credible only when its decision and recovery path remain visible through the normal operator surfaces.

**Independent Test**: Can be fully tested by evaluating one ready plan, one plan blocked by a missing validation strategy, and one ready plan with an accepted low-impact omission, then verifying that status, orchestration snapshots, and traces expose the expected readiness state, findings, assumptions, and transitions.

**Acceptance Scenarios**:

1. **Given** a plan-quality evaluation has completed, **When** the operator inspects session status, **Then** the operator sees the current plan-quality state and any relevant findings or accepted assumptions.
2. **Given** a plan-quality evaluation blocks execution handoff, **When** the operator inspects orchestration output or trace history, **Then** the operator sees the blocking finding, the emitted clarification request, and the fact that execution handoff was withheld.
3. **Given** a plan omits a low-impact detail for which Boundline applies an accepted default, **When** the operator inspects status or traces, **Then** the accepted assumption remains visible and does not block execution handoff.

---

### User Story 3 - Resume Planning Through Assistant Surfaces (Priority: P3)

As an assistant user, I receive a consistent planning response across supported hosts when goal quality or plan quality prevents progress, so I can answer one focused question and resume the same session safely.

**Why this priority**: The runtime gate must be preserved by every supported assistant surface; otherwise users can be routed around the safety decision accidentally.

**Independent Test**: Can be fully tested by validating the supported assistant planning assets and exercising a blocked planning response to confirm that each host preserves the quality projection, the single `phase_request`, and the resume command.

**Acceptance Scenarios**:

1. **Given** goal quality or plan quality blocks planning progress, **When** a supported assistant host renders the planning response, **Then** it preserves the blocked state, findings, accepted assumptions, emitted `phase_request`, and resume routing without inventing an execution step.
2. **Given** a blocked planning response includes one actionable clarification request, **When** the user answers and resumes through the assistant command, **Then** Boundline continues the existing session and re-evaluates readiness before offering execution.
3. **Given** the planning assets are distributed for a release, **When** package validation runs, **Then** every supported host asset contains the standardized planning sections and blocked-quality routing rules.

### Edge Cases

- Goal quality is unresolved when planning is requested; Boundline preserves the existing goal-quality block and does not evaluate plan quality prematurely.
- A plan has no validation strategy; Boundline emits the highest-impact missing-validation finding and exactly one clarification request.
- Multiple plan-quality findings are possible; Boundline records concise findings but asks exactly one question at a time, prioritizing scope and safety before user-facing behavior and technical detail.
- A low-impact planning detail is omitted; Boundline records an accepted assumption and allows the plan to remain ready when no blocking finding exists.
- An older session snapshot has no plan-quality fields; status and orchestration surfaces continue to read it without failure.
- A consumer ignores the additive plan-quality fields; existing session and orchestration behavior remains compatible.
- Planning recovers after clarification; the trace history retains the blocked assessment and the later ready assessment.
- A supported assistant host receives a blocked response; the host must not synthesize execution handoff from chat-only assumptions.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST evaluate plan quality after goal quality is satisfied and before execution handoff is offered.
- **FR-002**: The first independently valuable release slice MUST treat a missing validation strategy as a blocking plan-quality finding.
- **FR-003**: The system MUST represent plan quality with the states `ready`, `clarification_required`, and `blocked`.
- **FR-004**: When plan quality is insufficient but recoverable through operator input, the system MUST keep planning non-terminal, use `clarification_required`, emit exactly one actionable `phase_request`, and preserve the existing assistant resume routing.
- **FR-005**: The system MUST prioritize the one-question clarification backlog by scope and safety first, user-facing behavior second, and technical detail third.
- **FR-006**: The system MUST record concise machine-readable plan-quality findings for missing or weak planning inputs.
- **FR-007**: The system MUST record inferred low-impact defaults as accepted plan-quality assumptions and MUST expose them without blocking execution when no blocking finding remains.
- **FR-008**: The system MUST expose plan-quality state, findings, and accepted assumptions in session status when a plan exists or planning is blocked.
- **FR-009**: The system MUST expose plan-quality state, findings, and accepted assumptions in orchestration session snapshots when a plan exists or planning is blocked.
- **FR-010**: The system MUST trace plan-quality evaluation, blocking decisions, emitted clarification requests, accepted assumptions, recovery after clarification, and final readiness state with reproducible session context and without secrets or personally identifiable information.
- **FR-011**: The system MUST preserve compatibility with older persisted session snapshots that do not contain plan-quality fields and with consumers that ignore additive fields.
- **FR-012**: The system MUST preserve the existing goal-quality gate and MUST NOT evaluate plan quality as a replacement for unresolved goal quality.
- **FR-013**: The system MUST update the supported assistant planning assets so each host uses the standardized sections `User Input`, `Pre-Execution Checks`, `Execution Flow`, `Plan Quality Validation`, `Reasonable Defaults`, `Gate Handling`, `Output Interpretation`, `Next-Step Routing`, and `Done When`.
- **FR-014**: Supported assistant planning assets MUST preserve goal-quality and plan-quality blocked states, findings, accepted assumptions, any emitted `phase_request`, and resume routing without deriving execution handoff from chat-only assumptions.
- **FR-015**: The release MUST update the workspace version and aligned release metadata, `README.md`, user-facing documentation under `docs/`, engineering documentation under `tech-docs/`, and `CHANGELOG.md`.
- **FR-016**: The release MUST pass formatting and clippy validation with warnings rejected.
- **FR-017**: The release MUST demonstrate at least 95% patch coverage for changed or created implementation files and MUST use the repository patch-coverage helper when reporting that result.
- **FR-018**: The feature packet MUST record a current public-provider catalog refresh result, including an explicit no-change rationale when no catalog update is needed.
- **FR-019**: The feature MUST remain a runtime-owned planning-readiness gate and MUST NOT require generated Speckit files, a new CLI subcommand, Canon control flow, provider abstractions, background workers, parallel execution, or hidden fallback behavior.

### Task State, Recovery, and Terminal Conditions

- Plan-quality evaluation starts only after goal quality is satisfied and a plan is available for evaluation.
- A ready assessment permits execution handoff only when no blocking finding remains.
- A recoverable missing-validation-strategy finding keeps planning active, records `clarification_required`, emits exactly one `phase_request`, and waits for explicit operator input.
- A blocked assessment is reserved for a non-recoverable or explicitly blocked planning condition surfaced by the runtime; it never silently degrades into execution.
- Recovery reuses the existing session, records the supplied planning input, re-evaluates readiness, and appends a trace-visible transition.
- The feature is terminal for a planning attempt only when readiness is visible as `ready` or when a visible blocked condition stops progress pending operator action.

### Scope Boundaries

- This feature adds the first plan-readiness gate only; backlog-readiness and cross-artifact analysis gates remain separate roadmap slices.
- This feature does not add a new planning command, a second planning runtime, or a file-first Speckit workflow.
- This feature does not change Canon ownership boundaries; Canon may supply planning packets, but Boundline owns readiness evaluation and execution admission.
- This feature does not add provider, sandbox, browser, gateway, memory, council, adaptive-governance, or recursive-refinement behavior.
- This feature does not introduce parallel planning work, autonomous background work, or hidden heuristics.

### Key Entities

- **Plan Quality Assessment**: The current readiness decision for an active plan, including state, concise findings, accepted assumptions, and the session context needed for status and trace projection.
- **Plan Quality Finding**: A machine-readable reason a plan is missing or weak in a quality dimension, including whether the finding blocks execution handoff and whether focused operator input can resolve it.
- **Plan Quality Assumption**: A low-impact inferred default accepted by the runtime, retained for inspection without blocking execution handoff.
- **Planning Clarification Request**: The single highest-priority operator question emitted through the existing `phase_request` contract while planning remains non-terminal.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In all validation scenarios where goal quality is satisfied but the plan omits its validation strategy, execution handoff is withheld and exactly one actionable clarification request is returned.
- **SC-002**: In all validation scenarios where the plan contains an explicit validation strategy and no other blocking finding, plan quality is reported as ready and execution handoff remains available.
- **SC-003**: In all recovery scenarios where the operator supplies an adequate missing validation strategy, the same session transitions from requiring clarification to ready without losing the earlier blocked assessment from trace history.
- **SC-004**: In all compatibility scenarios using older persisted session snapshots without plan-quality fields, status and orchestration inspection complete successfully.
- **SC-005**: All supported assistant planning assets contain the nine standardized planning sections and preserve blocked-quality routing through one clarification request and the existing resume command.
- **SC-006**: Release validation completes with formatting checks passing, clippy producing zero warnings, and changed or created implementation files meeting at least 95% patch coverage.
- **SC-007**: The feature packet records a provider-catalog refresh result dated during the feature cycle, with either the applied catalog delta or an explicit evidence-backed no-change rationale.

## Assumptions

- The existing goal-quality gate, `phase_request`, assistant resume command, assistant next command, session status, orchestration snapshot, and trace surfaces remain the reusable runtime contracts.
- Missing validation strategy is the only newly enforced blocking quality dimension in the first delivery slice; additional findings may be represented for inspection only when they do not expand enforcement scope.
- Assistant host support remains aligned with the currently distributed Copilot, Claude, Codex, and Antigravity planning assets.
- Release closure uses the next minor pre-1.0 workspace version because the additive runtime projection and planning admission behavior form a new feature slice.
- Catalog refresh work is evidence-only unless current public provider documentation reveals a difference in the bundled assistant model catalog.
