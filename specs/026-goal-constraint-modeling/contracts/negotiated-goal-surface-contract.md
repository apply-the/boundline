# Contract: Negotiated Goal Surface

**Feature**: 026-goal-constraint-modeling  
**Date**: 2026-05-01

## Purpose

Define the bounded operator-facing surface for negotiated goal intake before
planning begins.

## Required Surface

- `goal` output must identify that a negotiated delivery packet now exists
  for the active session.
- The surface must expose the normalized requested outcome, the active
  acceptance boundary, and the current binding constraints.
- When the packet is not yet credible, the surface must name the clarification
  or conflict that blocks planning.
- Goal-only intake must still produce explicit defaults rather than omitting
  the negotiated story.

## Explicit Boundaries

- The goal surface must not hide negotiation inside planner internals.
- The surface must not imply that planning may proceed when required constraints
  remain materially ambiguous or contradictory.
- The goal surface must not invent a second workflow or background loop just
  to collect negotiation state.
