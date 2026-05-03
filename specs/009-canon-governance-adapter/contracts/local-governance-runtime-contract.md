# Contract: Local Governance Runtime

## Purpose

Define the deterministic request and response semantics for `LocalGovernanceRuntime`, the default governance path that keeps Boundline independently testable and executable when Canon is unavailable or not selected.

## Request Shape

```json
{
  "request_kind": "start",
  "governance_attempt_id": "gov-attempt-1",
  "stage_key": "bug-fix:investigate",
  "goal": "Capture the bounded investigation context for the add regression",
  "workspace_ref": "/abs/workspace",
  "autopilot": false,
  "bounded_context": {
    "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
    "stage_brief": ".boundline/governance/bug-fix-investigate/brief.md",
    "reused_packets": []
  }
}
```

For refreshes on an existing governed stage, Boundline reuses the same contract with `request_kind = "refresh"` and the same `governance_attempt_id` plus the current `packet_ref` when one already exists.

## Response Shape

```json
{
  "status": "governed_ready",
  "packet_ref": ".boundline/governance/bug-fix-investigate/attempt-1",
  "document_refs": [
    ".boundline/governance/bug-fix-investigate/attempt-1/brief.md"
  ],
  "approval_state": "not_needed",
  "packet_readiness": "reusable",
  "missing_sections": [],
  "headline": "local governance packet ready for the investigate stage",
  "message": "Local governance accepted the bounded stage context"
}
```

## Adapter Guarantees

- `status` must be one of `governed_ready`, `awaiting_approval`, `blocked`, or `failed`.
- `request_kind` must be either `start` or `refresh`.
- `governance_attempt_id` must remain stable across `start` and later `refresh` requests for the same governed attempt.
- `packet_ref` must be returned whenever the local runtime materializes or reuses a governed packet for the stage.
- `approval_state` must be explicit even though the normal local-runtime default is `not_needed`.
- `status = governed_ready` is valid only when `packet_readiness = reusable`.
- `status = awaiting_approval` is reserved for future local policies that can legitimately request approval; the first slice normally returns `not_needed`.
- `missing_sections` must be an ordered list of symbolic section references formatted as `<document_ref>#<section_slug>`.

## Deterministic Behavior

- The local runtime must never invent Canon-specific fields or pretend that a Canon run occurred.
- The local runtime may only use bounded stage context supplied by Boundline: selected read targets, authored stage brief, and bounded reused packet references.
- Packet readiness must be evaluated with the same deterministic rules used by Canon-backed governance: every expected document exists, authored body content is non-empty, and `missing_sections` is empty.
- When the stage is retried with narrowed context, the new request must represent a strict subset of the prior bounded context.

## Failure Semantics

- Missing `stage_key`, `goal`, or bounded context inputs must return `status = blocked`; the adapter must not infer them silently.
- `request_kind = refresh` must preserve the original `governance_attempt_id` and `packet_ref` lineage rather than creating a second governed attempt.
- If the local packet can be materialized but the authored body is empty or required sections are missing, the adapter must return `packet_readiness = incomplete` or `rejected` and Boundline must treat the stage as not ready.
- If the stage cannot continue under required governance, the adapter must return `status = blocked` with a non-empty blocking message.

## Refresh Semantics

- When Boundline revisits a stage through `status`, `step`, or `run`, the local runtime may refresh the latest packet or approval state for that stage, but it must preserve attempt lineage rather than overwriting prior governed evidence.
- The rest of Boundline must depend only on this contract, so unit and integration tests can replace the local runtime with deterministic fakes.