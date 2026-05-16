# Contract: Routing Decision Surface

**Feature**: 027-routing-assistant-decoupling  
**Date**: 2026-05-01

## Intent

The active route for a bounded delivery slot must be visible on the same read
surfaces operators already use to understand execution and follow-up state.

## Scenarios

### 1. Session-native run exposes the active route

**Given** workspace, cluster, global, and built-in routing defaults can all
contribute to the effective route  
**When** `cargo run --bin boundline -- run` succeeds or
stops in a bounded terminal condition  
**Then** the runtime summary must expose:
- the active bounded slot or routing headline
- the selected runtime and model
- the authoritative config source
- `route_owner`
- `route_config_projection` or an equivalent concise explanation of why the
  route is authoritative for the current run

### 2. Follow-up surfaces repeat the same route story

**When** the operator runs `status`, `next`, or workspace-based `inspect` after
planning or execution  
**Then** the output must preserve the same routing facts instead of forcing the
operator to call a separate routing command.

### 3. Route visibility survives non-success terminal outcomes

**When** the active step ends as blocked, failed, or needs clarification  
**Then** the follow-up surface must still expose the route that owned the step,
including source and route owner, so the operator can diagnose whether the
behavior followed configured intent.

## Acceptance Notes

- Route visibility must work for session-native and explicit compatibility
  follow-up states.
- The projection must stay compact enough for the existing CLI summary style.
- The contract is satisfied only when the operator can tell, from runtime
  output alone, which configured route owned the current bounded step.