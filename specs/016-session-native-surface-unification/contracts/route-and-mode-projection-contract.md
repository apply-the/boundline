# Contract: Route Precedence And Mode Projections

## Purpose

Define how session-native routing, explicit compatibility behavior, and optional bounded mode projections coexist after unification.

## Route precedence

### Rule 1: Ready session-native state is the default authoritative route

If the workspace has a valid active session with a ready goal plan or runnable bounded task state, Boundline MUST project the route as `native` unless the operator explicitly selected compatibility behavior.

### Rule 2: Compatibility remains explicit

If the operator invokes an explicit compatibility path or the workspace only provides a credible compatibility manifest, Boundline MAY project the route as `compatibility`.

Compatibility routing MUST remain visibly labeled across `run`, `status`, and `inspect`.

### Rule 3: Missing context blocks rather than guesses

If Boundline cannot derive a credible session-native route or explicit compatibility route, it MUST project a blocked condition with remediation guidance instead of silently guessing.

## Optional bounded mode projections

### Review

If review state is present, Boundline MUST project review trigger, vote, outcome, or headline using stable summary fields without changing the active route label.

### Adaptive execution

If adaptive execution state is present, Boundline MUST project workspace-slice and attempt-lineage details using stable summary fields without changing the active route label.

### Governance

If governance state is present, Boundline MUST project stage, runtime, mode, decision, blocked or waiting reason, and next action using stable summary fields.

Governance state MUST NOT imply that Canon owns the per-action control loop.

## Acceptance Examples

### Ready session-native plan plus compatibility manifest

**Given** a workspace with a ready session-native plan and an existing `.boundline/execution.json`

**When** the operator runs `boundline status --workspace .`

**Then** Boundline projects:

- route `native`
- a reason that the ready session-native plan is authoritative
- compatibility only if the operator explicitly selects it

### Explicit compatibility invocation

**Given** a workspace with both a ready session-native plan and a valid compatibility manifest

**When** the operator runs an explicit compatibility command path

**Then** Boundline projects:

- route `compatibility`
- a reason that the operator selected explicit compatibility behavior
- no hidden fallback phrasing that suggests native routing remained active

### Governance waiting state

**Given** a native session that is waiting on stage-boundary governance approval

**When** the operator runs `boundline status --workspace .`

**Then** Boundline projects:

- route `native`
- execution condition `waiting`
- governance summary fields
- the same next action that `inspect` would recommend