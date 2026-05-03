# Feature Specification: Canon Governance Adapter

**Feature Branch**: `009-canon-governance-adapter`  
**Created**: 2026-04-26  
**Status**: Draft  
**Input**: User description: "Integrate Canon 0.18.0 as the next Boundline stage-scoped governance adapter so built-in flow stages can open governed Canon runs, reuse governed documents as bounded reasoning inputs, enforce governance-required behavior, and optionally use autopilot to choose a compliant governed path without bypassing Canon guardrails."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Govern A Boundline Stage Through A Local-First Runtime (Priority: P1)

As a developer using Boundline for bug-fix, change, or delivery work, I want Boundline to route the current stage through an explicit governance runtime so that the stage can either use a local default runtime or a Canon-backed runtime while preserving one bounded orchestration path.

**Why this priority**: The first independently valuable slice is not Canon wiring by itself. It is a visible governance boundary per stage that keeps Boundline in control and remains testable even when Canon is unavailable.

**Independent Test**: Start a built-in flow stage with governance enabled, verify that Boundline selects a governance runtime, records the stage governance state, and continues or stops explicitly according to the runtime result without introducing a second hidden execution path.

**Acceptance Scenarios**:

1. **Given** an active Boundline session in a built-in flow with governance enabled for the current stage, **When** Boundline enters that stage, **Then** it selects one explicit governance runtime, records the stage as governance-backed, and keeps the rest of execution inside the existing Boundline session and trace lifecycle.
2. **Given** Canon integration is disabled or unavailable and governance is not required for the current stage, **When** Boundline reaches the governance boundary, **Then** it uses the local governance runtime, preserves explicit evidence of that fallback, and continues bounded execution without opening a Canon run.
3. **Given** governance is required for the current stage and Boundline cannot create a credible governance path because the mode binding is invalid, Canon is unavailable, or the authored stage input is missing or structurally insufficient, **When** the stage is requested, **Then** Boundline stops in an explicit governance-blocked terminal path and does not silently continue ungoverned.

---

### User Story 2 - Reuse Governed Canon Packets As Bounded Stage Input (Priority: P2)

As a developer running Boundline with Canon enabled, I want a governed Canon run for meaningful stages so that Boundline can reuse the resulting governed packet as bounded reasoning input for later planning, execution, review, or verification.

**Why this priority**: Canon only adds delivery value if governed artifacts materially improve later Boundline stages. A stage-scoped adapter without packet reuse is just a wrapper around an external CLI.

**Independent Test**: Run a governed Boundline stage with Canon enabled, produce a completed governed packet, and verify that later Boundline behavior consumes that packet as bounded stage input rather than reconstructing the context from scratch.

**Acceptance Scenarios**:

1. **Given** Canon is enabled and the current stage has a valid stage-to-mode binding, **When** Boundline enters the stage, **Then** it opens exactly one Canon run for that stage, records the Canon mode, risk, zone, owner, and system context, and preserves the Canon run reference in session-visible state.
2. **Given** a completed governed Canon packet for the current or an earlier stage, **When** a later Boundline stage needs that context, **Then** Boundline reuses the governed packet as bounded reasoning input instead of rebuilding the stage context from unrelated workspace state.
3. **Given** a Canon stage packet that contains only structural scaffolding or explicit missing-authored-body markers in required sections, **When** Boundline evaluates whether that packet can satisfy the stage boundary, **Then** Boundline treats the packet as incomplete and stops or retries explicitly instead of accepting it as valid stage completion.

---

### User Story 3 - Use Autopilot To Choose A Compliant Governed Path (Priority: P3)

As a developer operating Boundline in a governed environment, I want an optional autopilot mode that can decide how to proceed when governance is required so that Boundline can keep moving through compliant stage-level choices without bypassing Canon guardrails or hiding those decisions.

**Why this priority**: Once governance becomes required, small operational choices can stall delivery. Autopilot is useful only if it narrows those choices within policy while keeping approval and blocking behavior explicit.

**Independent Test**: Run a governed Boundline stage with governance required and autopilot enabled, create a decision point with more than one compliant path, and verify that Boundline records the autopilot decision, follows the chosen compliant path, or stops explicitly when no compliant path exists.

**Acceptance Scenarios**:

