# Feature Specification: Control Graduation And Adaptive Governance

**Feature Branch**: `057-adaptive-governance`  
**Created**: 2026-05-16  
**Status**: Draft  
**Input**: User description: "su canon e boundline con due nuovi feature branch per implementare roadmap/S4 - control-graduation-and-adaptive-governance-spec.md. Definisci contratti tra i due se necessario"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Adopt Governance Progressively (Priority: P1)

As a Boundline operator using the primary session-native workflow, I want governance to begin with low-friction runtime guidance and only graduate into stronger enforcement when confidence, trust, and authority posture justify it, so teams can adopt S4 without jumping immediately to hard-stop delivery behavior.

**Why this priority**: S4 only delivers value if teams can start using it safely on ordinary work. If the first experience is immediate hard enforcement, the feature will create bypass behavior instead of trusted governance.

**Independent Test**: Run representative governed stages with different authority posture and runtime evidence quality, and verify that the runtime surfaces one explicit governance state, one explicit rollout profile, and one explicit continue, degrade, escalate, or stop outcome through the normal session-native flow.

**Acceptance Scenarios**:

1. **Given** a newly governed workspace with no prior calibration history, **When** a governed boundary is evaluated for the first time, **Then** Boundline starts in advisory mode and surfaces recommendations, warnings, and next actions without pretending stronger enforcement is already trustworthy.
2. **Given** a governed boundary with credible evidence, stable trust history, and proportional authority posture, **When** Boundline evaluates the boundary, **Then** it graduates the runtime into an explicit stronger governance state with the corresponding rollout profile instead of leaving the operator to infer hidden policy changes.
3. **Given** a governed boundary whose evidence or confidence is insufficient for the requested maturity, **When** Boundline evaluates the boundary, **Then** it degrades, escalates, or stops explicitly rather than silently continuing as though governance had succeeded.

---

### User Story 2 - Degrade And Escalate Safely (Priority: P2)

As an operator managing risky delivery work, I want Boundline to degrade safely and escalate authority when governance conditions cannot be satisfied, so the system remains usable, inspectable, and honest during reviewer gaps, low confidence, missing evidence, or unsupported governed input.

**Why this priority**: S4 is operational, not decorative. If degradation and escalation are undefined, governance will fail exactly when conditions become risky.

**Independent Test**: Trigger reviewer unavailability, low-confidence evidence, unsupported Canon semantics, and repeated override scenarios, and verify that Boundline records one explicit degradation or escalation outcome with rationale and traceable next action.

**Acceptance Scenarios**:

1. **Given** a required governed boundary where the requested council or evidence posture cannot be satisfied, **When** Boundline evaluates the boundary, **Then** it chooses an explicit degradation or escalation path instead of weakening governance invisibly.
2. **Given** a governed boundary where repeated overrides or failed outcomes have reduced trust, **When** Boundline reevaluates the same class of work, **Then** it increases governance friction, escalation likelihood, or human gating proportionally.
3. **Given** a required governed boundary with incompatible or unavailable Canon semantic input, **When** Boundline cannot satisfy the compatibility requirement, **Then** it records the failure as an explicit blocked or escalated runtime decision rather than guessing fallback semantic meaning.

---

### User Story 3 - Preserve The Canon And Boundline Contract Boundary (Priority: P3)

As a cross-repo maintainer, I want Boundline to consume Canon-owned governance semantics without handing Canon ownership of runtime confidence, trust, councils, or stop transitions, so the S4 contract stays stable while Boundline remains the runtime decision-maker.

**Why this priority**: S4 spans two repositories with different responsibilities. If the semantic and runtime boundaries blur, both repos will drift and operators will not know where governance behavior actually comes from.

**Independent Test**: Compare Boundline behavior across a stage with only the required Canon posture contract, a stage with the same posture plus an optional adaptive companion contract, and a stage with an unsupported companion contract, and verify that local runtime governance remains authoritative in every case.

**Acceptance Scenarios**:

