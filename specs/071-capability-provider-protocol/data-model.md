# Data Model: External Capability Provider Protocol

## Entity: Capability Provider Registration

Represents an operator-approved provider entry that Boundline may activate and
route capability requests to.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `provider_id` | Stable identifier | Yes | Must be unique within the workspace configuration and remain stable across activation and trace records. |
| `display_name` | Operator-readable label | Yes | Must describe the provider without embedding transport secrets or filesystem-specific details. |
| `transport_kind` | `command` or `http` | Yes | Determines which transport descriptor is required. |
| `registration_source` | Stable label such as `operator_cli`, `guided_setup`, or `migrated_config` | Yes | Keeps onboarding provenance traceable. |
| `discovery_state` | `explicit`, `discovered`, or `unresolved` | Yes | Discovery never implies activation. |
| `activation_state` | `inactive`, `activating`, `active`, `blocked`, or `invalid` | Yes | Only `active` providers are eligible for execution admission. |
| `config_refs` | Ordered list of non-secret config refs | No | Stores validated non-secret setup values. |
| `secret_handle_refs` | Ordered list of opaque handle refs | No | Must never contain raw secret values. |
| `setup_requirements` | Ordered list of `Provider Setup Requirement` | Yes | Records which required and optional setup items still gate activation. |
| `capability_ids` | Ordered list of stable capability identifiers | Yes | Boundline uses these IDs for routing and admission. |
| `active_profile_id` | Optional specialized profile ref | No | Present only when an overlay profile is selected. |

## Entity: Provider Transport Descriptor

Represents the transport-specific details needed to contact a provider.

### Command Transport

| Field | Shape | Required | Rules |
|---|---|---|---|
| `command_ref` | Relative or operator-approved executable reference | Yes | Discovery may populate a candidate, but activation still requires explicit acceptance. |
| `args` | Ordered list | No | Arguments must remain non-secret and inspectable. |
| `working_directory_ref` | Optional relative ref | No | Must not persist absolute host-specific workspace paths in stable files. |
| `environment_ref_names` | Ordered list of env-handle names | No | Names may be persisted; resolved secret values may not. |

### HTTP Transport

| Field | Shape | Required | Rules |
|---|---|---|---|
| `endpoint_ref` | Operator-provided endpoint reference | Yes | Must identify the target without embedding secrets. |
| `auth_scheme` | Stable bounded label | No | Examples include bearer-token handle or signed gateway handle. |
| `headers_ref` | Ordered list of non-secret header refs | No | Secret-bearing headers must be represented only by handle refs. |
| `tls_policy` | Stable bounded label | No | Keeps trust policy explicit for remote endpoints. |

## Entity: Provider Setup Requirement

Represents one operator-visible requirement that must be satisfied before
activation can complete.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `requirement_id` | Stable identifier | Yes | Unique within one provider registration. |
| `kind` | `config_value`, `secret_handle`, `filesystem_ref`, or `connectivity_check` | Yes | Drives guided setup and missing-state reporting. |
| `required_state` | `required` or `optional` | Yes | Missing required items block activation. |
| `resolution_state` | `present`, `missing`, `invalid`, or `unchecked` | Yes | Activation requires all required items to be `present`. |
| `display_label` | Operator-readable label | Yes | Must explain the setup item without exposing secret values. |
| `source_ref` | Optional config or handle ref | No | Present when the requirement maps to persisted non-secret state. |

## Entity: Provider Capability Declaration

