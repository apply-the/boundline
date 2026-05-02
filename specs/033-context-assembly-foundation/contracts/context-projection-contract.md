# Contract: Context Projection Surface

## Goal

Operators can recover context-pack meaning from the existing primary Synod CLI surfaces.

## Required Surfaces

- `plan`
- `run`
- `status`
- `next`
- `inspect`

## Required Projection Fields

Each authoritative surface must expose bounded context information when a context pack exists:

- summary of selected context
- explicit credibility state
- primary inputs or narrowed targets
- provenance or rationale lines

## Compatibility Rule

When the authoritative path is explicit compatibility follow-up, the same vocabulary may be projected when the data exists, but compatibility authority must remain explicit.
