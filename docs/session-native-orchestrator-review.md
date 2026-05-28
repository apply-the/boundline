# Session-Native Orchestrator Review

## Thesis

Boundline should feel like a session-native orchestrator that takes a user from
bootstrap to working code through one primary loop:

`init -> goal -> plan -> run -> status -> inspect`

`init` remains bootstrap. `goal -> plan -> run` is the product story. Canon
remains downstream as governance and artifact control, not the place where
orchestration logic lives.

This review keeps only the parts of the older architecture critique that still
matter after the session-native and assistant follow-through work already
landed.

## What Already Exists

The current codebase already has the backbone of a real orchestrator.

### Session Harness

- [src/cli/session.rs](../src/cli/session.rs) exposes `goal`, `plan`, `step`,
  `run`, `status`, `next`, and `inspect` as resumable session commands after
  bootstrap.
- [src/orchestrator/session_runtime.rs](../src/orchestrator/session_runtime.rs)
  turns those commands into workspace-scoped state transitions, execution
  resumption, and operator-facing control.

### Execution Harness

- [src/orchestrator/engine.rs](../src/orchestrator/engine.rs) already provides
  the bounded loop: execute, normalize, retry, replan, terminalize, persist
  trace, and enforce stage governance.

### Adapter And Tool Harness

- [src/registry/agent_registry.rs](../src/registry/agent_registry.rs) and
  [src/registry/tool_registry.rs](../src/registry/tool_registry.rs) resolve
  named endpoints.
- [src/adapters/agent.rs](../src/adapters/agent.rs) and
  [src/adapters/tool.rs](../src/adapters/tool.rs) turn them into executable
  runtime adapters.

### Persistence And Inspection Harness

- [src/adapters/session_store.rs](../src/adapters/session_store.rs) persists
  active session state.
- [src/adapters/trace_store.rs](../src/adapters/trace_store.rs) persists
  execution traces for inspection and recovery.

### Governance Boundary

- [src/adapters/governance_runtime.rs](../src/adapters/governance_runtime.rs)
  keeps governance behind a Boundline-owned runtime boundary instead of moving
  orchestration into Canon.

The architectural problem is no longer "there is no runtime". The problem is
that the dominant input and decision model still reflects too much of the
compatibility fixture path.

## What Should Stay

Keep these invariants:

- the session-native command surface remains the primary product spine
- workspace-scoped session state and traces remain the inspectability backbone
- the bounded engine loop remains responsible for retries, replans, and
  terminal states
- governance remains a stage overlay: Boundline orchestrates and Canon governs
- assistant-facing follow-through remains a projection over the same session and
  trace authority, not a second reporting runtime

These are the parts that already make Boundline feel like orchestration rather
than prompt choreography.

## What Is Still Mismatched

### Bootstrap Still Shapes Too Much Of The Product Story

`init` and templates are still too visible as the operator mental model.
Bootstrap is useful, but it should stay setup, not the center of the runtime
story.

### Planning Still Feels Too Fixture-First

The main planning path still inherits too much from the declarative execution
profile model. The default product path should derive bounded work from goal,
workspace evidence, authored docs, and Canon artifacts rather than from a
pre-authored profile contract.

### Flow Selection Is Still Too Rigid

Static flow identity still matters, but it should be a bounded policy surface,
not a script the operator has to pre-author or micro-manage. The safe model is:
infer, show, confirm lightly.

### Next-Action Selection Is Not Explicit Enough

The central runtime question is still: what should happen next? Today too much
of that answer is implicit in fixture-shaped step synthesis, adapter-local
behavior, or flow-specific logic.

### Decision Shape Is Not First-Class Enough

Boundline needs an explicit inspectable decision object rather than hiding
important control choices inside local planner or adapter behavior.

### Tool Use Is Still Underdescribed In The Runtime Model

Agents matter, but tool use is what makes the loop real: read files, patch
files, run tests, inspect diffs, and interpret command output. The runtime
model should say that directly.

### Canon Artifacts Are Still Too Peripheral To Planning

Canon integration exists, but governed artifacts and project memory should be
bounded planning inputs rather than only stage-boundary side effects. Canon
must still stay out of per-action orchestration.

## Refactor Target

The next architectural slice should move the product farther away from
fixture-first execution without throwing away the bounded runtime that already
works.

### What Should Leave `fixture.rs` As The Dominant Product Path

- initial task shaping as the default user story
- runtime assembly as the main session execution surface
- flow-specific step synthesis as the default next-action model
- manifest-declared attempts as the normal execution contract

`fixture.rs` should remain a compatibility layer and helper surface, not the
place where the product decides the next move.

### What Should Stay In `engine.rs`

- bounded execution discipline
- generic dispatch for decisions, agents, and tools
- retry, replan, and terminalization semantics
- trace emission and recovery evidence
- governance enforcement as overlay

### What Should Stay In `session_runtime.rs`

- workspace-scoped session lifecycle
- operator-facing control flow for goal, plan, run, status, next, and inspect
- persistence and reload of active session state
- confirmation points for clarification, governance pauses, and inferred flow

### What Must Be Introduced

Boundline needs a first-class bounded decision object that is:

- selected from current workspace and trace evidence
- constrained by flow and stage policy
- persisted into session and trace state
- inspectable as rationale, target, expected outcome, and evidence inputs

Without this primitive, the refactor risks moving code around without changing
the actual behavior model.

## How Flow Should Work

Flow should become a policy surface over the decision loop:

- flow selects allowed decision families and stage boundaries
- the runtime chooses the next bounded decision inside those constraints
- stage transitions happen only after verifiable outcomes
- explicit override remains operator-controlled

That keeps flow useful without turning it back into a script.

## What The Next Spec Should Ship

One coherent slice should deliver all of this together:

1. A user can bootstrap with `init` and then begin from `goal` without treating
   compatibility-manifest authoring as the main entry point.
2. `plan` derives an initial bounded task draft from goal, workspace state,
   authored docs, and available Canon artifacts.
3. Boundline proposes an inferred flow and asks for lightweight confirmation
   unless the user already made an explicit choice.
4. `run` owns bounded next-action selection from live state, validation
   evidence, and accumulated context.
5. Each next action is represented as an explicit inspectable decision object.
6. The runtime loop becomes explicitly observe -> decide -> act -> verify ->
   update-context.
7. Flow constrains allowed decisions and stage boundaries instead of acting as
   a rigid script.
8. The loop is explicitly tool-driven through file reads, writes, diffs, tests,
   and command execution.
9. Governance remains a Boundline-owned stage overlay backed by Canon artifacts
   and approvals, with Canon invoked at stage boundaries rather than per action.

Guardrails for that slice:

- bounded planning, not a generic black-box planner
- explicit decision objects, not hidden adapter-local heuristics
- inferred flow with lightweight confirmation, not silent auto-run
- tool-driven execution, not text-only orchestration
- Canon at stage boundaries, not per action
- Boundline orchestrates and Canon governs

## Bottom Line

The current codebase already contains the right backbone:

- real executable harness
- session-native loop
- persisted session and trace state
- bounded engine
- governance overlay

What is still wrong is the dominant input and decision model.

The next step is not just better planning language. The next step is an
explicit decision model that makes the runtime easier to trust, debug, and keep
consistent across adapters.