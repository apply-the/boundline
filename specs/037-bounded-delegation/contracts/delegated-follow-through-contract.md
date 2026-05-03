# Contract: Delegated Follow-Through Surface

**Feature**: 037-bounded-delegation  
**Date**: 2026-05-03

## Purpose

Define which delegated continuity fields must align across native, workflow,
governance-aware, and explicit compatibility follow-up surfaces.

## Required Surface

- `run`, `status`, `next`, and `inspect` must expose the same bounded
  vocabulary for active delegation packet, continuity mode, decisive evidence,
  target owner, recommended next command, and stuck or superseded state.
- Equivalent blocked, handoff-required, escalation-required, resolved, stuck,
  exhausted, and inspect-only states must use aligned wording where the bounded
  meaning is the same.
- When compatibility follow-up is authoritative, the delegated continuity
  surface must still name that route authority explicitly.

## Explicit Boundaries

- Surface alignment must not hide which route owns the current follow-through
  story.
- Compatibility follow-up must not be presented as resumable native session
  state when no active native session owns it.
- Delegated follow-through must not invent packet fields that the authoritative
  route never produced.