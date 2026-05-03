# Contract: Adaptive Route Guidance

**Feature**: 021-adaptive-repair-depth  
**Date**: 2026-05-01

## Purpose

Define how adaptive compatibility execution must be explained when the same workspace also exposes session-native workflows, review, or governance surfaces.

## Route Guarantees

- Adaptive execution remains an explicit compatibility path backed by `.boundline/execution.json` for this slice.
- Session-native workflows remain available when invoked, but they must not be described as owning an active adaptive compatibility run.
- Review or governance projection may appear in the same summaries when configured, but those surfaces must remain additive and must not replace the adaptive route explanation.

## Unsupported Expectations

- No hidden promotion of adaptive execution to the primary session-native route in this release.
- No workflow-owned adaptive control flow.
- No Canon-owned adaptive replanning or mutation decisions.