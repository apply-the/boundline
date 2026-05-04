# Quickstart: Checkpoint Rewind

## Scenario 1: Mutating run creates a checkpoint first

1. Start or resume a normal workspace session.
2. Capture and confirm a bounded mutating goal.
3. Run `boundline run --workspace <repo>`.
4. Verify that the run output shows a checkpoint identity before or alongside
   the mutating execution result.
5. Verify that `.boundline/checkpoints/` now contains a persisted checkpoint.

Expected result: the mutating run has an explicit rollback point even if the
workspace is dirty or not under Git.

## Scenario 2: Safe restore refuses unrelated newer edits

1. Create a checkpoint through a mutating run.
2. Make unrelated newer edits in one of the captured files.
3. Run `boundline checkpoint restore <checkpoint-id> --workspace <repo>`.

Expected result: Boundline refuses the restore explicitly, names the conflicting
paths, and shows the forced restore command instead of overwriting those edits.

## Scenario 3: Forced restore rolls back the bounded mutation

1. Use the same checkpoint from Scenario 2.
2. Run `boundline checkpoint restore <checkpoint-id> --workspace <repo> --force`.
3. Inspect the workspace and the latest restore metadata.

Expected result: the captured file states are restored, the restore attempt is
recorded, and trace history remains append-only.

## Scenario 4: Clustered mutation links member checkpoints explicitly

1. Use a registered cluster with a primary workspace and one or more members.
2. Run a mutating `boundline step --cluster <primary-workspace>`.
3. List checkpoints from the primary scope.

Expected result: Boundline exposes one checkpoint group with explicit per-member
snapshots instead of hiding cluster ownership.

## Scenario 5: Workspace refoundation keeps repo-root commands stable

1. Run `cargo run --bin boundline -- --help` from the repository root.
2. Run `cargo test --workspace` from the repository root.
3. Run at least one existing session-native command from the repository root.

Expected result: the Rust workspace layout is visible to maintainers, but the
documented repo-root product surface still works.

## Scenario 6: Release validation for 0.41.0

1. Run `cargo fmt --all`.
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Run focused unit, integration, and contract tests for checkpoint manifests,
   restore conflicts, clustered restore behavior, and CLI checkpoint guidance.
4. Run `cargo test --workspace --all-targets`.
5. Run `cargo nextest run --workspace --all-features`.
6. Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
7. Verify modified or new Rust files stay above 95% coverage.

Expected result: `0.41.0` ships as a complete checkpoint-and-rewind release
with the workspace refoundation, command surface, docs, and validation evidence
aligned.