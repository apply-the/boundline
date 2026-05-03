# Data Model: Session & Interaction Model Unification

## ActiveSessionRecord

Represents the persisted workspace-scoped interaction state for one bounded Boundline task.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `session_id` | UUID-like string | Yes | Stable identifier reused across session-backed planning and execution requests |
| `workspace_ref` | Path-like string | Yes | Canonical workspace anchor for the active session |
| `goal` | Non-empty string or null | No | Present after capture or any command that scopes the current task |
| `active_task` | `TaskSnapshot` or null | No | Persisted bounded task state required for `plan`, `step`, and `run` continuity |
| `latest_status` | `SessionStatus` | Yes | Current session lifecycle state |
| `latest_terminal_reason` | `TerminalReason` or null | No | Present when the latest execution transition ended in a terminal or recovery-worthy state |
| `latest_trace_ref` | Path-like string or null | No | Last known persisted trace file for the active task |
| `created_at` | Unix timestamp millis | Yes | First creation time for the active session |
| `updated_at` | Unix timestamp millis | Yes | Last successful write time for the active session |

### Validation Rules

- `session_id` must not be empty.
- `workspace_ref` must not be empty.
- `goal` must be present before `plan`, `step`, or `run` can proceed.
- `active_task` must be present before `step` or session-backed `run` can continue from an existing plan.
- `latest_trace_ref`, when present, must point to a trace path inside the active workspace.
- `updated_at` must be greater than or equal to `created_at`.

### Relationships

- One `ActiveSessionRecord` belongs to exactly one workspace.
- One `ActiveSessionRecord` may contain zero or one `TaskSnapshot`.
- One `ActiveSessionRecord` may reference zero or one latest trace file.

### State Transitions

`missing` -> `initialized`

`initialized` -> `goal-captured`

`goal-captured` -> `planned`

`planned` -> `running`

`running` -> `planned`

`running` -> `succeeded | failed | exhausted | aborted`

Any state -> `invalid`

## TaskSnapshot

Represents the persisted in-progress task state carried across CLI invocations.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `task_id` | UUID-like string | Yes | Stable identifier for the bounded task instance |
| `goal` | Non-empty string | Yes | Bound delivery goal for the current task |
| `input` | Structured value | Yes | Initial request input used to create the task |
| `context` | `TaskContext` | Yes | Current shared execution context |
| `plan` | `Plan` | Yes | Current mutable plan snapshot, including revision and current step index |
| `status` | `TaskStatus` | Yes | Latest task execution status |
| `limits` | `RunLimits` | Yes | Explicit execution limits carried from task creation |
| `terminal_reason` | `TerminalReason` or null | No | Present when the task last reached a terminal state |
| `retry_count` | Integer | Yes | Number of retries consumed so far |
| `replan_count` | Integer | Yes | Number of replans consumed so far |
| `total_step_attempts` | Integer | Yes | Total attempts consumed so far across all steps |

### Validation Rules

- `goal` must not be empty.
- `context.workspace_ref` must equal the parent session `workspace_ref`.
- `plan` must validate under the existing `Plan` rules.
- `status` must remain compatible with `terminal_reason` presence.
- `total_step_attempts` must not be less than `retry_count` or `replan_count`.

### Relationships

- Embedded inside one `ActiveSessionRecord`.
- Supplies state for `SessionTransition` and `SessionStatusView`.

### State Transitions

`planned` -> `running`

`running` -> `running` after a non-terminal successful step

`running` -> `running` after retry or replan scheduling

`running` -> `succeeded | failed | exhausted | aborted`

## SessionStatus

Represents the user-facing lifecycle state of the active session.

### Values

| Value | Meaning |
|-------|---------|
| `initialized` | Session exists but no goal is captured yet |
| `goal-captured` | Goal exists but no active plan has been created yet |
| `planned` | A plan exists and is ready for explicit execution |
| `running` | Execution is in progress or resumable from a non-terminal state |
| `succeeded` | Latest task completed successfully |
| `failed` | Latest task ended in a failed terminal state |
| `exhausted` | Latest task ended after consuming retry, replan, or step limits |
| `aborted` | Latest task stopped for an explicit non-success reason |
| `invalid` | Session exists but cannot be trusted without recovery |

### Validation Rules

- `initialized` must not contain `active_task`.
- `goal-captured` must contain `goal` and must not require a terminal reason.
- `planned` must contain `goal` and `active_task` with an active plan.
- Terminal statuses must carry `latest_terminal_reason`.

## SessionTransition

Represents one persisted change applied to the active session after a command or execution event.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `trigger_command` | Enum | Yes | `start`, `capture`, `plan`, `step`, `run`, `status`, `next`, or `inspect` when session state is updated from inspection evidence |
| `from_status` | `SessionStatus` or null | No | Null only when the session is created for the first time |
| `to_status` | `SessionStatus` | Yes | New persisted state |
| `trace_ref` | Path-like string or null | No | Updated latest trace reference when the transition emitted or reused one |
| `reason` | String | Yes | Human-readable explanation of why the transition occurred |

### Validation Rules

