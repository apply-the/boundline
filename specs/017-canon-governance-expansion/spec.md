# Feature Specification: Canon Governance Expansion

**Feature Branch**: `017-canon-governance-expansion`  
**Created**: 2026-04-29  
**Status**: Draft  
**Input**: User description: "Deepen Boundline's Canon governance and escalation coverage by extending the existing stage-boundary Canon adapter to support a first bounded governed follow-on analysis slice. Keep Boundline's built-in bug-fix, change, and delivery flows unchanged, but expand the Canon governance surface so existing stages can escalate into security-assessment first, while shaping the model and operator surfaces so supply-chain-analysis can follow with minimal structural rework. Preserve Canon as a stage-boundary governance overlay rather than the per-action control plane, continue to reuse only bounded packet references and readiness metadata, and surface escalation target, selected Canon mode, approval state, blocked reason, packet provenance, and next action consistently across run, status, next, and inspect."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Route Existing Verification Through Governed Security Analysis (Priority: P1)

A developer using Boundline's primary session-native workflow can route an existing-system verification stage through Canon `security-assessment` when deeper governed analysis is required, without leaving the same bounded session and trace lifecycle.

**Why this priority**: This is the smallest independently valuable expansion of Boundline's real Canon usage. It deepens governance coverage with a concrete newer Canon mode while preserving the existing flow model and session-native control plane.

**Independent Test**: Can be fully tested by preparing a `bug-fix` or `change` session that reaches `verify`, enabling Canon governance for that stage, and confirming that Boundline opens a governed `security-assessment` path, records the selected Canon mode, and either continues, waits, or blocks explicitly.

**Acceptance Scenarios**:

1. **Given** an active `bug-fix` or `change` session at the `verify` stage with Canon governance enabled, **When** the governed stage selects `security-assessment`, **Then** Boundline starts one Canon-governed security analysis for that stage and records the selected mode, run reference, packet reference, and bounded next action in the active session and trace.
2. **Given** a verification stage that requests `security-assessment` but cannot satisfy the required existing-system context or Canon runtime contract, **When** the operator runs or resumes the session, **Then** Boundline stops in an explicit governance-blocked path instead of silently falling back to an ungoverned route.
3. **Given** a governed `security-assessment` packet that requires approval or remains non-reusable, **When** the operator later runs `status`, `step`, or `run`, **Then** Boundline refreshes the governance state and returns the updated approval or blocked condition without advancing hidden work.

---

### User Story 2 - Surface Governed Follow-On Analysis Through One Session Story (Priority: P2)

A developer can understand governed follow-on analysis through the same session-native summaries already used for runtime work, instead of treating Canon escalation as a separate hidden workflow.

**Why this priority**: Governance expansion is not credible if the operator has to infer which governed analysis ran, why it was selected, or what action is required next. The value lands only when the session surfaces stay coherent.

**Independent Test**: Can be fully tested by running a governed verification session that selects `security-assessment` and verifying that `run`, `status`, `next`, and `inspect` expose the selected Canon mode, approval or blocked state, packet provenance, and next-step guidance consistently.

**Acceptance Scenarios**:

1. **Given** a verification stage governed through `security-assessment`, **When** the developer checks `run`, `status`, `next`, or `inspect`, **Then** each surface explains the chosen Canon mode, the current governance condition, and the next command without contradicting the primary session-native routing story.
2. **Given** a governed follow-on packet reused from an earlier attempt or earlier stage within the same bounded session, **When** the developer inspects the session state, **Then** Boundline exposes only bounded packet references, packet headlines, readiness, and binding reasons rather than the full Canon artifact tree.

---

### User Story 3 - Keep The Governance Expansion Bounded And Extensible (Priority: P3)

A maintainer can expand Boundline's Canon governance support in a way that leaves room for later `supply-chain-analysis` support without reopening the built-in flow model or turning Canon into Boundline's per-action controller.

**Why this priority**: The next slice should deepen governance coverage now, but it should not solve that by overfitting the model to a single new mode or by broadening scope into a full multi-mode rewrite.

**Independent Test**: Can be fully tested by validating that the new mode-selection and escalation model supports `security-assessment` for the targeted stages, rejects unsupported Canon modes explicitly, and preserves the existing built-in flow boundaries.

**Acceptance Scenarios**:

1. **Given** a stage policy that binds a Canon mode outside the supported bounded expansion for this slice, **When** Boundline validates or executes that policy, **Then** it rejects the unsupported governed mode explicitly instead of passing it through as unchecked Canon configuration.
2. **Given** an existing session-native flow with no governance expansion configured, **When** the developer runs the workflow, **Then** Boundline preserves the current non-expanded behavior without forcing new Canon analysis paths.

### Edge Cases

