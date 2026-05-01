# Contract: Unified Route Summary Surface

**Feature**: 024-unify-route-summaries  
**Date**: 2026-05-01

## Purpose

Define which follow-up summary fields must align across native, workflow, review/governance, and explicit compatibility routes.

## Required Surface

- `status`, `next`, `inspect`, and workflow follow-up surfaces must expose the same bounded vocabulary for route owner, continuity authority, execution condition, state headline, and recommended next action.
- Equivalent paused, blocked, failed, completed, exhausted, and inspect-only states must use aligned summary wording where the bounded meaning is the same.
- When route-specific detail exists, it must appear as route evidence instead of forcing a divergent summary model.

## Explicit Boundaries

- Summary alignment must not hide which route owns the current follow-up story.
- Compatibility follow-up must not be presented as resumable session-native state when no active session exists.
- Unified wording must not invent fields that the authoritative route never produced.
