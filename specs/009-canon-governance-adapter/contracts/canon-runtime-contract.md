# Contract: Canon Runtime Adapter

## Purpose

Define the request and response semantics between Boundline's `CanonCliRuntime` adapter and the rest of the Boundline runtime so Canon-backed stage governance remains replaceable, testable, and visible.

## Request Shape

```json
{
  "request_kind": "start",
  "governance_attempt_id": "gov-attempt-1",
  "stage_key": "delivery:requirements",
  "goal": "Prepare a governed requirements packet for the parser fix",
  "workspace_ref": "/abs/workspace",
  "mode": "requirements",
  "system_context": "existing",
  "risk": "medium",
  "zone": "repo",
  "owner": "developer",
  "autopilot": false,
  "reused_packets": [
    {
      "stage_key": "delivery:requirements",
      "packet_ref": ".canon/runs/canon-run-100",
      "headline": "requirements packet ready for downstream architecture"
    }
  ],
  "input_documents": [
    {
      "kind": "stage-brief",
      "path": ".boundline/governance/delivery-requirements/input.md"
    }
  ]
}
```

For approval refreshes on a previously started Canon run, Boundline reuses the same contract with `request_kind = "refresh"` and includes the current `run_ref` plus the existing `packet_ref` when one already exists.

## Response Shape

```json
{
  "status": "governed_ready",
  "run_ref": "canon-run-123",
  "packet_ref": ".canon/runs/canon-run-123",
  "expected_document_refs": [
    ".canon/runs/canon-run-123/requirements.md"
  ],
  "document_refs": [
    ".canon/runs/canon-run-123/requirements.md"
  ],
  "approval_state": "not_needed",
  "packet_readiness": "reusable",
  "missing_sections": [],
  "headline": "requirements packet ready for downstream planning",
  "message": "Canon completed the governed requirements run"
}
```

## Supported Mode Packet Expectations (First Slice)

Boundline's first slice treats the following primary document as required for packet readiness under each supported Canon mode:

- `requirements` -> `<packet_ref>/requirements.md`
- `architecture` -> `<packet_ref>/architecture.md`
- `backlog` -> `<packet_ref>/backlog.md`
- `change` -> `<packet_ref>/change.md`
- `discovery` -> `<packet_ref>/discovery.md`
- `implementation` -> `<packet_ref>/implementation.md`
- `verification` -> `<packet_ref>/verification.md`
- `pr-review` -> `<packet_ref>/pr-review.md`

`expected_document_refs` must enumerate these Boundline-supported primary document paths for the selected mode.

## Adapter Guarantees

- `status` must be one of `governed_ready`, `awaiting_approval`, `blocked`, or `failed`.
- `run_ref` must be returned whenever Canon actually starts a run.
- `request_kind` must be either `start` or `refresh`.
- `governance_attempt_id` must remain stable across `start` and later `refresh` requests for the same governed attempt.
- `reused_packets` must be bounded to packet references, headlines, and source stage keys; the adapter must not require the whole governed artifact tree as unbounded input.
- `packet_readiness` must be explicit even when `status` is not `governed_ready`.
- `status = governed_ready` is valid only when `packet_readiness = reusable`.
- `status = awaiting_approval` requires `approval_state = requested`.
- `status = blocked` requires a non-empty blocking message.
- `expected_document_refs` must enumerate the minimum required packet documents for the selected mode so Boundline can validate packet readiness deterministically.
- `missing_sections` must be an ordered list of symbolic section references formatted as `<document_ref>#<section_slug>`.

## Failure Semantics

- Missing required request fields such as `mode`, `system_context`, `risk`, or `zone` must return `status = blocked`; the adapter must not invent them silently.
- `request_kind = refresh` must preserve the original `run_ref`, `packet_ref`, and `governance_attempt_id` lineage rather than creating a second governed attempt.
- If Canon returns success but the resulting packet is empty, scaffold-only, or missing authored required sections, the adapter must return `packet_readiness = incomplete` or `rejected` and Boundline must treat the stage as not ready.
- If Canon cannot be executed at all, times out, or returns malformed output, the adapter must return `status = failed` or `blocked` with no false `run_ref`.
- When a later `status`, `step`, or `run` refreshes a stage that is `awaiting_approval`, the adapter must report the new approval state without losing the original `run_ref` or `packet_ref` lineage.

## Local Testability

- The Boundline runtime must be able to replace this adapter with a deterministic fake or with `LocalGovernanceRuntime` in unit, contract, and integration tests.
- The rest of Boundline must depend only on these request and response semantics, not on Canon's internal implementation details.