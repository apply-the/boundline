# Session-Native Orchestrator Review

## Thesis

Synod should feel like a session-native orchestrator that takes a user from
problem intake to working code through one primary loop:

`start -> capture -> plan -> run -> status -> inspect`

`init` may remain as bootstrap, but it should stop being the center of the
product story. Canon should remain downstream from Synod as governance and
artifact control, not the place where orchestration logic lives.

This review is intentionally blunt: it calls out what the current code already
supports, what is misaligned with that thesis, and what the next spec should
treat as refoundation work instead of polish.

## Status Update

Feature `013-session-native-orchestrator` established the session-native planning
and decision-loop primitives, feature `014-native-loop-integration` moved the
real CLI path onto them, and feature `015-runtime-refoundation` completed the
product-story shift by making `GoalPlan`, routing state, flow state, and
decision summaries explicit operator-facing surfaces.

Current behavior is now:

- `plan` persists `GoalPlan` plus confirmed, proposed, or absent flow state in the active session
- `run` prefers the native decision loop whenever a session already has a `goal_plan`
- decision execution is adapter-backed and persisted into both session state and traces
- `fixture.rs` remains an explicit compatibility layer for declarative execution profiles rather than the default session path

The rest of this document remains useful as the architectural review that drove
those changes. Sections that describe fixture-first planning or implicit fixture
defaults should be read as historical rationale unless they are still explicitly
called out as open follow-up gaps.

## Where The Harness Actually Is

This is the part that separates Synod from a framework made of prompt files and
Markdown-defined agents.

The harness is not the Markdown. The harness is the executable control plane
that owns session state, step dispatch, recovery, validation, governance, and
traces.

### 1. User-facing session harness

- [src/cli/session.rs](../src/cli/session.rs) exposes `start`, `capture`, `plan`, `step`, `run`, `status`, `next`, and `inspect` as resumable session commands.
- [src/orchestrator/session_runtime.rs](../src/orchestrator/session_runtime.rs) turns those commands into workspace-scoped state transitions, execution resumption, and operator-facing control.

This is the daily UX harness.

### 2. Core execution harness

- [src/orchestrator/engine.rs](../src/orchestrator/engine.rs) is the bounded loop: execute a step, normalize the result, retry, replan, terminalize, persist trace, and apply stage governance.

This is the part that makes Synod more than prompt choreography.

### 3. Dispatch and adapter harness

- [src/registry/agent_registry.rs](../src/registry/agent_registry.rs) and [src/registry/tool_registry.rs](../src/registry/tool_registry.rs) resolve named endpoints.
- [src/adapters/agent.rs](../src/adapters/agent.rs) and [src/adapters/tool.rs](../src/adapters/tool.rs) turn those endpoints into executable runtime adapters.

Without this layer, "agent" is still just documentation.

### 4. Concrete workspace harness and compatibility layer

- [src/orchestrator/session_runtime.rs](../src/orchestrator/session_runtime.rs) now assembles the native session path around persisted `GoalPlan` state, adapter-backed execution, and persisted decisions.
- [src/fixture.rs](../src/fixture.rs) remains the explicit compatibility layer for declarative execution profiles and low-level workspace mutation or validation helpers.

This is why Synod is already more than a Markdown framework while keeping the
legacy declarative path available without making it the default product story.

### 5. Persistence and inspection harness

- [src/adapters/session_store.rs](../src/adapters/session_store.rs) persists the active session.
- [src/adapters/trace_store.rs](../src/adapters/trace_store.rs) persists execution traces and makes inspection and recovery possible.

This is what keeps the system resumable and inspectable instead of ephemeral.

## What To Keep

### 1. Session-native command surface

Keep the current session loop as the primary product spine.

- [src/cli.rs](../src/cli.rs) already exposes `start`, `capture`, `plan`, `run`, `status`, `next`, and `inspect` as first-class commands.
- [src/cli/session.rs](../src/cli/session.rs) already treats session state as the user-facing control surface.

This is the right direction. The next spec should strengthen it, not replace it.

### 2. Session and trace persistence

Keep workspace-scoped session state and traces as the inspectability backbone.

- [src/orchestrator/session_runtime.rs](../src/orchestrator/session_runtime.rs) persists and resumes active sessions.
- [src/adapters/trace_store.rs](../src/adapters/trace_store.rs) and the existing CLI inspection surfaces already support trace-driven recovery and debugging.

This is part of what makes Synod feel like orchestration rather than chat.

### 3. The bounded orchestration loop

Keep the current engine invariants around bounded execution, recovery, and
terminal states.

- [src/orchestrator/engine.rs](../src/orchestrator/engine.rs) already gives you a real sequential run loop with retries, replans, and terminalization.

The product thesis does not require throwing this away. It requires feeding this
engine better plans and better execution adapters.

### 4. Governance as stage overlay

Keep the governance boundary where Synod orchestrates and Canon governs.

- [src/orchestrator/engine.rs](../src/orchestrator/engine.rs) already applies governance at flow stages instead of moving orchestration into Canon.
- [src/adapters/governance_runtime.rs](../src/adapters/governance_runtime.rs) already models both local governance and Canon CLI integration as runtimes behind a Synod-owned request/response boundary.

