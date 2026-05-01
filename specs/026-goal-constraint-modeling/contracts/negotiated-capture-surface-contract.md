# Contract: Negotiated Capture Surface

**Feature**: 026-goal-constraint-modeling  
**Date**: 2026-05-01

## Purpose

Define the bounded operator-facing surface for negotiated capture before
planning begins.

## Required Surface

- `capture` output must identify that a negotiated delivery packet now exists
  for the active session.
- The surface must expose the normalized requested outcome, the active
  acceptance boundary, and the current binding constraints.
- When the packet is not yet credible, the surface must name the clarification
  or conflict that blocks planning.
- Goal-only capture must still produce explicit defaults rather than omitting
  the negotiated story.

## Explicit Boundaries

- The capture surface must not hide negotiation inside planner internals.
- The surface must not imply that planning may proceed when required constraints
  remain materially ambiguous or contradictory.
- The capture surface must not invent a second workflow or background loop just
  to collect negotiation state.