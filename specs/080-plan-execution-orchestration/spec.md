# Feature Specification: Plan Execution Orchestration

**Feature Branch**: `080-plan-execution-orchestration`

**Created**: 2026-06-14

**Status**: Draft

**Input**: User description: "Add a runtime-owned execution control plane to Boundline so accepted plans and validated backlogs can run as an inspectable sequence of bounded tasks with checkpointing, pause and resume, and explicit blocked-state handling." Original spec at ./080-plan-execution-orchestration/spec-plan-execution-orchestration.md.

## Clarifications

### Session 2026-06-14

- Q: What CLI surface triggers execution? → A: Extend `boundline run` with `--plan <plan-ref>`, `--accepted-plan`, and `--resume <execution-run-id>`. Avoid `--orchestrate`.
- Q: Where are checkpoints persisted? → A: `.boundline/execution/checkpoints/<run-id>.json` — atomically replaced, one canonical file per run.
- Q: Is spec 079 a hard dependency? → A: Yes, hard prerequisite. When verification is unavailable, execution pauses with a blocked projection identifying the missing dependency.
- Q: How are task dependencies declared? → A: Explicit `depends_on: [task-id, ...]` per task. Topological sort with deterministic tie-breaking. Cycles block before execution.
- Q: How does the orchestrator relate to session state? → A: Distinct `execution_run_id` with checkpoint-owned state. `ActiveSessionRecord` stores minimal linkage (`active_execution_run_id`). `SessionStatusView` projects execution fields additively by reading the linked checkpoint.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Execute Accepted Plans One Task At A Time (Priority: P1)

An operator has an accepted plan or validated backlog ready for execution. They invoke `boundline run --plan <ref>` or `boundline run --accepted-plan`, and Boundline advances one task at a time in dependency order, checkpointing after each verified outcome. They can inspect progress at any point and see exactly which task is running, which are blocked, and which have completed. A paused or interrupted run is resumed with `boundline run --resume <execution-run-id>`. Invoking `boundline run` without a plan option preserves existing single-task behavior.

**Why this priority**: This is the minimum viable orchestration slice. Without sequential, checkpointed task execution, multi-task plans cannot be trusted to run to completion without losing state or repeating work.

**Independent Test**: Load a validated three-task plan, start execution, and verify that tasks execute in dependency order with checkpoints persisted after each completed task. Inspect the run midway and confirm the current task, completed count, and blocked tasks are projected. Resume any interrupted run from the last checkpoint without re-executing completed tasks.

**Acceptance Scenarios**:

1. **Given** a validated plan with three tasks in dependency order, **When** execution starts, **Then** Boundline selects the first runnable task, locks its mutation surface, and dispatches execution.
2. **Given** a task has completed with passing validation and completion-verification proof, **When** the checkpoint is written, **Then** the next runnable task is selected and execution continues without re-executing completed work.
3. **Given** a run is interrupted mid-execution, **When** the operator runs `boundline run --resume <execution-run-id>`, **Then** Boundline reloads the last checkpoint and continues from the next uncompleted task.
4. **Given** an already active execution plan, **When** `boundline run --plan <ref>` is invoked again, **Then** Boundline returns the current execution state rather than creating a duplicate run.
5. **Given** an invalid, unaccepted, or cyclic plan reference, **When** `boundline run --plan <ref>` is invoked, **Then** Boundline blocks before any task execution and projects the validation failure.

---

### User Story 2 - Handle Blocked Tasks Without Losing Progress (Priority: P2)

A task becomes blocked — a finding requires resolution, proof is missing, or governance approval is pending. Boundline halts downstream execution, persists the blocked state, and projects the exact reason and resume command through status and inspect surfaces. No completed work is lost, and the operator can resolve the blocking condition and resume from the same checkpoint.

**Why this priority**: Multi-task plans inevitably encounter blocking conditions. Without explicit blocked-state handling, operators lose context, and runs silently fail or require full restarts.