1. **Given** a governed boundary with compatible Canon `authority-governance-v1` semantics and no adaptive companion metadata, **When** Boundline evaluates the runtime state, **Then** it still computes local confidence, trust, degradation, escalation, and stop behavior without waiting for a second Canon contract.
2. **Given** a governed boundary with optional Canon `adaptive-governance-v1` metadata, **When** Boundline consumes it, **Then** it treats that contract as semantic input and keeps runtime confidence, council, and stop transitions under local control.
3. **Given** a required adaptive companion contract that is unsupported or incompatible, **When** Boundline evaluates the boundary, **Then** it enters an explicit compatibility failure path instead of silently inventing equivalent local semantics.

### Edge Cases

- A newly enabled governance surface has no calibration history yet, so the runtime must begin with explicit low-friction advisory behavior rather than assuming mature enforcement.
- A required boundary loses reviewer availability, evidence coverage, or route credibility mid-run, so the runtime must degrade or escalate explicitly without losing session continuity.
- Repeated overrides or post-approval failures reduce trust over time, so the same category of work must become harder to auto-continue on later runs.
- Canon posture semantics are absent but governance is optional, so the normal local compatibility path must remain explicit instead of forcing governed mode.
- Canon posture semantics are required but incompatible, so the runtime must fail closed on the required contract without erasing already captured session evidence.
- A suspended or downgraded governance posture is resumed later, so the operator can still see why the runtime changed maturity and what is required for recovery.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST keep the primary session-native workflow as the operator story for this slice and apply adaptive governance as a runtime-owned overlay on `plan`, `run`, `status`, `next`, and `inspect` rather than as a hidden parallel workflow.
- **FR-002**: Boundline MUST support these runtime governance states for this slice: `advisory`, `catch`, `rule`, and `hook`.
- **FR-003**: Boundline MUST support these operator-visible rollout profiles for governance maturity: `minimal`, `guided`, `governed`, and `strict`, and MUST keep them distinct from S3 council profiles.
- **FR-004**: Boundline MUST begin newly enabled or low-trust governance surfaces in `advisory` mode unless an operator has explicitly approved a stronger initial posture.
- **FR-005**: Boundline MUST compute one effective runtime governance decision for each governed boundary using Canon posture semantics together with local evidence sufficiency, review credibility, calibration history, and trust state.
- **FR-006**: Boundline MUST support promotion, downgrade, rollback, temporary suspension, and recovery of governance maturity without losing traceability for why the runtime changed state.
- **FR-007**: Boundline MUST compute governance confidence and trust locally from runtime evidence and MUST NOT treat Canon as the owner of runtime confidence, trust evolution, degradation choice, or escalation choice.
- **FR-008**: Boundline MUST support explicit degradation outcomes including advisory fallback, smaller council, human gate, reduced autonomy, verification-only, and execution block, and MUST map each outcome onto the existing S3 stop-semantics vocabulary.
- **FR-009**: Boundline MUST keep every degradation outcome visible, explainable, and traceable, and MUST NOT silently weaken governance.
- **FR-010**: Boundline MUST trigger explicit escalation when runtime confidence is insufficient, required governance conditions cannot be satisfied, or required Canon semantic contracts are unavailable or incompatible.
- **FR-011**: Boundline MUST preserve explicit override events with rationale, affected boundary, resulting governance posture, and the lineage or trace context needed for later inspection.
- **FR-012**: Boundline MUST keep human authority intact so that stronger governance recommendations, rollout-profile changes, and resumed automation do not take effect without explicit operator approval.
- **FR-013**: Boundline MUST continue to treat Canon `authority-governance-v1` as the required Canon posture contract for the first S4 runtime slice.
- **FR-014**: Boundline MUST treat Canon `adaptive-governance-v1`, if present, as an optional companion contract for semantic governance-maturity input and MUST NOT require it for first-slice runtime operation unless stage policy explicitly requires the companion contract.
- **FR-015**: Boundline MUST preserve a two-tier cross-repo contract boundary for S4: `authority-governance-v1` remains the required Canon posture baseline, while any `adaptive-governance-v1` companion remains additive and semantic rather than runtime-authoritative.
- **FR-016**: Boundline MUST keep Canon-owned semantics, approval semantics, readiness semantics, governance metadata, project memory, lineage, and promotion state distinct from Boundline-owned confidence, trust evolution, degradation, escalation, council assembly, and stop-transition behavior.
- **FR-017**: Boundline MUST preserve explicit compatibility behavior when Canon semantic input is absent and governance is not required, rather than forcing governed execution onto ordinary delivery.
- **FR-018**: Boundline MUST produce traces for governance-state transitions, rollout-profile changes, confidence changes, degradation events, escalation events, override events, and trust evolution.
- **FR-019**: Boundline MUST explain why governance activated, why the current rollout profile applies, why confidence changed, why degradation or escalation occurred, and why execution continued, paused, or stopped.
- **FR-020**: Boundline MUST keep this first slice bounded to runtime governance progression, degradation, escalation, confidence, trust, and operator-visible projection, leaving new provider-routing semantics, distributed governance, permanent automatic lockout, and advanced multi-agent reasoning profiles out of scope.

