# Contract: Compatibility And Cluster Routing Boundaries

**Feature**: 027-routing-assistant-decoupling  
**Date**: 2026-05-01

## Intent

Routing transparency must preserve the existing ownership boundaries for
explicit compatibility follow-up and clustered delivery.

## Scenarios

### 1. Compatibility follow-up keeps route ownership explicit

**Given** the latest actionable state comes from an explicit compatibility trace  
**When** the operator runs `status`, `next`, or `inspect`  
**Then** the output must preserve `continuity_authority`, `route_owner`, and the
trace-owned routing explanation instead of implying that a resumable native
session owns the route.

### 2. Clustered delivery keeps the primary workspace authoritative

**Given** the workspace participates in a registered cluster  
**When** the operator runs clustered `start`, `run`, `status`, `next`, or
`inspect` flows  
**Then** routing visibility must preserve the primary workspace as the
authoritative owner while still exposing the active route and assistant binding.

### 3. Follow-up guidance does not collapse boundary-specific commands

**When** the CLI provides a compatibility- or cluster-specific `next_command` or
`corrected_command`  
**Then** routing projection must reinforce that command instead of replacing it
with a generic workspace-scoped recommendation.

## Acceptance Notes

- Route transparency is only correct if ownership boundaries remain clear.
- The feature must not regress existing cluster or compatibility continuity
  cues.
- Tests should cover both workspace-scoped and trace-scoped inspection paths.