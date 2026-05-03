# Research: Dynamic Planning And Flow Inference

## Decision 1: Make planning proposal state explicit inside GoalPlan

- **Decision**: Extend the existing goal-plan model with explicit proposal,
  confirmation, and revision metadata instead of keeping proposal state implicit
  in CLI behavior or transient runtime flags.
- **Rationale**: `GoalPlan` already owns inferred flow, context credibility,
  workflow progress, and task candidates. Keeping proposal lineage there
  preserves the existing session-native authority model and lets `status`,
  `next`, and `inspect` render the same authoritative state.
- **Alternatives considered**:
  - Store proposal metadata only in traces. Rejected because `run` and `status`
    need durable session state before any new trace is written.
  - Introduce a second planning-state file. Rejected because it would duplicate
    authority and violate the sequential session model.

## Decision 2: Replace keyword-first flow inference with evidence scoring

- **Decision**: Infer candidate flows from a scored evidence bundle built from
  context-pack inputs, selected targets, workspace signals, authored brief or
  negotiation projections, latest trace evidence, and goal language as one
  signal rather than the only one.
- **Rationale**: The pre-035 behavior chooses a flow from goal keywords and then
  shapes the plan almost statically. Evidence scoring lets Synod prefer bug-fix
  when it sees failing-test or trace signals, prefer change when workspace and
  selected targets point to bounded code additions, and prefer delivery only
  when the evidence indicates cross-cutting completion work.
- **Alternatives considered**:
  - Keep keyword matching and add more keywords. Rejected because it preserves
    the root problem: the plan shape is still driven by textual coincidence.
  - Require explicit flow selection for every plan. Rejected because it removes
    the planning assistance that the feature is meant to improve.

## Decision 3: Use `synod plan` as proposal, `synod plan --confirm` as commit point

- **Decision**: Keep the existing `plan` command as the operator-facing entry
  point, but change its semantics so the default invocation produces a proposal,
  `--confirm` confirms the current proposal, and `--replan` produces a bounded
  successor proposal when evidence has changed.
- **Rationale**: This reuses the current operator path and keeps the main state
  transition visible. It also avoids overloading `run` with hidden confirmation
  behavior and gives `status` and `next` a stable recommendation surface.
- **Alternatives considered**:
  - Auto-confirm on every successful `plan`. Rejected because it prevents the
    explicit operator checkpoint required by the feature.
  - Add a new `confirm-plan` command. Rejected because the surface area is small
    enough to live under `plan` without introducing a new top-level command.

## Decision 4: Keep workflows as guardrails, not plan owners

- **Decision**: Workflow progress may bias flow inference and task ordering, but
  the inferred plan remains derived from evidence and bounded context rather than
  copied from workflow phases.
- **Rationale**: Existing workflows are useful operator hints, yet Spec 035
  explicitly requires that they not be the sole source of plan shape. The plan
  must remain adaptable when the workspace evidence disagrees with the nominal
  workflow.
- **Alternatives considered**:
  - Convert workflow phases directly into planned tasks. Rejected because it
    recreates the static-plan problem under a different name.
  - Ignore workflows entirely. Rejected because it would lose useful guardrails
    already present in the product.

## Decision 5: Model replanning as bounded supersession, not in-place mutation

- **Decision**: When new evidence changes targets, verification strategy, or
  flow choice, create a new revision of the goal plan, supersede the prior
  confirmed revision, and keep an explicit reason for the revision.
- **Rationale**: Replanning needs an inspectable lineage so operators can see
  why the plan changed and which proposal is authoritative. Silent mutation
  would erase the context needed to trust the runtime.
- **Alternatives considered**:
  - Mutate tasks and flow in place. Rejected because it hides causal history.
  - Fork a new session for every replan. Rejected because it fragments the
    bounded delivery record for a single goal.

## Decision 6: Block native run on unconfirmed proposal, with actionable output

- **Decision**: Native run should remain blocked until the current proposal is
  confirmed, and the blocking reason should tell the operator whether to confirm,
  replan, or gather more context.
- **Rationale**: The runtime already blocks on pending flow confirmation. Extending
  that checkpoint to full plan confirmation preserves operator control and makes
  the new planning model real instead of cosmetic.
- **Alternatives considered**:
  - Let `run` auto-confirm. Rejected because it hides the confirmation boundary.
  - Allow unconfirmed runs in some flows. Rejected because it makes the planning
  contract inconsistent across delivery modes.

## Decision 7: Preserve compatibility follow-up as an explicit fallback only

- **Decision**: Compatibility execution remains available for explicit operator
  intent or legacy active-task continuity, but the goal-plan proposal remains the
  authoritative planning surface for new session-native work.
- **Rationale**: Spec 035 changes planning quality, not the authority model. The
  compatibility path should not silently bypass proposal confirmation or replan
  lineage.
- **Alternatives considered**:
  - Collapse compatibility and native planning into one abstract route. Rejected
    because it would blur the trace-authoritative fallback boundary.

## Decision 8: Release the slice with docs, roadmap, and coverage closure

- **Decision**: Treat `0.35.0` closeout as part of the feature, including version
  bump, release notes, roadmap update, assistant/docs refresh, clippy/fmt, and
  coverage for modified Rust files above 95%.
- **Rationale**: The roadmap request is release-complete, and the planning model
  materially affects user-facing behavior and operator guidance.
- **Alternatives considered**:
  - Defer docs and release notes. Rejected because the behavioral contract is
    operator-visible and must ship together with the code.