# Quickstart: Assistant Command Packs

## Prerequisites

1. Work from the repository root on branch `003-assistant-command-packs`.
2. Have Rust 1.95.0 with `cargo` available so the local Boundline CLI can run.
3. Use a writable workspace so Boundline can persist traces under `.boundline/traces/`.
4. Choose one supported assistant environment: Claude, Codex, or Copilot.

## Asset Layout

- Shared installation and usage guidance lives in `assistant/README.md`.
- Claude command files live in `assistant/claude/commands/`.
- Codex command files live in `assistant/codex/commands/`.
- Copilot prompt files live in `assistant/copilot/prompts/`.

## Shell-Enabled Walkthrough

### 1. Start from chat

Invoke `/boundline-start` in your assistant.

Expected outcome:

- The assistant asks for the workspace only if it is missing.
- The assistant runs or recommends:

```bash
cargo run --bin boundline -- doctor --workspace "$PWD"
```

- The assistant summarizes whether the workspace is ready and what prerequisite, if any, must be fixed.

### 2. Bound the goal

Invoke `/boundline-plan`.

Expected outcome:

- The assistant asks only for the missing goal details.
- The assistant turns the goal into a bounded `boundline run` objective.
- The assistant routes directly to `/boundline-run`.

### 3. Execute the workflow

Invoke `/boundline-run`.

Expected outcome:

- The assistant runs or recommends:

```bash
cargo run --bin boundline -- run --workspace "$PWD" --goal "Summarize the current bounded developer flow"
```

- The assistant summarizes the terminal status, recovery signals, and trace location.

### 4. Check latest status or next step

Invoke `/boundline-step`, `/boundline-status`, or `/boundline-next`.

Expected outcome:

- The assistant runs or recommends:

```bash
cargo run --bin boundline -- inspect --workspace "$PWD"
```

- `/boundline-step` recommends one explicit next command using the latest confirmed context or pasted inspection output.
- `/boundline-status` summarizes the latest trace.
- `/boundline-next` uses that same evidence to recommend the most relevant follow-up command.

### 5. Inspect a specific trace

Invoke `/boundline-inspect` with a trace path when you need a specific run rather than the latest one.

Expected outcome:

- The assistant runs or recommends:

```bash
cargo run --bin boundline -- inspect --trace "$PWD/.boundline/traces/<task-id>.json"
```

- The assistant summarizes final status, recovery events, and next action guidance.
- The assistant surfaces `inspection_target: explicit-trace` when the user selected a specific trace.

### 6. Recover from an unreadable trace

Invoke `/boundline-inspect` with a missing or stale trace path.

Expected outcome:

- The assistant runs or recommends:

```bash
cargo run --bin boundline -- inspect --trace "$PWD/.boundline/traces/<task-id>.json"
```

- The assistant surfaces `terminal_reason: failed to read the requested trace`.
- The assistant surfaces `next_command: /boundline-inspect`.
- The assistant surfaces `corrected_command: cargo run --bin boundline -- inspect --trace <trace>` so the user can retry with a corrected reference.

## Chat-Only Walkthrough

1. Invoke the same assistant command.
2. Let the assistant ask only for missing inputs.
3. Copy the provided `cargo run --bin boundline -- ...` command into your terminal.
4. Paste the command output back into the chat.
5. Follow the assistant's summary and next-step recommendation.

Minimum fallback checkpoints:

- `/boundline-start` must recover from a not-ready workspace.
- `/boundline-run` must surface a trace location even for non-success outcomes.
- `/boundline-status`, `/boundline-next`, and `/boundline-inspect` must continue from either a workspace or an explicit trace path.
- `/boundline-inspect` must surface `inspection_target` for successful inspection and `corrected_command` for trace-read failures.

## Validation Commands

Run these commands from the repository root:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Minimum Validation Scenarios

1. Each supported assistant exposes the full seven-command pack.
2. `/boundline-start` and `/boundline-run` work in both shell-enabled and chat-only modes.
3. `/boundline-status` and `/boundline-next` can summarize the latest trace without requiring raw log inspection.
4. `/boundline-inspect` can explain a specific run using only a trace path or workspace reference.
5. `/boundline-step` can continue routing from either confirmed context or pasted inspection output.
6. Trace-read failures expose a replacement inspect command instead of a raw error blob.