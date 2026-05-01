# Contract: Compatibility Negotiation Boundary

**Feature**: 026-goal-constraint-modeling  
**Date**: 2026-05-01

## Purpose

Define how explicit compatibility behavior stays visibly separate from the
primary session-native negotiation story.

## Required Surface

- When explicit compatibility follow-up is authoritative, the operator-facing
  surface must say so clearly.
- Compatibility inspection may reference negotiation-related context only when
  that context is trace-backed or otherwise explicitly available.
- Session-native negotiation authority must remain explicit when a clustered
  session projects the packet from the primary workspace.
- Recommended next action must stay aligned with the authoritative route rather
  than implying a hidden session-native resumability story.

## Explicit Boundaries

- Compatibility surfaces must not silently synthesize a session-owned
  negotiation packet when none is authoritative.
- The boundary contract must not duplicate negotiation ownership into every
  cluster member workspace.
- Compatibility behavior must not become the default negotiation control path.