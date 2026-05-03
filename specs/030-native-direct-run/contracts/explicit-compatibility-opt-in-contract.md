# Contract: Explicit Compatibility Opt-In

## Purpose

Define how Boundline preserves the execution-profile compatibility route without
letting it remain the implicit default for direct run.

## Contract

- Compatibility execution must occur only when the operator chooses it
  deliberately.
- When compatibility execution is chosen, Boundline must keep routing, ownership,
  execution path, and follow-up surfaces explicitly compatibility-owned.
- Native direct-run bootstrap must not silently fall back to compatibility just
  because an execution profile exists.

## Required Visible Outcomes

- Operators can tell whether a run was native or compatibility-owned from the
  normal CLI output surfaces.
- Compatibility traces remain inspectable and continuity-aware.
- The default product story no longer depends on compatibility inference.

## Boundary Conditions

- This contract does not delete `.boundline/execution.json` support.
- This contract does not merge native and compatibility ownership into one
  hidden route.
- This contract does not make compatibility execution the recovery path for
  native bootstrap failures.