# Quickstart: Chat-First Host-Integrated Runtime

## Goal

Validate the first host-integrated runtime slice by exercising the structured
output mode on the existing session-native and inspection commands.

## Scenario 1: Start And Inspect Session State From A Host Surface

1. Create a temporary fixture workspace.
2. Run:

   ```bash
   cargo run --bin boundline -- start --json
   ```

3. Verify the command exits successfully and returns:
   - `command_name: start`
   - `exit_status: succeeded`
   - `session_status.latest_status`
   - `session_status.next_command`
   - `rendered_output`

## Scenario 2: Capture, Plan, And Resume Through Structured Session Output

1. Run:

   ```bash
   cargo run --bin boundline -- goal --goal "Fix the failing add test" --json
   cargo run --bin boundline -- plan --json
   cargo run --bin boundline -- status --json
   cargo run --bin boundline -- next --json
   ```

2. Verify the structured payload preserves:
   - `session_status.goal`
   - `session_status.latest_status`
   - `session_status.continuity_authority` when present
   - `session_status.context_summary` when present
   - `session_status.next_command`

## Scenario 3: Run And Inspect Through Structured Trace Output

1. Run:

   ```bash
   cargo run --bin boundline -- run --json
   cargo run --bin boundline -- inspect --json
   ```

2. Verify the payload preserves:
   - `trace_summary.trace_ref`
   - `trace_summary.routing_summary` or `trace_summary.routing_projection`
   - `trace_summary.terminal_status`
   - `trace_summary.terminal_reason`
   - `trace_location` when available
   - `rendered_output`

## Scenario 4: Host Guidance Still Supports Chat-Only Fallback

1. Open the assistant command pack assets for `run`, `status`, `next`, and
   `inspect`.
2. Verify shell-enabled paths prefer the structured output mode.
3. Verify chat-only fallback still presents copyable plain-text commands and
   tells the user to paste the output before continuing.

## Validation Commands

Use focused validation while implementing the slice:

```bash
cargo test --test contract session_command_contract
cargo test --test contract assistant_command_definition_contract
cargo test --test integration assistant_shell_enabled_flow
cargo test --test integration assistant_chat_fallback
cargo test -p boundline-cli session::tests
cargo test -p boundline-cli inspect::tests
```