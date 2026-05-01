# Contract: Adaptive Change Kind Surface

**Feature**: 023-broaden-bounded-adaptive-repair  
**Date**: 2026-05-01

## Purpose

Define the required contract for broader bounded adaptive mutation families.

## Required Surface

- Adaptive execution profiles must continue to declare allowed change kinds inside the existing `adaptive.allowed_change_kinds` manifest field.
- Every supported change kind must remain deterministic, built-in, and bounded to the selected workspace slice.
- The runtime must preserve stable candidate signatures across materially identical synthesized changes.

## Explicit Boundaries

- No change kind may search outside manifest-declared `read_targets`.
- No change kind may depend on open-ended autonomous code generation.
- No change kind may hide its resulting candidate family from traces or CLI summaries.