- `to_status` must stay consistent with the embedded `ActiveSessionRecord`.
- `trace_ref`, when present, must match the session's `latest_trace_ref` after the transition.

## SessionStatusView

Represents the user-visible summary returned by session-aware `status` and `next` commands.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `session_id` | String | Yes | Current active session identifier |
| `workspace_ref` | String | Yes | Workspace currently targeted |
| `goal` | String or null | No | Present when the task is already scoped |
| `plan_revision` | Integer or null | No | Present when an active task exists |
| `current_step_id` | String or null | No | Present when an executable step remains |
| `current_step_index` | Integer or null | No | Present when an active task exists |
| `latest_status` | `SessionStatus` | Yes | Current session state |
| `latest_trace_ref` | String or null | No | Latest trace available for inspection |
| `next_command` | String or null | No | Recommended follow-up command when determinable |
| `explanation` | String | Yes | Concise user-facing explanation for the current state or recommendation |

### Validation Rules

- `next_command` must be one valid follow-up for the current session state.
- `current_step_id` and `current_step_index` must agree with the embedded plan when present.
- `explanation` must never be empty.

*** Add File: /Users/rt/workspace/boundline/specs/004-session-model-unification/contracts/session-command-contract.md
# Contract: Session Command Surface

## Purpose

Defines the required behavior, inputs, and user-visible outputs of the session-backed Boundline CLI commands introduced by the session model feature.

## Command Set

| Command | Purpose |
|---------|---------|
| `boundline start` | Establish a new active session for the current workspace |
| `boundline capture` | Store or replace the current bounded goal in the active session |
| `boundline plan` | Create an executable plan from the active session goal |
| `boundline step` | Execute exactly one next step from the active session |
| `boundline run` | Continue execution until the task reaches a terminal state |
| `boundline status` | Summarize the active session state |
| `boundline next` | Recommend exactly one next valid command for the active session |

## Required Behavioral Rules

- Every session-backed command MUST resolve the active session automatically from the current workspace or an explicit workspace override when one is provided.
- `start` MUST create or replace the active session only through an explicit user action.
- `capture` MUST fail clearly when no active session exists.
- `plan` MUST fail clearly when the active session has no goal.
- `step` MUST execute at most one executable step and MUST persist updated session and trace state before returning.
- `run` MUST reuse the current active task snapshot when one exists; otherwise it may initialize execution from the active goal and a freshly created plan.
- `status` MUST surface goal, current execution position when available, latest status, and latest trace reference.
- `next` MUST return exactly one valid follow-up command and a short explanation.

## Required Non-Success Handling

| Situation | Required Result |
|-----------|-----------------|
| No active session | Explicit message telling the user to run `boundline start` |
| Session exists without goal | Explicit message telling the user to use `boundline capture` |
| Session exists without plan | Explicit message telling the user to use `boundline plan` |
| Session is corrupted or unreadable | Explicit recovery message and no hidden fallback |
| Latest trace reference is missing | Status or next output must surface the mismatch and guide the user to recover deliberately |
| Terminal session receives more execution commands | Command must fail clearly or require explicit reset rather than silently continuing |

## Output Guarantees

- `status` and `next` outputs MUST be readable without opening raw JSON files.
- Any command that updates the latest trace MUST surface the new trace reference.
- Any command that reaches a terminal outcome MUST surface the terminal reason.
- Any command that preserves a non-success recovery state MUST make the next recommended action explicit.

*** Add File: /Users/rt/workspace/boundline/specs/004-session-model-unification/contracts/session-record-contract.md
# Contract: Session Record Surface

## Purpose

Defines the persisted shape and behavioral guarantees of the workspace-scoped session record stored for active Boundline work.

## Storage Location

- The active session record MUST live at `<workspace>/.boundline/session.json`.
- The file MUST be human-readable JSON.
- The session record MUST remain local to the workspace and MUST NOT require an external service.

## Required Fields

| Field | Requirement |
|-------|-------------|
| `session_id` | MUST identify the active session uniquely |
| `workspace_ref` | MUST identify the workspace the session belongs to |
| `goal` | MUST be present after goal capture and before planning or execution |
| `active_task` | MUST be present whenever stepwise or resumable execution is possible |
| `latest_status` | MUST represent the current lifecycle state of the session |
| `latest_trace_ref` | MUST be present after any execution that emits a persisted trace |
| `updated_at` | MUST change whenever the session record is persisted after a meaningful transition |

## Behavioral Guarantees

- The session record MUST be the authoritative persisted interaction state for the current workspace.
- The session record MUST keep enough state to continue execution without reconstructing task state from raw traces.
- The session record MUST remain consistent with the latest persisted trace reference when one is present.
- The session record MUST remain readable even after non-success execution outcomes.
- The session record MUST reject or clearly surface malformed, stale, or workspace-mismatched content rather than silently repairing it.

## Update Guarantees

- `start` MUST initialize a fresh session record.
- `capture` MUST update goal-related fields without dropping unrelated valid session state.
- `plan` MUST write a new active task snapshot and reset execution position.
- `step` and `run` MUST persist updated task, status, and trace fields before returning control to the user.
- Terminal execution MUST preserve the latest task outcome until the user explicitly starts fresh or replaces the active goal.

