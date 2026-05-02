# Research: Governed Delivery With Canon Inside The Loop

**Feature**: 031-canon-delivery-loop  
**Date**: 2026-05-02

## R1: Reuse the existing session-native governance path instead of inventing a new governed workflow surface

**Decision**: Build the slice on top of the current `SessionRuntime` governance
hooks, the existing `bug-fix` flow stages (`investigate`, `implement`,
`verify`), and the current Canon adapter contract.

**Rationale**: The repo already supports governed stage execution, packet reuse,
approval refresh, and verify-stage Canon modes. The product gap is not the
absence of a governance runtime, but the absence of a strong claim that Synod
now delivers real code changes inside that governed loop.

**Alternatives Considered**:
- Create a new governed workflow or separate Canon-owned execution path:
  rejected because it would widen product surfaces instead of proving value in
  the existing one.
- Wait for Feature 032 unification work first: rejected because the roadmap
  explicitly requires real delivery proof before more abstraction.

## R2: Add an explicit delivery-completion gate in `SessionRuntime`

**Decision**: Tighten terminal success in the session-native runtime so goal
completion requires all of the following: no blocking or approval-pending
governance state, at least one material changed file in task context, and a
credible validation outcome recorded in task context.

**Rationale**: The current runtime can finalize a task successfully simply
because the last step succeeded or the plan ran out of steps. That is the exact
behavioral gap behind the criticism that Synod still does not demonstrably
deliver.

**Alternatives Considered**:
- Keep last-step completion and rely on docs or guidance to explain what “real
  delivery” means: rejected because it leaves false-positive success paths.
- Add a brand-new completion engine or artifact store: rejected because the
  runtime already persists `latest_changed_files` and `latest_validation_status`.

## R3: Treat missing diff or missing validation evidence as bounded delivery failure, not as silent success

**Decision**: When the current plan reaches terminal evaluation without
material diff or without passed validation evidence, stop with an explicit
terminal reason such as `TaskNotCredible` instead of claiming goal satisfaction.

**Rationale**: A bounded delivery system must stop explicitly when it cannot
support a credible claim that the requested code change was actually delivered.

**Alternatives Considered**:
- Mark the task succeeded but surface warnings in `inspect`: rejected because it
  still pollutes the main session story with a false success state.
- Replan automatically forever until a diff appears: rejected because the
  feature must remain bounded and sequential-first.

## R4: Keep governed and non-governed follow-through on the same CLI surfaces

**Decision**: Reuse existing session and trace projections in `status`, `next`,
`inspect`, and follow-through summaries, adding only the extra delivery-gate
signals needed to explain why a governed run completed or stopped.

**Rationale**: Feature 031 is only credible if Canon participation stays inside
the Synod product story rather than creating a second diagnostic surface.

**Alternatives Considered**:
- Add a dedicated “governed delivery” read command: rejected because it would
  fragment the operator experience again.
- Hide delivery gating inside trace-only details: rejected because success or
  blocked semantics need to be visible on the current primary surfaces.

## R5: Use the existing verify-stage Canon mapping for the first full governed proof

**Decision**: For the initial proof, continue to rely on existing Canon stage
mapping where `bug-fix:investigate` and `bug-fix:implement` can be governed and
`bug-fix:verify` can route through `security-assessment` when configured.

**Rationale**: The repository already contains test fixtures and runtime support
for governed investigate, implementation packet reuse, verify-stage security
assessment, and approval refresh. Reusing those paths keeps the slice minimal.

**Alternatives Considered**:
- Introduce a brand-new dedicated “review” flow stage in this slice: rejected
  because it expands the flow model before the core delivery proof is complete.
- Govern every built-in flow in one release: rejected because one valuable
  governed bug-fix proof is the roadmap minimum.

## R6: Treat release closeout as first-class work for `0.31.0`

**Decision**: Include version bump, impacted docs plus changelog, assistant
guidance, coverage for modified or created Rust files above 95%, `cargo
clippy`, and `cargo fmt` as explicit tasks in the implementation plan.

**Rationale**: This feature changes the product claim for Synod. Runtime,
docs, assistant guidance, roadmap, and release evidence must land together or
the release will contradict itself.

**Alternatives Considered**:
- Defer docs and release hygiene until after runtime work: rejected because the
  user explicitly requested release-complete delivery discipline.
- Skip coverage on modified Rust files: rejected because the request makes that
  a shipping constraint, not a best-effort goal.