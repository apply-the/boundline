# Quickstart: Inspectable Routing And Assistant Decoupling

**Feature**: 027-routing-assistant-decoupling  
**Date**: 2026-05-01

This walkthrough shows the intended operator story for `0.27.0`: configure a
slot route, verify the effective decision, run the existing bounded workflow,
and confirm that follow-up surfaces keep both the routing decision and the
assistant/backend binding explicit.

## 1. Configure a slot route

Set the implementation slot for the current workspace:

```bash
cargo run --bin boundline -- config set \
  --workspace <workspace> \
  --scope workspace \
  --slot implementation \
  --runtime codex \
  --model gpt-5.4
```

Set verification at cluster scope when the workspace belongs to a registered
cluster:

```bash
cargo run --bin boundline -- config set \
  --workspace <member-workspace> \
  --cluster <primary-workspace> \
  --scope cluster \
  --slot verification \
  --runtime claude \
  --model sonnet-4
```

## 2. Inspect the effective routing decision

```bash
cargo run --bin boundline -- config show --workspace <workspace> --scope effective
```

Expected behavior:
- The output shows the resolved route for each bounded slot.
- Each route includes its authority source such as `workspace`, `cluster`,
  `global`, or `built_in`.
- The operator can identify which assistant/backend family should be bound
  before execution starts.

## 3. Run the existing session-native workflow

```bash
cargo run --bin boundline -- start --workspace <workspace>
cargo run --bin boundline -- capture --workspace <workspace> --goal "Summarize routing ownership while updating backend binding"
cargo run --bin boundline -- plan --workspace <workspace>
cargo run --bin boundline -- run --workspace <workspace>
```

Expected behavior:
- `plan` and `run` keep the existing command surface.
- Runtime output surfaces the active route, its authority source, and the bound
  assistant or command-pack family for the slot being executed.
- Backend binding does not imply a second orchestration runtime; Boundline remains
  the owner of the run.

## 4. Continue with runtime follow-up surfaces

```bash
cargo run --bin boundline -- status --workspace <workspace>
cargo run --bin boundline -- next --workspace <workspace>
cargo run --bin boundline -- inspect --workspace <workspace>
```

Expected behavior:
- `status`, `next`, and workspace-based `inspect` repeat the active route,
  source, and assistant-binding explanation without requiring a separate route
  inspection workflow.
- When the latest run came from an explicit compatibility path, those same
  surfaces keep `continuity_authority` and routing ownership explicit instead of
  pretending a resumable native session exists.

## 5. Preserve clustered or compatibility authority

Clustered delivery keeps the primary workspace authoritative:

```bash
cargo run --bin boundline -- status --cluster <primary-workspace>
```

Explicit compatibility inspection stays explicit about the owning trace:

```bash
cargo run --bin boundline -- inspect --trace <trace-ref>
```

Expected behavior:
- Cluster-aware output preserves primary-workspace ownership while still making
  the effective route and assistant binding visible.
- Compatibility inspection preserves trace ownership and uses the same routing
  vocabulary instead of inventing a session-owned route story.