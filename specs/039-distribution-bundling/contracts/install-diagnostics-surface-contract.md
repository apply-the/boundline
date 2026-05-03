# Contract: Install Diagnostics Surface

**Feature**: 039-distribution-bundling  
**Date**: 2026-05-03

## Purpose

Define how Boundline exposes install, version, Canon pairing, and repair state to
an operator after using an official package channel or the source-install
fallback.

## Required Surface

- `doctor` must support an install-focused verification path that does not
  require a workspace.
- Install diagnostics must report the running Boundline version, the supported
  Canon companion version or requirement, and an explicit pairing state.
- Install diagnostics must emit stable check identifiers for missing Boundline
  prerequisites, missing Canon companion state, and Canon version mismatch.
- When repair is possible, install diagnostics must emit bounded next actions
  aligned with the active supported channel or fallback path.

## Explicit Boundaries

- The surface must not imply that Canon owns Boundline orchestration behavior.
- The surface must not hide whether the install is ready, blocked, or in a
  repair-needed state.
- The surface must not require a repository checkout just to verify an
  installed package.