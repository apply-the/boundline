# Advanced Context Semantic Acceleration Projection Contract

## Purpose

Define the minimum operator-facing projection surface for S5.v2 semantic
acceleration so hybrid retrieval, fallback, and Canon semantic compatibility
remain visible on the normal Boundline runtime path.

This contract extends the S5 V1 advanced-context projection contract rather than
replacing it.

## Compact Runtime Surface Requirements

When semantic acceleration is evaluated for an active decision point, Boundline
must keep the following fields, or equivalent structured projection, available
to compact runtime surfaces such as `plan`, `status`, and `next`:

- `semantic_policy_state: disabled|local`
- `semantic_capability_state: ready|unavailable|unsupported|degraded`
- `hybrid_outcome: baseline_only|expanded|reranked|skipped|fallback`
- `retrieval_state: selected|degraded|insufficient|exhausted|unavailable`
- `retrieval_authority_order: structured>canon>workspace_override>semantic`
- `retrieval_index_state: ready|stale|building|insufficient`
- `semantic_terminal_reason: <reason>` whenever hybrid outcome is `skipped` or
  `fallback`
- `semantic_selected_count: <count>` when semantic evaluation selected or
  promoted candidates
- `semantic_rejected_count: <count>` when semantic evaluation rejected or
  skipped candidates

These compact projections must be available on the primary session-native path.
Any compatibility-route use remains explicitly labeled as secondary.

## Inspect Surface Requirements

`inspect` must make it possible to answer all of the following without reading
source code:

- whether semantic acceleration was disabled, unavailable, unsupported, ready,
  or degraded
- whether a candidate came from the V1 set, a semantic expansion, a semantic
  rerank, or explicit structured fallback
- why a semantic candidate was selected, downgraded, rejected, or skipped
- whether a Canon artifact was eligible, excluded, unsupported, or missing
  required semantic metadata
- which semantic fallback reason caused a return to the V1 path

Minimum detailed fields, or equivalent structured output:

- `semantic_policy_state`
- `semantic_capability_state`
- `hybrid_outcome`
- `semantic_terminal_reason`
- `selected_evidence[*].match_origin`
- `selected_evidence[*].source_kind`
- `selected_evidence[*].source_ref`
- `selected_evidence[*].authority_rank`
- `selected_evidence[*].selection_reason`
- `selected_evidence[*].lexical_score` when available
- `selected_evidence[*].semantic_score` when available
- `selected_evidence[*].canon_semantic_contract_line` when the source is Canon-backed
- `selected_evidence[*].canon_semantic_provenance_ref` when the source is Canon-backed
- `rejected_candidates[*].match_origin`
- `rejected_candidates[*].selection_reason`
- `rejected_candidates[*].compatibility_state`

## Trace Projection Requirements

Trace output must preserve a step-by-step account of semantic behavior.
Minimum trace events, or equivalent typed trace records:

- semantic capability evaluated
- semantic index refreshed because eligible content changed
- semantic chunk blocked because eligibility or capability rules failed
- semantic candidate expanded the V1 set
- semantic candidate reranked the V1 set
- semantic candidate rejected or skipped
- Canon artifact skipped because of semantic incompatibility
- semantic acceleration fell back to the V1 path
- hybrid retrieval ended in selected, degraded, insufficient, exhausted, or
  unavailable state

Each trace record must preserve enough provenance to connect the event back to
one hybrid retrieval query and its supporting evidence.

## Failure Projection Rules

When semantic acceleration does not contribute a selected result, the runtime
must make the failure mode explicit:

- `baseline_only`: the V1 path remained sufficient and semantic acceleration was
  disabled or not attempted
- `skipped`: semantic acceleration was considered but excluded before affecting
  candidate ranking
- `fallback`: semantic acceleration was requested but the runtime returned to
  the V1 path because capability, compatibility, or refresh constraints failed
- `degraded`: semantic acceleration ran in a lower-confidence path with visible
  caveats
- `unavailable`: required local semantic capability was not available

The projection must never imply that semantic expansion succeeded when the
runtime actually stayed on, or returned to, the V1 path.

## Explicit Exclusions

This contract does not require:

- a new UI surface outside the existing CLI and trace projections
- hidden vector-ranking logic without surfaced rationale
- Canon-owned control over ranking, fallback thresholds, or delivery stop policy
- remote-provider-specific output in the local-only first slice
