# Data Model: Control Graduation And Adaptive Governance

## AdaptiveGovernanceInput

Represents the optional Canon-owned `adaptive-governance-v1` companion contract
that Boundline may consume together with the required
`authority-governance-v1` posture baseline.

Fields:
- `contract_line`: companion contract identifier; this slice only accepts `adaptive-governance-v1` when the companion contract is present.
- `governance_state`: Canon semantic governance-state label such as `advisory`, `catch`, `rule`, or `hook`.
- `rollout_profile`: Canon semantic maturity label such as `minimal`, `guided`, `governed`, or `strict`.
- `state_rationale`: optional Canon semantic explanation for why the state label applies.
- `profile_rationale`: optional Canon semantic explanation for why the rollout profile applies.

Validation rules:
- The entire companion object is optional for first-slice runtime behavior.
- If the companion object is present, `contract_line`, `governance_state`, and `rollout_profile` are required for companion compatibility.
- An incompatible or incomplete companion contract does not repair a missing required `authority-governance-v1` baseline.

## GovernanceRuntimeState

Represents the local Boundline operational governance posture for one boundary.

Values:
- `advisory`
- `catch`
- `rule`
- `hook`

Transition rules:
- state may promote, downgrade, roll back, suspend, or recover
- state changes must be recorded with rationale and visible outcome

## GovernanceRolloutProfile

Represents the operator-visible maturity profile for governance adoption.

Values:
- `minimal`
- `guided`
- `governed`
- `strict`

Validation rules:
- rollout profile remains distinct from S3 council profile
- rollout profile may change only through explicit runtime decision plus traceable rationale

## ConfidenceAssessment

Represents the runtime-owned assessment of whether current evidence justifies
stronger or weaker governance behavior.

Fields:
- `confidence_level`: bounded level such as `low`, `medium`, `high`, or `critical`.
- `supporting_signals`: ordered evidence signals such as review diversity, artifact coverage, verification quality, trace completeness, or historical success.
- `blocking_gaps`: missing evidence, reviewer, or credibility gaps that prevent stronger automation.
- `summary`: concise explanation projected to operators.
- `evidence_refs`: ordered references supporting the assessment.

Validation rules:
- confidence is runtime-owned and must not be overwritten by Canon semantic contracts
- low confidence may reduce autonomy or trigger escalation

## TrustEvolutionRecord

Represents how governance trust changes over time for a work class or boundary.

Fields:
- `trust_state`: current posture such as growing, stable, decayed, suspended, or recovering.
- `change_reason`: explanation for why trust changed.
- `supporting_events`: incidents, overrides, successful deliveries, review outcomes, or verification results that caused the change.
- `effective_from`: boundary or session point where the trust change became active.
- `next_recovery_condition`: explicit operator-visible condition for regaining stronger autonomy.

Validation rules:
- repeated bypass, ignored findings, incidents, or incomplete traces may decay trust
- recovery must remain staged and explicit rather than automatic and silent

## DegradationOutcome

Represents an explicit narrowing of governance behavior when required conditions
cannot be satisfied.

Fields:
- `degradation_mode`: operational outcome such as `advisory_fallback`, `smaller_council`, `human_gate`, `reduced_autonomy`, `verification_only`, or `execution_block`.
- `mapped_stop_semantics`: existing S3 stop result used for runtime progression.
- `reason`: explanation for why degradation was needed.
- `unsatisfied_conditions`: ordered list of missing evidence, reviewer, route, or compatibility conditions.
- `next_action`: operator or runtime action required after degradation.

Validation rules:
- degradation must remain visible, explainable, and traceable
- degradation may not silently weaken governance

## EscalationEvent

Represents the explicit transfer of authority when runtime confidence or
conditions are insufficient for continuation.

Fields:
- `trigger`: cause such as low confidence, conflicting findings, missing mandatory reviewer, unsupported contract, or blocked governance.
- `target`: stronger authority path such as additional review, human approval, governance review, or security review.
- `required_gate`: whether a human or higher-trust gate is required.
- `summary`: concise explanation projected to operators.
- `blocking_refs`: ordered references to unresolved blockers that forced escalation.

Validation rules:
- escalation is a runtime-owned action and must not be implied silently
- escalation may coexist with degradation but both outcomes must remain separately inspectable

## OverrideRecord

Represents an operator-approved deviation from the current governance posture.

Fields:
- `override_id`: stable identifier for the override event.
- `boundary_ref`: governed boundary affected by the override.
- `previous_state`: runtime state before the override.
- `resulting_state`: runtime state or stop posture after the override.
- `operator_rationale`: explicit human rationale for the deviation.
- `lineage_refs`: trace or artifact references that preserve the decision trail.

Validation rules:
- overrides must be explicit, traced, and justified
- repeated overrides may reduce trust and increase future governance friction

## Session Adaptive Governance Projection

Represents the read-side projection later commands must reuse.

Projected fields:
- `latest_governance_runtime_state`
- `latest_governance_rollout_profile`
- `latest_governance_confidence_level`
- `latest_governance_trust_state`
- `latest_governance_degradation_mode`
- `latest_governance_escalation_target`
- `latest_governance_override_summary`
- `latest_governance_contract_lines`
- `latest_governance_reason`
- `latest_governance_next_action`

## Relationship Notes

- `AdaptiveGovernanceInput` is optional semantic input from Canon; it may influence explanation or initial maturity seeding but does not own runtime behavior.
- `ConfidenceAssessment` and `TrustEvolutionRecord` inform the transition between `GovernanceRuntimeState` values.
- `DegradationOutcome`, `EscalationEvent`, and `OverrideRecord` attach to governance-state changes and must be preserved in session and trace projection.
- `GovernanceRolloutProfile` describes adoption depth; S3 council profiles still describe council shape.