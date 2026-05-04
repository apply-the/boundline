# Contract: Checkpoint Projection

## Purpose

Define how checkpoint state appears on normal Boundline read-side surfaces.

## Surfaces

- `boundline run`
- `boundline status`
- `boundline next`
- `boundline inspect`
- `boundline checkpoint list`

## Required behavior

- When a latest checkpoint exists for the authoritative follow-up state, the
  surface must show the checkpoint identity and a restore cue.
- When checkpoint restore was refused, the surface must preserve the conflict
  story rather than collapsing back into generic failure text.
- Clustered or compatibility follow-up must preserve explicit route authority
  while reusing the same checkpoint vocabulary.

## Consistency rules

- The same authoritative checkpoint story must drive `status`, `next`, and
  `inspect`.
- Projection must not imply that checkpoint restore rewrites or deletes trace
  history.