**Independent Test**: Execute a plan where the second task hits a blocking finding. Verify that the first task's checkpoint is preserved, the blocked task is projected with its stop reason, downstream tasks remain unstarted, and the resume command restores the exact blocked state.

**Acceptance Scenarios**:

1. **Given** a task is blocked by a missing completion-verification proof, **When** the block is detected, **Then** downstream tasks are halted, the blocked task is projected in status with its stop reason, and the run state is checkpointed.
2. **Given** a run is paused with a blocked task, **When** the operator resolves the block and resumes, **Then** execution continues from the blocked task without re-executing completed work.
3. **Given** a task is blocked and the operator inspects the run, **When** status is rendered, **Then** the output shows `execution_plan_state: blocked`, the blocked task ID, and the exact resume command.

---

### User Story 3 - Surface Execution State In Runtime Output (Priority: P3)

An operator, reviewer, or downstream governance consumer inspects Boundline status, orchestrate snapshots, or assistant-rendered output during execution. They can see the execution plan state, current task, completed count, blocked tasks, checkpoint references, and the exact resume command — all as additive fields that do not break existing runtime consumers.

**Why this priority**: Operational visibility is essential, but the core execution engine must work first. This story ensures the execution state is visible where operators already look for progress.

**Independent Test**: Execute a multi-task plan and inspect status, orchestrate, and assistant output at each state transition. Verify that `execution_plan_state`, `execution_current_task_id`, `execution_completed_task_count`, `execution_blocked_task_ids`, `execution_checkpoint_ref`, and `execution_resume_command` are present and accurate without breaking existing status consumers.

**Acceptance Scenarios**:

1. **Given** a run is executing, **When** status is rendered, **Then** the output includes `execution_plan_state: running`, `execution_current_task_id`, and `execution_completed_task_count`.
2. **Given** a run has blocked tasks, **When** status is rendered, **Then** `execution_blocked_task_ids` lists the blocked tasks and the output does not use completion language.
3. **Given** a run is paused, **When** status is rendered, **Then** `execution_resume_command` shows the exact resume route and `execution_checkpoint_ref` shows the last durable checkpoint.
4. **Given** all tasks have completed, **When** status is rendered, **Then** `execution_plan_state: completed` and existing status consumers continue to work unchanged.

---

### Edge Cases

- What happens when the task registry is empty? The execution plan state transitions immediately to `completed` with zero completed tasks.
- What happens when a task dependency is circular? The execution engine must detect circular dependencies at plan-load time and refuse to start, projecting the cycle as a `dependency_cycle` finding with the involved task IDs.
- What happens when a checkpoint write fails? The run must pause with an explicit filesystem-error finding and preserve the in-memory state for operator recovery. The previous valid checkpoint must be preserved.
- What happens when a previously completed task's proof becomes stale before the full plan completes? Each task's completion-verification proof is independently checkpointed; downstream tasks do not invalidate upstream proofs.
- What happens when the mutation surface of two adjacent tasks overlaps? The second task is blocked until the first task's mutation surface is released at checkpoint.
- What happens when completion verification is unavailable? Execution pauses before task completion with a `verification_pending` state and a blocked projection identifying `completion_verification_unavailable` as the blocking dependency.
- What happens when the operator invokes `boundline run --plan <ref>` for an already active run? Boundline returns the current execution state rather than creating a duplicate run.
- What happens when a `depends_on` task is skipped or deferred? The downstream task evaluates the dependency outcome; skipped tasks may unblock downstream execution, deferred tasks block until resolved.
- What happens when the checkpoint file is unreadable or missing? Status projection returns a degraded or blocked state rather than silently clearing the session linkage.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST support loading an accepted plan or validated backlog as a task registry in dependency order.
- **FR-002**: Boundline MUST select one runnable task at a time and lock its mutation surface to prevent overlapping concurrent execution.
- **FR-003**: Boundline MUST require task-local validation and completion-verification proof before marking a task complete.
- **FR-004**: Boundline MUST checkpoint state after each completed, blocked, skipped, or deferred task to a single durable checkpoint format.
- **FR-005**: Boundline MUST resume from the last explicit checkpoint without re-executing completed tasks or recomputing state from scratch.
- **FR-006**: Boundline MUST halt downstream execution when a task is blocked and project the blocked task ID, stop reason, and resume command.
- **FR-007**: Boundline MUST expose `execution_plan_state`, `execution_current_task_id`, `execution_completed_task_count`, `execution_blocked_task_ids`, `execution_checkpoint_ref`, and `execution_resume_command` as additive projection fields in status, orchestrate, and inspect output.
- **FR-008**: Boundline MUST detect circular task dependencies at plan-load time and refuse execution with a blocking finding.
- **FR-009**: Boundline MUST NOT allow overlapping mutation surfaces to execute concurrently; adjacent tasks with shared mutation targets must serialize.
- **FR-010**: The first slice MUST support one sequential runner only, with no parallel task execution, no autonomous replanning, and no implicit task creation.
- **FR-012**: Checkpoints MUST be written atomically (temp file → flush → rename) with a monotonically increasing `checkpoint_sequence`; the previous valid checkpoint MUST be preserved if the new write fails.
- **FR-013**: Resume MUST reject a checkpoint whose plan identity, plan fingerprint, or workspace binding is incompatible with the current execution context.