That separation is strategically right and should be reinforced.

## What Must Be Refounded

### 1. `init` and templates as the operator story

Today, bootstrap still shapes too much of the product surface.

- [src/cli/init.rs](../src/cli/init.rs) writes a generated execution profile from `bug-fix`, `change`, or `delivery` templates.

That is acceptable as a transitional bootstrap tool, but it is the wrong mental
model for the end product. The next spec should make session entry and capture
the default path, and treat bootstrap as optional setup or fallback.

### 2. Static execution-profile-first planning

Today, planning still depends on a pre-authored workspace execution profile as
the primary contract.

- [src/orchestrator/session_runtime.rs](../src/orchestrator/session_runtime.rs) builds task requests and plans from the current workspace execution profile.
- [src/fixture.rs](../src/fixture.rs) still owns plan construction and runtime assembly for the main execution path.

That is the biggest architectural mismatch with the intended product. The next
spec should shift plan construction toward goal + workspace + documents + Canon
artifacts, with execution profiles becoming optional low-level inputs rather
than the primary UX contract.

This must remain bounded and inspectable. If this turns into a generic black-box
"make me a smart plan" LLM planner, Synod becomes harder to trust, harder to
debug, and easier to derail.

### 3. Manual flow selection as required user work

Today, built-in flows are rigid and user-selected.

- [src/domain/flow.rs](../src/domain/flow.rs) defines the static `bug-fix`, `change`, and `delivery` flows.
- [src/orchestrator/session_runtime.rs](../src/orchestrator/session_runtime.rs) requires explicit `select_flow` when a flow is used.

The next spec should preserve explicit override, but make flow proposal or flow
inference the default.

The safe rule is: infer, show, require lightweight confirmation. Not: infer and
auto-run.

### 4. Fixture-driven runtime naming and adapter shape

The core runtime is already doing real work on the workspace, but the primary
execution path is still shaped like a fixture engine.

- [src/fixture.rs](../src/fixture.rs) still assembles `analyzer`, `coder`, `reviewer`, and `tester` adapters around manifest-declared attempts and validation commands.

Fixture is not the root problem. Fixture is the symptom.

Today the system uses a declarative structure to simulate a dynamic runtime. The
weakness is not that the engine is fake. The weakness is that the main path
still thinks in predeclared change sets instead of model-guided read/modify/test/fix loops.

### 5. Next-action selection is not yet explicit enough

The missing capability is not "a better plan". The missing capability is clear,
bounded ownership of the question: what should happen next?

- [src/orchestrator/engine.rs](../src/orchestrator/engine.rs) already owns the bounded loop where this decision can run safely.
- [src/orchestrator/session_runtime.rs](../src/orchestrator/session_runtime.rs) already owns the session-facing control state that should feed that decision.
- [src/fixture.rs](../src/fixture.rs) still answers that question mostly through prebuilt attempts and flow-shaped step synthesis.

The next spec should make the runtime operate more like this:

```text
while not done:
	observe workspace and evidence
	decide next bounded action
	execute
	verify
	update context
```

If Synod does not own that loop explicitly, it risks becoming a static planner
with better wording instead of a real orchestrator.

### 6. Decision shape is not yet formalized enough

Even if Synod starts choosing the next bounded action, that still is not enough
unless the decision itself becomes a first-class runtime object.

Right now, too much of the decision is implicit in flow-shaped step synthesis,
adapter-local logic, or manifest-declared attempts.

The next spec should formalize a decision shape closer to this:

```text
Decision:
	type: analyze | code | test | fix | replan
	target: file | test | subsystem | workspace slice
	rationale: short inspectable reason
	expected_outcome: verifiable claim
	evidence_inputs: files, traces, failures, docs, Canon artifacts
```

If that structure is missing, every adapter will decide in its own way, traces
will become harder to compare, and debugging the loop will become guesswork.

### 7. Flow and decision loop are not yet connected cleanly enough

The next spec should not treat flow as a script. It should treat flow as a
constraint on what the decision loop is allowed to do next.

- [src/domain/flow.rs](../src/domain/flow.rs) already gives Synod stage metadata and flow identity.
- [src/orchestrator/session_runtime.rs](../src/orchestrator/session_runtime.rs) already owns the operator-facing flow state.

That should evolve toward a model like this:

```text
flow: bug-fix

allowed decisions by stage:
	investigate -> analyze
	implement -> code | fix
	verify -> test | replan
```

If flow remains a rigid script, the loop cannot become adaptive. If the loop
ignores flow, the flow becomes ornamental. The next spec should make flow a
bounded policy surface over decision-making.

### 8. Tools are still underspecified in the runtime model

Agents matter, but tools are what make the loop real.

- [src/registry/tool_registry.rs](../src/registry/tool_registry.rs) and [src/adapters/tool.rs](../src/adapters/tool.rs) already provide the runtime slot for concrete tool execution.
- [src/fixture.rs](../src/fixture.rs) already shows the important reality: file mutation and command execution are what turn orchestration into behavior.

The next spec should make the decision loop explicitly tool-driven:

