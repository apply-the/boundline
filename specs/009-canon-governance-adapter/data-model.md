# Data Model: Canon Governance Adapter

## WorkspaceExecutionProfile (extended)

- Purpose: Continues to define the bounded delivery configuration for one workspace run and now optionally carries governance policy for built-in flow stages.
- New field:
  - `governance`: Optional `GovernanceProfile` that controls local or Canon-backed governance behavior.
- Validation rules:
  - Existing execution, adaptive, review, and run-limit validation remains unchanged.
  - When `governance` is present, the governance profile must validate independently.

## GovernanceProfile

- Purpose: Declares how Synod should govern built-in flow stages for one workspace.
- Fields:
  - `default_runtime`: `GovernanceRuntimeKind` used when no stage-specific runtime overrides it.
  - `canon`: Optional `CanonRuntimeConfig` used only when the Canon runtime is selected for at least one stage.
  - `stages`: Ordered list of `StageGovernancePolicy` records keyed by flow and stage.
- Validation rules:
  - `stages` must not contain duplicate `(flow_name, stage_id)` pairs.
  - Every `(flow_name, stage_id)` must map to a supported built-in Synod flow stage.
  - If any stage selects the Canon runtime, `canon` must be present and valid.
  - Canon stage policies must validate against the first-slice stage-to-mode whitelist before the workspace profile loads successfully.

## GovernanceRuntimeKind

- Purpose: Names the runtime selected for the current governed stage.
- Values:
  - `local`
  - `canon`
- Validation rules:
  - The runtime must be explicit before a governed stage continues.

## CanonRuntimeConfig

- Purpose: Provides the Canon CLI adapter inputs required to open governed stage runs.
- Fields:
  - `command`: CLI program name or path, typically `canon`.
  - `default_owner`: Optional owner applied when a stage policy does not override it.
  - `default_risk`: Optional default risk used when a stage policy does not override it.
  - `default_zone`: Optional default zone used when a stage policy does not override it.
  - `default_system_context`: Optional default `new` or `existing` system context used when a stage policy does not override it.
- Validation rules:
  - `command` must not be empty.
  - Default values may be omitted only if every Canon-governed stage provides its own explicit value or can deterministically derive one before execution.

## StageGovernancePolicy

- Purpose: Describes governance behavior for one built-in Synod flow stage.
- Fields:
  - `flow_name`: One of `bug-fix`, `change`, or `delivery`.
  - `stage_id`: Built-in stage identifier inside the selected flow.
  - `enabled`: Whether the stage has an explicit governance boundary.
  - `required`: Whether the stage may proceed only through a compliant governed path.
  - `autopilot`: Whether bounded autopilot decisions are allowed for this stage.
  - `runtime`: Optional `GovernanceRuntimeKind` override for this stage.
  - `canon_mode`: Optional Canon mode, required when the effective runtime is `canon`.
  - `system_context`: Optional `new` or `existing` binding for Canon-governed stages.
  - `risk`: Optional risk classification for Canon-governed stages.
  - `zone`: Optional governance zone for Canon-governed stages.
  - `owner`: Optional stage owner for Canon-governed stages.
- Validation rules:
  - `required` implies `enabled`.
  - `autopilot` implies `enabled`.
  - If the effective runtime is `canon`, `canon_mode` must be present unless exactly one compliant whitelist mode exists and Synod can derive it deterministically at load time.
  - If `canon_mode` is present, it must be allowed for the `(flow_name, stage_id)` pair in the first-slice mapping.
  - `system_context = existing` means the stage is grounded in the current repository or an earlier governed packet; `system_context = new` means the stage is grounded only in a newly authored governed brief.
  - For the first slice, `change`, `backlog`, `implementation`, `verification`, and `pr-review` bindings must use `existing`; `requirements`, `discovery`, and `architecture` may bind either `new` or `existing` when the selected Canon mode allows it.
  - If `runtime` is `local`, Canon-specific fields are ignored and must not be used to claim Canon governance occurred.

## GovernedStageRecord

- Purpose: Persists the governance lifecycle state for the current or latest governed stage inside the active task context, with session fields derived from this stored record.
- Fields:
  - `stage_key`: Stable key such as `bug-fix:investigate`.
  - `runtime`: Effective `GovernanceRuntimeKind` used for the stage.
  - `lifecycle_state`: `GovernanceLifecycleState` for the stage.
  - `required`: Whether the stage is required to pass governance.
  - `autopilot_enabled`: Whether autopilot was permitted for this stage.
  - `approval_state`: `ApprovalState` for the current stage.
  - `canon_run_ref`: Optional Canon run identifier.
  - `governance_attempt_id`: Stable identifier for this governed stage attempt.
  - `previous_governance_attempt_id`: Optional earlier attempt for the same stage.
  - `packet_ref`: Optional reference to the governed stage packet reused by Synod.
  - `decision_ref`: Optional reference to the latest `AutopilotDecisionRecord`.
  - `blocked_reason`: Optional reason when governance could not continue.
- Validation rules:
  - `canon_run_ref` is required once the Canon runtime has successfully started a run.
  - `blocked_reason` is required when `lifecycle_state` is `blocked`.
  - `approval_state` must be explicit even when approval is not needed.
  - `governance_attempt_id` must change only when a rerun or escalation opens a new governed attempt.

## GovernanceLifecycleState

- Purpose: Captures the major state transitions of a governed stage.
- Values:
  - `pending_selection`
  - `running`
  - `governed_ready`
  - `awaiting_approval`
  - `blocked`
  - `completed`
  - `failed`
