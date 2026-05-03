# Contract: Delegation Packet Lifecycle

**Feature**: 037-bounded-delegation  
**Date**: 2026-05-03

## Purpose

Define the allowed lifecycle and authority rules for handoff and escalation
packets.

## Lifecycle Rules

- A handoff packet may be created only when direct continuation on the current
  route is non-credible but another bounded continuation path still exists.
- An escalation packet may be created only when no declared continuation path
  remains credible inside the current limits.
- Every packet must preserve decisive evidence, continuity reason, target owner,
  and recommended next action.
- A packet may transition from active to resolved, superseded, stuck, or
  exhausted only through an explicit recorded event.
- Superseding a packet must preserve the historical packet and point to the
  successor packet.

## Explicit Boundaries

- Packet lifecycle must not depend on an external inbox, daemon, or background
  patrol.
- Packet creation must not widen the accepted bounded goal or introduce generic
  multi-agent orchestration.
- A resolved packet must not remain the authoritative continuity source.