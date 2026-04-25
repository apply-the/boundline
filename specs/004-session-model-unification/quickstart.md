# Quickstart: Session & Interaction Model Unification

## Prerequisites

1. Work from the repository root on branch `004-session-model-unification`.
2. Have Rust 1.95.0 with `cargo` available.
3. Use a writable workspace so Synod can persist both `.synod/session.json` and `.synod/traces/`.
4. Start from the workspace you want the active session to belong to.

## Session-Backed CLI Walkthrough

### 1. Start a new session

Run:

```bash
cargo run --bin synod -- start
```

Expected outcome:

- Synod creates `.synod/session.json` in the current workspace.
- The session becomes the active interaction state for later commands.
- No goal or task plan is required yet.

### 2. Capture a bounded goal

Run:

```bash
cargo run --bin synod -- capture --goal "Summarize the current bounded developer flow"
```

Expected outcome:

- Synod stores the goal in the active session.
- Later planning and execution commands no longer require the goal to be re-entered.

### 3. Create a plan

Run:

```bash
cargo run --bin synod -- plan
```

Expected outcome:

- Synod creates an executable plan for the active session goal.
- The active session now includes a persisted task snapshot with current execution position at the first executable step.

### 4. Execute one step at a time

Run:

```bash
cargo run --bin synod -- step
```

Expected outcome:

- Synod executes exactly one next step.
- The active session updates task context, plan position, latest status, and latest trace reference.
- If the step fails, retries, or triggers replanning, the active session preserves the latest actionable state.

### 5. Continue to a terminal outcome

Run:

```bash
cargo run --bin synod -- run
```

Expected outcome:

- Synod resumes from the active session task snapshot and continues until success, failure, exhaustion, or abort.
- The session record captures the final state and latest trace reference.

### 6. Inspect status and the next action

Run:

```bash
cargo run --bin synod -- status
cargo run --bin synod -- next
```

Expected outcome:

- `status` reports the active goal, current step position when present, latest status, and latest trace reference.
- `next` returns exactly one recommended follow-up command with a short explanation.

### 7. Inspect the detailed trace

Run:

```bash
cargo run --bin synod -- inspect --workspace "$PWD"
```

Expected outcome:

- Synod uses the active session's latest trace reference when available.
- The output reconstructs step progression, recovery events, and terminal reason.

## Assistant Walkthrough

1. Start from an assistant command that maps to the active session flow, such as `/synod-start`.
2. Let the assistant establish or reuse the active session.
3. Use `/synod-plan`, `/synod-step`, `/synod-run`, `/synod-status`, or `/synod-next` without restating already preserved session context.
4. Use `/synod-inspect` only when the active session or an explicit prior trace needs detailed inspection.

## Recovery Scenarios

### Missing session

If a session-backed command is invoked before `start`, expected output should tell the user to establish an active session first.

### Missing goal

If `plan`, `step`, or `run` is invoked before goal capture, expected output should route the user to `capture`.

### Corrupted or stale session

If `.synod/session.json` is unreadable, workspace-mismatched, or points at a missing trace, expected output should surface the exact problem and avoid hidden continuation.

### Terminal session reuse

If a task already ended in a terminal state, expected output should route the user to inspect the result or start fresh explicitly instead of silently resuming.

## Validation Commands

Run these commands from the repository root:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Minimum Validation Scenarios

1. A developer can start a session, capture a goal, plan, and run without re-entering goal context.
2. A planned session can advance through repeated `step` invocations while preserving task context and trace continuity.
3. `status` and `next` provide explicit, aligned guidance from the same active session.
4. Assistant commands reuse the active session instead of asking for preserved goal or trace information again.
5. Missing, corrupted, or stale session state fails clearly and does not continue with hidden assumptions.