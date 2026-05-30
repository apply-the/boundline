# Quickstart: Goal Negotiation And Constraint Modeling

## Goal

Exercise the complete `0.26.0` operator story: capture one bounded goal,
derive one negotiated delivery packet, verify that planning honors the packet,
and confirm that follow-up surfaces keep the active acceptance boundary and
binding constraints explicit.

## Prerequisites

- A local workspace with writable `.boundline/` session and trace state.
- Use `cargo run --bin boundline -- ...` from the repository root when validating
  locally.
- Have either a direct goal or one Markdown brief available so capture can
  derive a negotiated delivery packet.
- Treat direct explicit compatibility execution as a separate path; this
  quickstart focuses on the primary session-native flow.

## Flow

1. Start a new session-native delivery story.
2. Record a bounded goal with optional authored brief inputs.
3. Confirm that capture output shows the negotiated outcome, acceptance
   boundary, and binding constraints.
4. Plan the work and verify that the negotiated story remains visible.
5. Run or inspect the session and confirm that non-success follow-up identifies
   the active binding constraint or tradeoff when one exists.
6. Validate release docs and repository checks.

## Example Validation Sequence

```text
cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "Fix the failing parser test without widening the public API" --brief docs/example-negotiation-brief.md --risk medium --owner platform
cargo run --bin boundline -- status
cargo run --bin boundline -- plan --flow bug-fix
cargo run --bin boundline -- run
cargo run --bin boundline -- status
cargo run --bin boundline -- next
cargo run --bin boundline -- inspect
```

## Expected CLI Behavior

### Negotiated capture

- `goal` reports one negotiated delivery packet for the active session.
- The packet summary makes the intended outcome, acceptance boundary, and any
  binding constraints explicit even when the operator supplied only a direct
  goal.
- If negotiation is not credible, capture or follow-up output explains what
  clarification or conflict blocks planning.

### Negotiated planning and follow-up

- `plan` preserves the active acceptance boundary and selected tradeoff story
  instead of collapsing back to goal-only output.
- `status`, `next`, and `inspect` identify the currently binding constraint,
  acceptance boundary, or unresolved tradeoff when the session is blocked,
  failed, exhausted, or inspect-only.
- The explicit compatibility route remains visibly separate and does not imply
  hidden session-native negotiation authority.

## Validation Checklist

- Capture produces an inspectable negotiated packet for both goal-only and
  authored-brief scenarios.
- Planning stops before confirmation when required constraints are ambiguous or
  conflicting.
- Follow-up surfaces preserve the active constraint or tradeoff story in both
  success and non-success paths.
- Docs, assistant guidance, version metadata, and changelog describe the same
  `0.26.0` negotiation behavior as the runtime output.