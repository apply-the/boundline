# Contract: Workspace Participation Surface

**Feature**: 025-multi-workspace-delivery  
**Date**: 2026-05-01

## Purpose

Define how Synod reports which member workspaces participated in one clustered
delivery story.

## Required Surface

- Clustered run and inspection output must identify which member workspaces were
  read, mutated, blocked, or skipped when that distinction matters to follow-up.
- Participation reporting must preserve the bounded ordering of workspace
  involvement when the delivery story traverses multiple repositories.
- When a workspace produces the authoritative trace or blocking condition, the
  participation surface must make that relationship explicit.
- Participation reporting must remain inspectable from persisted clustered
  follow-up or trace-derived views.

## Explicit Boundaries

- Participation output must not claim that a workspace mutated when it was only
  inspected or read.
- The surface must not dump every cluster member indiscriminately when only a
  subset materially participated in the delivery story.
- Missing or stale member state must be surfaced as a gap, not inferred away.