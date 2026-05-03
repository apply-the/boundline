# Research: Session & Interaction Model Unification

## Decision 1: Persist one workspace-scoped active session at `<workspace>/.boundline/session.json`

- **Decision**: Store the active session as a single JSON document at `<workspace>/.boundline/session.json`, parallel to the existing trace directory under `<workspace>/.boundline/traces/`.
- **Rationale**: The existing trace store already anchors Boundline state under `.boundline/` inside the developer workspace. A sibling `session.json` keeps the session local, explicit, and easy to inspect without adding discovery or indexing logic. This aligns with the feature's single-active-session-per-workspace scope.
- **Alternatives considered**:
  - Persist sessions in memory only: rejected because CLI invocations must survive process boundaries.
  - Add a database or external store: rejected because the slice must remain local, file-based, and dependency-light.
  - Use one file per command invocation: rejected because the feature needs one active interaction state rather than another append-only history surface.

## Decision 2: Persist a task snapshot inside the session record rather than only goal and trace references

- **Decision**: Store the active task snapshot in the session record using the existing serializable `Task`, `Plan`, and `TaskContext` models, alongside session metadata such as latest trace reference and latest terminal reason.
- **Rationale**: The current orchestrator engine only exposes `run()`, but the new feature requires `plan` and `step` across separate invocations. Persisting only the goal, current step index, and latest trace would be insufficient because stepwise continuation also needs the current plan, task context, retry counters, and replan counters. The existing domain models are already serializable and therefore suitable as a persisted working snapshot.
- **Alternatives considered**:
  - Persist only a minimal summary (goal, step index, trace): rejected because `step` could not resume accurately after process exit.
  - Persist a separate plan file and context file: rejected because it spreads one active interaction state across multiple mutable files.
  - Persist only traces and reconstruct state from them: rejected because traces are history-oriented and not optimized for fast, explicit continuation.

## Decision 3: Add a `SessionStore` adapter parallel to the existing `TraceStore`

- **Decision**: Introduce a new adapter for session persistence with a file-backed implementation instead of embedding session I/O directly into CLI or orchestrator modules.
- **Rationale**: The repository already uses a trait-plus-file-implementation pattern for traces. Mirroring that pattern for sessions keeps persistence isolated, testable, and replaceable without coupling CLI command routing to filesystem details.
- **Alternatives considered**:
  - Put JSON reads and writes directly in `cli.rs`: rejected because session lifecycle logic would become harder to test and reuse.
  - Persist the session only through orchestrator code: rejected because `start`, `capture`, `status`, and `next` must work even when no orchestration loop is running.

## Decision 4: Reuse existing orchestrator semantics by extracting a session runtime adapter instead of inventing a second engine

- **Decision**: Add a small orchestration adapter that reuses the current planner, step execution, recovery, terminal, and trace logic to support `plan`, `step`, and `run` against a persisted task snapshot.
- **Rationale**: `Orchestrator::run()` already defines the trusted semantics for step execution, retries, replanning, and terminal outcomes. The session feature needs new entry points, not new orchestration rules. An adapter that factors reusable transitions from the engine preserves behavior while making stepwise commands possible.
- **Alternatives considered**:
  - Reimplement planning and step logic in CLI code: rejected because it would drift from the orchestrator's recovery and terminal semantics.
  - Extend traces into a resumable execution engine: rejected because traces are for inspection, not as the authoritative mutable runtime state.
  - Add a background worker that keeps the task alive between commands: rejected because the constitution requires explicit, user-invoked sequential execution.

## Decision 5: Introduce an explicit session-backed CLI surface with `start`, `capture`, `plan`, `step`, `run`, `status`, and `next`

- **Decision**: Add session-native CLI commands that operate on the active workspace session, while keeping `inspect` as the trace-focused command for detailed run history and retaining `doctor` for readiness checks.
- **Rationale**: The feature's value is reducing repeated inputs and aligning CLI with assistant workflows. A dedicated session-native command set gives both surfaces the same explicit vocabulary for establishing, advancing, and inspecting work state.
- **Alternatives considered**:
  - Overload the existing `run` and `inspect` commands with hidden session behavior only: rejected because the user would still lack explicit `start`, `capture`, and `next` lifecycle entry points.
  - Make assistant-only commands reuse hidden session state while leaving CLI unchanged: rejected because the feature must unify, not split, interaction models.

## Decision 6: Resolve the workspace from the current directory by default, with optional explicit override

- **Decision**: Session-backed commands operate against the current working directory by default and may allow an explicit workspace override when necessary.
- **Rationale**: The feature aims to reduce repeated parameter passing and make chat-first workflows natural. Defaulting to the current workspace lets `boundline start`, `boundline plan`, and follow-up assistant-invoked commands work naturally from the repository root while still leaving room for explicit targeting when a user needs it.
- **Alternatives considered**:
  - Require `--workspace` on every session command: rejected because it preserves the repetitive UX this feature is meant to remove.
  - Infer workspace from the latest trace only: rejected because the session itself must be the authoritative interaction anchor.

## Decision 7: Validate continuity and recovery with Rust tests across unit, integration, and contract layers

- **Decision**: Add unit tests for the session record and store, integration tests for session-backed CLI flows and failure recovery, and contract tests for command behavior and assistant continuity.
- **Rationale**: This feature spans persisted local state, bounded orchestration behavior, and cross-surface continuity. The repository already uses Cargo-based validation, so extending the same harness keeps drift visible and avoids a second tooling path.
- **Alternatives considered**:
  - Rely on manual CLI testing: rejected because session corruption, stale state, and continuity regressions are easy to miss without executable checks.
  - Use shell-only validation scripts: rejected because they duplicate the existing Rust test surface and make portability harder.