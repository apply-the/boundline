# Session Model

Boundline eschews the ephemeral "chat memory" model in favor of a strictly defined, locally persistent **Session Model**. Every interaction, plan, and execution step is grounded in authoritative state files residing in your repository workspace under the `.boundline/` directory.

## Core State Files

The session state is distributed across a few transparent JSON and TOML files to separate concerns:

- **`.boundline/session.json`**: The primary session tracker. It holds the current operator goal, the active context boundary, and the current state machine position (e.g., `planning`, `running`, `blocked`, `completed`).
- **`.boundline/execution.json`**: Tracks the granular step-by-step progress of the active plan. It records which steps succeeded, which failed, and what the latest subprocess exit codes were.
- **`.boundline/workflows.toml`**: Defines the approved workflow templates, limits, and optional preflight checks required before a session can transition states.

## The State Machine

Boundline models engineering work as a deterministic state machine:

1. **`Init`**: A fresh workspace. No active goal.
2. **`Goal`**: The operator injects a requirement. The session now has a purpose.
3. **`Plan`**: Boundline (or an external adapter) generates a sequence of bounded actions based on the goal and context.
    - **Refinement (optional sub-state)**: When `--refine` is active, the plan
      stage enters a bounded refinement loop (`planner → critic → planner →
      finalizer`) before the plan is considered ready. Each round produces a
      structured round packet persisted in the trace store with confidence
      scoring, findings, and a closed stop reason.
4. **`Run`**: The execution engine processes the plan sequentially.
5. **`Inspect/Status`**: At any point, the runtime can pause and report the exact state without needing to "ask the LLM what it did."

Because the state is fully persisted to disk at each transition, you can interrupt a `run` with `Ctrl+C`, walk away, and resume it tomorrow with `boundline run` exactly where it left off.

## Traces and Checkpoints

To ensure total transparency, the session model incorporates append-only logging and state rollback mechanisms:

- **`traces/`**: Every subprocess invocation, planner prompt, adapter handoff,
  and refinement round is recorded as an immutable JSON trace file. Refinement
  rounds emit `RefinementRoundCompleted` events with round number, stop reason,
  critic and effective confidence scores, findings, and trace-linked artifact
  references.
- **`checkpoints/`**: Before destructive edits, Boundline can snapshot the workspace state, allowing the operator to reject an execution and revert the session cleanly.