*** Add File: /Users/rt/workspace/boundline/specs/004-session-model-unification/contracts/assistant-session-continuity-contract.md
# Contract: Assistant Session Continuity

## Purpose

Defines how assistant command packs must reuse and respect the active Boundline session introduced by the unified session model.

## Required Assistant Behavior

- Assistant commands MUST prefer the active Boundline session over asking the user to restate goal, workspace, or latest trace information that Boundline already preserves.
- Assistant commands MUST route through session-backed CLI commands when they need current status or next-step guidance.
- Assistant commands MAY still use explicit trace inspection when the user asks about a specific historical run instead of the active session.

## Command Alignment Rules

| Assistant Command | Preferred Session-Backed CLI Surface |
|-------------------|--------------------------------------|
| `/boundline-start` | `boundline start` |
| `/boundline-plan` | `boundline capture` followed by `boundline plan` when a goal must be established first |
| `/boundline-step` | `boundline step` |
| `/boundline-run` | `boundline run` |
| `/boundline-status` | `boundline status` |
| `/boundline-next` | `boundline next` |
| `/boundline-inspect` | `boundline inspect` |

## Continuity Guarantees

- If the active session contains a valid goal, assistant commands MUST NOT ask for that goal again unless the user explicitly changes it.
- If the active session contains a latest trace reference, assistant commands MUST reuse it before requesting manual trace lookup.
- If the active session is invalid or missing, assistant commands MUST say so explicitly and route the user to `boundline start` or another concrete recovery action.
- Assistant commands MUST preserve the rule that exactly one next command is recommended at a time.

## Non-Success Handling

- When the session is corrupted, stale, or workspace-mismatched, assistant commands MUST surface the session problem rather than inventing context.
- When execution reaches a terminal state, assistant commands MUST treat the session as complete and route the user to inspect, restart, or replace the goal explicitly.
- When the user refers to an explicit prior trace, assistant continuity MUST not overwrite the active session silently.

*** Add File: /Users/rt/workspace/boundline/specs/004-session-model-unification/quickstart.md
# Quickstart: Session & Interaction Model Unification

## Prerequisites

1. Work from the repository root on branch `004-session-model-unification`.
2. Have Rust 1.95.0 with `cargo` available.
3. Use a writable workspace so Boundline can persist both `.boundline/session.json` and `.boundline/traces/`.
4. Start from the workspace you want the active session to belong to.

## Session-Backed CLI Walkthrough

### 1. Start a new session

Run:

```bash
cargo run --bin boundline -- start
```

Expected outcome:

- Boundline creates `.boundline/session.json` in the current workspace.
- The session becomes the active interaction state for later commands.
- No goal or task plan is required yet.

### 2. Capture a bounded goal

Run:

```bash
cargo run --bin boundline -- capture --goal "Summarize the current bounded developer flow"
```

Expected outcome:

- Boundline stores the goal in the active session.
- Later planning and execution commands no longer require the goal to be re-entered.

### 3. Create a plan

Run:

```bash
cargo run --bin boundline -- plan
```

Expected outcome:

- Boundline creates an executable plan for the active session goal.
- The active session now includes a persisted task snapshot with current execution position at the first executable step.

### 4. Execute one step at a time

Run:

```bash
cargo run --bin boundline -- step
```

Expected outcome:

- Boundline executes exactly one next step.
- The active session updates task context, plan position, latest status, and latest trace reference.
- If the step fails, retries, or triggers replanning, the active session preserves the latest actionable state.

### 5. Continue to a terminal outcome

Run:

```bash
cargo run --bin boundline -- run
```

Expected outcome:

- Boundline resumes from the active session task snapshot and continues until success, failure, exhaustion, or abort.
- The session record captures the final state and latest trace reference.

### 6. Inspect status and the next action

Run:

```bash
cargo run --bin boundline -- status
cargo run --bin boundline -- next
```

Expected outcome:

- `status` reports the active goal, current step position when present, latest status, and latest trace reference.
- `next` returns exactly one recommended follow-up command with a short explanation.

### 7. Inspect the detailed trace

Run:

```bash
cargo run --bin boundline -- inspect
```

Expected outcome:

- Boundline uses the active session's latest trace reference when available.
- The output reconstructs step progression, recovery events, and terminal reason.

## Assistant Walkthrough

1. Start from an assistant command that maps to the active session flow, such as `/boundline-start`.
2. Let the assistant establish or reuse the active session.
3. Use `/boundline-plan`, `/boundline-step`, `/boundline-run`, `/boundline-status`, or `/boundline-next` without restating already preserved session context.
4. Use `/boundline-inspect` only when the active session or an explicit prior trace needs detailed inspection.

## Recovery Scenarios

### Missing session

If a session-backed command is invoked before `start`, expected output should tell the user to establish an active session first.

### Missing goal

If `plan`, `step`, or `run` is invoked before goal capture, expected output should route the user to `capture`.

### Corrupted or stale session

If `.boundline/session.json` is unreadable, workspace-mismatched, or points at a missing trace, expected output should surface the exact problem and avoid hidden continuation.

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