# Research: Control Graduation And Adaptive Governance

## Decision: Keep S4 runtime behavior owned by Boundline

- **Decision**: Boundline remains the owner of runtime confidence, trust evolution, degradation, escalation, councils, and stop transitions.
- **Rationale**: The S4 roadmap explicitly places semantic posture in Canon and operational governance behavior in Boundline. Keeping those responsibilities separate prevents cross-repo drift and preserves local runtime control.
- **Alternatives considered**:
  - Push confidence or trust scoring into Canon: rejected because it would make Canon a runtime decision-maker.
  - Duplicate confidence logic across Canon and Boundline: rejected because it would create competing trust models.

## Decision: Reuse existing governance, session, trace, and CLI surfaces

- **Decision**: The first slice should extend `src/domain/governance.rs`, `src/orchestrator/governance.rs`, `src/domain/session.rs`, `src/orchestrator/review_trace.rs`, `src/cli/session.rs`, and `src/cli/output.rs` instead of introducing a separate adaptive-governance runtime.
- **Rationale**: Boundline already has the persisted session and trace surfaces needed to make governance progression inspectable. Reuse keeps the slice sequential, stateful, and debuggable.
- **Alternatives considered**:
  - Create a dedicated adaptive-governance subsystem or service: rejected because it adds orchestration complexity before the first independently valuable slice is proven.
  - Keep S4 purely in traces and not session state: rejected because operators need the latest posture visible from `status`, `next`, and `inspect` without reconstructing history manually.

## Decision: Model adaptive governance as explicit typed records

- **Decision**: Represent governance state, rollout profile, confidence, trust, degradation, escalation, and override history as explicit typed domain records rather than loosely assembled maps or ad hoc trace payload fragments.
- **Rationale**: The repository constitution and Rust language rules require stable shapes to use typed models. Explicit records also make later session and CLI projection deterministic.
- **Alternatives considered**:
  - Reuse untyped JSON fields inside task state: rejected because stable runtime projection would drift and violate the stable-shape rules.
  - Infer adaptive state from existing council fields only: rejected because S4 introduces governance-maturity concepts that are distinct from S3 council composition.

## Decision: Preserve a two-tier Canon contract boundary

- **Decision**: `authority-governance-v1` remains the required Canon posture baseline, while `adaptive-governance-v1` is treated as an optional, additive, semantic companion contract when present.
- **Rationale**: This keeps S4 compatible with existing governed packets and lets Boundline ship the runtime slice without blocking on a new required Canon machine contract.
- **Alternatives considered**:
  - Require `adaptive-governance-v1` for all S4 behavior immediately: rejected because it would stall the first runtime slice behind a second hard dependency.
  - Overload `authority-governance-v1` with adaptive behavior: rejected because it would blur the S3 posture baseline and create silent meaning drift.

## Decision: Start new governed surfaces in advisory mode by default

- **Decision**: A newly enabled or low-trust governed surface begins in `advisory` mode unless an operator has explicitly approved a stronger maturity.
- **Rationale**: The roadmap makes progressive adoption a core outcome. Advisory-first behavior minimizes governance fatigue and reduces early bypass incentives.
- **Alternatives considered**:
  - Start in `rule` mode by default: rejected because the first-run operator experience becomes punitive instead of calibrating.
  - Leave starting mode implicit: rejected because hidden maturity assumptions would violate the required explainability principles.

## Decision: Map degradation onto the existing S3 stop semantics

- **Decision**: Degradation stays an operational mechanism and must map onto existing stop semantics such as `proceed_with_advisory`, `proceed_with_warning`, `degraded_proceed`, `human_gate_required`, and `hard_stop`.
- **Rationale**: S4 defines runtime behavior, not a second stop-language. Reusing the existing stop vocabulary keeps operator reading surfaces consistent.
- **Alternatives considered**:
  - Create a second S4-only stop vocabulary: rejected because it would force operators to learn two separate outcome systems.
  - Collapse degradation into generic warnings: rejected because degradation must remain explicit and operationally meaningful.

## Provider-Doc Audit

- **Decision**: Carry forward the 2026-05-16 provider-doc audit recorded in the spec with a no-change result for `assistant/catalog/model-catalog.toml`.
- **Rationale**: The planning workflow requires catalog currency evidence, and the spec already records a same-day audit against current OpenAI, Anthropic, and Google model documentation.
- **Alternatives considered**:
  - Re-open catalog selection during S4 design: rejected because this slice changes governance behavior, not route inventory.

## Likely Touchpoints

- `src/adapters/governance_runtime.rs`
- `src/domain/governance.rs`
- `src/domain/review.rs`
- `src/domain/session.rs`
- `src/orchestrator/governance.rs`
- `src/orchestrator/decision_loop.rs`
- `src/orchestrator/review_trace.rs`
- `src/cli/output.rs`
- `src/cli/session.rs`
- `tests/unit/governance_policy.rs`
- `tests/unit/governance_runtime.rs`
- `tests/contract/governance_session_contract.rs`
- `tests/contract/governance_trace_contract.rs`
- `tests/integration/session_governance_flow.rs`
- `tests/integration/governance_autopilot_flow.rs`
- `tests/integration/workflow_follow_through.rs`
- `tech-docs/control-graduation-model.md`
- `tech-docs/adaptive-governance.md`
- `tech-docs/runtime-confidence-and-calibration.md`
- `tech-docs/degradation-and-escalation.md`