# Contract: Adaptive Credibility Summary Surface

**Feature**: 023-broaden-bounded-adaptive-repair  
**Date**: 2026-05-01

## Purpose

Define which adaptive credibility decisions must be visible through run, session, and trace summaries.

## Required Surface

- `run`, `status`, `next`, and `inspect` must expose the latest adaptive selection headline and candidate credibility reason when a bounded adaptive candidate was chosen.
- When bounded alternatives were rejected, the runtime must surface enough summary evidence to explain why the current candidate was preferred.
- Credibility summaries must stay aligned with the existing route plus `execution_condition` vocabulary used across native and compatibility surfaces.

## Explicit Boundaries

- Summary alignment must not hide that adaptive execution still ran on the explicit compatibility route.
- Credibility summaries must not imply that workflows, review, or Canon selected the adaptive candidate when they did not.
- Missing credibility evidence must result in an explicit stop or omission, not invented explanatory text.