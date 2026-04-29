# Session Planning Contract

**Feature**: 014-native-loop-integration  
**Date**: 2026-04-29

## Overview

This contract defines what the session CLI must persist and report when native planning succeeds.

## Planning Inputs

| Field | Source | Required | Notes |
|-------|--------|----------|-------|
| `goal` | active session | Yes | Planning requires a captured goal |
| `authored_brief` | active session | No | Used when present to enrich planning context |
| `active_flow` | active session | No | Explicit operator-selected flow remains authoritative |
| workspace signals | filesystem | Yes | Derived from the current workspace |
| Canon artifacts | `.canon/` | No | Used only as bounded evidence inputs |

## Planning Outputs

| Field | Destination | Required | Contract |
|-------|-------------|----------|----------|
| `goal_plan` | session record | Yes | Non-empty bounded plan after successful planning |
| `goal_plan.flow` | session record | No | Present when explicit or inferred flow is available |
| `active_flow_policy` | session record | No | Present only when flow is explicitly confirmed |
| `latest_status` | session record | Yes | Set to `planned` after successful native planning |
| `latest_trace_ref` | session record | Optional | May be populated if planning emits trace artifacts |
| planning summary | CLI output | Yes | Must tell the operator whether flow is confirmed, proposed, or absent |

## Flow Confirmation Rules

1. If the operator supplies an explicit flow during planning, the stored flow is confirmed.
2. If the operator explicitly disables flow, the stored plan records no active flow constraint.
3. If Synod infers a flow without explicit confirmation, the plan stores the proposal but execution may not silently treat it as confirmed.
4. A previously confirmed session flow remains authoritative unless the operator explicitly resets or replaces it.

## Failure Contract

- Planning without a captured goal MUST return an explicit remediation error.
- Planning blocked by unresolved authored-input clarification MUST preserve the session and return a next-step message.
- Planning that cannot derive a bounded plan MUST fail explicitly instead of creating an empty or placeholder session plan.
