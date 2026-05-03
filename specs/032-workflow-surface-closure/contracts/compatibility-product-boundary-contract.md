# Contract: Compatibility Product Boundary

**Feature**: 032-workflow-surface-closure  
**Date**: 2026-05-02

## Intent

Workflow and direct native execution must read as one primary Boundline product
story, while explicit compatibility execution remains a visible subordinate
route.

## Scenarios

### 1. Named workflows stay on the primary Boundline story

**Given** an operator starts work through `boundline workflow run`  
**When** follow-through output or assistant guidance is rendered  
**Then** the result must describe a primary Boundline workflow on the same
session-native model used by direct native execution.

### 2. Compatibility remains explicit and subordinate

**Given** an operator explicitly chose `run --compatibility`  
**When** follow-up guidance is rendered later  
**Then** the output must preserve compatibility ownership explicitly and must
not imply that workflows or direct native runs are compatibility-backed by
default.

### 3. Canon remains visible but secondary

**Given** governance cues appear during workflow or native follow-through  
**When** the operator reads assistant guidance or runtime summaries  
**Then** Canon participation must remain visible as bounded governance inside
Boundline rather than as a competing product surface.

## Acceptance Notes

- Product identity wording must stay consistent across runtime output,
  assistant assets, docs, roadmap, and changelog.
- Compatibility continuity cues remain authoritative only when the operator
  explicitly chose the compatibility route.
- The contract should be validated by workflow compatibility tests plus
  assistant continuity or documentation tests.