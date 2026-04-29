# Session Run Routing Contract

**Feature**: 014-native-loop-integration  
**Date**: 2026-04-29

## Overview

This contract defines how `synod run` selects between the native decision-loop path and the fixture compatibility path.

## Routing Priority

| Priority | Condition | Selected Mode | Required Operator Visibility |
|----------|-----------|---------------|------------------------------|
| 1 | Session has persisted `goal_plan` and no unresolved native-path block | Native decision loop | CLI and trace indicate native routing |
| 2 | Operator explicitly requests declarative compatibility path | Fixture compatibility | CLI reports explicit compatibility routing |
| 3 | No `goal_plan`, explicit execution profile available | Fixture compatibility | CLI reports declarative routing |
| 4 | Goal captured but no `goal_plan` | Blocked | CLI instructs operator to run planning first |
| 5 | Goal plan exists but flow proposal still requires confirmation | Blocked | CLI explains how to confirm or skip flow |
| 6 | No usable execution context | Blocked | CLI returns explicit remediation |

## Native Execution Contract

When native routing is selected:

1. `DecisionLoop` owns next-action selection.
2. Actions are dispatched through registered runtime adapters rather than direct in-loop filesystem or process calls.
3. The resulting decisions are written to both session state and trace output.
4. Terminal status is reported as a session-visible result with inspectable evidence.

## Compatibility Contract

When fixture compatibility is selected:

1. Existing declarative execution behavior remains unchanged.
2. The route is explicit, not implicit or silent.
3. Compatibility mode does not overwrite native planning state unless the operator chooses to reset the session.

## Inspectability Contract

The developer must be able to answer all of the following from CLI state and traces:

- Did `run` use the native path or fixture compatibility?
- Was a goal plan present at the time of routing?
- Was a flow confirmed, proposed, or absent?
- Which decisions were chosen, executed, verified, failed, or recovered?
- Why did the run terminate?
