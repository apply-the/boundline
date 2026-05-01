# Contract: Adaptive Exhaustion Follow-Up Surface

**Feature**: 023-broaden-bounded-adaptive-repair  
**Date**: 2026-05-01

## Purpose

Define the required terminal and follow-up behavior when broader bounded adaptive repair can no longer continue.

## Required Surface

- When adaptive replanning ends because no remaining bounded candidate is credible or allowed, the runtime must emit an explicit failed or exhausted terminal condition.
- `status`, `next`, and `inspect` must preserve the latest adaptive exhaustion reason, routing ownership, and recommended compatibility follow-up.
- Exhaustion output must distinguish between validation failure on the selected candidate and depletion of remaining bounded candidates.

## Explicit Boundaries

- Exhaustion must not be hidden behind a generic validation failure when no further bounded replan is available.
- Follow-up guidance must not pretend that an active session-native task is resumable if the authoritative state is the latest compatibility trace.
- Exhaustion semantics must not create hidden retries beyond configured limits.