1. **Given** governance is required and autopilot is enabled for a stage with more than one credible compliant governance path, **When** Boundline reaches that decision point, **Then** it records one explicit autopilot decision, chooses one compliant path, and preserves the rationale in inspectable state.
2. **Given** a governed stage where Canon guardrails require explicit human approval or force a recommendation-only posture because of the declared risk or zone, **When** autopilot reaches that boundary, **Then** Boundline enters an explicit awaiting-approval state and autopilot does not bypass the approval or continue through a hidden local substitute.
3. **Given** governance is required and no compliant governed path exists after autopilot evaluates the bounded choices, **When** Boundline must decide how to proceed, **Then** Boundline stops in an explicit governance-blocked state and records why autopilot could not resolve the stage.

---

### Edge Cases

- If a Boundline stage maps to more than one credible Canon mode, Boundline must make the selected mapping explicit rather than switching modes invisibly.
- If a Canon mode requires explicit `system_context` and the current Boundline stage does not provide enough information to bind `new` or `existing`, Boundline must stop or block explicitly rather than inventing the context.
- If risk, zone, or owner are missing and the current governance policy requires them for the stage, Boundline must stop or block explicitly rather than inventing governance attributes.
- If a governed stage is rerun after a blocked, incomplete, or rejected Canon packet, Boundline must preserve lineage between the previous and current governance attempts.
- If approval is required and not granted within the bounded lifecycle, Boundline must remain in an explicit awaiting-approval or governance-blocked state instead of continuing locally.
- If approval is granted, rejected, or expires outside Boundline while a stage is waiting, the next `status`, `step`, or `run` command must refresh governance state before any stage execution continues.
- If governance remains required and autopilot is disabled, Boundline must still preserve a compliant path by blocking or awaiting approval rather than bypassing governance.
- If Canon is unavailable during a stage that is already mid-governance, Boundline must preserve the partial governance evidence and stop or fall back explicitly according to the declared policy for that stage.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST support stage-scoped governance for the built-in `bug-fix`, `change`, and `delivery` flows.
- **FR-002**: Boundline MUST represent stage governance through an explicit `GovernanceRuntime` abstraction with a local default runtime and an optional Canon-backed runtime.
- **FR-003**: Boundline MUST remain independently testable and executable through the local governance runtime when Canon is unavailable or not enabled.
- **FR-004**: Boundline MUST record the governance state of the current stage in the active task context inside `.boundline/session.json` and project it into session-visible state rather than hiding it as internal behavior.
- **FR-005**: Boundline MUST support governance-enabled behavior that may use Canon when available without making Canon mandatory for all stages or all runs.
- **FR-006**: Boundline MUST support governance-required behavior that forbids silent fallback to ungoverned execution for the governed stage.
- **FR-007**: When Boundline selects the Canon-backed runtime for a stage, it MUST bind the stage to one explicit Canon mode before governed execution can proceed.
- **FR-008**: When the selected Canon mode requires `system_context`, Boundline MUST provide one explicit binding for that stage, where `existing` means the stage is grounded in the current repository or an earlier governed packet and `new` means the stage is grounded only in the newly authored governed brief.
- **FR-009**: Boundline MUST record the Canon run reference, mode, risk, zone, owner, and lifecycle state for every Canon-governed stage.
- **FR-010**: Boundline MUST reuse completed governed Canon stage documents as bounded reasoning input for later Boundline planning, execution, review, or verification when those documents are relevant to the current task.
- **FR-011**: Boundline MUST distinguish between completed governed packet content and incomplete Canon scaffolding.
- **FR-012**: Boundline MUST NOT treat governed output classified as `incomplete` or `rejected` under FR-031, including packets with required missing-authored-body markers, as valid stage completion or valid reasoning input.
- **FR-013**: Boundline MUST preserve lineage between a Boundline stage and all related governance attempts when a stage is rerun, remapped, escalated, blocked, or approved.
- **FR-014**: Boundline MUST support an optional autopilot mode that can choose among compliant governance paths for the current stage.
- **FR-015**: Autopilot MUST be limited to this bounded stage-level action vocabulary: `select_mode`, `retry_stage_with_narrowed_context`, `escalate_verification`, `escalate_pr_review`, `await_approval`, and `block_stage`.
- **FR-016**: Autopilot MUST NOT disable governance, downgrade the declared risk or zone, bypass required approvals, or replace Canon or the local governance runtime as the source of governed stage truth.
- **FR-017**: Boundline MUST enter an explicit awaiting-approval state when the selected governance path requires human approval before the stage can continue.
- **FR-018**: Boundline MUST enter an explicit governance-blocked state when governance is required and no compliant governance path or approval resolution exists for the current stage.
- **FR-019**: Boundline MUST expose governed stage status, selected runtime, Canon mode when applicable, system context, risk, zone, approval state, autopilot state, and governed packet references through user-visible `status`, `next`, `run`, and `inspect` surfaces.
- **FR-020**: Boundline MUST keep overall task orchestration, coding, testing, adaptive retry, and bounded review inside Boundline rather than delegating overall delivery control to Canon.
- **FR-021**: Boundline MUST support this initial stage-to-mode mapping in the first governed slice:
  - `delivery`: `requirements -> requirements`, `architecture -> architecture`, `backlog -> backlog`, `implementation -> implementation`
  - `change`: `understand-change -> change`, `implement -> implementation`, `verify -> verification` with optional `pr-review`
  - `bug-fix`: `investigate -> discovery` or `change`, `implement -> implementation`, `verify -> verification` with optional `pr-review`
