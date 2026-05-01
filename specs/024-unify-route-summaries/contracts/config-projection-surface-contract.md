# Contract: Config Projection Surface

**Feature**: 024-unify-route-summaries  
**Date**: 2026-05-01

## Purpose

Define which routing and configuration facts must be projected onto aligned follow-up surfaces.

## Required Surface

- Follow-up summaries must show explicit route overrides when the current command or trace chose a route directly.
- When workspace or global defaults materially explain the current route interpretation, the summary must surface those defaults.
- Workflow metadata and governance mode must appear only when they materially affect the current follow-up story.
- The projection should explain why a route owns follow-up, not dump every known config field.

## Explicit Boundaries

- Stale or irrelevant config must not appear just because it exists in persisted config files.
- Config projection must not override the explicit route owner named by the runtime state.
- Summary surfaces must not imply that Canon or workflow metadata controls native or compatibility execution when that control path is not active.
