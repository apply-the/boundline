# Contract: Native Direct-Run Bootstrap

## Purpose

Define how direct `run --goal` becomes an entry to the primary native session
route instead of the explicit compatibility path.

## Contract

- Direct `run --goal <goal>` must create or continue the
  same bounded native execution story that manual `start -> capture -> plan ->
  run` would have created.
- The route choice must be visible as native on `run`, `status`, `next`, and
  `inspect`.
- Direct run bootstrap must not stop on a pending flow-confirmation detour when
  bounded native execution can proceed safely.

## Required Visible Outcomes

- Operators can reach changed files, validation, and trace inspection from one
  direct run command.
- Native session state remains available after the direct run completes.
- Decision and trace output reflects native goal-plan execution rather than a
  compatibility-only run.

## Boundary Conditions

- This contract does not require a new execution engine.
- This contract does not remove bounded stop conditions.
- This contract does not make compatibility execution disappear.