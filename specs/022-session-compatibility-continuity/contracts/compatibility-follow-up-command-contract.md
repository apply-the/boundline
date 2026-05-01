# Contract: Compatibility Follow-Up Command Surface

**Feature**: 022-session-compatibility-continuity  
**Date**: 2026-05-01

## Purpose

Define the required user-visible follow-up behavior after an explicit compatibility run.

## Required Surfaces

- `run` must keep compatibility routing explicit and surface enough trace information for later commands to identify the resulting follow-up state.
- `status` must explain whether the authoritative follow-up state comes from the active session, the latest compatibility trace, or an explicit absence of follow-up state.
- `next` must recommend only bounded follow-up commands supported by the authoritative state and must not imply hidden resumability.
- `inspect` must continue to resolve the latest compatibility trace when workspace-based follow-up depends on it.

## Explicit Non-Success Requirements

- If a compatibility run leaves only inspectable terminal evidence, `next` must recommend inspection rather than resumption.
- If no active session and no latest trace exist, the CLI must stop explicitly instead of synthesizing a fake follow-up state.