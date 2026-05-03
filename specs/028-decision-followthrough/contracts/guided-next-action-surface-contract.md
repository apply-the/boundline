# Contract: Guided Next-Action Surface

## Purpose

Define how `status`, `next`, and `inspect` must explain the next bounded action
for `0.28.0` without widening the Boundline control plane.

## Contract

- `status`, `next`, and `inspect` must each project one coherent follow-through
  story for the current bounded task or authoritative trace.
- The story must include one explicit next bounded action or one explicit stop
  condition.
- The story must name the evidence that made that action or stop condition
  credible when that evidence is not obvious from the lifecycle status alone.
- Generic state labels such as `planned`, `running`, or `failed` are not
  sufficient on their own when more specific decision, recovery, validation, or
  governance evidence is already available.

## Required Visible Outcomes

- Operators can tell what Boundline wants to do next.
- Operators can tell why that next step is currently credible.
- Operators can tell when Boundline is intentionally stopping because no further
  bounded action is credible.

## Boundary Conditions

- This contract does not add new commands.
- This contract does not permit hidden retry or replanning behavior.
- This contract does not allow the projected next action to contradict the
  actual continuity authority.