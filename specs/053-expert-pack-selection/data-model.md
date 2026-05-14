# Data Model: Expert Pack Selection

## ExpertPackDefinition

Represents one built-in Boundline expertise entry that may be selected for a
bounded workspace or target.

Fields:
- `pack_id`: stable identifier for the built-in expert pack.
- `label`: operator-facing name for the pack.
- `supported_families`: domain families the pack may attach to.
- `recommended_runtime_roles`: reviewer or runtime roles the pack recommends when selected.
- `selection_summary`: concise explanation of why the pack exists and when it is relevant.
- `required_signals`: cues that must be present before the pack is credible.

## ExpertPackSignal

Represents one local or Canon-derived cue considered during expert-pack
selection.

Fields:
- `kind`: signal class such as domain-template match, reviewer-role route, external-context status, or Canon expertise input.
- `reference`: concrete operator-visible reference for the cue.
- `source`: where the cue came from.
- `status`: whether the cue supported, suppressed, or blocked the candidate.
- `rationale`: operator-facing explanation of why the cue matters.

## RejectedExpertCandidate

Represents a candidate expert pack that was considered but not accepted.

Fields:
- `pack_id`: rejected expert-pack identifier.
- `reason`: explicit rejection or suppression reason.
- `blocking_signals`: ordered signals that explain the rejection.

## ExpertPackSelectionOutcome

Represents the persisted expert-pack selection result for a bounded planning
attempt.

Fields:
- `target_ref`: bounded workspace target or workspace-level selection scope.
- `selection_state`: `selected` or `none-selected`.
- `selected_expert_packs`: ordered list of selected expert-pack identifiers.
- `suggested_runtime_roles`: ordered runtime-role recommendations derived from the selected packs.
- `supporting_signals`: ordered cues that justified the final outcome.
- `rejected_expert_candidates`: rejected or suppressed candidates with explicit reasons.
- `canon_inputs_considered`: Canon expertise inputs that were used, ignored, or rejected, each retaining `contract_version`, `mode`, `expertise_input.expertise_kind`, `expertise_input.domain_families`, `source_ref`, `promotion_state`, `publication_target_class`, final disposition, and `disposition_reason`.
- `summary`: operator-facing summary used by plan and trace projection.

## GoalPlan Expert Projection

Represents the read-side projection carried into session-native inspection
surfaces.

Projected fields:
- `expert_selection_summary`
- `expert_selection_target`
- `selected_expert_packs`
- `suggested_runtime_roles`
- `rejected_expert_candidates`
- `expert_selection_provenance`
- `canon_expertise_inputs`: projected Canon expertise entries preserving `contract_version`, `mode`, `expertise_input.expertise_kind`, `expertise_input.domain_families`, `source_ref`, `promotion_state`, `publication_target_class`, used or ignored outcome, and `disposition_reason`.
