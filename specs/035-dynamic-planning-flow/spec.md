# Feature Specification: Dynamic Planning And Flow Inference

**Feature Branch**: `035-dynamic-planning-flow`  
**Created**: 2026-05-02  
**Status**: Draft  
**Input**: User description: "Implement Spec 035 Dynamic Planning And Flow Inference: replace keyword-first flow inference and stage-static planning with infer -> propose -> confirm planning derived from context packs and observed workspace evidence; allow bounded replanning to reshape targets, verification strategy, and flow choice without losing operator control or acceptance-boundary visibility; preserve workflows as operator-facing guidance and guardrails instead of the sole source of execution shape; ship the feature complete with version bump, docs, changelog, roadmap update, cargo fmt, cargo clippy, and line coverage above 95% for modified Rust files."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Infer A Bounded Plan From Workspace Evidence (Priority: P1)

An operator can capture a bounded goal and have Synod infer a credible flow,
target set, and verification approach from the current context pack and observed
workspace evidence instead of relying on keyword-first flow matching and a mostly
static analyze-then-implement plan shape.

**Why this priority**: This is the operating-model change named by roadmap Spec
035. Without it, planning still behaves like a lightly contextualized wrapper
around static task derivation.

**Independent Test**: Run `start -> capture -> plan` on representative Rust
workspaces with distinct goals and verify that the proposed flow and tasks are
derived from observed files, symbols, tests, and acceptance cues rather than
from keyword matches alone.

**Acceptance Scenarios**:

1. **Given** a goal whose wording is ambiguous but whose workspace evidence
  strongly indicates a bug in existing Rust code, **When** the operator runs
  `plan`, **Then** Synod proposes a bounded bug-fix style plan with evidence-backed
  targets and verification steps even when the goal text alone would not have
  produced that flow.
2. **Given** a goal whose workspace evidence indicates a new capability with no
  existing failing test, **When** the operator runs `plan`, **Then** Synod
  proposes a bounded change-style plan with explicit implementation and
  verification intent derived from the context pack instead of replaying a fixed
  task sequence.

---

### User Story 2 - Confirm Or Adjust The Proposed Plan Explicitly (Priority: P2)

An operator can see the inferred flow and proposed bounded plan, then confirm it
explicitly or rerun planning with bounded adjustments while keeping the operator
in control of the execution shape.

**Why this priority**: Dynamic planning is only credible if Synod remains
inspectable and operator-controlled instead of silently changing the plan shape
behind the normal session workflow.

**Independent Test**: After `plan`, verify that `status`, `next`, `run`, and
`inspect` surface the inferred flow, proposed plan state, confirmation status,
and any bounded operator action needed before execution continues.

**Acceptance Scenarios**:

1. **Given** a newly inferred bounded plan, **When** the operator checks
  `status` or `next`, **Then** Synod surfaces the proposed flow, why it was
  inferred, which evidence justified it, and whether execution is waiting on
  explicit confirmation.
2. **Given** a workflow-aware session with a proposed dynamic plan, **When** the
  operator confirms it, **Then** Synod preserves workflow guidance as a guardrail
  while recording that the plan shape is now session-authoritative and ready for
  execution.

---

### User Story 3 - Replan Boundedly When New Evidence Changes The Best Path (Priority: P3)

An operator sees bounded replanning reshape targets, verification strategy, or
flow choice when execution reveals stronger evidence, while keeping acceptance
boundary, stop conditions, and reasoning explicit.

**Why this priority**: The roadmap requires dynamic planning rather than a one-time
proposal. Without bounded replanning, Synod would still be locked into a
mostly stage-static execution model.

**Independent Test**: Run representative bounded tasks where the first inferred
plan becomes non-credible after analysis or validation and verify that Synod
records a bounded replan revision instead of silently continuing the old plan or
jumping to an unrelated fallback.

**Acceptance Scenarios**:

1. **Given** a planned bounded change whose first analysis reveals a different
  target family than originally proposed, **When** Synod replans, **Then** it
  records a new bounded proposal with updated targets or verification strategy
  and keeps the acceptance boundary visible.
2. **Given** a bounded execution path whose validation evidence invalidates the
  current flow choice, **When** Synod replans, **Then** it can change the flow
  choice or stop explicitly, but it must keep the rationale, revision lineage,
  and operator-facing next action inspectable.

---

### User Story 4 - Ship The Dynamic Planner As 0.35.0 (Priority: P4)

A maintainer can ship `0.35.0` with the dynamic planning model reflected
consistently in runtime behavior, roadmap closure, docs, changelog, version
metadata, and repository validation evidence.

**Why this priority**: The feature is incomplete if the runtime changes land
without the release surfaces, roadmap closure, and repository validation needed
to present one coherent product story.

**Independent Test**: Follow the updated docs on a representative workspace,
run the release validation suite, and verify that the version bump, roadmap,
docs, changelog, coverage, linting, and formatting all align with the shipped
dynamic planning operating model.

**Acceptance Scenarios**:

1. **Given** the `0.35.0` release artifacts, **When** a maintainer follows the
  documented native path, **Then** the runtime, roadmap, and docs all describe
  evidence-driven infer -> propose -> confirm planning as the primary Synod
  planning model.
