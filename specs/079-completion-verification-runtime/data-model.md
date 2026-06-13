# Data Model: Boundline Completion Verification Runtime

## Overview

The first slice adds typed runtime records that sit between existing task or
session lifecycle state and operator-facing closeout surfaces. The model stays
additive: it does not replace `TaskStatus`, `SessionStatusView`, or existing
trace ownership.

## Entities

### CompletionClaim

- Purpose: The concrete engineering outcome Boundline is asserting before
  closeout.
- Key fields:
  - `claim_id`: stable identifier for the active closeout claim
  - `kind`: `tests_pass`, `bug_fixed`, `build_clean`, `migration_valid`
  - `scope`: `task`, `stage`, or `run`
  - `source`: `explicit_metadata`, `runtime_inference`,
    `operator_confirmed`, `operator_override`
  - `confidence`: `high`, `medium`, `low` for inferred claims only
  - `summary`: short human-readable claim sentence
  - `supporting_signals`: bounded evidence used to derive the claim
- Validation rules:
  - Every closeout attempt has at most one active dominant claim in the first
    slice.
  - Explicit metadata claims omit inference confidence.
  - Runtime-inferred claims must be surfaced before proof selection proceeds.

### ProofCommandSelection

- Purpose: The deterministic mapping from a claim to one proving command.
- Key fields:
  - `claim_id`
  - `command_ref`: stable reference to the selected proof command rule
  - `command_line`: exact proving command to execute
  - `selection_reason`: why this command is the narrowest falsifying proof
  - `coverage_note`: whether the proof fully covers the claim
- Validation rules:
  - One active proof command per claim in the first slice.
  - The proof command is selected against the claim, never vice versa.
  - Partial-coverage proof requires confirmation or blocked closeout.

### WorkspaceContentFingerprint

- Purpose: The normalized view of meaningful workspace content used to decide
  whether a passing proof is still fresh.
- Key fields:
  - `fingerprint_id`
  - `captured_at`
  - `included_roots`
  - `excluded_roots`
  - `content_digest`
  - `tracked_path_count`
  - `untracked_path_count`
  - `changed_paths_sample`
  - `truncated_changed_paths`: bool
- Inclusion rules:
  - Tracked source, config, test, build, and claim-relevant documentation
  - Non-ignored untracked workspace files
- Exclusion rules:
  - `.git/`
  - `.boundline/traces/`
  - `.boundline/artifacts/`
  - `.boundline/cache/`
  - Boundline-written proof evidence
  - Ignored or configured volatile paths such as `target/`, `node_modules/`,
    `dist/`, `build/`, `.next/`, `.venv/`

### ProofRunRecord

- Purpose: The fresh execution result for the selected proving command.
- Key fields:
  - `proof_ref`
  - `claim_id`
  - `command_ref`
  - `started_at`
  - `finished_at`
  - `exit_code`
  - `outcome`: `passed`, `failed`, `interrupted`, `unsupported`
  - `summary_lines`
  - `pre_fingerprint_ref`
  - `post_fingerprint_ref`
  - `evidence_refs`
- Validation rules:
  - Every passing proof records both pre and post fingerprints.
  - Failed or interrupted runs still persist summary lines and proof refs.
  - Evidence refs must come from the proof that produced the recorded outcome.

### CompletionVerificationFinding

- Purpose: The typed explanation for why closeout is not ready.
- Key fields:
  - `kind`: `missing_proof`, `stale_proof`, `failed_proof`, `mismatched_proof`,
    `stale_child_proof`, `missing_child_proof`, `failed_child_proof`,
    `claim_conflict`
  - `severity`: `blocking`, `warning`
  - `message`
  - `proof_ref`: optional
  - `task_id`: optional for child findings
  - `changed_paths`: bounded sample
  - `required_action`: `run_proof`, `rerun_proof`, `confirm_claim`,
    `override_claim`, `resolve_conflict`
- Validation rules:
  - `stale_proof` findings require `required_action = rerun_proof`.
  - Child findings require a child task or stage reference.
  - Blocking findings must align with `completion_verification_state != ready`.

### ChildVerificationSummary

- Purpose: Aggregated readiness counts for stage and run closeout.
- Key fields:
  - `scope`: `stage` or `run`
  - `ready_children`
  - `blocked_children`
  - `failed_children`
  - `stale_children`
  - `missing_proof_children`
  - `deferred_children`
  - `skipped_children`
  - `findings`
- Validation rules:
  - Optional or deferred children are counted separately and do not block
    required readiness.
  - Required blocked, stale, failed, or missing-proof children block parent
    closeout.

### CompletionVerificationProjection

- Purpose: The additive operator-facing projection rendered in `status`,
  `inspect`, and `orchestrate`.
- Key fields:
  - `completion_verification_state`: `ready`, `proof_required`, `blocked`,
    `failed`
  - `scope`: `task`, `stage`, `run`
  - `claim`
  - `blocked_claims`
  - `findings`
  - `evidence_refs`
  - `child_summary`: optional for parent scopes
- Validation rules:
  - `ready` requires no blocking findings.
  - `proof_required` or `blocked` must carry at least one actionable finding.
  - Parent scopes surface child summaries instead of hiding child failures.

## Relationships

- `CompletionClaim 1 -> 1 ProofCommandSelection`
- `CompletionClaim 1 -> many ProofRunRecord`
- `ProofRunRecord 1 -> 2 WorkspaceContentFingerprint` for pre/post capture
- `CompletionVerificationProjection 1 -> many CompletionVerificationFinding`
- `CompletionVerificationProjection 0..1 -> 1 ChildVerificationSummary`

## State Transitions

### Claim Lifecycle

`derived -> surfaced -> confirmed_or_accepted -> proved -> ready`

Failure paths:

- `derived -> blocked_for_clarification`
- `proved -> stale`
- `proved -> failed`

### Parent Closeout Lifecycle

`aggregate_children -> evaluate_parent_claim -> ready_or_blocked`

Failure paths:

- Any required child stale, failed, or missing proof -> `blocked`
- Explicit parent claim without fresh proof -> `proof_required` or `blocked`

## Persistence Notes

- These entities are additive extensions to existing task, session, and trace
  persistence surfaces.
- No new external database or service is required.
- Stable serialized shapes should use typed serde structs/enums rather than ad
  hoc JSON map assembly.