### Scope Boundaries *(mandatory)*

- **In Scope**: runtime governance-state progression; rollout-profile projection; confidence and trust evaluation; explicit degradation and escalation; override traceability; operator approval boundaries; reuse of existing stop semantics; explicit session-native and trace-native explanation; continued Canon posture consumption; and the additive S4 contract boundary with Canon.
- **Out of Scope**: redefining S3 authority posture; Canon-owned semantic authoring; automatic provider or model-route assignment; distributed governance services; permanent autonomous lockout; always-on councils for every step; UI work; deployment pipelines; and advanced reasoning-profile orchestration.

### Key Entities *(include if feature involves data)*

- **Governance Runtime State**: The current operational governance posture for a boundary, expressed as `advisory`, `catch`, `rule`, or `hook` together with the reason it applies.
- **Governance Rollout Profile**: The operator-visible maturity level for governance adoption, expressed as `minimal`, `guided`, `governed`, or `strict`, independent of council size.
- **Confidence Assessment**: The runtime-owned decision record that explains whether current evidence, review quality, and trust justify stronger or weaker governance behavior.
- **Degradation Outcome**: The explicit record that names how governance was weakened or narrowed, why that happened, and which stop posture it mapped to.
- **Escalation Event**: The explicit transfer-of-authority record describing why runtime governance was insufficient and what stronger authority path is required next.
- **Trust Evolution Record**: The persisted runtime evidence that shows how successful deliveries, overrides, incidents, and review quality changed governance trust over time.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative newly enabled governed workspaces, 100% of first-run governed boundaries begin in an explicit advisory posture unless an operator has already approved a stronger rollout profile.
- **SC-002**: In representative low-confidence, missing-evidence, unsupported-contract, and reviewer-gap scenarios, 100% of governed boundaries end in an explicit continue, degrade, escalate, wait, or stop outcome without silent governance weakening.
- **SC-003**: Operators can identify the current governance state, rollout profile, confidence rationale, and next required action from normal runtime surfaces in under 2 minutes.
- **SC-004**: In representative override, degradation, and escalation scenarios, 100% of governance state changes produce inspectable trace records that explain what changed and why.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: OpenAI Models documentation at `https://developers.openai.com/api/docs/models`, Anthropic Models overview at `https://platform.claude.com/docs/en/docs/about-claude/models`, and Google Gemini Models documentation at `https://ai.google.dev/gemini-api/docs/models`, reviewed on 2026-05-16.
- **Catalog Delta**: No bundled catalog changes were required for this adaptive-governance slice.
- **No-Change Rationale**: The bundled catalog at `assistant/catalog/model-catalog.toml`, version `0.56.0` and updated on `2026-05-16`, still matches the current coding-relevant model families surfaced by the reviewed provider documentation, including OpenAI `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.4-nano`; Anthropic `claude-opus-4-7`, `claude-sonnet-4-6`, and `claude-haiku-4-5`; and Google `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite`, `gemini-3.1-pro-preview`, `gemini-3-flash-preview`, `gemini-3.1-flash-lite`, and `gemini-3.1-flash-lite-preview`. This slice changes governance behavior rather than model availability.

## Assumptions

- Canon `authority-governance-v1` remains the required posture baseline for governed Boundline boundaries in the first S4 slice.
- If Canon publishes `adaptive-governance-v1` during this slice, it remains optional, additive, and semantic rather than mandatory for every governed run.
- Existing session, trace, and CLI surfaces are reused for governance explainability rather than replaced by a new operator interface.
- Operators remain the final authority for promoting governance maturity, accepting stronger enforcement, and clearing human-gate transitions.
