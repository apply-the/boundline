# Research: Authority-Zoned Delivery Councils

## Current Implementation Surfaces

- `src/adapters/governance_runtime.rs` already owns the Canon governance adapter boundary and is the correct entry point for consuming compatible `authority_governance` metadata.
- `src/domain/governance.rs` already defines `GovernanceLifecycleState`, `ApprovalState`, `PacketReadiness`, and the core governed-stage packet model that this slice needs to extend.
- `src/domain/review.rs` already models review triggers and voting boundaries, so bounded councils should extend the existing review surface instead of creating a second runtime.
- `src/domain/session.rs` and `src/orchestrator/review_trace.rs` already persist session-visible governance and review state that later commands can project without recomputing hidden decisions.
- `src/cli/output.rs` and `src/cli/session.rs` already own operator-visible `plan`, `run`, `status`, `next`, and `inspect` projections.

## Boundaries Confirmed During Planning

- Canon remains the semantic authority for governed posture; Boundline remains the runtime owner for council composition, runtime roles, and stop semantics.
- Existing review and voting boundary concepts are the correct owner for this first council slice; the feature should not introduce an always-on debate engine or distributed review subsystem.
- Local governance fallback must remain valid when Canon is absent and governance is not required.
- Optional Canon provenance fields such as `persona_anti_behaviors`, `primary_artifact`, `artifact_order`, and `promotion_refs` stay inspectable only in this slice and must not change control resolution by themselves.

## Implementation Direction

- Extend the governance adapter reader to accept optional `authority_governance` metadata and fail closed on incompatible or incomplete required `authority-governance-v1` inputs.
- Add explicit authority-control resolution and bounded council-profile projection close to the existing governance-domain types so the same decision can be reused by session, trace, and CLI output paths.
- Reuse the current review and session state to persist findings, producer responses, adjudication outcomes, and stop semantics instead of inventing a parallel persistence layer.
- Project the consumed Canon contract line, resolved control class, council profile, findings, responses, and stop semantics through the existing session-native operator surfaces.
- Keep feature-local contracts aligned with Canon's stable integration docs so Boundline stays a strict consumer rather than redefining Canon semantics.

## Provider-Doc Audit

- Reviewed current OpenAI, Anthropic, and Google model documentation on 2026-05-15 against `assistant/catalog/model-catalog.toml`.
- No catalog changes are required for this slice.
- The bundled catalog already reflects the documented coding-relevant model families needed for this governance feature, so the implementation can stay focused on runtime control rather than model availability.

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