# Quickstart: Unify Route Summaries And Config Projection

## Goal

Exercise the complete `0.24.0` operator story: aligned route summaries, explicit ownership, and material config projection across mixed-route follow-up.

## Prerequisites

- A workspace with session-native state available through `.synod/session.json`.
- Optional `.synod/workflows.toml` and `.synod/config.toml` to exercise workflow and routing defaults.
- At least one explicit compatibility run or persisted compatibility trace to verify inspect-only follow-up.
- Optional governance-enabled workspace state to observe paused or blocked review/governance summaries.

## Flow

1. Start a session-native workflow and capture a goal.
2. Run a named workflow that can pause in review or governance.
3. Run an explicit compatibility path in the same workspace.
4. Compare `status`, `next`, `inspect`, and workflow follow-up commands.
5. Verify that the same summary vocabulary appears across routes while route ownership remains explicit.

## Example Validation Sequence

```text
cargo run --bin synod -- start --goal "Unify route summaries"
cargo run --bin synod -- workflow run governed-delivery
cargo run --bin synod -- status
cargo run --bin synod -- next
cargo run --bin synod -- inspect
cargo run --bin synod -- run --workspace . --goal "Compatibility route check"
cargo run --bin synod -- status
cargo run --bin synod -- inspect --workspace .
```

## Expected CLI Behavior

### Native and workflow follow-up

- `status`, `next`, and workflow-aware surfaces share route owner, authority, execution condition, and next-action wording.
- If workflow, review, or governance owns the current pause or block, that ownership remains explicit.

### Compatibility follow-up

- Compatibility follow-up uses the same summary family for authority and next-step guidance.
- When no active session is resumable, the surface remains explicitly inspect-only and does not imply hidden native ownership.

### Config projection

- Relevant route overrides, workflow metadata, and routing defaults appear when they materially explain the current follow-up story.
- Irrelevant or stale config values do not appear just because they exist in workspace or global config.

## Validation Checklist

- Route owner and continuity authority are visible across native, workflow, review/governance, and compatibility follow-up.
- Aligned `execution_condition` and next-action wording does not hide which route owns the work.
- Material config projection is visible when it explains the route choice and absent when it does not.
- Docs, assistant guidance, version metadata, and changelog describe the same `0.24.0` behavior as the runtime output.
