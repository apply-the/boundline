# Contract: Cluster Follow-Up Authority

**Feature**: 025-multi-workspace-delivery  
**Date**: 2026-05-01

## Purpose

Define how clustered follow-up surfaces preserve explicit authority after a
clustered run.

## Required Surface

- `status`, `next`, and `inspect` must identify which route owns the clustered
  follow-up story.
- The follow-up surface must name the authoritative workspace context whenever a
  clustered run touched more than one member workspace.
- Inspect-only clustered follow-up must remain explicit when no resumable
  continuation exists.
- Recommended next action must stay aligned with the authoritative route and
  workspace context.

## Explicit Boundaries

- Follow-up surfaces must not imply that another workspace is resumable when the
  authoritative state is inspect-only.
- Cluster-wide summaries must not hide the workspace responsible for a blocked
  or failed condition.
- Compatibility authority, when present, must stay explicit rather than being
  promoted silently into native clustered ownership.