# Governed Reasoning Profile Contracts Quickstart

**Version**: 0.61.0  
**Audience**: Boundline maintainers, Canon maintainers, release reviewers  
**Purpose**: Exercise the first bilateral reasoning-profile contract in under 15 minutes

## What ships in this feature

Boundline 061 introduces an explicit reasoning-profile runtime layer that can
activate bounded challenge inside the normal session-native workflow.

The first release should make the following visible:

- selected profile id
- activation reason
- Canon posture provenance
- participant topology and independence result
- degraded, blocked, or escalated outcomes
- confidence contribution and next action

Canon 0.57.0 publishes the matching provider-side challenge-posture contract.

## Scenario 1: Activate A Blind Review Profile

Goal: confirm that a governed verification stage can activate
`independent_pair_review` without leaving the current session lifecycle.

Expected flow:

1. Start a normal session-native workflow.
2. Reach a stage whose posture requires stronger challenge.
3. Confirm that Boundline records a reasoning-profile activation instead of a
   second hidden workflow.
4. Confirm that the profile result remains bounded and returns a next action.

Expected projection shape:

```text
reasoning_profile.profile_id: independent_pair_review
reasoning_profile.status: active|completed|degraded|blocked|escalated
reasoning_profile.activation_reason: ...
reasoning_profile.independence.result: passed|degraded|failed
reasoning_profile.outcome.headline: ...
reasoning_profile.confidence.summary: ...
```

## Scenario 2: Inspect Reasoning Trace And Confidence Handoff

Goal: confirm that the trace and inspect surfaces explain why the profile ran
and how it affected confidence.

Expected flow:

1. Run a representative profile-backed stage.
2. Open `status`, `next`, and `inspect` on the same session.
3. Confirm that profile activation, participant lifecycle, disagreement or
   convergence, and confidence contribution are visible.

Focused validation targets:

- `tests/contract/reasoning_profile_trace_contract.rs`
- `tests/integration/reasoning_profile_inspect.rs`

## Scenario 3: Fail Closed On Insufficient Independence

Goal: confirm that Boundline does not silently downgrade a required stronger
challenge posture.

Expected flow:

1. Prepare a profile request whose required independence cannot be met.
2. Run the same session-native workflow.
3. Confirm that Boundline blocks, degrades, or escalates explicitly.
4. Confirm that the next action tells the operator how to proceed.

Focused validation targets:

- `tests/unit/reasoning_profile_selection.rs`
- `tests/integration/reasoning_profile_degradation.rs`

## Scenario 4: Reject Contract Drift Between Boundline And Canon

Goal: confirm that incompatible posture contract lines or version windows fail
before runtime execution begins.

Expected flow:

1. Point the Boundline contract test at a mismatched Canon contract fixture or
   sibling Canon repo revision.
2. Run the contract-alignment tests.
3. Confirm that the failure identifies whether the issue is a version-window
   mismatch, missing contract line, or vocabulary drift.

Focused validation targets:

- `tests/contract/canon_reasoning_posture_contract.rs`
- sibling Canon docs or contract checks for
  `docs/integration/governed-reasoning-posture-contract.md`

## Completion Commands

Run these commands from the Boundline repository root during closeout:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo test --test contract reasoning_profile_contract
cargo test --test contract canon_reasoning_posture_contract
cargo test --test integration reasoning_profile_
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Run the matching Canon checks from the sibling Canon repository root during
closeout:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --test contract governed_reasoning_posture_contract
```