# Contract: Decision Projection Surface

## Goal

Operators can understand selector-driven execution from the existing Boundline
read-side surfaces without reading raw persisted traces.

## Required Surfaces

- `run`
- `status`
- `next`
- `inspect`

## Required Projection Fields

Each authoritative surface must expose decision-driven state when available:

- current selector kind
- selector rationale
- bounded evidence basis
- verification or recovery intent
- explicit stop reason when execution cannot continue

## Compatibility Rule

When the authoritative path is explicit compatibility follow-up, the same
decision vocabulary may be projected when present, but compatibility authority
must remain explicit.