# Expert Pack Selection Contract

## Purpose

Define the minimum operator-visible selection outcome that Boundline must
produce before planning continues for the bounded expert-pack slice.

## Required Outcome Fields

The expert-pack selection outcome must be able to project:
- `target_ref`: the bounded workspace target or workspace-level scope.
- `selection_state`: `selected` or `none-selected`.
- `selected_expert_packs`: ordered selected pack identifiers.
- `suggested_runtime_roles`: ordered runtime-role recommendations derived from the selected packs.
- `supporting_signals`: ordered local or Canon-derived cues that supported the outcome.
- `rejected_expert_candidates`: rejected or suppressed candidates with explicit reasons.
- `canon_inputs_considered`: Canon expertise inputs that were used, ignored, or rejected, including `contract_version`, `mode`, `expertise_input.expertise_kind`, `expertise_input.domain_families`, `source_ref`, `promotion_state`, `publication_target_class`, the final disposition, and `disposition_reason`.
- `summary`: concise operator-facing selection headline.

## Selection Rules

- Selection must use effective Boundline configuration rather than raw per-scope config fragments.
- Selection must remain deterministic for the same bounded target and effective inputs.
- Local-only selection remains valid when no Canon expertise input is available.
- Canon compatibility for this slice is limited to `v1` expertise inputs of
  kind `domain-language` and `domain-model`.
- Canon expertise input only applies when `expertise_input.domain_families`
	intersects the current selected domain families.
- Canon expertise input may support a candidate or be ignored with an explicit
	reason, but it must not choose the runtime role directly or suppress a locally
	credible candidate.

## Rejection Rules

A candidate pack or runtime role must be rejected or suppressed explicitly when:
- the bounded target does not match the pack's supported families
- required local context is unavailable
- the recommended runtime role is unroutable in the effective config
- effective precedence disables or overrides the candidate
- Canon expertise input is incompatible with the current supported contract line
- Canon expertise input is blocked, pending, or otherwise unavailable for
	use in the current bounded selection attempt
- Canon expertise input is published to a proposal, evidence, index, pending,
	blocked, conflicting, or other non-usable Canon surface for this slice