- Validation rules:
  - `awaiting_approval` may occur only when approval is genuinely required by the selected governance path.
  - `completed` implies that the stage may continue to normal Synod execution.
  - First-slice transitions are limited to `pending_selection -> running -> governed_ready|awaiting_approval|blocked|failed`, `governed_ready -> completed`, and `awaiting_approval -> governed_ready|blocked`.
  - `blocked`, `failed`, and `completed` are terminal for the current governed attempt.

## ApprovalState

- Purpose: Normalizes approval status across local and Canon-governed stages.
- Values:
  - `not_needed`
  - `requested`
  - `granted`
  - `rejected`
  - `expired`
- Validation rules:
  - Approval state must stay aligned with the effective governance runtime and lifecycle state.

## GovernedStagePacket

- Purpose: Represents the governed document set and readiness state that later Synod stages may reuse as bounded reasoning input.
- Fields:
  - `packet_ref`: Stable reference used in session and trace surfaces.
  - `runtime`: Source runtime that produced the packet.
  - `canon_mode`: Optional Canon mode that produced the packet.
  - `expected_document_refs`: Ordered list of required documents for the selected runtime and stage.
  - `document_refs`: Ordered list of governed document references.
  - `readiness`: `PacketReadiness` classification.
  - `missing_sections`: Ordered list of symbolic section references formatted as `<document_ref>#<section_slug>`.
  - `headline`: Short rendered summary of what the packet is safe to support.
- Validation rules:
  - `readiness = reusable` requires at least one document reference, every expected document reference to be present, and no required missing sections.
  - `readiness = incomplete` or `rejected` prevents the packet from satisfying stage completion or downstream reuse.
  - `missing_sections` entries must be unique and sorted in the same order the missing sections appear across the expected documents.

## PacketReadiness

- Purpose: Distinguishes a reusable governed packet from empty or invalid output.
- Values:
  - `pending`
  - `incomplete`
  - `reusable`
  - `rejected`
- Validation rules:
  - Stub-only scaffolding, empty authored sections, or explicit missing-body markers classify the packet as `incomplete` or `rejected`.
  - The validator must combine runtime-declared `missing_sections` with a local non-whitespace authored-body check for every expected document.
  - Classification is deterministic for the first slice: `pending` before runtime completion; `rejected` when the runtime explicitly rejects the packet or every expected document fails the authored-body check; `incomplete` when any expected document is missing, any `missing_sections` entry exists, or any expected document body is empty; `reusable` only when every expected document exists, every expected document body is non-empty, and `missing_sections` is empty.

## PacketReuseBinding

- Purpose: Records why one governed packet was made available to a later stage as bounded reasoning input.
- Fields:
  - `upstream_stage_key`: Stage that produced the reusable packet.
  - `downstream_stage_key`: Stage that consumed or attempted to consume it.
  - `packet_ref`: Reused governed packet reference.
  - `binding_reason`: Short explanation such as `same_stage_rerun` or `upstream_stage_context`.
- Validation rules:
  - `packet_ref` must refer to a `GovernedStagePacket` with `readiness = reusable`.
  - `upstream_stage_key` and `downstream_stage_key` must belong to the same active Synod session.
  - For the first slice, `upstream_stage_key` must equal either `downstream_stage_key` on rerun, the immediately previous stage in the same built-in flow, or the explicit escalation source stage for a newly opened downstream governed attempt.
  - The first slice assumes linear built-in flows only; branching packet lineage is invalid for this feature version.

## AutopilotAction

- Purpose: Defines the only compliant actions autopilot may consider at a governed stage boundary.
- Values:
  - `select_mode`
  - `retry_stage_with_narrowed_context`
  - `escalate_verification`
  - `escalate_pr_review`
  - `await_approval`
  - `block_stage`
- Validation rules:
  - `retry_stage_with_narrowed_context` must remove the last eligible read target from the current ordered target list or, when no further read target can be removed, the last reused packet reference, while leaving goal, risk, zone, and owner unchanged.
  - `escalate_pr_review` is valid only where the stage-to-mode whitelist permits `pr-review`.
  - `escalate_verification` is valid only for `implement` stages.
  - `await_approval` is valid only when the selected governance path can legitimately require human approval.

## AutopilotDecisionRecord

- Purpose: Stores one inspectable bounded decision taken by autopilot at a governed stage boundary.
- Fields:
  - `decision_id`: Stable identifier.
  - `stage_key`: Governed stage this decision belongs to.
  - `candidate_actions`: Ordered list of `AutopilotAction` choices considered.
  - `candidate_modes`: Ordered list of Canon modes considered when `select_mode` is a candidate.
  - `selected_action`: Optional chosen `AutopilotAction`.
  - `selected_mode`: Optional Canon mode chosen when `selected_action = select_mode`.
  - `selected_target_stage_key`: Optional downstream stage selected by an escalation action.
  - `rationale`: Short explanation of why the selected action was chosen or why none was possible.
  - `blocked_reason`: Optional reason when no compliant action existed.
- Validation rules:
  - `selected_action` must be present unless the decision ended in a blocked outcome.
  - Candidate actions must stay inside the bounded autopilot vocabulary approved by the plan.
  - `candidate_actions` must be ordered by the first-slice decision priority: mode selection, approval wait, narrowed retry, verification escalation, PR-review escalation, block.
  - `candidate_modes` is required when `candidate_actions` includes `select_mode`.
  - `selected_mode` is required when `selected_action = select_mode`.
  - `selected_target_stage_key` is required when `selected_action` is an escalation action.
  - When `stage_key = bug-fix:investigate` and `candidate_modes` contains both `discovery` and `change`, `discovery` must appear first in the candidate order.
  - When `stage_key` is `bug-fix:verify` or `change:verify` and `candidate_modes` contains both `verification` and `pr-review`, `verification` must appear first in the candidate order.