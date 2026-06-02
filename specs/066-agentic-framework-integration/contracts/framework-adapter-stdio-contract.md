# Contract: Framework Adapter Stdio Protocol

## Purpose

Define the initial host-to-adapter protocol used by Boundline to discover
capabilities, validate config, execute claimed stages, and deliver declared
hooks. The protocol is intentionally bounded and sequential: one-shot subprocess
commands, JSON over stdin/stdout, a standard host-visible success or error
response envelope, optional best-effort structured stderr, and no background
daemon.

## Transport Rules

- The adapter is a trusted local subprocess launched from the command persisted
  in `.boundline/config.toml`.
- Every protocol interaction is a one-shot process invocation.
- Requests and response envelopes are UTF-8 JSON.
- The adapter reads its request from stdin when the command expects a payload.
- The adapter writes exactly one JSON response envelope to stdout.
- Human-oriented diagnostics go to stderr only.
- If the adapter emits structured stderr, it should write one self-contained
  JSON object per line so the host may ingest those lines into trace records.
- Structured stderr is optional in V1 and never replaces the stdout response
  envelope.
- Exit code `0` means the adapter emitted a protocol-valid response envelope.
- Any non-zero exit code is treated as a transport failure by the host.

## Standard Response Envelope

Every stdout response uses the same host-visible envelope.

**Success envelope**:

```json
{
  "success": true,
  "data": {
    "status": "ready"
  }
}
```

**Error envelope**:

```json
{
  "success": false,
  "error": {
    "code": "unsupported_transport",
    "message": "The adapter did not declare a V1-supported transport",
    "details": {
      "supported_transports": ["socket-json"]
    }
  }
}
```

**Envelope rules**:

- command-specific domain outcomes such as `blocked`, `failed`, and
  `delivered` remain inside `data`
- `success = false` is reserved for protocol-level or request-validation
  failures that still returned a protocol-valid envelope on exit code `0`
- when `success = true`, `error` must be absent
- when `success = false`, `data` must be absent

## Optional Structured Stderr

If the adapter chooses to emit structured diagnostics on stderr, each line
should be a standalone JSON object.

**Example stderr line**:

```json
{"severity":"warn","code":"template_repo_missing","message":"Configured template repo path was not found"}
```

These diagnostics are optional. Boundline may capture them into traces when
parseable, but malformed or plain-text stderr must not invalidate an otherwise
valid stdout response envelope.

## Host-Owned Catalog Rules

- Stage IDs are owned by Boundline.
- Hook IDs are owned by Boundline.
- The adapter may only claim or subscribe to IDs that the host recognizes.
- Unknown or unsupported IDs invalidate the capability manifest and block
  activation before stage execution begins.

## Normative Known-Speckit Stage Mapping

For the corrected Spec 066 slice, the known Speckit profile uses the following
host-owned stage map:

| Boundline surface | Ownership rule | Workflow ID | Command surface |
|-------------------|----------------|-------------|-----------------|
| `goal` | Boundline-owned only; the adapter must not claim it | `boundline-native-goal` | Native Boundline goal capture only |
| `plan` | May be adapter-owned when `plan` is declared and preflight is ready | `speckit-planning` | `speckit.specify`; `speckit.clarify` when required; `speckit.plan`; `speckit.tasks`; mandatory `speckit.analyze`; bounded remediation plus analyze re-checks |
| `run` | May be adapter-owned when `run` is declared and preflight is ready | `speckit-implementation` | `speckit.implement` plus implementation validation or status capture only |
| `status` / `inspect` | Boundline-owned only; the adapter contributes evidence but does not own the surface | `boundline-native-visibility` | Native Boundline visibility over adapter ownership, findings, traces, and validation refs |

Within one claimed `plan` attempt, the known Speckit bridge inherits the
host's existing retry and stop controls and may execute at most one initial
analyze pass plus two remediation or analyze re-check cycles. If blocking
findings still remain after the second re-check, the adapter must return a
blocked stage outcome with the remaining findings and a recovery action.

The corrected workflow assets live at
`.specify/workflows/speckit/planning.yml` and
`.specify/workflows/speckit/implementation.yml`. When the real Speckit CLI is
invoked, the bridge may execute those local YAML paths directly through
`specify workflow run <path>` while still returning the semantic workflow IDs
`speckit-planning` and `speckit-implementation` in the response payload.
Those YAML assets are the workflow entrypoints only. The adapter bridge remains
the authoritative layer for conditional clarify execution, the mandatory
planning-readiness analyze loop, bounded remediation or analyze re-check
cycles, and run-stage validation or status capture that the response payload
must expose to Boundline.

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
  "success": true,
  "data": {
    "protocol_line": "framework-adapter-v1",
    "adapter_id": "speckit",
    "adapter_version": "0.1.0",
    "supported_boundline_range": ">=0.66.0,<0.67.0",
    "supported_transports": [
      {
        "transport": "stdio",
        "encoding": "json",
        "request_channel": "stdin",
        "response_channel": "stdout"
      }
    ],
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
}
```

**Host guarantees**:

- the host validates the protocol line, version range, supported transports,
  stage IDs, hook IDs, and field definitions before activation
- missing `supported_transports`, or a list that does not include the V1 JSON
  over stdin/stdout declaration, blocks activation
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
  "success": true,
  "data": {
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
}
```

**Blocked response example**:

