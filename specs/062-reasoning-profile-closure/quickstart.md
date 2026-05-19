# Reasoning Profile Closure Quickstart

**Target Boundline Release**: 0.62.0  
**Companion Canon Release**: 0.59.0  
**Audience**: Boundline maintainers, Canon maintainers, release reviewers  
**Purpose**: Exercise the full S6.1 closure slice in under 20 minutes

## What closes in this feature

Boundline 062 finishes the residual S6.1 work by:

- proving `independent_pair_review` with a credible positive-path outcome
- proving `heterogeneous_security_review` through a dedicated runtime story
- proving `bounded_reflexion` through a real runtime activation path
- classifying debate as bounded substrate carried by shared primitive behavior
- classifying adjudication as a shared primitive rather than a standalone
   shipped profile
- aligning runtime claims, docs, roadmap, validation, and release artifacts
- clearing the release-blocking maintainability findings in the touched session
  and reasoning surfaces

## Scenario 1: Close Independent Pair Review Positively

Goal: confirm that `independent_pair_review` reaches a converged or adjudicated
positive-path outcome inside the existing session-native workflow.

Expected flow:

1. Start a normal governed delivery workflow.
2. Reach a stage whose policy or posture requires blind independent review.
3. Confirm that the profile activates with distinct reviewer routes.
4. Confirm that `status`, `inspect`, and trace surfaces show a positive-path
   terminal result plus confidence output.

Focused validation targets:

- `tests/integration/reasoning_profile_activation.rs`
- `tests/contract/reasoning_profile_contract.rs`

## Scenario 2: Close Heterogeneous Security Review And Reflexion

Goal: confirm that the remaining residual concrete profiles run end-to-end.

Expected flow:

1. Run a representative security-sensitive stage with heterogeneous review
   enabled.
2. Confirm explicit activation, inspectable participant diversity, terminal
   outcome, and confidence contribution.
3. Run a representative bounded reflexion scenario and confirm explicit critique
   or revise progression with bounded non-success handling when interrupted or
   exhausted.

Focused validation targets:

- `tests/integration/reasoning_profile_activation.rs`
- `tests/integration/reasoning_profile_degradation.rs`
- `tests/contract/reasoning_profile_trace_contract.rs`

## Scenario 3: Validate Debate And Adjudication Classification

Goal: confirm that the repository no longer overstates debate or adjudication.

Expected flow:

1. Review the closure classification contract and validation report.
2. Confirm that runtime and docs expose debate only as substrate and
   adjudication only as a shared primitive.
3. Confirm that no operator-facing surface invents a standalone shipped profile
   id for those capabilities.

Focused validation targets:

- `specs/062-reasoning-profile-closure/contracts/profile-closure-classification-contract.md`
- `tests/contract/reasoning_profile_contract.rs`
- `tests/contract/reasoning_profile_trace_contract.rs`

## Scenario 4: Validate Release Alignment And Companion Compatibility

Goal: confirm that the published Boundline `0.62.x` and Canon `0.59.x`
compatibility artifacts stay aligned even when the sibling Canon repo is
absent.

Expected flow:

1. Run the Boundline contract tests with and without the sibling Canon repo
   available.
2. Confirm that Boundline validates the active release pair using the sibling
   repo when present or the local snapshot when absent.
3. Confirm that the Canon changelog, docs, tests, and version anchors match the
   published compatibility window.

Focused validation targets:

- `tests/contract/canon_reasoning_posture_contract.rs`
- companion Canon docs and contract tests

## Completion Commands

Run these commands from the Boundline repository root during closeout:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

The closure is not complete until the existing SonarCloud quality workflow in
`.github/workflows/quality.yml` reports the touched cognitive-complexity
findings as cleared.

Run the matching Canon checks from the Canon repository root during closeout:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
```