Represents the provider-published metadata returned by `capabilities`.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `provider_id` | Stable identifier | Yes | Must match the active registration candidate. |
| `protocol_line` | Stable line identifier | Yes | Identifies the provider protocol family and major compatibility line. |
| `protocol_version` | Version string | Yes | Used for compatibility checks and fail-closed validation. |
| `capability_id` | Stable identifier | Yes | Unique within one provider declaration. |
| `supported_lifecycle_phases` | Ordered list | Yes | Boundline uses this to reject unsupported phase routing before execution. |
| `supported_inputs` | Ordered list of typed input kinds | Yes | Prevents ambiguous request construction. |
| `supported_outputs` | Ordered list of typed output kinds | Yes | Supports validation and evidence expectations. |
| `mutation_support` | `read_only`, `proposal_only`, or `mutating` | Yes | Boundline validation rules still apply regardless of provider claim. |
| `required_permissions` | Ordered list of permission kinds | Yes | Must be subset-checked against Boundline runtime policy at admission time. |
| `evidence_formats` | Ordered list | Yes | Defines what collect-evidence may return. |

## Entity: Provider Health Snapshot

Represents the latest readiness report returned by `health`.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `provider_id` | Stable identifier | Yes | Ties the snapshot to one registration. |
| `readiness_state` | `ready`, `degraded`, or `unavailable` | Yes | `unavailable` blocks provider-backed execution. |
| `missing_dependencies` | Ordered list | No | Operator-facing recovery hints may reference these values. |
| `warnings` | Ordered list | No | Warnings remain inspectable even when readiness is not blocked. |
| `runtime_environment` | Structured bounded summary | No | Captures supported OS, shell, endpoint, or feature gates relevant to the provider. |
| `checked_at` | Timestamp | Yes | Used to determine freshness of the readiness state. |

## Entity: Provider Preparation Report

Represents the pre-execution report returned by `prepare`.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `request_id` | Stable identifier | Yes | Must match the later execute request. |
| `required_context_refs` | Ordered list | Yes | Missing required context blocks execution admission. |
| `optional_context_refs` | Ordered list | No | Informational inputs only. |
| `missing_evidence_refs` | Ordered list | No | Supports pre-execution repair or degrade decisions. |
| `expected_artifacts` | Ordered list | No | Used to validate execute and collect-evidence output. |
| `risk_observations` | Ordered list | No | Provider-supplied risks remain non-authoritative hints. |
| `estimated_cost_or_runtime` | Structured bounded estimate | No | Informational only; route economics remain out of scope for this slice. |

## Entity: Provider Permission Envelope

Represents the least-privilege execution grant attached to one request.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `read_files` | Boolean | Yes | Explicitly granted or denied. |
| `write_files` | Boolean | Yes | Explicitly granted or denied. |
| `run_commands` | Boolean | Yes | Explicitly granted or denied. |
| `network` | Boolean | Yes | Explicitly granted or denied. |
| `read_secrets` | Boolean | Yes | Indicates whether secret handles may be resolved. |
| `write_artifacts` | Boolean | Yes | Controls provider-generated artifact persistence. |
| `allowed_paths` | Ordered list of relative refs | No | Empty means no path grant. |
| `max_runtime_ms` | Integer | Yes | Runtime-bound request limit. |
| `max_output_bytes` | Integer | Yes | Output-bound request limit. |

## Entity: Provider Execution Request

Represents the bounded request sent to `execute`.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `request_id` | Stable identifier | Yes | Shared across prepare, execute, and collect-evidence. |
| `session_ref` | Stable session identifier | Yes | Preserves runtime traceability. |
| `step_or_stage_ref` | Stable identifier | Yes | Boundline must be able to map provider work back to one bounded runtime step. |
| `capability_id` | Stable identifier | Yes | Must resolve to an active capability declaration. |
| `goal_summary` | Bounded text | Yes | Must describe the requested outcome without becoming the sole evidence source. |
| `lifecycle_phase` | Stable phase label | Yes | Checked against the capability declaration before admission. |
| `authority_zone` | Stable label | Yes | Distinguishes workspace, runtime, or governed evidence boundaries. |
| `context_pack_refs` | Ordered list | Yes | Every provider-backed request must identify the context it used. |
| `permission_envelope` | `Provider Permission Envelope` | Yes | Explicit least-privilege grant. |
| `expected_outputs` | Ordered list | Yes | Used to validate returned artifacts and evidence. |

## Entity: Provider Execution Result

