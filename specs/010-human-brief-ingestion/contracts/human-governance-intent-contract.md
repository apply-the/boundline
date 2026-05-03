# Contract: Human Governance Intent

## Purpose

Define the human-facing governance input accepted alongside direct text and Markdown briefs, and define how that input maps into Boundline's existing governed execution path without exposing stage-level wiring.

## Accepted Inputs

```text
--governance <local|canon>
--risk <value>
--zone <value>
--owner <value>
```

- `--governance` is an optional runtime preference, not a stage selector.
- `--risk`, `--zone`, and `--owner` are optional business values that refine governed execution intent.
- Assistant-driven entry points must provide the same semantic fields even if they are collected from chat rather than flags.

## Mapping Rules

- If none of the governance fields are present, Boundline proceeds with the normal ungoverned path unless an advanced manifest already requires governance.
- If `--governance` is present, governed execution is requested explicitly.
- If any of `--risk`, `--zone`, or `--owner` is present without `--governance`, governed execution is still considered requested, but the runtime preference remains unspecified until Boundline can derive it credibly from workspace defaults or targeted clarification.
- Internal stage IDs, Canon modes, packet references, and manifest keys are never valid user inputs in this contract.
- After normalization, Boundline maps the accepted governance intent into the existing governance runtime and session projection path.

## Clarification Rules

- If governed execution is requested but a required business field is missing, Boundline must ask only for the missing business value.
- Clarification must never ask the user to translate governance intent into stage mappings or internal runtime configuration.
- If no credible governed path can be derived from the accepted business inputs and workspace configuration, Boundline must stop before planning or execution and report why.

## Output Rules

- `status` and `inspect` must show whether governance was requested and what business values were accepted.
- When governed execution becomes blocked or approval-gated, the user-facing output must continue to report the blocked or awaiting-approval state through the existing session surfaces.
- The resulting governed execution must continue to use Boundline's local-first governance abstraction so that the feature remains independently testable when Canon is absent.