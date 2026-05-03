# Quickstart: Dynamic Planning And Flow Inference

## Scenario 1: Capture a goal and generate a bounded proposal

1. Start or resume a session.
2. Capture a goal and any authored brief material.
3. Run `boundline plan --workspace <repo>`.
4. Confirm that the output shows:
   - inferred flow with evidence-based rationale,
   - selected target files or symbols,
   - verification strategy,
   - proposal revision `1`, and
   - an explicit next action recommending confirmation or more context.

Expected result: the session stores a proposed goal plan, `status` and `next`
report that execution is blocked on plan confirmation, and `inspect` shows the
same proposal evidence.

## Scenario 2: Confirm the proposal and execute natively

1. Review the proposed plan with `boundline status --workspace <repo>` or
   `boundline inspect --workspace <repo>`.
2. Confirm the current proposal with `boundline plan --workspace <repo> --confirm`.
3. Run `boundline run --workspace <repo>`.

Expected result: the session records the proposal as confirmed, native goal-plan
execution becomes available, and the runtime uses the confirmed flow plus
verification strategy instead of a static analyze/fix/test template.

## Scenario 3: Replan after new evidence changes the shape

1. After a failed verification or a new trace/context input, run
   `boundline plan --workspace <repo> --replan`.
2. Confirm that the new output shows:
   - proposal revision incremented from the prior revision,
   - a summary of what changed,
   - whether flow, targets, tasks, or verification strategy moved, and
   - the reason the prior revision was superseded.
3. Re-confirm with `boundline plan --workspace <repo> --confirm` before running
   again.

Expected result: the previous confirmed plan is retained as superseded lineage,
the new revision becomes the active proposal, and `run` remains blocked until
the operator confirms the revision.

## Scenario 4: Insufficient context remains explicit

1. Capture a vague goal with no credible workspace or authored context.
2. Run `boundline plan --workspace <repo>`.

Expected result: planning does not silently invent a plan. The CLI reports that
bounded context is required, persists the insufficient-context proposal state,
and tells the operator which clarifying evidence is missing.

## Scenario 5: Release validation for 0.35.0

1. Run `cargo fmt --all`.
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Run targeted unit, integration, and contract coverage for the changed slices.
4. Run `cargo test --no-run --all-targets`.
5. Run `cargo nextest run --workspace --all-features`.
6. Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
7. Verify modified and new Rust files remain above 95% coverage.

Expected result: the release ships as `0.35.0` with docs, roadmap, assistant
guidance, changelog, and validation outputs aligned to the new planning contract.