- **FR-014**: `ActiveSessionRecord` MUST store only minimal execution linkage (`active_execution_run_id`); the authoritative execution state MUST reside in the checkpoint file.
- **FR-015**: `SessionStatusView`, `status`, `next`, and `inspect` MUST project execution fields additively by reading the linked checkpoint without redefining existing session states.
- **FR-016**: Starting the same accepted plan again MUST return or resume the existing non-terminal execution run rather than creating a duplicate.
- **FR-017**: Resume MUST validate that the session, workspace, accepted plan, and checkpoint all refer to the same execution context before allowing continuation.
- **FR-018**: Each task MUST declare direct dependencies through an explicit `depends_on` list; an empty list means no task-level prerequisite.
- **FR-019**: The task dependency graph MUST be validated before execution: cycles, self-dependencies, missing references, and duplicates are blocking findings.
- **FR-020**: A task becomes runnable ONLY when all its `depends_on` tasks have reached an acceptable terminal state; task order in the plan is the deterministic tie-breaker.

### Key Entities

- **Execution Run**: A distinct `execution_run_id` (e.g., `ER-20260614-abc123`) linked to an active session for the duration of plan execution.
- **Accepted Plan**: A validated plan or backlog with explicit per-task `depends_on` declarations, loaded as the task registry.
- **Execution Checkpoint**: A durable canonical JSON file at `.boundline/execution/checkpoints/<run-id>.json` containing run identity, plan fingerprint, checkpoint sequence, completed/blocked/skipped task IDs, evidence refs, and resume state. Written atomically (temp → flush → rename).
- **Execution Plan State**: An enumerated lifecycle state (`ready`, `running`, `paused`, `blocked`, `completed`) reflecting the overall orchestration status.
- **Mutation Surface Lock**: A per-task filesystem scope lock that prevents overlapping writes across concurrent or adjacent tasks.
- **Session Linkage**: A minimal `active_execution_run_id` field in `ActiveSessionRecord` that enables `SessionStatusView` to project execution fields without duplicating state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can execute a validated three-task plan from start to finish in dependency order, with each task checkpointed and independently verifiable.
- **SC-002**: A run interrupted mid-execution resumes from the last checkpoint without re-executing any completed task, validated by comparing task IDs and checkpoint refs.
- **SC-003**: A blocked task halts downstream execution within one orchestration cycle, and the blocked task ID with stop reason is visible in status output within the same cycle.
- **SC-004**: Status and inspect output includes all six execution projection fields (`execution_plan_state`, `execution_current_task_id`, `execution_completed_task_count`, `execution_blocked_task_ids`, `execution_checkpoint_ref`, `execution_resume_command`) without breaking any existing status consumer contract.
- **SC-005**: Circular task dependencies are detected at plan-load time and reported as a blocking finding before any task execution begins.
