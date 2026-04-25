# Feature Specification: Test-Fix Loop Vertical Slice Demo

**Feature Branch**: `005-test-fix-loop-demo`  
**Created**: 2026-04-25  
**Status**: Draft  
**Input**: User description: "Vertical slice demo: orchestrator drives a failing test to passing through analyzer, coder, tester with retry and replan"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run the demo and watch a failing test become a passing test (Priority: P1)

A developer wants to confirm that Synod can take a real failing test in a real workspace
and drive it to a passing state using its existing orchestrator (analyzer → coder →
tester). The developer runs a single command, watches the orchestrator step through the
plan, and at the end sees the previously failing test pass and the bug fixed in the
workspace file.

**Why this priority**: This is the proof of execution that justifies every other Synod
feature. Without an observed end-to-end "broken → fixed" loop, none of the orchestrator,
session, or assistant work demonstrates delivery value.

**Independent Test**: Can be fully tested by running `synod run-demo`, inspecting the
emitted step-by-step output, and verifying that the demo workspace file ends in the
fixed state and the demo test runner reports a passing run.

**Acceptance Scenarios**:

1. **Given** a clean checkout, **When** the developer runs `synod run-demo`, **Then**
   the command provisions an isolated demo workspace that contains exactly one buggy
   source file and exactly one failing test, executes the analyzer → coder → tester
   plan, and prints the final task status `Succeeded` along with the path to the
   updated source file and the trace file.
2. **Given** the demo has finished successfully, **When** the developer reads the
   updated source file, **Then** the bug introduced in the seeded source is no longer
   present and a re-run of the demo's test runner against the same workspace reports
   that the test passes.

---

### User Story 2 - See at least one retry within a single demo run (Priority: P2)

A developer wants visible evidence that the orchestrator's retry behavior is real, not
hypothetical. During the demo run, the first coder attempt MUST fail in a recoverable
way, the orchestrator MUST retry the same step, and the second attempt MUST succeed and
let the plan continue forward.

**Why this priority**: Retry is the simplest non-success recovery path. If retry does
not visibly happen on a real test-fix loop, the orchestrator's recovery story is not
demonstrated end-to-end.

**Independent Test**: Can be tested by running `synod run-demo` and inspecting the
printed step-by-step output (and the trace file) for at least one entry showing a
recoverable failure on the coder step followed by a successful retry of the same step.

**Acceptance Scenarios**:

1. **Given** the demo plan is executing, **When** the coder step is reached for the
   first time, **Then** the orchestrator records a recoverable failure for that step,
   increments the retry counter, and the next observed event for the same step is a
   successful retry attempt that produced a code change.

---

### User Story 3 - See at least one replan within a single demo run (Priority: P3)

A developer wants visible evidence that the orchestrator's replan behavior is real on
the same demo. During the demo run, the tester step MUST report a failure that the
orchestrator MUST treat as `ReplanRequired`, causing an additional analyzer (debug)
step and an additional coder (fix again) step to be inserted, after which a final
tester step MUST pass.

**Why this priority**: Replan is the second non-success recovery path called out in the
input. Demonstrating it on top of an already-working retry shows the full mutable plan
behavior the orchestrator promises.

**Independent Test**: Can be tested by running `synod run-demo` and inspecting the
printed step-by-step output (and the trace file) for one tester failure event followed
by inserted analyzer and coder steps and a final tester success.

**Acceptance Scenarios**:

1. **Given** the demo plan has reached the first tester step, **When** that tester step
   reports a `ReplanRequired` failure, **Then** the orchestrator inserts one analyzer
   step and one coder step into the plan and re-runs the tester step, and the
   re-executed tester step reports success.

---

### Edge Cases

- **Retry limit reached**: If the coder step fails recoverably more times than the
  configured retry limit, the demo MUST stop in a `Failed` terminal state, MUST NOT
  silently mark the run as success, and MUST surface the retry-exhaustion reason in
  the printed output and the trace file.
- **Replan exhaustion**: If the orchestrator already replanned the maximum number of
  times allowed for the demo and the tester still fails with `ReplanRequired`, the
  demo MUST stop in a `Failed` terminal state and MUST identify replan-exhaustion as
  the stop reason.
- **Demo workspace already exists**: If a previous demo workspace from an earlier run
  is still present on disk, the demo MUST reset it to the seeded buggy state before
  executing the plan; it MUST NOT continue against an already-fixed workspace and
  falsely report success.
- **Trace file not writable**: If the trace file location is not writable, the demo
  MUST stop before executing the plan and MUST report the unwritable trace path; it
  MUST NOT execute the plan without a trace.
- **Test runner unavailable**: If the test-runner step cannot execute (for example,
  the simulated runner is misconfigured), the demo MUST stop in a `Failed` terminal
  state with that cause recorded in the trace, instead of silently skipping the
  verification.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST expose a single CLI command (`synod run-demo`) that
  performs the full demo end-to-end without requiring additional flags or prior setup.
- **FR-002**: The system MUST create or reset an isolated demo workspace that
  contains exactly one source file with a known bug and exactly one failing test
  before plan execution begins.