- **FR-022**: Boundline MUST make the selected mapping explicit and inspectable whenever more than one compliant Canon mode could satisfy the current stage.
- **FR-023**: Boundline MUST use only Canon modes available in the supported Canon release for this slice and MUST fail or block explicitly when a required stage has no supported governed mode.
- **FR-024**: When a stage needs governed packet reuse, Boundline MUST resolve the newest `reusable` packet from the same session whose stage key is either the current stage on rerun or the immediately previous stage in the same built-in flow.
- **FR-025**: Boundline MUST inject only bounded packet references, packet headlines, and declared missing-section metadata into downstream stage input rather than exposing the full governed artifact tree as unbounded context.
- **FR-026**: `retry_stage_with_narrowed_context` MUST reduce the current stage input to a strict subset of the current bounded context by removing the last eligible read target from the current ordered stage target list or, when no further read target can be removed, the last reused packet reference, while leaving the goal, risk, zone, and owner unchanged; if neither can be removed, the retry candidate is unavailable and MUST NOT be emitted.
- **FR-027**: While a stage is `awaiting_approval`, every later `status`, `step`, or `run` invocation for the active workspace session MUST issue exactly one refresh request to the selected governance runtime and return immediately with the updated approval state; Boundline does not poll or wait inside the command, and `rejected` or `expired` approvals MUST transition the stage to `governance-blocked`.
- **FR-028**: When a Canon-governed stage omits `canon_mode`, Boundline MAY derive the mode only from the whitelist in FR-021 when exactly one compliant mode exists; otherwise the stage must remain pending selection until autopilot or the operator chooses a valid mode.
- **FR-029**: Stage-to-mode validation MUST reject manifest configurations at load time when a stage selects a Canon mode outside the supported whitelist for that stage in the first slice.
- **FR-030**: Packet readiness MUST be determined by a deterministic validator that requires all expected document references, non-empty authored body content, and no declared missing sections from the selected governance runtime.
- **FR-031**: For the first slice, the packet-readiness validator MUST classify governed output as `pending`, `incomplete`, `reusable`, or `rejected` using this order: `pending` before runtime completion; `rejected` when the runtime explicitly rejects the packet or every expected document fails the authored-body check; `incomplete` when any expected document is missing, any required section is reported missing, or any expected document has an empty authored body; `reusable` only when every expected document exists, every authored body is non-empty, and no required section is missing.
- **FR-032**: Autopilot candidate generation MUST follow a deterministic policy: if the stage policy already binds `canon_mode`, no `select_mode` candidate may be emitted; otherwise generate `select_mode` candidates from the per-stage whitelist in this order when multiple modes are allowed: `bug-fix:investigate = discovery, change`, `bug-fix:verify = verification, pr-review`, and `change:verify = verification, pr-review`; generate `await_approval` when the selected path requires approval; generate at most one `retry_stage_with_narrowed_context` candidate after a failed or incomplete governed attempt when a strict subset of the current bounded context exists; generate `escalate_verification` only from `implement` stages; generate `escalate_pr_review` only from `verify` stages that permit `pr-review`; and always generate `block_stage` when governance remains required but no compliant continuation exists.
- **FR-033**: When autopilot records `select_mode`, it MUST also record the candidate Canon modes considered and the selected mode; when it records an escalation action, it MUST record the target downstream stage that will receive the escalated governed attempt.
- **FR-034**: Boundline MUST expose the latest autopilot candidate actions and, when a governed packet is reused, the packet source stage and binding reason through inspectable session or trace surfaces.
- **FR-035**: When an escalation action opens a governed attempt for a downstream stage, that new stage MAY reuse only the newest reusable packet from the escalation source stage within the same session lifecycle as a bounded forward state transition; no second concurrent execution path and no additional upstream packet chaining are permitted within the same escalation transition.
- **FR-036**: Governance lifecycle transitions in the first slice are limited to `pending_selection -> running -> governed_ready|awaiting_approval|blocked|failed`, `governed_ready -> completed`, and `awaiting_approval -> governed_ready|blocked`; `blocked`, `failed`, and `completed` may only transition through an explicit rerun or escalation that creates a new governed attempt.

