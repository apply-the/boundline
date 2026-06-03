# Boundline Plan Execution Orchestration

## Summary

Add a runtime-owned execution control plane to Boundline so accepted plans and
validated backlogs can run as an inspectable sequence of bounded tasks with
checkpointing, pause and resume, and explicit blocked-state handling. Canon may
consume progress and handoff packets, but Boundline owns task ordering,
mutation-surface control, validation loops, and resume semantics.

This seed is not the same as recursive stage refinement in seed 12. Seed 12 is
about repeated planning rounds within one stage. This seed is about executing an
already accepted multi-task plan over time.

## Speckit Seed Notes

- Seed role: runtime control plane for multi-task execution.
- First slice: one opt-in sequential execution profile that advances one task at
  a time, checkpoints after each verified outcome, and can resume from the last
  explicit checkpoint.
- Depends on: planning-readiness gates, backlog validation, completion
  verification runtime, and existing status/inspect/session surfaces.
- De-duplication: seed 12 owns recursive refinement rounds; seed 07 owns
  provider lifecycle; seed 13 owns sandbox enforcement; Canon owns progress and
  handoff packet semantics when those artifacts are exported.

## Public And Runtime Interface Changes

Add optional execution-orchestration projection fields to session status,
orchestrate snapshots, and rendered output when a run is executing from a task
registry:

- `execution_plan_state`: `ready`, `running`, `paused`, `blocked`, or
  `completed`
- `execution_current_task_id`: the task currently locked for execution
- `execution_completed_task_count`: total completed tasks in the active run
- `execution_blocked_task_ids`: tasks blocked on findings or missing proof
- `execution_checkpoint_ref`: the last durable checkpoint or handoff ref
- `execution_resume_command`: exact resume route when a run is paused or
  interrupted

The fields are additive and must not break existing runtime consumers.

## Runtime Behavior

When execution orchestration is active, Boundline should:

- load the accepted plan and task registry in dependency order
- select one runnable task at a time
- lock the active mutation surface so overlapping tasks cannot run concurrently
- dispatch the bounded execution path for that task using existing runtime,
  provider, or sandbox surfaces as appropriate
- require task-local validation and completion-verification proof before moving
  the task to complete
- checkpoint state after each completed, blocked, skipped, or deferred task
- expose the active task, stop reason, and next action in status and inspect
- resume from the last explicit checkpoint rather than recomputing state from
  chat or inferred diffs

The first slice should stay intentionally narrow:

- one sequential runner only
- no overlapping parallel task execution
- no autonomous replanning
- no implicit task creation
- one checkpoint format and one resume path

## Assistant Asset Updates

Update Boundline run, status, and inspect assets so they:

- preserve `execution_plan_state`, `execution_current_task_id`,
  `execution_blocked_task_ids`, `execution_checkpoint_ref`, and
  `execution_resume_command`
- do not report a run as completed when tasks remain blocked, skipped, or
  deferred without explicit projection
- explain the difference between paused, blocked, and finished
- route interrupted runs to the exact resume command instead of suggesting a
  new ad hoc start

## Tests

Add unit, contract, and integration coverage for:

- sequential execution respects task dependency order
- overlapping mutation surfaces do not run concurrently
- a blocked task halts downstream execution until the operator resolves it
- a verified completed task advances the checkpoint
- paused runs resume from the last checkpoint instead of recomputing progress
- status and inspect surfaces project current task, state, and resume command

Run:

- `cargo test --test unit`
- `cargo test --test contract`
- `cargo test --test integration`

## Canon Boundary

No Canon files are changed as part of this Boundline planning document.

Canon owns:

- progress and handoff packet semantics
- task-state artifact meaning
- evidence-ref schema consumption

Boundline owns:

- runnable-task selection
- task locking
- execution dispatch
- checkpoint persistence
- resume behavior
- completion gating before task closeout

## Assumptions

- The first slice can operate on an already validated backlog or equivalent task
  registry.
- Existing provider and sandbox surfaces can be reused rather than redefined.
- Exporting Canon progress or handoff packets is optional in the first slice;
  Boundline may project equivalent state internally before Canon integration is
  wired.