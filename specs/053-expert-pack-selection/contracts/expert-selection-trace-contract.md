# Expert Selection Trace Contract

## Purpose

Define the operator-visible trace projection for expert-pack selection so
session-native read surfaces can explain why a runtime role recommendation was
selected, suppressed, or rejected.

## Required Projection Fields

The trace projection must be able to expose:
- `expert_selection_summary`
- `expert_selection_target`
- `expert_selection_state`
- `selected_expert_packs`
- `suggested_runtime_roles`
- `rejected_expert_candidates`
- `expert_selection_provenance`
- `canon_expertise_inputs` with `contract_version`, `mode`, `expertise_input.expertise_kind`, `expertise_input.domain_families`, `source_ref`, `promotion_state`, `publication_target_class`, used or ignored disposition, and `disposition_reason`

## Projection Rules

- Projection must preserve the ordered selected and rejected candidates.
- Projection must distinguish local selection cues from optional Canon expertise input.
- Projection must preserve the Canon metadata needed to explain why an input was
  used, ignored, blocked, pending, or rejected.
- Projection must surface `none-selected` explicitly rather than implying a hidden default.
- Projection must preserve rejection reasons for candidates and roles that failed compatibility checks.
- Session-native surfaces and trace summaries must read from the same persisted outcome rather than recomputing a second selection path.