- What happens when `security-assessment` is requested for a stage that does not have credible existing-system context or lacks a reusable bounded packet lineage?
- What happens when the governed security packet is structurally present but still `pending`, `incomplete`, or `rejected` at the point where Boundline must decide whether the session can continue?
- What happens when the Canon runtime is available for the initial start request but unavailable during a later approval refresh?
- What happens when a workspace contains both older supported Canon stage mappings and the new security-analysis expansion so the operator must still understand one coherent session route?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST preserve the session-native `bug-fix`, `change`, and `delivery` workflows as the primary operator story while expanding Canon governance only at bounded stage boundaries.
- **FR-002**: System MUST support `security-assessment` as an explicitly governed Canon mode for the targeted existing-system verification stages in the first slice.
- **FR-003**: System MUST keep Canon mode selection explicit and inspectable whenever a governed stage may route through more than one compliant Canon mode.
- **FR-004**: System MUST enforce the existing-system context requirements of `security-assessment` before a governed security-analysis path can continue.
- **FR-005**: System MUST preserve explicit `awaiting approval`, `blocked`, `failed`, and reusable-packet outcomes for governed follow-on analysis instead of collapsing them into generic terminal messages.
- **FR-006**: System MUST refresh the governance state of approval-gated security-analysis runs through later `status`, `step`, or `run` invocations before any further stage execution continues.
- **FR-007**: System MUST expose the selected Canon mode, run reference, packet reference, packet readiness, packet binding reason, and next action across `run`, `status`, `next`, and `inspect` when governed follow-on analysis is active.
- **FR-008**: System MUST continue to inject only bounded packet references, packet headlines, readiness, and declared missing metadata into downstream governance context rather than exposing the full Canon artifact tree.
- **FR-009**: System MUST preserve explicit policy validation and reject unsupported Canon modes for the current bounded expansion rather than allowing arbitrary newer Canon modes through unchecked.
- **FR-010**: System MUST preserve the existing non-expanded governance and non-governed behavior for sessions that do not opt into the new security-analysis path.
- **FR-011**: System MUST treat `security-assessment` as a governed analysis overlay and MUST NOT let Canon become the per-action control plane for Boundline's execution loop.
- **FR-012**: System MUST keep the expanded mode-selection and operator-surface model compatible with a later bounded `supply-chain-analysis` addition without requiring a new top-level Boundline flow family.

### Scope Boundaries *(mandatory)*

- **In Scope**: bounded expansion of Canon governance coverage for `security-assessment`; explicit mode validation and selection for the targeted verification stages; approval refresh, packet-readiness handling, packet provenance, and operator-surface summaries for the new governed analysis path; future-compatible model shaping for later `supply-chain-analysis` support.
- **Out of Scope**: adding new built-in Boundline flow families or stage ids; full parity with Canon's entire mode roster; direct support for `supply-chain-analysis`, `incident`, `migration`, `review`, `system-shaping`, or `refactor` in the first slice; exposing the full `.canon/` artifact tree; changing Canon into Boundline's per-action runtime controller.

### Key Entities *(include if feature involves data)*

- **Governed Analysis Mode Selection**: The explicit session-visible decision that binds a bounded Boundline stage to one Canon mode such as `security-assessment`, including candidate modes, the chosen mode, and the rationale for that choice.
- **Governed Analysis Packet**: The Canon packet reused or evaluated by Boundline for the governed follow-on analysis path, including run reference, packet reference, readiness, headline, and missing-section metadata.
- **Packet Reuse Binding**: The bounded relationship between the upstream session stage and a reused Canon packet, including source stage, downstream stage, packet reference, and binding reason.
- **Governance Condition**: The operator-facing state of the governed analysis path, such as running, awaiting approval, blocked, failed, or reusable, together with the next action required to continue or inspect the session.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative `bug-fix` and `change` verification scenarios, Boundline can route a governed stage through Canon `security-assessment` without introducing a second hidden execution workflow.
- **SC-002**: 100% of representative approval-gated or blocked security-analysis scenarios surface an explicit governance condition and at least one clear next action through the session-native operator surfaces.
- **SC-003**: Developers can identify the selected Canon mode, packet readiness, packet provenance, and next command for a governed security-analysis path from `status`, `next`, or `inspect` in under 2 minutes.
- **SC-004**: 100% of representative unsupported-mode configurations for the bounded expansion are rejected explicitly rather than passing through as unchecked Canon configuration.

## Assumptions

- Operators continue to use one active Boundline session per workspace and rely on persisted `.boundline/session.json` plus `.boundline/traces/` state between commands.
- The first governance-expansion slice should maximize delivery value by adding one newer Canon mode (`security-assessment`) before broadening to additional operational analysis modes.
- Existing-system verification stages in `bug-fix` and `change` are the most credible first attachment points for `security-assessment` because they already operate on modified repository context.
- `supply-chain-analysis` remains a follow-on slice because its clarification and tool-availability posture would widen the first slice beyond the intended bounded scope.
