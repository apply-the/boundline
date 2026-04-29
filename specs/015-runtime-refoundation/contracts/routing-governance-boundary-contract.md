# Routing And Governance Boundary Contract

**Feature**: 015-runtime-refoundation  
**Date**: 2026-04-29

## Overview

This contract defines how Synod selects between the session-native runtime path and the explicit compatibility path, and how Canon artifacts may participate without becoming the orchestration control plane.

## Routing Precedence

| Priority | Condition | Selected Mode | Required Operator Visibility |
|----------|-----------|---------------|------------------------------|
| 1 | Operator explicitly selects compatibility mode | compatibility | CLI explains that compatibility was chosen deliberately |
| 2 | Session has a confirmed or flow-skipped bounded task draft | native | CLI and trace identify session-native routing |
| 3 | Session has a proposed-but-unconfirmed flow | blocked | CLI explains how to confirm or skip flow before policy-bound execution |
| 4 | Declarative execution profile exists and no session-native bounded task draft exists | compatibility | CLI explains that declarative compatibility is driving the run |
| 5 | No credible execution context exists | blocked | CLI returns explicit remediation instead of silent fallback |

## Native Route Contract

When `native` routing is selected:

1. Synod owns next-action selection from live evidence.
2. Flow constraints apply only when flow is explicitly confirmed.
3. Decisions are persisted to session and trace state.
4. Canon may contribute bounded planning or stage-boundary evidence but does not choose each decision.

## Compatibility Route Contract

When `compatibility` routing is selected:

1. Declarative execution behavior remains available for explicit profiles.
2. Compatibility mode does not silently overwrite session-native planning state.
3. Session output still identifies that the run followed compatibility behavior.

## Canon Boundary Rules

1. Canon artifacts MAY be consumed during planning as bounded evidence inputs.
2. Canon artifacts MAY influence stage-boundary governance decisions when the current route and stage require that evidence.
3. Canon MUST NOT become the per-action runtime selector for bounded decisions.
4. Missing Canon artifacts MUST NOT prevent core session-native planning or execution unless an explicit stage-boundary governance rule requires them.

## Inspectability Contract

The operator must be able to answer all of the following from status, next, or inspect surfaces:

- which route was chosen and why
- whether a flow was proposed, confirmed, skipped, or absent
- whether Canon artifacts were used as bounded evidence
- whether the runtime was blocked because context was missing or confirmation was pending
- why the run terminated or stopped progressing