```json
{
  "success": true,
  "data": {
    "status": "blocked",
    "reason": "missing_required_config",
    "missing_fields": ["template_repo"],
    "recovery": "boundline adapter add speckit --workspace <workspace>"
  }
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

**Plan-stage response shape** (`stage_key = plan`):

```json
{
  "success": true,
  "data": {
    "status": "succeeded",
    "summary": "Speckit planning workflow completed with no blocking findings",
    "workflow_id": "speckit-planning",
    "executed_commands": [
      "speckit.specify",
      "speckit.plan",
      "speckit.tasks",
      "speckit.analyze"
    ],
    "produced_artifacts": [
      "specs/066-agentic-framework-integration/spec.md",
      "specs/066-agentic-framework-integration/plan.md",
      "specs/066-agentic-framework-integration/tasks.md",
      "specs/066-agentic-framework-integration/analysis.md"
    ],
    "planning_findings": {
      "blocking": 0,
      "non_blocking": 2,
      "report_ref": "specs/066-agentic-framework-integration/analysis.md"
    },
    "remediation_status": "not_needed",
    "analyze_pass_count": 1,
    "remediation_cycles_used": 0,
    "next_action": null
  }
}
```

**Plan-stage blocked response example**:

```json
{
  "success": true,
  "data": {
    "status": "blocked",
    "summary": "Planning readiness is still blocked after the bounded remediation budget",
    "workflow_id": "speckit-planning",
    "executed_commands": [
      "speckit.specify",
      "speckit.plan",
      "speckit.tasks",
      "speckit.analyze",
      "remediation",
      "speckit.analyze",
      "remediation",
      "speckit.analyze"
    ],
    "produced_artifacts": [
      "specs/066-agentic-framework-integration/spec.md",
      "specs/066-agentic-framework-integration/plan.md",
      "specs/066-agentic-framework-integration/tasks.md",
      "specs/066-agentic-framework-integration/analysis.md"
    ],
    "planning_findings": {
      "blocking": 2,
      "non_blocking": 1,
      "report_ref": "specs/066-agentic-framework-integration/analysis.md"
    },
    "remediation_status": "limit_reached",
    "analyze_pass_count": 3,
    "remediation_cycles_used": 2,
    "next_action": "Resolve the remaining blocking findings and retry `boundline plan`"
  }
}
```

**Run-stage response shape** (`stage_key = run`):

```json
{
  "success": true,
  "data": {
    "status": "succeeded",
    "summary": "Speckit implementation workflow completed",
    "workflow_id": "speckit-implementation",
    "executed_commands": [
      "speckit.implement",
      "implementation-validation"
    ],
    "produced_artifacts": [
      "src/orchestrator/session_runtime_planning_runtime.rs",
      ".boundline/traces/run-001-implementation.json"
    ],
    "implementation_status": "validated",
    "validation_refs": [
      ".boundline/traces/run-001-implementation.json"
    ],
    "next_action": null
  }
}
```

**Failed response example**:

```json
{
  "success": true,
  "data": {
    "status": "failed",
    "summary": "Speckit could not complete the claimed stage",
    "workflow_id": "speckit-implementation",
    "failure_class": "adapter_runtime",
    "next_action": "Inspect the adapter log and retry after correction"
  }
}
```

**Host guarantees**:

- once the host has routed a stage to the adapter, `failed` becomes a failed
  stage in Boundline and requires operator intervention
- `blocked` may map to existing Boundline host handoff or phase-request surfaces,
  but the stage remains incomplete and owned by the adapter until the operator
  resolves it or removes the adapter
- when `stage_key = plan`, the host must not mark the stage complete while
  `planning_findings.blocking > 0`; the stage remains blocked until the adapter
  returns a succeeded outcome or the operator resolves the blocked findings
- when `stage_key = run`, the adapter must not invoke planning commands such as
  `speckit.specify`, `speckit.clarify`, `speckit.plan`, `speckit.tasks`, or
  `speckit.analyze`

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
  "success": true,
  "data": {
    "status": "delivered",
    "summary": "Hook processed successfully"
  }
}
```

**Failure semantics**:

- when `stage_claimed = false`, hook failures remain observable warnings and do
  not retroactively fail a built-in stage
- when `stage_claimed = true`, the host may incorporate a failed hook into the
  current stage's failure classification

## Deferred Lifecycle Concerns

- V1 accepts only the declared JSON-over-stdin/stdout transport above.
- Graceful shutdown, connection keepalive, and other long-running transport
  lifecycle concerns are deferred to a future protocol line.
- The current one-shot subprocess model avoids orphan-process concerns in this
  slice without requiring additional shutdown handshakes.

## Protocol Invariants

- The adapter must report the same `adapter_id` in `describe` that the host uses
  in config and audit records.
- Every stdout response document must use the standard response envelope.
- The `protocol_line`, `supported_boundline_range`, and
  `supported_transports` fields must be present on every `describe` response.
- `supported_transports` must include at least one entry with
  `transport = stdio`, `encoding = json`, `request_channel = stdin`, and
  `response_channel = stdout` for V1 compatibility.
- The host may cache one validated capability snapshot per lifecycle run, but it
  must re-run `describe` on the next run.
- Secret config fields may appear in requests when needed, but they must not be
  echoed back in clear text unless the host explicitly marks that channel safe.
- Optional structured stderr may enrich traces when parseable, but it never
  substitutes for the stdout response envelope and the host may ignore malformed
  lines without changing the command outcome.
- The adapter must never assume that PATH discovery equals activation; it only
  becomes active after host registration succeeds.
- `execute-stage(plan)` responses for the known Speckit profile must report
  workflow ID `speckit-planning`, the executed command list, planning findings,
  remediation counts, and a planning-readiness artifact reference.
- `execute-stage(run)` responses for the known Speckit profile must report
  workflow ID `speckit-implementation`, the executed command list,
  implementation status, and at least one validation or status artifact
  reference.