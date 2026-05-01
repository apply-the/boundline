# Contract: Route Ownership Preservation

**Feature**: 024-unify-route-summaries  
**Date**: 2026-05-01

## Purpose

Define the ownership guarantees that must survive summary-model convergence.

## Required Surface

- Every aligned follow-up surface must explicitly state which route currently owns follow-up.
- Workflow, review, and governance states may reuse the shared summary vocabulary while still naming the bounded owner that controls the next action.
- Compatibility summaries must preserve inspect-only or trace-authority guidance when no active session is resumable.
- Continuity authority must remain visible whenever the latest authoritative state differs from the route used to start work.

## Explicit Boundaries

- Native, workflow, and compatibility ownership must not collapse into one generic owner label.
- Route ownership must not be inferred only from missing fields or absent commands.
- Converged wording must not hide when the correct next operator action is route-specific.