Represents the structured response returned by `execute`.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `request_id` | Stable identifier | Yes | Must match the originating request. |
| `status` | `succeeded`, `blocked`, `failed`, or `partial` | Yes | Used together with validation disposition to determine the final runtime outcome. |
| `observations` | Ordered list | No | Non-authoritative runtime notes. |
| `findings` | Ordered list | No | Claims returned by the provider; not automatically accepted. |
| `artifact_refs` | Ordered list | No | References to generated or retrieved artifacts. |
| `evidence_refs` | Ordered list | No | References that later feed collect-evidence. |
| `state_patch_proposals` | Ordered list | No | Optional proposals; never direct state mutation. |
| `limitations` | Ordered list | No | Must remain visible in status or inspect when present. |
| `next_actions` | Ordered list | No | Operator or runtime follow-up suggestions. |

## Entity: Provider Evidence Collection Record

Represents the normalized output of `collect_evidence`.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `request_id` | Stable identifier | Yes | Ties the record back to execute. |
| `claims` | Ordered list | Yes | Normalized provider claims. |
| `evidence_refs` | Ordered list | Yes | Required for accepted provider evidence. |
| `artifact_refs` | Ordered list | No | Additional supporting artifacts. |
| `findings` | Ordered list | No | Carries forward non-authoritative findings. |
| `limitations` | Ordered list | No | Preserved for later inspection. |
| `reproducibility_metadata` | Structured bounded summary | Yes | Must provide enough information for later replay or audit. |

## Entity: Provider Validation Disposition

Represents Boundline's final decision on provider output.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `request_id` | Stable identifier | Yes | Identifies which provider result was evaluated. |
| `disposition` | `accepted`, `rejected`, `blocked`, or `degraded` | Yes | Final state that governs whether provider output can influence runtime state. |
| `failure_class` | `readiness`, `permission_admission`, `execution`, or `post_execution_validation` | No | Required when the disposition is not fully accepted. |
| `accepted_evidence_refs` | Ordered list | No | Populated only for accepted or degraded outcomes. |
| `rejected_evidence_refs` | Ordered list | No | Populated when specific evidence was rejected. |
| `reason` | Operator-readable summary | Yes | Must be visible in runtime projections. |

## Entity: Specialized Execution Profile

Represents an optional overlay that maps generic provider capabilities to
Boundline stage semantics.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `profile_id` | Stable identifier | Yes | Unique within the workspace or packaged profile catalog. |
| `provider_id` | Stable identifier | Yes | Must reference an existing registration. |
| `capability_mappings` | Ordered list | Yes | Maps provider capability IDs to Boundline stage or hook semantics. |
| `profile_version` | Version string | Yes | Supports profile compatibility checks. |
| `conflict_policy` | Stable bounded label | Yes | Must never weaken Boundline runtime policy. |

## Identity And Uniqueness Rules

- `provider_id` is unique within the workspace.
- `capability_id` is unique within one `provider_id`.
- `request_id` is unique per provider-backed execution attempt and shared across
  prepare, execute, collect-evidence, and validation records.
- `profile_id` is unique within the active profile catalog.

## State Transitions

```text
provider discovered
  -> registration created
  -> activation attempted

activation attempted
  -> health validated
  -> active

activation attempted
  -> blocked or invalid
  -> previous active provider remains authoritative

active provider + capability request
  -> prepare report
  -> permission admission
  -> execute
  -> collect_evidence
  -> validation disposition

metadata conflict or permission mismatch
  -> fail closed before execute
```

## Compatibility Rules

- Older sessions or configs without provider-registration fields must continue
  to deserialize and render successfully.
- Provider state is additive; status and inspect may omit provider projections
  entirely when the provider protocol has never run in a workspace.
- Canon-governed artifacts may appear in context or evidence refs, but Canon
  never becomes a provider registration, capability declaration, or activation
  record.
- Secret handle refs may be persisted, but raw secret values must not be
  written to tracked config, session, or trace surfaces.
