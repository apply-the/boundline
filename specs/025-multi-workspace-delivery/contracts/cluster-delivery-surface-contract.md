# Contract: Cluster Delivery Surface

**Feature**: 025-multi-workspace-delivery  
**Date**: 2026-05-01

## Purpose

Define the bounded operator-facing surface for one clustered delivery story.

## Required Surface

- The clustered delivery path must expose one authoritative route owner for the
  overall run.
- Clustered run output must identify the authoritative workspace context for the
  current or terminal step.
- When more than one member workspace participates, the operator-facing surface
  must say so explicitly.
- Clustered non-success outcomes must remain explicit about the blocking or
  failed workspace instead of collapsing into a generic cluster failure.

## Explicit Boundaries

- Clustered execution must not silently create unrelated orchestration owners per
  repository.
- The clustered delivery surface must not imply parallel or background fan-out.
- Single-workspace delivery output must remain valid when no clustered path is
  requested.