- read files and collect evidence
- write or patch files
- run tests and validation commands
- inspect diffs and execution output

If the loop is not grounded in tool use, Synod will regress into a planner that
mostly emits text.

### 9. Canon artifacts are not yet first-class planning inputs

Governance integration exists, but Canon-produced artifacts are not yet central
to plan derivation.

The next spec should let Synod consume Canon outputs as bounded planning inputs
without turning Canon into the orchestration brain.

Operationally, the rule should stay strict: Canon is invoked at stage
boundaries, not per action inside the loop.

## Refactor Target For The Next Spec

This is not a normal feature slice. It is an architecture shift across entry
point, planning, and execution semantics.

### What must leave `fixture.rs`

- initial task shaping as the primary product path, especially the logic centered on `build_fixture_plan_for_goal`
- runtime assembly as the main session execution surface, especially the logic centered on `build_fixture_runtime_for_flow`
- flow-specific step synthesis as the default way to decide what happens next
- adaptive candidate selection as a substitute for general next-action selection
- manifest-declared attempts as the dominant execution contract for normal work

`fixture.rs` should stop being where the product decides the next move.

### What should remain in `engine.rs`

- the bounded execution loop
- generic step dispatch for agents, tools, and decisions
- retry, replan, and terminalization semantics
- trace emission and recovery evidence
- governance stage enforcement as overlay
- result normalization and recoverability handling

`engine.rs` is the right home for execution discipline. It should not become a
prompt-heavy planning surface.

### What should remain in `session_runtime.rs`

- workspace-scoped session lifecycle
- capture, clarify, resume, step, run, status, next, and inspect control flow
- persistence and reloading of active session state
- operator-facing confirmation points for clarification, governance pauses, and inferred flow selection
- translation from session state into bounded execution state

`session_runtime.rs` should own operator-facing orchestration, not manifest-first
attempt synthesis.

### What `fixture.rs` should become

- a compatibility layer for declarative execution profiles
- a low-level helper module for workspace mutation and validation primitives
- an optional fallback path for explicitly declarative or test-oriented workflows

The goal is not to delete `fixture.rs`. The goal is to stop making it define the
product.

### What must be introduced as a new runtime primitive

- a first-class bounded decision object owned by Synod, not hidden inside adapter-specific logic
- explicit mapping from observed state to next bounded action
- persistence of the chosen decision into session and trace evidence
- a clean handoff from decision selection to agent and tool execution

Without this primitive, the refactor risks moving code around without changing
the actual behavior model.

### How flow should work after the refactor

- flow selects allowed decision families and stage boundaries
- the runtime chooses the next bounded decision within those constraints
- stage transitions occur only after verifiable outcomes
- explicit flow override remains operator-controlled

This keeps flow useful without turning it back into a script.

## Next Spec Guidance

The next spec should ship one coherent slice with these outcomes together:

1. A user can begin from `start` or `capture` without treating `init` as the main entry point.
2. `plan` derives an initial bounded task draft from goal, workspace state, collected docs, and available Canon artifacts.
3. Synod proposes an inferred flow, shows it, and asks for lightweight confirmation unless the user already made an explicit choice.
4. `run` owns bounded next-action selection from live state, validation evidence, and accumulated context instead of only replaying static declarations.
5. Each next action is represented as an explicit, inspectable decision object with type, target, rationale, expected outcome, and evidence inputs.
6. The execution model becomes an explicit observe -> decide -> act -> verify -> update-context loop.
7. Flow constrains allowed decisions and stage boundaries instead of acting as a rigid script.
8. The loop is explicitly tool-driven through file reads, writes, diffs, tests, and command execution.
9. Governance remains a Synod-owned stage overlay backed by Canon artifacts and approvals, with Canon invoked at stage boundaries rather than per action.

The real target is not a fancier planner. The real target is an execution loop
guided by real state while staying bounded, inspectable, and debuggable.

Guardrails for that slice:

- bounded planning, not a generic black-box planner
- explicit decision objects, not adapter-local hidden heuristics
- inferred flow with lightweight confirmation, not silent auto-run
- flow as bounded policy, not scripted playback
- dynamic next-action selection, not dressed-up manifest playback
- tool-driven execution, not text-only orchestration
- Canon at stage boundaries, not per action
- Synod orchestrates and Canon governs

If that means cutting secondary features, cut them. Do not split this product
realignment across a long train of meta-specs.

## Non-goals For The Next Spec

- redesigning the whole orchestrator core
- replacing bounded execution with open-ended autonomy
- moving orchestration logic into Canon
- polishing more template variants before the session-native path is dominant

## Bottom Line

The current codebase already contains the right backbone:

- real executable harness
- session-native loop
- persisted session and trace state
- bounded engine
- governance overlay

What is still wrong is the dominant input and decision model.

The next step is not just to improve planning language. It is to make the
decision model explicit enough that the runtime can be trusted, debugged, and
implemented consistently across adapters.

The next spec should not ask, "How do we add one more framework surface?"
It should ask, "How do we make Synod feel like the orchestrator it already wants
to be, by letting it decide the next bounded action from real state?"