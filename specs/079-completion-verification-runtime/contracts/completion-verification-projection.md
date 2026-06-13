# Completion Verification Projection Contract

## Purpose

Define the additive runtime projection shared by `status`, `inspect`, and
`orchestrate` when completion verification is relevant to task, stage, or run
closeout.

## Top-Level Fields

```json
{
  "completion_verification_state": "ready | proof_required | blocked | failed",
  "scope": "task | stage | run",
  "completion_blocked_claims": ["tests_pass"],
  "completion_evidence_refs": ["trace:proof-20260612-abc123"],
  "completion_verification_findings": []
}
```

Rules:

- The fields are additive and optional; consumers that do not understand them
  may ignore them.
- Success language must not be rendered when
  `completion_verification_state != "ready"`.
- `scope = stage` and `scope = run` aggregate child verification rather than
  replacing task-level proof ownership.

## Claim Projection

```json
{
  "claim": {
    "claim_id": "claim-task-014",
    "kind": "bug_fixed",
    "source": "runtime_inference",
    "confidence": "medium",
    "summary": "The bug fix is ready for closeout."
  }
}
```

Rules:

- Explicit claims may omit `confidence`.
- Inferred claims must be surfaced before proof selection continues.
- Claim source values are limited in the first slice to:
  `explicit_metadata`, `runtime_inference`, `operator_confirmed`,
  `operator_override`.

## Finding Projection

```json
{
  "kind": "stale_proof",
  "severity": "blocking",
  "message": "The previously passing proof is stale because workspace content changed after proof execution.",
  "proof_ref": "proof-20260612-abc123",
  "changed_paths": [
    "src/lib.rs",
    "Cargo.toml"
  ],
  "required_action": "rerun_proof"
}
```

Rules:

- `stale_proof` is a finding kind, not a top-level state value.
- Blocking findings require `completion_verification_state` to be
  `proof_required`, `blocked`, or `failed`.
- If changed-path output is capped, the projection must include a truncation
  marker in the human-readable rendering or structured equivalent in the record.

## Parent-Scope Aggregation Projection

```json
{
  "completion_verification_state": "blocked",
  "scope": "stage",
  "child_summary": {
    "ready_children": 7,
    "blocked_children": 2,
    "failed_children": 0,
    "stale_children": 1,
    "missing_proof_children": 1,
    "deferred_children": 0,
    "skipped_children": 0
  },
  "completion_verification_findings": [
    {
      "kind": "stale_child_proof",
      "task_id": "T-014",
      "required_action": "rerun_proof"
    },
    {
      "kind": "missing_child_proof",
      "task_id": "T-019",
      "required_action": "run_proof"
    }
  ]
}
```

Rules:

- Parent scopes must not report success while any required child verification is
  blocked, stale, failed, or missing.
- Optional or deferred children must be represented explicitly as skipped or
  deferred with reason and must not count as required blocked children.
- If an explicit stage-level or run-level claim exists, it adds another proof
  requirement after child readiness is satisfied; it does not replace child
  verification.

## Confirmation Prompt Contract

When operator confirmation is required, the runtime must surface:

```json
{
  "claim_confirmation": {
    "inferred_claim": "migration_valid",
    "confidence": "low",
    "evidence_used": [
      "task_title",
      "changed_files",
      "recent_execution_trace"
    ],
    "selected_proof_command": "cargo test --test migration",
    "alternative_claims": [
      "build_clean"
    ],
    "consequence_of_proceeding": "Proof will validate only the migration path."
  }
}
```

Rules:

- Confirmation is required for low-confidence or ambiguous inference, partial
  proof coverage, risky surfaces, or metadata/runtime conflicts.
- High-confidence single-claim inference may proceed silently when policy
  allows.
