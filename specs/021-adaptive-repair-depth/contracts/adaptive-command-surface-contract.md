# Contract: Adaptive Command Surface

**Feature**: 021-adaptive-repair-depth  
**Date**: 2026-05-01

## Purpose

Define the user-visible command outputs required when validation-guided adaptive replanning changes the bounded repair path.

## Required Surfaces

- `run` must keep compatibility routing explicit and surface the current adaptive workspace slice, updated attempt lineage, validation outcome, and terminal or recovery condition.
- `status` must surface `latest_workspace_slice`, `latest_selection_headline`, `latest_attempt_lineage`, and latest validation status after a validation-guided replan.
- `next` must keep the same adaptive route story and must not invent a session-native or workflow-owned follow-up when the active run remains on the compatibility path.
- `inspect` must summarize the adaptive slice change, validation-guided rationale, recovery path, and terminal reason without requiring raw log inspection.

## Explicit Non-Success Requirements

- If validation guidance cannot justify a new bounded candidate, the command surfaces must report an explicit failed or exhausted condition.
- If a new bounded candidate is selected, the surfaces must explain that the attempt changed because of validation evidence rather than hidden heuristics.