# Contract: Reasoning Trace

## Purpose

Define the additive trace events and projection expectations that make
reasoning-profile execution inspectable through the existing Boundline trace
model.

## Required Event Families

The first release adds additive reasoning-profile events to the current trace
model. At minimum, the trace contract must represent:

- profile activation
- participant started
- participant completed
- convergence or disagreement recorded
- debate round completed
- reflexion revision completed
- adjudication recorded
- confidence contribution recorded
- profile blocked or escalated

## Required Payload Fields

Each reasoning-profile event payload must preserve the relevant subset of these
fields when applicable:

- `profile_id`
- `stage`
- `activation_id`
- `participant_id`
- `role`
- `iteration_kind`
- `iteration_index`
- `outcome_kind`
- `independence_result`
- `confidence_level`
- `summary`
- `next_action`
- `canon_posture_ref`

## Projection Requirements

- `status` and `next` MUST surface the latest reasoning-profile condition when
  reasoning activity exists.
- `inspect` MUST be able to summarize activation reason, participant topology,
  independence outcome, disagreement or convergence, confidence contribution,
  and next action from the recorded trace events.
- Trace summaries MUST keep reasoning-profile output distinct from existing
  governance, review, and decision-loop timelines even when they are rendered in
  one combined surface.

## Compatibility Rules

- Reasoning events are additive to the existing trace contract.
- Sessions with no reasoning-profile activity remain valid and require no
  placeholder reasoning projection.
- Unknown future reasoning events must be ignorable by older readers without
  corrupting the rest of the trace summary.

## Explicit Exclusions

- No separate reasoning trace file family
- No hidden reasoning steps omitted from the persisted trace
- No confidence contribution projected without a matching trace event