# Research: Bounded Delegated Execution

## Decision 1: Extend routing configuration with explicit runtime capability descriptors

- **Decision**: Add a structured runtime capability declaration to Boundline's
  existing routing configuration so slot routing is no longer limited to
  runtime plus model identity.
- **Rationale**: Boundline already persists effective routing and assistant
  bindings, but it cannot currently express whether a route can resume, support
  validation-oriented follow-through, or credibly own a delegated handoff.
  Declared capability descriptors make blocked continuity predictable before a
  run reaches an opaque backend mismatch.
- **Alternatives considered**:
  - Continue treating `assistant_runtimes` as the only capability declaration.
    Rejected because runtime presence does not tell the planner or runtime what
    continuity behaviors a route can support.
  - Discover capabilities only at execution time from backend probing. Rejected
    because the feature needs stable, inspectable policy in config and traces,
    not hidden runtime guesses.

## Decision 2: Model effort as slot policy, not as a provider abstraction layer

- **Decision**: Represent effort as an explicit per-slot policy that can shape
  route selection and delegation rationale while keeping the current runtime and
  model routing surface intact.
- **Rationale**: The useful lesson from Gas Town is not provider theatricality;
  it is that operators need to declare where high-cost reasoning is worthwhile
  and where lighter effort is acceptable. Slot policy preserves Boundline's product
  identity and avoids a second abstraction stack.
- **Alternatives considered**:
  - Add provider-wide cost tiers detached from slot routing. Rejected because it
    would compete with the current route-slot model instead of clarifying it.
  - Ignore effort entirely and let model IDs imply reasoning budget. Rejected
    because that would keep a hidden heuristic exactly where this feature needs
    explicit operator intent.

## Decision 3: Persist delegation as session-owned packets in existing state

- **Decision**: Store handoff and escalation packets in the same authoritative
  session and task-context state used by goal plans, follow-through guidance,
  and Canon-grounded memory.
- **Rationale**: Delegated continuity must be visible to planning, execution,
  `status`, `next`, and `inspect`. Existing session and task-context state are
  already the authority for bounded continuity, so extending them preserves one
  runtime story instead of creating an external inbox or separate ledger.
- **Alternatives considered**:
  - Create a new mailbox or packet file under `.boundline/`. Rejected because it
    would split continuity authority away from the active session.
  - Store packets only in traces. Rejected because later decisions need current
    authoritative continuity state before a new trace event exists.

## Decision 4: Use evidence-based stuck detection instead of heartbeat-style monitoring

- **Decision**: Detect stuck delegated continuity from repeated blocked
  attempts, unchanged decisive evidence, unresolved packet reuse, or stale route
  declarations rather than introducing background health checks.
- **Rationale**: Boundline is a bounded sequential runtime, not a daemonized agent
  manager. Evidence-based stuck detection fits the current execution loop and
  constitution while still making failure and exhaustion explicit.
- **Alternatives considered**:
  - Add background patrol or watchdog behavior. Rejected because it violates the
    sequential-first design and imports the wrong architectural lesson.
  - Treat every repeated block as generic failure. Rejected because the feature
    specifically needs continuity-aware recovery, not only terminal failure.

## Decision 5: Resolve or supersede delegation packets explicitly when evidence changes

- **Decision**: Packet lifecycle must include active, resolved, superseded,
  stuck, and exhausted states, with explicit transitions triggered by new route
  declarations, new validation evidence, replanning, or blocked retries.
- **Rationale**: Delegation is only credible if stale packets do not linger as
  false authority. Explicit supersession also keeps the continuity story
  inspectable when an operator changes routing or a later run makes the prior
  handoff obsolete.
- **Alternatives considered**:
  - Replace packets in place without history. Rejected because the constitution
    requires visible control-flow changes and traceability.
  - Leave old packets open until manual cleanup. Rejected because it would turn
    continuity into ambiguous state rather than bounded orchestration.

## Decision 6: Keep compatibility follow-up explicit but reuse the same delegation vocabulary

- **Decision**: The compatibility route may surface delegation and escalation
  language when its traces provide it, but native sessions remain the primary
  operator path and compatibility continuity remains explicitly trace-owned.
- **Rationale**: Boundline already distinguishes native session continuity from
  compatibility follow-up. The delegated execution slice should unify vocabulary
  without collapsing those authority boundaries.
- **Alternatives considered**:
  - Make compatibility traces look resumable as native sessions. Rejected
    because that would blur route ownership and violate current product rules.
  - Ignore compatibility surfaces completely. Rejected because the existing CLI
    still projects explicit follow-through from compatibility traces.

## Decision 7: Ship delegated execution as a release-aligned 0.37.0 macrofeature

- **Decision**: Treat `0.37.0` closeout as part of the slice, including version
  bump, docs and assistant guidance updates, roadmap activation/closure, and
  validation above 95% line coverage for modified Rust files.
- **Rationale**: Delegated continuity changes operator-visible execution
  behavior and must ship as one coherent product story, not as hidden internal
  wiring.
- **Alternatives considered**:
  - Defer docs and release closure until a later cleanup. Rejected because the
    user explicitly requested feature-complete delivery and the product surface
    changes are externally visible.