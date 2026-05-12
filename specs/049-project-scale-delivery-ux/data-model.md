# Data Model: Boundline Project-Scale Delivery UX

## GlobalAssistantPackage

Represents a user-scoped assistant package installed once per host before any workspace is initialized.

**Fields**:

- `host`: `claude`, `codex`, `cursor`, `copilot`, `gemini`, or future host identifier.
- `scope`: `user` for true global/user-scoped installation; `manual` or `prompt-pack` where the host does not support global commands.
- `commands`: bootstrap-safe command set.
- `metadata`: name, display name, version, description, author, homepage, repository, license, keywords, capabilities, icon/logo refs.
- `shell_execution`: whether the host can execute Boundline CLI directly.
- `fallback_guidance`: exact CLI commands or manual import instructions.

**Validation Rules**:

- Must not require `.boundline/session.json`.
- Must include `/boundline:init`, `/boundline:doctor`, `/boundline:help`, `/boundline:continue`, and `/boundline:status`.
- Must not claim true global installation when host support is manual or prompt-pack only.
- Version must align with the Boundline package version.

## RepoLocalAssistantPackage

Represents the workspace-generated assistant package produced by `boundline init --assistant <host>`.

**Fields**:

- `workspace_root`
- `host`
- `generated_paths`
- `commands`
- `prompts`
- `metadata`
- `state_source`: always CLI-backed `.boundline/session.json`.

**Validation Rules**:

- Commands must read state through Boundline CLI or documented CLI-equivalent surfaces.
- Host-specific folders should mostly contain manifests, metadata, command bindings, assets, prompts, and glue.
- Runtime behavior must remain shared rather than duplicated in host markdown.

## DeliveryInitiative

Represents a broad project-scale goal decomposed into bounded stages and work units.

**Fields**:

- `id`
- `brief`
- `current_stage_id`
- `proposed_path`
- `confirmed_path`
- `risk_profile`
- `context_evidence_refs`
- `trace_refs`
- `checkpoint_refs`
- `governance_refs`
- `voting_refs`
- `next_action`
- `status`: `draft`, `awaiting_confirmation`, `active`, `blocked`, `terminal`.

**Validation Rules**:

- Must not claim the whole initiative can be completed in one unchecked run.
- Must have one active stage or work unit at a time.
- Must stop when required context, approval, validation, voting, or risk policy is missing.

## BoundedStage

Represents a named stage in a project-scale delivery path.

**Fields**:

- `id`
- `canon_mode`: optional mode from the governed stage catalog.
- `category`: planning, execution guidance, review, verification, assessment, or operational.
- `entry_conditions`
- `required_context`
- `completion_evidence`
- `stop_conditions`
- `confirmation_required`
- `can_lead_to`: next allowed stage categories or modes.
- `recommendation_only`: whether Canon output is advisory from Canon's perspective.

**Validation Rules**:

- A stage with `canon_mode` must validate availability against Canon capabilities before running governed work.
- Material stage transitions require confirmation.
- Canon-governed stages must record packet/provenance/approval refs when run.

## BoundedWorkUnit

Represents a concrete implementation, refactor, verification, review, or recovery slice inside a stage.

**Fields**:

- `id`
- `stage_id`
- `goal`
- `target_surface`
- `expected_changes`
- `validation_commands`
- `retry_budget`
- `checkpoint_ref`
- `trace_ref`
- `terminal_outcome`
- `next_action`

**Validation Rules**:

- Must have a bounded goal and validation expectation.
- Retry exhaustion must route to recovery, voting, review, or terminal blocked state.
- Must not silently expand beyond the confirmed stage boundary.

## GovernedStageCatalogEntry

Maps a Canon mode to Boundline-owned stage selection behavior.

**Fields**:

- `mode`
- `consider_when`
- `required_system_context`
- `category`
- `voting_applicability`
- `can_lead_to_implementation_or_refactor`
- `recommendation_only`
- `capability_status`: `available`, `unavailable`, `unknown`, or `incompatible`.

**Validation Rules**:

- Must cover all current Canon modes named in the spec.
- Must stop explicitly for unavailable, unknown, or incompatible modes when governance is requested.
- Must not be exposed as a primary top-level `/boundline-<mode>` command set.

## GovernedPacketRef

Represents Canon-produced stage-boundary output visible inside Boundline.

**Fields**:

- `packet_id`
- `mode`
- `artifact_path`
- `provenance_ref`
- `approval_state`
- `readiness`
- `missing_sections`
- `created_at`
- `reviewed_by`

**Validation Rules**:

- Must remain a reference in Boundline state, not a replacement for Boundline orchestration.
- Approval-gated packets must block continuation until approved, adjudicated, repaired, or explicitly overridden by allowed policy.

## VotingDecision

Represents stage-boundary review voting for risky quality boundaries.

**Fields**:

- `id`
- `stage_id`
- `reviewed_evidence_ref`
- `trigger`
- `reviewers`
- `strategy`: `majority`, `weighted`, `reject_on_blocking`, or configured combination.
- `findings`
- `votes`: approve, concern, block.
- `result`: approved, concern, blocked, inconclusive, adjudicated, overridden.
- `adjudication_ref`
- `blocking`
- `next_action`

**Validation Rules**:

- Must not run by default for low-risk local stages.
- Blocking findings must prevent continuation unless adjudicated or explicitly overridden according to policy.
- Status, next, and inspect must project the latest voting state.

## DeliveryPilotLoopEvent

Represents one observed iteration of `observe -> decide -> act -> verify -> update context`.

**Fields**:

- `event_id`
- `initiative_id`
- `stage_id`
- `work_unit_id`
- `phase`: observe, decide, act, verify, update_context.
- `inputs`
- `decision_rationale`
- `action`
- `verification_result`
- `context_updates`
- `trace_ref`
- `next_action`

**Validation Rules**:

- Decisions and hidden heuristics must be traceable through event output.
- Failed, blocked, exhausted, clarification-required, and terminal states must be explicit.

## State Transitions

```text
no workspace state
  -> global bootstrap diagnostics
  -> initialized workspace
  -> delivery initiative drafted
  -> path proposed
  -> path confirmed
  -> bounded stage active
  -> bounded work unit active
  -> verification
  -> review/voting/governance as required
  -> continue, publish-ready, blocked, or terminal
```

Governed stage transitions:

```text
mode requested/inferred
  -> capability checked
  -> context validated
  -> confirmation required when material
  -> Canon stage call
  -> packet ref persisted
  -> approval/readiness evaluated
  -> next Boundline action selected
```

Voting transitions:

```text
risk trigger detected
  -> evidence packet selected
  -> reviewer findings collected
  -> vote strategy applied
  -> approved, blocked, inconclusive, adjudicated, or overridden
  -> session next action updated
```
