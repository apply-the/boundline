# Quickstart: Boundline Completion Verification Runtime

## Prerequisites

1. Work on branch `079-completion-verification-runtime`.
2. Use the existing Boundline CLI entrypoints; this slice adds no new closeout
   command.
3. Ensure tests can run in a temporary fixture workspace rather than the
   repository root when runtime state would otherwise dirty this repo.

## Scenario 1: Task closeout blocked because proof is missing

1. Prepare a fixture workspace with a task that claims `tests_pass`.
2. Attempt task closeout without a matching proof run.
3. Verify `status` or `inspect` reports:
   - `completion_verification_state: proof_required`
   - one blocked claim
   - a finding explaining that proof is missing
   - the exact proving command as the next action

## Scenario 2: Task closeout succeeds after fresh proof

1. Prepare a fixture workspace with a task that claims `build_clean`.
2. Trigger closeout so Boundline selects the narrowest proving command.
3. Let the proof command pass.
4. Verify:
   - pre and post workspace fingerprints were recorded
   - evidence refs were attached to the passing proof
   - `completion_verification_state: ready`
   - no success text is emitted before the proof passes

## Scenario 3: Passing proof becomes stale after workspace change

1. Reuse a fixture task with a previously passing proof.
2. Modify a meaningful tracked or non-ignored untracked workspace file.
3. Re-run `status` or attempt closeout again.
4. Verify:
   - the current fingerprint no longer matches the passing proof fingerprint
   - state is `blocked` or `proof_required`
   - a `stale_proof` finding is present
   - changed paths are surfaced with a truncation marker if needed

## Scenario 4: Inferred claim requires operator confirmation

1. Use a task with no explicit completion claim and ambiguous closeout context.
2. Attempt closeout.
3. Verify the runtime surfaces:
   - inferred claim
   - confidence level
   - evidence used for inference
   - selected proof command
   - a confirmation-required action before proof execution proceeds
4. Confirm or override the claim through the operator follow-up path before
   rerunning proof.

## Scenario 5: Stage closeout aggregates child verification

1. Prepare a stage with required child tasks in mixed states:
   - several ready
   - one stale
   - one missing proof
2. Attempt stage closeout.
3. Verify:
   - stage state remains `blocked`
   - findings identify the blocking child tasks and required actions
   - stale and missing-child findings remain visible in `status`, `inspect`,
     and orchestrate output

## Verification Commands

Run the feature test suites after implementation:

```bash
cargo test --test unit
cargo test --test contract
cargo test --test integration
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
