# Data Model: Agentic Framework Integration

This slice extends the existing workspace configuration, session runtime, and
trace models so Boundline can keep its built-in Canon-aware path as the default
while supporting one explicit external adapter per lifecycle run. The entities
below describe the additive state needed in the host repo, plus the compatibility
state that the sibling template and Speckit repos must honor.

## 1. AdapterSelectionRecord

**Purpose**: Represents the optional workspace-level adapter selection persisted
in `.boundline/config.toml`.

**Key Fields**:

- `selection_mode`: `none`, `known_profile`, or `custom`
- `adapter_id`: stable adapter identity such as `speckit`
- `display_name`: operator-facing label for the selected adapter
- `command`: executable name or explicit command path Boundline will spawn
- `args`: optional fixed arguments passed to the adapter binary
- `registration_source`: `adapter_add`, `init`, or `config_migration`
- `discovery_state`: `explicit_command`, `discovered_on_path`, or `unresolved`
- `compatibility_line`: host-owned contract line such as `framework-adapter-v1`
- `updated_at`: timestamp of the most recent selection change

**Relationships**:

- references zero or one `KnownAdapterProfileDefinition`
- owns one `ResolvedAdapterConfigSet`
- is sampled into one `AdapterCapabilitySnapshot` per lifecycle run when active

**Validation Rules**:

- when no selection record exists, Boundline must use built-in behavior
- exactly one selection may be active per workspace
- `selection_mode != none` requires non-empty `adapter_id` and `command`
- `discovery_state = discovered_on_path` may help prefill setup, but it must not
  create or activate the record without an explicit registration action

## 2. KnownAdapterProfileDefinition

**Purpose**: Represents host-shipped metadata for known profiles such as
Speckit.

**Key Fields**:

- `adapter_id`: stable profile identity; `speckit` for this slice
- `display_name`: operator-facing profile label
- `default_command`: default binary name, such as `boundline-adapter-speckit`
- `registration_alias`: short CLI alias used by `boundline adapter add`
- `adapter_repo_ref`: repository reference for the concrete adapter
- `template_repo_ref`: repository reference for the reusable template
- `compatibility_line`: protocol line the profile expects
- `discovery_names`: bounded list of executable names that may be suggested
- `prefilled_fields`: host-known field defaults or hints the guided setup flow
  can populate automatically

**Relationships**:

- may be selected by many `AdapterSelectionRecord` states across workspaces
- constrains one `AdapterCapabilitySnapshot` when the profile is active
- is governed by one `ProtocolCompatibilityRecord`

**Validation Rules**:

- `adapter_id` must be unique across all known profiles
- the Speckit profile must map to `boundline-adapter-speckit`
- discovery names may be suggested to the operator, but they do not activate a
  profile on their own

## 3. AdapterCapabilitySnapshot

**Purpose**: Represents the capability manifest returned by the adapter for one
lifecycle run after `describe` and before stage routing begins.

**Key Fields**:

- `run_id`: lifecycle run identifier that owns the snapshot
- `adapter_id`: adapter identity returned by the binary
- `protocol_line`: declared protocol line such as `framework-adapter-v1`
- `adapter_version`: semantic version reported by the adapter
- `supported_boundline_range`: machine-readable Boundline version range
- `supported_transports`: ordered list of adapter-declared transport
  descriptors; each descriptor carries `transport`, `encoding`,
  `request_channel`, and `response_channel`, and V1 expects at least one JSON
  over stdin/stdout entry
- `declared_stage_overrides`: ordered list of host-known stages the adapter
  wants to own
- `declared_hook_subscriptions`: ordered list of host-known hooks the adapter
  wants to observe
- `config_schema_fingerprint`: stable digest of the required field definitions
- `snapshot_state`: `validated`, `blocked`, `invalid_manifest`, or
  `incompatible`

**Relationships**:

- belongs to one `AdapterSelectionRecord`
- owns many `AdapterConfigFieldDefinition` records
- constrains many `StageRoutingDecisionRecord` and `HookEventDispatchRecord`
  records
