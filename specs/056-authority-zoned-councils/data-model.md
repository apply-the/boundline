# Data Model: Authority-Zoned Delivery Councils

## AuthorityGovernanceInput

Represents the Canon-owned `authority-governance-v1` input that Boundline may
consume at a governed boundary.

Fields:
- `contract_line`: Canon contract identifier; this slice only accepts `authority-governance-v1`.
- `authority_zone`: Canon posture classification such as `green`, `yellow`, `red`, or `restricted`.
- `change_class`: Canon change classification such as `low-impact`, `bounded-impact`, `systemic-impact`, or `critical-operations`.
- `intended_persona`: Canon-owned semantic persona for the governed packet.
- `approval_state`: Canon approval lifecycle input used by Boundline when governance is required.
- `packet_readiness`: Canon readiness lifecycle input used by Boundline when deciding whether a governed packet can support progression.
- `risk`: Canon risk classification used during local control resolution.
- `persona_anti_behaviors`: optional Canon provenance-only semantics.
- `primary_artifact`: optional Canon provenance-only artifact reference.
- `artifact_order`: optional Canon provenance-only artifact ordering.
- `promotion_refs`: optional Canon provenance-only promotion lineage.
- `stage_role_hints`: optional advisory Canon hints that Boundline may inspect but not execute directly.

## AuthorityControlResolution

Represents the local Boundline decision produced for one governed stage
boundary.

Fields:
- `stage_ref`: governed stage or workflow boundary being evaluated.
- `boundary_trigger`: local review or governance trigger that caused evaluation.
- `canon_contract_line`: consumed Canon contract line or explicit absence.
- `effective_control_class`: local control outcome derived from Canon inputs plus local evidence posture.
- `council_profile`: bounded council profile selected for the current boundary.
- `stop_semantics`: explicit progression outcome such as `proceed`, `council_required`, `human_gate_required`, or `hard_stop`.
- `human_gate_required`: whether a human approval gate is still required before continuation.
- `rationale`: operator-visible explanation of why the control outcome was chosen.
- `provenance_refs`: ordered Canon and local references supporting the decision.

## CouncilProfileDecision

Represents the persisted runtime council composition and eligibility result for
one resolved boundary.

Fields:
- `profile_id`: one of `none`, `light_single`, `yellow_pair`, `red_five`, or `restricted_manual`.
- `mandatory_roles`: runtime roles that must be present for the profile to be credible.
- `selected_roles`: ordered runtime roles actually assigned locally.
- `independence_state`: whether reviewer independence succeeded, degraded, or failed.
- `quorum_state`: whether the profile met its minimum participation rule.
- `selection_summary`: concise projection headline used by status and inspect surfaces.

## CouncilFindingRecord

Represents one persisted council finding tied to a governed boundary.

Fields:
- `finding_id`: stable identifier for the finding within the session.
- `reviewer_ref`: reviewer identity and local runtime provenance.
- `runtime_role`: the local role under which the reviewer contributed.
- `severity`: bounded severity such as note, concern, or block.
- `summary`: concise operator-facing description of the finding.
- `required_action`: explicit remediation or adjudication requirement.
- `confidence`: reviewer confidence level or equivalent explicit confidence signal.
- `evidence_refs`: ordered references supporting the finding.
- `disposition`: current resolution state for the finding.

## ProducerResponseRecord

Represents the explicit producer response recorded for a concern or blocking
finding.

Fields:
- `finding_id`: target finding identifier.
- `response_state`: `accepted`, `rejected`, or `deferred`.
- `rationale`: producer justification for that response.
- `follow_up_refs`: remediation tasks, plan updates, or escalation references created from the response.
- `resolved`: whether the response actually clears the finding for progression.

## AdjudicationOutcome

Represents the explicit resolution of mixed or blocking council results.

Fields:
- `outcome_state`: adjudicated result such as proceed, warn, require-human-gate, or hard-stop.
- `decision_owner`: local authority that made the final adjudication decision.
- `blocking_findings`: unresolved findings that still constrain continuation.
- `next_action`: required next operator or runtime action.
- `summary`: concise explanation projected to session-native surfaces.

## Session Governance Projection

Represents the read-side session projection that later commands must reuse.

Projected fields:
- `latest_governance_contract_line`
- `latest_authority_zone`
- `latest_change_class`
- `latest_effective_control_class`
- `latest_council_profile`
- `latest_stop_semantics`
- `latest_findings_summary`
- `latest_producer_response_summary`
- `latest_adjudication_summary`
- `latest_optional_canon_provenance`