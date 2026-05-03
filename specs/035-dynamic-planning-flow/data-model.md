# Data Model: Dynamic Planning And Flow Inference

## GoalPlan

- **Purpose**: Remains the authoritative persisted record for inferred work on a
  captured goal.
- **Existing fields reused**:
  - `goal_text`
  - `tasks`
  - `workspace_signals`
  - `context_pack`
  - `workflow_progress`
  - `flow`
  - `status`
- **New or expanded responsibilities**:
  - Persist the current planning proposal state.
  - Persist whether the proposal is confirmed for execution.
  - Persist a bounded revision number and prior revision summary when replanning.
  - Persist a verification strategy summary alongside planned tasks.
  - Persist proposal rationale and the evidence lines that justify it.

## PlanningEvidenceBundle

- **Purpose**: Normalized evidence assembled from the workspace and current
  session context before inference.
- **Inputs**:
  - Goal text.
  - Context-pack primary inputs and selected targets.
  - Workspace signals such as language, tests, config, Canon presence, and file
    density.
  - Authored brief or negotiation projection.
  - Latest trace reference when present.
  - Workflow progress when present.
- **Outputs**:
  - Evidence lines suitable for inspect/status surfaces.
  - Flow-scoring hints.
  - Target-selection hints.
  - Verification-strategy hints.
- **Validation rules**:
  - Must preserve bounded provenance for every non-derived signal.
  - Must not include empty evidence references.

## FlowProposal

- **Purpose**: Captures the inferred flow candidate and why it was selected.
- **Fields**:
  - `flow_name`
  - `confidence_reason`
  - `score_summary`
  - `workflow_guardrail_summary`
  - `confirmed`
- **Validation rules**:
  - `flow_name` must be one of the supported built-in flows unless explicitly
    skipped.
  - `confidence_reason` must explain the decisive evidence, not only a keyword.

## VerificationStrategy

- **Purpose**: Makes the plan's validation intent explicit.
- **Fields**:
  - `mode`: targeted test, workspace test, static validation, inspect-only, or
    bounded no-test justification.
  - `targets`: files, suites, or commands the runtime should prefer.
  - `rationale`: why this strategy matches the evidence.
- **Validation rules**:
  - Every proposed plan must carry either a verification strategy or an explicit
    bounded reason why verification is deferred.

## PlanProposalState

- **Purpose**: Tracks whether the current plan is a draft proposal, confirmed,
  or superseded by a later bounded revision.
- **Fields**:
  - `mode`: proposed, confirmed, superseded.
  - `revision`: monotonically increasing integer starting at 1.
  - `revision_reason`: required when revision > 1.
  - `confirmed_at`: timestamp when explicitly confirmed.
  - `superseded_by_revision`: optional integer reference.
- **Validation rules**:
  - Only one revision can be authoritative at a time.
  - A superseded proposal cannot be re-confirmed.

## PlannedTask

- **Purpose**: Represents a bounded unit of planned work.
- **Existing fields reused**:
  - `task_id`
  - `description`
  - `target`
  - `expected_outcome`
  - `decision_type_hint`
- **New or expanded responsibilities**:
  - Reflect evidence-selected targets rather than generic placeholder stages.
  - Differentiate investigation, modification, verification, and replanning steps
    using bounded descriptions and decision hints.

## ReplanRevisionRecord

- **Purpose**: Captures why a new proposal replaced the prior one.
- **Fields**:
  - `from_revision`
  - `to_revision`
  - `trigger`: new trace, operator request, changed workspace evidence, or
    workflow-guardrail conflict.
  - `summary`
  - `changed_fields`: flow, targets, verification strategy, tasks.
- **Validation rules**:
  - Must mention at least one changed field.
  - Must point to an existing prior revision.

## Session Projection

- **Purpose**: Exposes the authoritative planning state to CLI and runtime
  routing.
- **Derived fields**:
  - `execution_path`: pending plan proposal, pending confirmation, native goal
    plan, compatibility fallback, or blocked.
  - `next_action`: confirm current proposal, replan, capture more context, or
    continue execution.
  - `proposal_summary`: flow, targets, verification strategy, revision, and
    evidence headline.

## Trace Projection

- **Purpose**: Records proposal generation, confirmation, supersession, and run
  decisions in the same inspectable trace vocabulary as native execution.
- **Required payload concepts**:
  - Proposal state.
  - Revision lineage.
  - Evidence summary.
  - Verification strategy.
  - Blocking reason when run is refused.