- is checked by one `ProtocolCompatibilityRecord`

**Validation Rules**:

- unknown stage IDs or hook IDs force `snapshot_state = invalid_manifest`
- mismatched protocol or version compatibility force
  `snapshot_state = incompatible`
- a missing supported transport declaration, or a declaration that does not
  include the V1 JSON-over-stdin/stdout transport, forces
  `snapshot_state = incompatible`
- the `describe` command must return a protocol-valid success envelope before
  the host parses the capability body; a protocol error envelope or transport
  failure blocks activation before manifest validation continues
- a `validated` snapshot may exist only when the selected adapter identity,
  command, manifest identity, and supported transport declaration agree

**State Transitions**:

- `validated -> blocked`
- `validated -> incompatible`
- `validated -> invalid_manifest`

## 4. AdapterConfigFieldDefinition

**Purpose**: Represents one adapter-declared field that the host may need to
collect or validate before execution.

**Key Fields**:

- `field_key`: stable field identifier
- `display_label`: operator-facing prompt label
- `value_kind`: `string`, `path`, `boolean`, `integer`, or `enum`
- `required`: whether the field must be present before execution
- `secret`: whether the field must not be echoed in normal CLI output or traces
- `default_value_text`: optional textual default supplied by the adapter or
  known profile
- `prompt_text`: prompt copy used in guided setup
- `help_text`: actionable recovery text for non-interactive failures
- `non_interactive_policy`: `fail`, `use_default`, or `skip_if_unowned`

**Relationships**:

- belongs to one `AdapterCapabilitySnapshot`
- is resolved into one or more `AdapterConfigValueRecord` values over time

**Validation Rules**:

- `field_key` must be unique within one capability snapshot
- `secret = true` fields must not be rendered in plain-text status output
- `non_interactive_policy = use_default` is only valid when a default exists

## 5. ResolvedAdapterConfigSet

**Purpose**: Represents the host's persisted and runtime-validated adapter
configuration for the currently selected adapter.

**Key Fields**:

- `adapter_id`: selected adapter identity
- `schema_fingerprint`: fingerprint of the capability schema this config matches
- `completeness_state`: `complete`, `missing_required`, or `invalid`
- `interactive_resolution`: whether the latest successful resolution used guided
  prompts
- `last_validated_at`: timestamp of the most recent successful validation
- `value_count`: number of stored field values

**Relationships**:

- belongs to one `AdapterSelectionRecord`
- owns many `AdapterConfigValueRecord` values
- is checked during `AdapterCapabilitySnapshot` preflight

**Validation Rules**:

- `completeness_state = complete` requires every required field to have a valid
  value compatible with the current schema fingerprint
- non-interactive execution with `missing_required` must block before any
  adapter-owned stage or hook call begins
- a schema fingerprint change requires re-validation before reuse

## 6. AdapterConfigValueRecord

**Purpose**: Represents one resolved field value stored for the active adapter.

**Key Fields**:

- `field_key`: stable field identifier
- `value_kind`: mirrors the field definition type
- `string_value`: populated when `value_kind = string` or `enum`
- `path_value`: populated when `value_kind = path`
- `bool_value`: populated when `value_kind = boolean`
- `int_value`: populated when `value_kind = integer`
- `value_source`: `operator_prompt`, `cli_flag`, `known_profile_default`, or
  `migrated_config`
- `resolution_state`: `present`, `missing`, or `invalid`

**Relationships**:

- belongs to one `ResolvedAdapterConfigSet`
- resolves one `AdapterConfigFieldDefinition`

**Validation Rules**:

- exactly one typed value slot may be populated for a given record
- secret fields may be stored, but they must be redacted in operator-visible
  status and trace projections
- `resolution_state = invalid` blocks adapter-owned execution until corrected

## 7. StageRoutingDecisionRecord

**Purpose**: Represents Boundline's per-stage decision about whether the built-in
path or the adapter owns the stage.

