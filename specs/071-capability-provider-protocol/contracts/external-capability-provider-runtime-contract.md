# Contract: External Capability Provider Runtime Projection

## Purpose

Define the additive Boundline-owned runtime contract for external capability
provider discovery, registration, activation, bounded execution, evidence
collection, and fail-closed validation.

## Lifecycle Order

```text
provider discovered
  -> operator registration
  -> activation attempt
  -> health
  -> prepare
  -> execute
  -> collect_evidence
  -> validation disposition
  -> status / inspect / trace projection
```

Discovery is informational only. Registration and activation are explicit
operator-controlled state transitions.

## Protocol Calls

### `capabilities`

Returns the provider's declared identity and capability metadata.

Required response fields:

- `provider_id`
- `protocol_line`
- `protocol_version`
- `capabilities[]`
- `supported_lifecycle_phases`
- `supported_inputs`
- `supported_outputs`
- `mutation_support`
- `required_permissions`
- `evidence_formats`

### `health`

Returns the provider's readiness state before Boundline attempts execution.

Required response fields:

- `provider_id`
- `readiness_state`
- `missing_dependencies[]`
- `warnings[]`
- `runtime_environment`

### `prepare`

Returns pre-execution context and evidence expectations.

Required response fields:

- `request_id`
- `required_context_refs[]`
- `optional_context_refs[]`
- `missing_evidence_refs[]`
- `expected_artifacts[]`
- `risk_observations[]`
- `estimated_cost_or_runtime`

### `execute`

Consumes one bounded execution request and returns one structured result.

Required request fields:

- `request_id`
- `session_ref`
- `step_or_stage_ref`
- `capability_id`
- `goal_summary`
- `lifecycle_phase`
- `authority_zone`
- `context_pack_refs[]`
- `permission_envelope`
- `expected_outputs[]`

Required response fields:

- `request_id`
- `status`
- `observations[]`
- `findings[]`
- `artifact_refs[]`
- `evidence_refs[]`
- `state_patch_proposals[]`
- `limitations[]`
- `next_actions[]`

### `collect_evidence`

Normalizes provider results into Boundline-owned evidence records after
execution.

Required response fields:

- `request_id`
- `claims[]`
- `evidence_refs[]`
- `artifact_refs[]`
- `findings[]`
- `limitations[]`
- `reproducibility_metadata`

## Permission Envelope Rules

Every provider-backed execution request must carry an explicit permission
envelope with these logical fields:

- `read_files`
- `write_files`
- `run_commands`
- `network`
- `read_secrets`
- `write_artifacts`
- `allowed_paths`
- `max_runtime_ms`
- `max_output_bytes`

Default policy is least privilege. Missing permission fields are invalid. A
provider may request broader permissions during `prepare`, but Boundline must
either reject the request or require a new explicit operator-approved admission
decision before execution.

## Additive Runtime Projection

When the provider protocol has run, session, status, inspect, and machine
output may include additive fields like:

```json
{
  "capability_provider_projection": {
    "provider_id": "browser-provider",
    "activation_state": "active",
    "capability_id": "browser.fetch_dom",
    "failure_class": "post_execution_validation",
    "validation_disposition": "blocked",
    "limitations": [
      "provider could not supply reproducible evidence refs"
    ],
    "accepted_evidence_refs": [],
    "rejected_evidence_refs": [
      "provider://browser-provider/request-42/evidence/dom-snapshot"
    ]
  }
}
```

These fields are additive. Older snapshots may omit them entirely.

## Blocking Rules

Boundline must block provider-backed execution before `execute` when any of
these conditions is true:

- the provider is unregistered
- the provider is inactive
- the provider is unhealthy or unavailable
- the provider capability declaration is missing or incompatible
- the requested lifecycle phase is unsupported
- required permission fields are missing
- the granted permission envelope is narrower than what the provider declares
  it requires and no accepted degraded path exists
- provider metadata, profile metadata, and Boundline runtime policy conflict on
  capability identity, lifecycle phase support, permissions, or evidence
  requirements

## Validation Rules

- Provider output is always a claim, not truth.
- Evidence and artifacts may be accepted only after Boundline records a
  validation disposition.
- State patch proposals are proposals only; they never directly mutate
  Boundline-owned state.
- `collect_evidence` output must preserve limitations and reproducibility
  metadata regardless of whether the final disposition is accepted or rejected.
- Canon-governed evidence may be consumed as a context or evidence input, but
  Canon does not participate in provider activation or validate provider output
  on Boundline's behalf.

## Conflict Rule

When provider metadata, specialized profile metadata, and Boundline runtime
policy disagree, the stricter Boundline runtime policy wins. If the conflict
affects permissions, capability identity, lifecycle phase support, or evidence
requirements, provider-backed execution must fail closed before execution
starts.

## Minimal Observability

Provider-backed execution must emit or project enough structured state to
distinguish:

- readiness failure
- permission admission failure
- execution failure
- post-execution validation failure
- accepted provider evidence
- rejected provider evidence
- provider limitations

## Assistant Asset Contract

Copilot, Claude, Codex, and Antigravity plan, run, status, and inspect assets
must preserve:

- provider registration state
- activation status
- selected capability
- failure class
- validation disposition
- evidence refs
- provider limitations
- repair or continuation guidance

Assistant assets must not synthesize a provider-ready or provider-accepted
state when the runtime has blocked or rejected the provider lifecycle.

## Explicit Non-Goals

- No concrete browser, sandbox, RecursiveMAS, or static-analysis provider
  implementation in this slice
- No route economics or provider benchmarking
- No automatic activation from discovery
- No direct provider mutation of Boundline-owned state
- No full sandbox enforcement or secret-inheritance policy beyond the explicit
  permission envelope
