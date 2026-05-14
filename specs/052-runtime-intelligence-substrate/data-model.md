# Data Model: Runtime Intelligence Substrate

## RuntimeIndex

Represents the persisted, operator-visible index of bounded context inputs that
steer planning and follow-through.

Fields:
- `context_summary`: concise headline for the active bounded context.
- `context_credibility`: stored state value: `credible`, `stale`, or `insufficient`.
- `context_primary_inputs`: primary references or selected-target fallbacks.
- `context_provenance`: ordered provenance lines for local and optional Canon inputs.
- `context_staleness_reason`: optional explanation when credibility is `stale`.
- `suggested_next_command`: operator-facing next action derived from the active credibility state.

## ContextPack

Represents the bounded planning substrate persisted inside the goal plan.

Fields:
- `pack_id`: stable identifier for the assembled context pack instance.
- `summary`: operator-readable bounded context summary.
- `credibility`: `credible`, `stale`, or `insufficient`.
- `inputs`: ordered `ContextInput` records that explain what the pack was built from.
- `selected_targets`: fallback references when no explicit primary inputs are available.
- `staleness_reason`: optional reason emitted when the pack is stale.

## ContextCredibilityOutcome

Represents the stored credibility state plus the behavioral response it implies.

State values:
- `credible`: planning may continue normally.
- `stale`: runtime surfaces must warn or guide refresh before treating the context as current.
- `insufficient`: planning must stop, narrow, or replan before execution can continue.

Behavioral mapping:
- warning or refresh guidance is derived from `stale`
- replan or bounded stop is derived from `insufficient`
- terminal handling is a runtime outcome triggered when the current insufficient path cannot recover safely

## ContextInput

Represents one bounded context input considered during planning.

Fields:
- `kind`: source class such as `workspace_file`, `symbol_hint`, `authored_brief`, `recent_trace`, `canon_capability`, or `canon_memory`.
- `reference`: the concrete artifact or logical reference surfaced to the operator.
- `rationale`: why that input matters for the active goal.
- `source`: the producer or selection path that introduced the input.
- `primary`: whether the input is primary for planning and projection.

## GoalPlan Context Projection

The goal plan exposes read-side helpers used by session-native commands.

Projected fields:
- `context_summary`
- `context_credibility`
- `context_primary_inputs`
- `context_provenance_lines`
- `canon_memory_staleness_reason`

## TraceSummaryView Context Projection

The inspect and run summaries accumulate trace payload fields into:
- `context_summary`
- `context_credibility`
- `context_primary_inputs`
- `context_provenance`
- `context_staleness_reason`
- `governance_next_action`

## SubstrateTraceProjection

Represents the trace-owned reconstruction of the runtime index after planning or
execution events have been persisted.

Fields:
- `context_summary`
- `context_credibility`
- `context_primary_inputs`
- `context_provenance`
- `context_staleness_reason`
- Canon-derived `canon_memory_*` lines when optional governed evidence contributed to the projection

## Canon Compatibility Snapshot

Represents the supported Canon machine-facing surface used for optional enrichment.

Relevant fields:
- `canon_version`
- `supported_schema_versions`
- `operations`
- `supported_modes`
- `status_values`
- `approval_state_values`
- `packet_readiness_values`
- `compatibility_notes`