**Key Fields**:

- `run_id`: owning lifecycle run identifier
- `stage_key`: host-known lifecycle stage key
- `execution_source`: `built_in` or `adapter`
- `decision_reason`: `no_adapter_selected`, `undeclared_stage`,
  `declared_override`, `preflight_blocked`, `invalid_manifest`, or
  `compatibility_blocked`
- `claim_state`: `not_claimed`, `claimed`, `completed`, or `failed_after_claim`
- `adapter_id`: selected adapter identity when the source is `adapter`
- `recorded_at`: timestamp of the routing decision

**Relationships**:

- belongs to one `AdapterCapabilitySnapshot` when an adapter is active
- produces one `LifecycleStageExecutionRecord`

**Validation Rules**:

- every lifecycle stage must produce exactly one routing decision
- `execution_source = adapter` requires a validated capability snapshot and
  complete config
- `claim_state = failed_after_claim` requires the stage execution record to mark
  intervention required

**State Transitions**:

- `not_claimed -> claimed`
- `claimed -> completed`
- `claimed -> failed_after_claim`

## 8. LifecycleStageExecutionRecord

**Purpose**: Represents the operator-visible per-stage outcome recorded in
session state and traces.

**Key Fields**:

- `run_id`: owning lifecycle run identifier
- `stage_key`: host-known lifecycle stage key
- `execution_source`: `built_in` or `adapter`
- `adapter_id`: adapter identity when applicable
- `status`: `succeeded`, `failed`, `blocked`, or `skipped`
- `intervention_required`: whether the operator must act before continuing
- `failure_class`: `preflight`, `manifest`, `missing_config`, `adapter_runtime`,
  `adapter_protocol`, `adapter_transport`, `built_in`, or
  `hook_warning_only`
- `produced_artifacts`: bounded list of artifact refs returned by the stage
- `diagnostic_trace_refs`: optional trace refs for structured adapter stderr
  captured during stage execution, when emitted
- `started_at`: start timestamp
- `finished_at`: terminal timestamp

**Relationships**:

- belongs to one `StageRoutingDecisionRecord`
- may reference many `HookEventDispatchRecord` values
- is rendered into `.boundline/session.json` and trace outputs

**Validation Rules**:

- `intervention_required = true` is mandatory when an adapter fails after stage
  claim
- a `blocked` status must surface an actionable next step or missing-field list
- built-in stages must remain recordable even when an adapter is configured but
  undeclared for that stage

## 9. HookEventDispatchRecord

**Purpose**: Represents one host-to-adapter hook delivery attempt.

**Key Fields**:

- `run_id`: owning lifecycle run identifier
- `hook_key`: host-known hook identifier
- `stage_key`: related lifecycle stage, when applicable
- `adapter_id`: receiving adapter identity
- `delivery_status`: `ignored`, `delivered`, `warning`, or `failed`
- `stage_claim_state`: whether the current stage had already been claimed by the
  adapter when the hook fired
- `payload_ref`: trace or artifact ref for the hook payload
- `diagnostic_trace_refs`: optional trace refs for structured adapter stderr
  captured during hook delivery, when emitted
- `error_summary`: surfaced failure summary when delivery is not successful

**Relationships**:

- belongs to one `AdapterCapabilitySnapshot`
- may be referenced by one `LifecycleStageExecutionRecord`

**Validation Rules**:

- undeclared hooks resolve to `ignored` and do not spawn subprocess execution
- a failed non-owning hook may record `warning` or `failed`, but it must not
  retroactively convert a built-in stage into an adapter-owned failure
- a hook fired after an adapter-owned stage has been claimed may contribute to
  the stage's terminal failure classification
- missing or malformed structured stderr must not change `delivery_status` when
  the stdout response envelope is otherwise protocol-valid

## 10. AdapterCommandExchangeRecord

**Purpose**: Represents one host-to-adapter stdio exchange, whether it happens
during explicit adapter management or during lifecycle execution.

**Key Fields**:

