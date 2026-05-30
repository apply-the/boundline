# Quickstart: Context Selection Hardening

## Scenario 1: A failing test drives the bounded context

1. Start or resume a session for a Rust workspace with a failing test.
2. Record a goal such as `Fix the failing add test`.
3. Run `boundline plan --workspace <repo>`.
4. Confirm that the output shows:
   - a bounded context summary,
   - the selected test or source inputs,
   - why those inputs were chosen, and
   - whether the context is credible, stale, or insufficient.

Expected result: the goal plan persists one context pack whose primary inputs
come from the failing-test evidence rather than generic keyword matches.

## Scenario 2: An authored brief narrows a generic goal

1. Start a session in a workspace whose goal text is broad.
2. Capture the goal with one or more Markdown briefs that name the intended
   files or modules.
3. Run `boundline plan --workspace <repo>`.
4. Review `boundline status --workspace <repo>` or
   `boundline inspect --workspace <repo>`.

Expected result: the selected context is anchored by the authored brief
references, and the surfaced provenance explains that those authored inputs
overrode weak ambient repository similarity.

## Scenario 3: Weak ambient evidence stops planning explicitly

1. Capture a vague goal in a workspace where no failing tests, authored file
   refs, workflow targets, recent mutations, or Canon artifacts point to a
   bounded target.
2. Run `boundline plan --workspace <repo>`.

Expected result: planning does not silently invent a credible context. The CLI
reports an insufficient or stale context state and points to the bounded
recovery action.

## Scenario 4: Cluster boundaries remain explicit

1. Initialize or use an existing registered cluster with a primary workspace and
   one member workspace.
2. Record a goal from the primary workspace.
3. Run `boundline plan --cluster <primary-workspace>`.

Expected result: only cross-workspace files with direct evidence anchors enter
the active context, and the operator can see why a member file was selected or
excluded.

## Scenario 5: Release validation for 0.40.0

1. Run `cargo fmt --all`.
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Run targeted unit, integration, and contract coverage for the changed Rust
   slices.
4. Run `cargo test --no-run --all-targets`.
5. Run `cargo nextest run --workspace --all-features`.
6. Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
7. Verify modified and new Rust files stay above 95% coverage.

Expected result: the release ships as `0.40.0` with runtime behavior, docs,
roadmap, changelog, and validation evidence aligned to the hardened context
selection contract.