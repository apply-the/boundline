# Contract: Dashboard Snapshot

## Purpose

Define the stable snapshot shape the dashboard consumes to render Boundline runtime truth without parsing human-readable command output.

## Authority

- Boundline session state remains authoritative for active delivery state.
- Boundline trace state remains authoritative for historical execution events.
- Checkpoint, finding, guidance, guardian, and governed-reference facts remain authoritative only through the existing Boundline projections that already expose them.
- The dashboard snapshot is a projection, not a new source of truth.

## Snapshot Shape

```json
{
  "snapshot_id": "snapshot-uuid",
  "workspace_ref": "/workspace",
  "captured_at": "2026-05-19T00:00:00Z",
  "authority": "session_native",
  "session_revision": 3,
  "session": {
    "session_id": "session-uuid",
    "goal": "Fix the failing checkout flow",
    "route_kind": "native_goal_plan",
    "route_owner": "runtime",
    "active_flow": "bug-fix",
    "flow_state": "confirmed",
    "goal_plan_state": "confirmed",
    "goal_plan_revision": 1,
    "current_stage": "verify",
    "current_step_id": "run-tests",
    "current_step_index": 2,
    "execution_condition": "ready",
    "latest_status": "planned",
    "next_action_label": "Continue bounded execution",
    "next_command": "boundline run",
    "blocking_reason": null,
    "compatibility_context": null
  },
  "timeline": [
    {
      "event_id": "event-1",
      "event_kind": "plan",
      "occurred_at": "2026-05-19T00:00:00Z",
      "stage": "plan",
      "step_id": "build-goal-plan",
      "status": "succeeded",
      "headline": "Goal plan confirmed",
      "evidence_refs": ["session:goal_plan"],
      "trace_ref": null,
      "details": ["selected target src/lib.rs"]
    }
  ],
  "panels": {
    "goal_plan": {
      "revision": 1,
      "state": "confirmed",
      "verification_strategy": "run targeted verification",
      "targets": ["src/lib.rs"]
    },
    "evidence": [],
    "context_degradation": [],
    "stop_rules": [],
    "findings": [],
    "checkpoints": [],
    "governed_references": []
  },
  "actions": [
    {
      "action_kind": "continue",
      "label": "Run",
      "description": "Continue bounded execution",
      "requires_reason": false,
      "requires_confirmation": true,
      "target_session_revision": 3,
      "expected_result": "running_or_terminal",
      "disabled_reason": null
    }
  ],
  "degraded_state": null,
  "branding": {
    "wordmark_lines": ["boundline"],
    "color_profile": "color",
    "min_width": 20,
    "fallback_label": "boundline"
  }
}
```

## Required Behavior

- The snapshot must be serializable for contract fixtures.
- Missing optional panels must be represented as empty lists or explicit unavailable states, not as omitted unknowns.
- Mutating actions must include the target session revision.
- Compatibility context must be explicit when the snapshot is driven by compatibility trace follow-up rather than active session-native state.
- Governed references must never include write actions.

## Invalid Snapshot Cases

- Snapshot has no session and no degraded state.
- Snapshot exposes a mutating action without a target session revision.
- Snapshot marks a stopped, blocked, exhausted, or invalid session as ready without a blocking reason.
- Snapshot hides unavailable governed references while still implying governed evidence was loaded.
