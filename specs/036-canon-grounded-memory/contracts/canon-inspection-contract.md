# Contract: Canon-Grounded Inspection

## Purpose

Define how Boundline exposes Canon-grounded reasoning and compact memory through the
normal read-side operator surfaces.

## Covered surfaces

- `boundline run`
- `boundline status`
- `boundline next`
- `boundline inspect`

## Required behavior

- When Canon-grounded evidence materially influenced planning or later decisions,
  the read-side surfaces must show:
  - the active compact Canon-memory headline,
  - the current memory credibility state,
  - the decisive Canon context headline,
  - packet or artifact lineage when relevant, and
  - any capability constraint, refresh requirement, or stop reason.

## Compatibility continuity

- If the latest authoritative follow-up state comes from an explicit
  compatibility-governed trace, read-side surfaces must keep that route
  ownership explicit.
- When such a trace contains Canon-grounded reasoning or compact-memory
  evidence, the same vocabulary must be projected without implying that native
  session state owns the follow-up.

## Missing-evidence behavior

- If no Canon-grounded evidence exists, surfaces may omit Canon-specific
  projection but must not invent it.
- If Canon-grounded memory is non-credible, surfaces must prefer the explicit
  refresh, replan, or stop wording over generic continuation guidance.