### Scope Boundaries *(mandatory)*

- **In Scope**: stage-scoped governance runtime selection, local default governance behavior, optional Canon-backed stage governance, explicit mode and system-context binding, explicit risk and zone capture, governed packet reuse as bounded reasoning input, governance-required enforcement, optional autopilot for compliant path selection, approval-aware stage handling, and inspectable governance lineage.
- **Out of Scope**: direct dependency on Canon internals, one Canon run per Boundline micro-step, replacement of `.boundline` traces with Canon artifacts, new custom Boundline flows, governance policy authoring inside Boundline, autopilot bypass of approvals or guardrails, distributed orchestration, UI or UX work, and deployment pipelines.

### Key Entities *(include if feature involves data)*

- **Governance Runtime**: The explicit runtime selected for the current stage, either the local default runtime or the Canon-backed runtime, including the policy and state transitions that determine whether the stage may continue.
- **Governed Stage Packet**: The governed artifact set for one Boundline stage, including authored input, outputs, evidence, approval state, and the content Boundline may reuse as bounded reasoning input.
- **Governance Policy**: The per-stage or per-run declaration of whether governance is enabled, whether governance is required, whether autopilot is enabled, and which governance attributes constrain the stage.
- **Autopilot Decision**: The inspectable decision record that captures which compliant governance path Boundline selected, why that path was chosen, and what constraints limited the choice.
- **Governance Lifecycle State**: The explicit state of the governed stage, such as pending-governance, governed-ready, awaiting-approval, governance-blocked, or completed.
- **Stage Governance Mapping**: The explicit relationship between a Boundline flow stage and the governance path chosen to satisfy it, including runtime selection, Canon mode selection when applicable, and any reruns or escalations.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative `bug-fix`, `change`, and `delivery` scenarios with governance enabled, Boundline can route the current stage through an explicit governance runtime and continue or stop without creating a second hidden execution path.
- **SC-002**: In representative scenarios where Canon is disabled or unavailable and governance is not required, Boundline can complete the governance boundary through the local default runtime with explicit fallback evidence in 100% of observed runs.
- **SC-003**: In representative scenarios where governance is required, 100% of governed stages stop in an explicit governed-ready, awaiting-approval, governance-blocked, or completed state before ungoverned continuation could occur.
- **SC-004**: Developers can identify the current governed stage, selected runtime, Canon mode when applicable, risk, zone, approval state, autopilot state, and governance reference from status or inspection output in under 60 seconds.
- **SC-005**: In representative packet-quality scenarios, Boundline rejects 100% of stub-only or missing-authored-body governed packets as valid reasoning input for current-stage completion.
- **SC-006**: In representative decision-point scenarios with governance required and autopilot enabled, 100% of autopilot decisions either choose a compliant governance path or produce an explicit governance-blocked outcome, and none bypass approval-gated or recommendation-only boundaries.

## Assumptions

- The initial slice targets the Canon 0.18.0 mode set that is currently available and only maps the subset relevant to Boundline's built-in flows.
- Boundline remains the delivery orchestrator, while governance runtimes only govern meaningful stage boundaries and packet lifecycle.
- Most initial governed runs happen inside an existing repository and can provide the `existing` system context required by Canon modes such as `backlog`, `change`, `implementation`, `verification`, and `pr-review`.
- Human approval remains a real boundary that autopilot cannot override in this slice.
- Approval resolution happens outside Boundline in the first slice; Boundline observes the updated approval state on later `status`, `step`, or `run` invocations before resuming the stage.
- Boundline supports one active session per workspace in the first slice, so approval refresh and governance state projection apply only to that active session.
- Packet reuse in the first slice is limited to Boundline's linear built-in flows; branching flow graphs and multi-parent packet lineage are deferred.
- Governance policy values such as default risk, zone, and owner can come from Boundline configuration, per-run input, or deterministic workspace defaults, but must become explicit before a governed stage continues.
