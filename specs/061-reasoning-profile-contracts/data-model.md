# Data Model: Governed Reasoning Profile Contracts

## ReasoningProfileDefinition

- Purpose: Declares one bounded reasoning strategy that Boundline may execute
  inside an existing governed stage.
- Fields:
  - `profile_id`: Stable identifier such as `bounded_self_consistency`,
    `independent_pair_review`, `heterogeneous_security_review`, or
    `bounded_reflexion`.
  - `family`: High-level profile class such as `self_consistency`,
    `blind_review`, `heterogeneous_review`, `reflexion`, or `debate_enabled`.
  - `allowed_stages`: Ordered list of stages where the profile may activate.
  - `limits`: One `ReasoningBudget` record.
  - `participant_roles`: Ordered list of required `ParticipantRoleDefinition`
    entries.
  - `adjudication_mode`: One explicit disagreement-resolution model.
  - `degradation_policy`: Rules for what happens when the requested topology or
    independence floor cannot be met.
- Validation rules:
  - `profile_id` must be non-empty and unique.
  - `allowed_stages` must be non-empty.
  - `participant_roles` must be non-empty.
  - The requested budget must be bounded and internally consistent.
  - Debate rounds and reflexion revisions may only be non-zero when the profile
    family explicitly allows them.

## ReasoningBudget

- Purpose: Defines the explicit execution bounds for one profile activation.
- Fields:
  - `max_participants`
  - `max_branches`
  - `max_debate_rounds`
  - `max_reflexion_revisions`
  - `max_calls`
  - `max_tokens`
  - `max_adjudication_steps`
- Validation rules:
  - Every numeric field must be greater than zero when present.
  - `max_participants` must satisfy the minimum required by the chosen profile.
  - `max_debate_rounds` must be zero when debate is disabled.
  - `max_reflexion_revisions` must be zero when reflexion is disabled.

## ParticipantRoleDefinition

- Purpose: Declares the roles a profile needs without hardcoding specific
  models or providers.
- Fields:
  - `role_id`: Stable role identifier.
  - `role_kind`: One of `independent_path`, `blind_reviewer`,
    `heterogeneous_reviewer`, `critic`, `reviser`, or `arbiter`.
  - `preferred_slot`: Existing Boundline routing slot used to resolve the role.
  - `independence_requirements`: One `IndependenceFloor` record.
  - `required`: Boolean.
- Validation rules:
  - `role_id` must be unique within a profile.
  - `role_kind` must come from the supported vocabulary.
  - `preferred_slot` must map to an existing routing slot or explicit
    adjudication path.

## CanonChallengePostureInput

- Purpose: Captures the Canon-owned signal that may require stronger challenge.
- Fields:
  - `contract_line`: Stable producer contract identifier such as
    `governed_reasoning_posture_v1`.
  - `compatibility_window`: Supported Boundline and Canon version window.
  - `required_profile_family`: Minimum profile family or explicit profile id.
  - `minimum_independence`: Requested minimum `IndependenceFloor`.
  - `admission_priority`: Whether the posture is advisory, required before
    continue, or required before acceptance.
  - `confidence_handoff_required`: Boolean.
  - `provenance_ref`: Canon reference that explains where the posture came from.
- Validation rules:
  - `contract_line` must be supported by Boundline.
  - `compatibility_window` must admit the active Boundline and Canon versions.
  - `required_profile_family` must map to a supported Boundline profile.
  - Unknown major contract lines are rejected.

## ProfileActivationRecord

- Purpose: Tracks one live or completed reasoning-profile lifecycle for a stage.
- Fields:
  - `activation_id`: Stable identifier for the profile execution.
  - `stage_key`: Governed stage where it activated.
  - `profile_id`: Selected profile definition.
  - `trigger`: Why the profile activated, such as Canon posture requirement,
    governance escalation, or explicit operator policy.
  - `status`: One of `pending`, `active`, `completed`, `degraded`, `blocked`,
    `interrupted`, `escalated`, or `failed`.
  - `participants`: Ordered list of `ParticipantAssignment` entries.
  - `budget`: The effective `ReasoningBudget` used at runtime.
  - `posture`: Optional `CanonChallengePostureInput` summary.
  - `outcome`: Optional `ReasoningOutcome`.
  - `confidence`: Optional `ReasoningConfidenceContribution`.