- `surface`: `adapter_add`, `adapter_show`, `lifecycle_preflight`,
  `stage_execution`, or `hook_delivery`
- `command_key`: `describe`, `preflight`, `execute_stage`, or `emit_hook`
- `transport_key`: normalized transport identifier; V1 is `stdio_json_v1`
- `response_class`: `success_envelope`, `error_envelope`, `transport_failure`,
  or `invalid_stdout`
- `domain_status`: optional command-specific outcome from `data.status`, such as
  `ready`, `blocked`, `succeeded`, `failed`, or `delivered`
- `stderr_trace_refs`: optional trace refs for structured adapter stderr
  captured during the exchange, when emitted and parseable
- `recorded_at`: timestamp of the completed exchange

**Relationships**:

- may seed one `AdapterCapabilitySnapshot`
- may block or normalize one `ResolvedAdapterConfigSet`
- may be referenced by one `LifecycleStageExecutionRecord` or one
  `HookEventDispatchRecord`

**Validation Rules**:

- `response_class = success_envelope` requires exit code `0` and a protocol-
  valid stdout body with `success = true` and a `data` object
- `response_class = error_envelope` requires exit code `0` and a protocol-valid
  stdout body with `success = false` and an `error` object
- non-zero exit codes, missing stdout, or malformed JSON classify the exchange
  as `transport_failure` or `invalid_stdout` instead of an in-band domain
  outcome
- structured stderr may add `stderr_trace_refs`, but it must not change
  `response_class` or `domain_status`

## 11. ProtocolCompatibilityRecord

**Purpose**: Represents the compatibility line shared across the host repo, the
template repo, and concrete adapter repos.

**Key Fields**:

- `compatibility_line`: stable protocol identifier such as
  `framework-adapter-v1`
- `boundline_version_range`: supported host version range for that line
- `contract_crate_ref`: versioned git-tag reference to `boundline-adapters`
- `template_version`: current template release that implements the line
- `adapter_version`: current adapter release that implements the line
- `compatibility_state`: `supported`, `deprecated`, or `blocked`

**Relationships**:

- governs one or more `KnownAdapterProfileDefinition` records
- is checked by every `AdapterCapabilitySnapshot`

**Validation Rules**:

- incompatible host or adapter versions must block activation before stage
  routing begins
- the template and adapter repos may release independently, but each release
  must declare the line it implements

## Cross-Entity Invariants

- When no `AdapterSelectionRecord` exists, every `StageRoutingDecisionRecord`
  must resolve to `execution_source = built_in`.
- Exactly one adapter may be active per workspace and per lifecycle run.
- PATH discovery may suggest or prefill `AdapterSelectionRecord.command`, but it
  must never create the selection implicitly.
- Any unknown stage override or hook subscription makes the
  `AdapterCapabilitySnapshot` invalid and blocks activation.
- Every validated `AdapterCapabilitySnapshot` must declare at least one host-
  supported transport; V1 accepts only JSON over stdin/stdout and defers
  long-running transport lifecycle management.
- Every host-to-adapter subprocess exchange must be classified first as a
  success envelope, error envelope, transport failure, or invalid stdout before
  routing, blocking, or audit decisions are finalized.
- Non-interactive execution with `ResolvedAdapterConfigSet.completeness_state =
  missing_required` must stop before any adapter-owned stage or hook delivery.
- Once an adapter-owned stage is claimed, a runtime failure must surface as a
  failed `LifecycleStageExecutionRecord` with `intervention_required = true`.
- Hook failures remain observable, but non-owning hook failures must not fail a
  built-in stage retroactively.
- Optional structured adapter stderr may enrich trace outputs when parseable,
  but absent or unstructured stderr must not invalidate an otherwise valid
  stdout response or change the host's response classification.
- The known `speckit` profile must stay aligned with adapter ID `speckit`,
  default binary `boundline-adapter-speckit`, and the sibling repos
  `../boundline-framework-template/` and `../boundline-adapter-speckit/`.