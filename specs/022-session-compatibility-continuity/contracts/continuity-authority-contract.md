# Contract: Continuity Authority

**Feature**: 022-session-compatibility-continuity  
**Date**: 2026-05-01

## Purpose

Define how Synod chooses the authoritative follow-up state after explicit compatibility execution.

## Authority Rules

- An active native session remains authoritative for native session state unless the CLI explicitly reports a different bounded follow-up authority.
- A latest compatibility trace may become authoritative for compatibility follow-up even when a native session still exists, but that relationship must be named explicitly.
- When no resumable compatibility state exists, the authoritative follow-up must be inspect-oriented rather than implied as resumable.

## Unsupported Expectations

- No hidden conversion of compatibility traces into native session ownership.
- No background reconciliation worker that mutates route ownership out of band.
- No Canon-owned selection of the next operator command.