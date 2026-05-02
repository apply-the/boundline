# Data Model: Context Assembly Foundation

## ContextPack

- **Purpose**: Captures the bounded planning context attached to one goal plan.
- **Key fields**:
  - `pack_id`: stable identifier for the assembled pack
  - `summary`: operator-facing description of the context selected for the goal
  - `credibility`: explicit credibility state for planning and follow-through
  - `inputs`: ordered list of selected context inputs
  - `selected_targets`: narrowed primary file or artifact references
  - `staleness_reason`: optional reason when the pack is no longer credible
- **Lifecycle**:
  - Created during planning before goal-plan confirmation
  - Projected through session and trace surfaces while authoritative
  - Marked stale or insufficient when later follow-through evidence contradicts it

## ContextInput

- **Purpose**: Represents one selected planning input.
- **Key fields**:
  - `kind`: file, symbol_hint, authored_brief, negotiation, trace, canon_artifact
  - `reference`: stable path, identifier, or summary handle
  - `rationale`: bounded explanation for why this input matters
  - `source`: the upstream source from which the input was derived
  - `primary`: whether this input is one of the pack’s main narrowed targets
- **Relationships**:
  - Belongs to exactly one `ContextPack`
  - May contribute to one or more planned task targets

## ContextCredibility

- **Purpose**: Declares whether the assembled context is trustworthy enough to plan or continue bounded work.
- **Values**:
  - `credible`: safe to use for plan confirmation and projection
  - `insufficient`: not enough bounded evidence exists to plan credibly
  - `stale`: earlier context exists but has been contradicted by later state
- **Rules**:
  - Planning may confirm only when credibility is `credible`
  - `insufficient` and `stale` states require explicit surfaced guidance

## ContextProjection

- **Purpose**: Operator-facing summary projected through CLI and trace views.
- **Key fields**:
  - `context_summary`
  - `context_credibility`
  - `context_primary_inputs`
  - `context_provenance_lines`
  - `context_next_action` when credibility is not sufficient
- **Relationships**:
  - Derived from `ContextPack`
  - Rendered on plan, run, status, next, and inspect surfaces

## PlanningSourceSnapshot

- **Purpose**: Captures the bounded upstream sources available when the pack was assembled.
- **Key fields**:
  - `workspace_signals`
  - `authored_input_summary`
  - `negotiation_goal_summary`
  - `latest_trace_ref`
  - `canon_artifact_refs`
- **Lifecycle**:
  - Created at planning time
  - Used for provenance and later staleness reasoning
