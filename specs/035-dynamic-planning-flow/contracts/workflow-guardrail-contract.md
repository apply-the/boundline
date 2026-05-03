# Contract: Workflow Guardrails In Planning

## Purpose

Define how workflow progress influences planning without becoming the sole source
of plan shape.

## Required behavior

- Workflow progress may contribute guardrail hints for preferred flow, expected
  sequencing, and next validation focus.
- The planner must still derive targets and verification strategy from the
  current evidence bundle, even when a workflow is active.
- If workflow guardrails and workspace evidence disagree, the proposal must say
  so explicitly and record which side was favored.

## Observable surfaces

- `plan` output must show whether a workflow guardrail influenced the proposal.
- `status` and `inspect` must preserve the workflow name or phase plus the
  guardrail summary.
- Replanning must mention when workflow guardrails changed the preferred flow or
  validation emphasis.

## Non-goals

- Workflow phases must not be copied directly into planned tasks without evidence.
- Workflow presence must not bypass clarification, confirmation, or replanning
  rules.
- Workflow guardrails must not silently force compatibility execution.