- **FR-003**: The system MUST execute, in order, the hardcoded plan
  `analyzer → coder → tester` against the demo workspace using the existing
  orchestrator, agents, and tool registries.
- **FR-004**: The system MUST cause the first coder attempt to fail recoverably and
  the second attempt to succeed, so that retry behavior is exercised on every demo run.
- **FR-005**: The system MUST cause the first tester attempt (after the successful
  retry) to fail with `ReplanRequired`, triggering insertion of an additional
  analyzer step and an additional coder step, and the subsequent tester step MUST
  succeed.
- **FR-006**: The system MUST apply the demo's "fix" as a real modification to the
  seeded source file in the demo workspace before the final tester step runs, so
  that re-running the demo's test runner against the file reports a passing test.
- **FR-007**: The system MUST stop the demo when the plan reaches a terminal state:
  either `Succeeded` after the final tester step passes, or `Failed` when the
  configured retry or replan limit is reached.
- **FR-008**: The system MUST emit step-by-step execution output to the developer's
  console for every step in the order it executes, including the kind of step
  (analyzer/coder/tester), its outcome (success / recoverable failure / replan
  required / terminal failure), and the attempt number.
- **FR-009**: The system MUST persist a trace of the demo run that records every
  step, every recovery decision, every plan mutation, and the final terminal state,
  and MUST print the trace file path on completion.
- **FR-010**: The system MUST not introduce any new framework, plugin system,
  multi-agent voting, model routing, or external Canon integration; it MUST use only
  the orchestrator, planner, agent registry, tool registry, session model, and CLI
  surfaces that already exist in the codebase.

### Scope Boundaries *(mandatory)*

- **In Scope**:
  - A single `synod run-demo` CLI command.
  - One isolated demo workspace with one buggy source file and one failing test.
  - One hardcoded `analyzer → coder → tester` plan with deterministic retry and
    replan behavior driven by existing fake agents and a built-in test runner.
  - One trace file capturing the run.
- **Out of Scope**:
  - Real LLM calls, model routing, or any AI provider integration.
  - Multi-agent voting, council patterns, or alternative agent selection logic.
  - Plugin systems or extensibility surfaces beyond the existing registries.
  - Canon artifact governance, persistent agent histories, or distributed execution.
  - Generic developer-facing demo configuration (no flags, no profiles, no custom
    workspaces in this slice).
  - Performance, scalability, and concurrency work.

### Key Entities

- **Demo workspace**: An on-disk folder owned by the demo containing one seeded
  source file with a known bug and one seeded failing test. Lifecycle: provisioned
  or reset at the start of every `synod run-demo` invocation; left in its post-run
  state on disk for inspection.
- **Demo plan**: The hardcoded ordered sequence of steps executed by the
  orchestrator (`analyzer → coder → tester`) plus the steps inserted by the single
  replan event (`analyzer → coder → tester`). The plan is mutable through the
  orchestrator's existing replan path; it is not user-configurable in this slice.
- **Demo trace**: The persisted record of the run, including every step attempt,
  recovery decision, plan mutation, and the final terminal state. Owned by the
  existing trace store; produced once per `synod run-demo` invocation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can run `synod run-demo` on a clean checkout and observe
  the demo reach the `Succeeded` terminal state in a single invocation, with the
  seeded source file ending in its fixed state on disk.
- **SC-002**: 100% of `synod run-demo` invocations on a clean checkout produce
  step-by-step output that contains at least one recoverable failure event followed
  by a successful retry of the same step.
- **SC-003**: 100% of `synod run-demo` invocations on a clean checkout produce
  step-by-step output that contains at least one `ReplanRequired` failure event
  followed by inserted analyzer and coder steps and a passing final tester step.
- **SC-004**: 100% of `synod run-demo` invocations write a trace file to a
  developer-readable path, print that path to the console on completion, and the
  trace contains a complete record of every step attempt, recovery decision, and
  the final terminal state.
- **SC-005**: A developer can identify, from the trace alone and without re-running
  the demo, the step where retry occurred, the step where replan occurred, and the
  final terminal state in under 5 minutes.

## Assumptions

- The demo runs in a single process on the developer's machine, sequentially, with
  no concurrency, background workers, or external services.
- The "test runner" used by the tester step is an in-process function that reads the
  seeded source file and returns pass/fail based on whether the bug pattern is still
  present (deterministic, no spawning of `cargo test` or external toolchains for
  this slice).
- The demo workspace lives under a stable path owned by the demo (for example
  `<repo>/.synod/demo-workspace/`) and is safe to recreate or reset on every run.
- The retry limit and the replan limit used by the demo come from the existing
  orchestrator default `RunLimits` and are not introduced or re-tuned by this slice.
- The existing analyzer, coder, and tester adapters in `src/demo/endpoints.rs` are
  the agents the demo uses, optionally extended in-place with the minimal logic
  required to read or modify the seeded source file. No new abstraction layer is
  introduced.
- The trace store and session model already in place are sufficient to persist the
  demo run; this slice does not extend either.