2. **Given** modified or newly created Rust files for this slice, **When** the
  maintainer runs release validation, **Then** those files remain above 95%
  line coverage, lint issues introduced by the slice are resolved, and
  formatting completes successfully.

### Edge Cases

- What happens when workspace evidence supports multiple plausible flows but no
  single bounded proposal is clearly credible?
- How does the system behave when a proposed flow conflicts with explicit
  workflow guardrails, negotiated acceptance boundaries, or previously captured
  operator intent?
- What happens when replanning would expand the scope beyond the bounded change
  implied by the accepted goal or negotiated delivery packet?
- How does the system surface proposed versus confirmed planning state on the
  primary session-native path versus an authoritative explicit compatibility
  follow-up trace?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST derive flow choice, bounded targets, and verification
  strategy from the current context pack and observed workspace evidence instead
  of relying primarily on keyword-first flow inference.
- **FR-002**: System MUST represent dynamic planning as an explicit infer ->
  propose -> confirm lifecycle on the primary session-native path.
- **FR-003**: System MUST persist the inferred planning rationale, the evidence
  that justified it, and the current proposal or confirmation state in the
  goal-plan/session story.
- **FR-004**: System MUST preserve operator control by making the proposed plan
  shape visible before execution proceeds when confirmation is required.
- **FR-005**: System MUST allow bounded replanning to revise targets,
  verification strategy, or flow choice when new evidence invalidates the prior
  proposal.
- **FR-006**: System MUST keep replanning bounded by the captured goal,
  negotiated delivery packet, and explicit acceptance boundary instead of
  silently broadening scope.
- **FR-007**: System MUST keep workflows as operator-facing guidance and
  guardrails rather than the sole source of execution shape.
- **FR-008**: System MUST surface inferred flow, planning rationale, proposal
  status, replan revision lineage, and any confirmation or stop requirement
  through the existing plan, run, status, next, and inspect surfaces plus
  persisted traces.
- **FR-009**: System MUST stop explicitly when no credible bounded plan or
  replan can be proposed within the available context and configured limits.
- **FR-010**: System MUST preserve explicit compatibility authority when the
  latest authoritative follow-up state comes from a compatibility trace, while
  reusing the same planning vocabulary when that trace contains it.
- **FR-011**: System MUST include contract, integration, and unit validation
  for evidence-driven flow inference, proposal confirmation, bounded replanning,
  and output projection.
- **FR-012**: System MUST include explicit release-closeout work for the version
  bump, roadmap closure, impacted docs, assistant guidance, and changelog.
- **FR-013**: System MUST finish with repository formatting, lint cleanliness,
  and line coverage above 95% for modified or newly created Rust files in this
  slice.

### Scope Boundaries *(mandatory)*

- **In Scope**: evidence-driven dynamic planning on the primary native path;
  explicit infer -> propose -> confirm planning state; bounded replanning that
  can revise targets, verification intent, and flow choice; workflow guardrails
  preserved as guidance; release closeout for `0.35.0`.
- **Out of Scope**: long-term memory beyond bounded session or trace reuse;
  distributed or parallel planning; provider abstraction refoundation; new UI
  surfaces; Canon-coupled reasoning that requires a new machine contract;
  unbounded autonomous scope expansion.

### Key Entities *(include if feature involves data)*

- **PlanningInference**: the bounded evidence-backed explanation of why Synod
  inferred one flow, target family, and verification strategy instead of another.
- **PlanProposal**: the proposed bounded execution shape produced by planning,
  including selected flow, selected targets, verification intent, and the
  operator-visible confirmation state.
- **PlanRevision**: a bounded replan record that supersedes a previous proposal
  while preserving revision lineage, rationale, and acceptance-boundary
  continuity.
- **WorkflowGuardrailProjection**: the workflow-owned guidance that remains
  visible during dynamic planning so workflows constrain and explain planning
  without becoming the only source of plan shape.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative native planning runs, operators can identify
  the inferred flow, selected targets, verification strategy, and confirmation
  state from standard Synod output in under 2 minutes.
- **SC-002**: 100% of representative planning and replanning runs end with one
  explicit proposed plan, one explicit confirmed plan, or one explicit stop
  reason instead of silently falling back to keyword-only flow selection.
- **SC-003**: In representative replanning scenarios, developers can identify
  why Synod revised the flow or target set and what revision is authoritative
  from `status`, `next`, or `inspect` without reading raw JSON.
- **SC-004**: All modified or newly created Rust files in this slice finish the
  release validation suite above 95% line coverage with clean formatting and
  lint results.

## Assumptions

- The primary operator path remains session-native, and explicit compatibility
  execution remains a subordinate, opt-in route.
- Existing goal-plan, session, trace, workflow, and decision surfaces can be
  extended without introducing a second planning runtime.
- Confirmation remains a bounded CLI-visible state transition rather than a new
  interactive chat loop inside `run`.
- Release closeout for `0.35.0` may update repository docs, assistant guidance,
  roadmap entries, version metadata, and changelog in the same delivered
  macrofeature.