- Validation rules:
  - Only one active record may exist per governed stage.
  - `profile_id` must reference a known definition.
  - `participants` must satisfy required roles before the record can enter
    `active`.

## ParticipantAssignment

- Purpose: Resolves one role into one effective runtime participant.
- Fields:
  - `role_id`
  - `participant_id`
  - `effective_route`
  - `provider_family`
  - `context_basis`
  - `prompting_pattern`
  - `status`: `pending`, `running`, `completed`, `failed`, or `omitted`.
  - `result_summary`: Optional short outcome text.
- Validation rules:
  - `role_id` must match a declared participant role.
  - `participant_id` must be unique within the activation record.
  - Every required role must resolve to one assignment before activation.

## IndependenceFloor

- Purpose: States how much separation the requested challenge posture requires.
- Fields:
  - `route_distinct`: Boolean.
  - `provider_distinct`: Boolean.
  - `context_distinct`: Boolean.
  - `prompt_pattern_distinct`: Boolean.
  - `minimum_participants`: Positive integer.
- Validation rules:
  - `minimum_participants` must be at least one.
  - Blind review requires at least two participants.
  - Heterogeneous review requires at least one distinct provider or route.

## IndependenceAssessment

- Purpose: Records whether the resolved participant topology actually met the
  requested floor.
- Fields:
  - `requested_floor`: The `IndependenceFloor` demanded by policy or Canon.
  - `observed_distinctions`: Summary of the distinct routes, providers,
    contexts, and prompting patterns actually used.
  - `result`: `passed`, `degraded`, or `failed`.
  - `reason`: Human-readable explanation.
- Validation rules:
  - `failed` blocks activation unless the degradation policy explicitly allows
    fallback.
  - `degraded` must carry a reason.

## ReasoningIterationRecord

- Purpose: Captures one bounded branch, debate round, or reflexion revision.
- Fields:
  - `iteration_kind`: `branch`, `debate_round`, `reflexion_revision`, or
    `adjudication_step`.
  - `iteration_index`: Zero-based order.
  - `participants`: Which participants contributed.
  - `summary`: Short description of what changed.
  - `novelty`: Whether new material evidence emerged.
  - `condition`: `active`, `stagnated`, `completed`, or `exhausted`.
- Validation rules:
  - Iterations must respect the effective budget.
  - Repeated non-novel iterations may trigger degradation or termination.

## ReasoningOutcome

- Purpose: Defines the terminal profile result.
- Fields:
  - `outcome_kind`: `converged`, `disagreed`, `adjudicated`, `degraded`,
    `blocked`, `interrupted`, `escalated`, or `failed`.
  - `headline`: Short operator-facing summary.
  - `disagreement_summary`: Optional explanation of the conflict.
  - `next_action`: Optional bounded operator action.
  - `iterations`: Ordered list of `ReasoningIterationRecord` summaries.
- Validation rules:
  - A non-success outcome must carry a reason or next action.
  - `adjudicated` must identify that adjudication occurred.

## ReasoningConfidenceContribution

- Purpose: Supplies profile-level evidence back into the existing governance
  confidence path.
- Fields:
  - `confidence_level`: `low`, `medium`, or `high`.
  - `basis`: Ordered list of confidence inputs such as convergence,
    disagreement, independence result, or exhaustion.
  - `admission_effect`: `none`, `warn`, `gate`, or `escalate`.
  - `summary`: Short explanation projected to operator surfaces.
- Validation rules:
  - `confidence_level` alone never authorizes acceptance.
  - `gate` or `escalate` must align with the existing governance path.

## ReasoningCompatibilityWindow

- Purpose: Expresses the supported bilateral release pair for the first shipped
  contract.
- Fields:
  - `boundline_min`
  - `boundline_max_exclusive`
  - `canon_min`
  - `canon_max_exclusive`
  - `contract_line`
- Validation rules:
  - Unsupported pairs are rejected before profile activation.
  - Additive fields inside the same contract line are allowed when explicitly
    marked compatible.