# Contract: Framework Adapter Stdio Protocol

## Purpose

Define the initial host-to-adapter protocol used by Boundline to discover
capabilities, validate config, execute claimed stages, and deliver declared
hooks. The protocol is intentionally bounded and sequential: one-shot subprocess
commands, JSON over stdin/stdout, and no background daemon.

## Transport Rules

- The adapter is a trusted local subprocess launched from the command persisted
  in `.boundline/config.toml`.
- Every protocol interaction is a one-shot process invocation.
- Requests and responses are UTF-8 JSON.
- The adapter reads its request from stdin when the command expects a payload.
- The adapter writes exactly one JSON response document to stdout.
- Human-oriented diagnostics go to stderr only.
- Exit code `0` means the adapter emitted a protocol-valid response.
- Any non-zero exit code is treated as a transport failure by the host.

## Host-Owned Catalog Rules

- Stage IDs are owned by Boundline.
- Hook IDs are owned by Boundline.
- The adapter may only claim or subscribe to IDs that the host recognizes.
- Unknown or unsupported IDs invalidate the capability manifest and block
  activation before stage execution begins.

## Commands

### 1. `describe`

**Invocation**:

```bash
boundline-adapter-speckit describe
```

**Request body**: none

**Response shape**:

```json
{
  "protocol_line": "framework-adapter-v1",
  "adapter_id": "speckit",
  "adapter_version": "0.1.0",
  "supported_boundline_range": ">=0.66.0,<0.67.0",
  "declared_stage_overrides": ["plan", "run"],
  "declared_hook_subscriptions": ["stage_completed", "stage_failed"],
  "required_config_fields": [
    {
      "field_key": "template_repo",
      "display_label": "Template repository",
      "value_kind": "path",
      "required": true,
      "secret": false,
      "prompt_text": "Path to the reusable template repo",
      "help_text": "Point this at ../boundline-framework-template or another checked-out template repo",
      "non_interactive_policy": "fail"
    }
  ]
}
```

**Host guarantees**:

- the host validates the protocol line, version range, stage IDs, hook IDs, and
  field definitions before activation
- malformed JSON or missing required fields in the response block activation

### 2. `preflight`

**Invocation**:

```bash
boundline-adapter-speckit preflight
```

**Request shape**:

```json
{
  "boundline_version": "0.66.0",
  "workspace_ref": "../tmp/example-workspace",
  "non_interactive": false,
  "config_values": [
    {
      "field_key": "template_repo",
      "value_kind": "path",
      "path_value": "../boundline-framework-template"
    }
  ]
}
```

**Response shape**:

```json
{
  "status": "ready",
  "normalized_config_values": [
    {
      "field_key": "template_repo",
      "value_kind": "path",
      "path_value": "../boundline-framework-template"
    }
  ],
  "warnings": []
}
```

**Blocked response example**:

```json
{
  "status": "blocked",
  "reason": "missing_required_config",
  "missing_fields": ["template_repo"],
  "recovery": "boundline adapter add speckit --workspace <workspace>"
}
```

**Host guarantees**:

- adapter-owned stage execution cannot start unless `preflight.status = ready`
- `blocked` preflight results are surfaced to the operator before stage claim
- `non_interactive = true` forbids prompt-only recovery inside the adapter

### 3. `execute-stage`

**Invocation**:

```bash
boundline-adapter-speckit execute-stage
```

**Request shape**:

```json
{
  "run_id": "b1d1d3c2-7f6d-4d8c-9f57-6e57fd2d1d02",
  "stage_key": "plan",
  "stage_attempt": 1,
  "workspace_ref": "../tmp/example-workspace",
  "adapter_id": "speckit",
  "config_values": [
    {
      "field_key": "template_repo",
      "value_kind": "path",
      "path_value": "../boundline-framework-template"
    }
  ],
  "context_artifacts": [
    "specs/066-agentic-framework-integration/spec.md"
  ]
}
```

**Response shape**:

```json
{
  "status": "succeeded",
  "summary": "Plan artifacts refreshed through the Speckit profile",
  "produced_artifacts": [
    "specs/066-agentic-framework-integration/plan.md",
    "specs/066-agentic-framework-integration/tasks.md"
  ],
  "next_action": null
}
```

**Blocked response example**:

```json
{
  "status": "blocked",
  "summary": "Operator input is required before the claimed stage can continue",
  "next_action": "Update the adapter config and retry the stage"
}
```

**Failed response example**:

```json
{
  "status": "failed",
  "summary": "Speckit could not complete the claimed stage",
  "failure_class": "adapter_runtime",
  "next_action": "Inspect the adapter log and retry after correction"
}
```

**Host guarantees**:

- once the host has routed a stage to the adapter, `failed` becomes a failed
  stage in Boundline and requires operator intervention
- `blocked` may map to existing Boundline host handoff or phase-request surfaces,
  but the stage remains incomplete and owned by the adapter until the operator
  resolves it or removes the adapter

### 4. `emit-hook`

**Invocation**:

```bash
boundline-adapter-speckit emit-hook
```

**Request shape**:

```json
{
  "run_id": "b1d1d3c2-7f6d-4d8c-9f57-6e57fd2d1d02",
  "hook_key": "stage_completed",
  "stage_key": "plan",
  "stage_claimed": true,
  "workspace_ref": "../tmp/example-workspace",
  "payload_ref": ".boundline/traces/<trace-id>.json"
}
```

**Response shape**:

```json
{
  "status": "delivered",
  "summary": "Hook processed successfully"
}
```

**Failure semantics**:

- when `stage_claimed = false`, hook failures remain observable warnings and do
  not retroactively fail a built-in stage
- when `stage_claimed = true`, the host may incorporate a failed hook into the
  current stage's failure classification

## Protocol Invariants

- The adapter must report the same `adapter_id` in `describe` that the host uses
  in config and audit records.
- The `protocol_line` and `supported_boundline_range` must be present on every
  `describe` response.
- The host may cache one validated capability snapshot per lifecycle run, but it
  must re-run `describe` on the next run.
- Secret config fields may appear in requests when needed, but they must not be
  echoed back in clear text unless the host explicitly marks that channel safe.
- The adapter must never assume that PATH discovery equals activation; it only
  becomes active after host registration succeeds.