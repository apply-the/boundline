# Advanced Context Vector Observability Contract

## Purpose

Define the minimum operator-facing observability surface for real `sqlite-vec`
activation in Boundline's advanced-context runtime. This contract extends the
existing semantic-acceleration projection instead of replacing it.

## Compact Runtime Surface Requirements

When semantic retrieval is evaluated for a bounded decision point, `plan`,
`status`, and `next` must surface the following fields, or an equivalent typed
projection:

- `semantic_policy_state: disabled|local`
- `semantic_capability_state: ready|missing|unsupported|degraded|corrupt`
- `semantic_engine: disabled|baseline_json|sqlite_vec`
- `hybrid_outcome: baseline_only|expanded|reranked|skipped|fallback`
- `retrieval_state: selected|degraded|insufficient|exhausted|unavailable`
- `retrieval_index_state: ready|stale|incompatible|degraded|corrupt|missing`
- `vector_query_count: <integer>` when semantic retrieval was attempted
- `vector_candidates_returned: <integer>` when vector search returned chunk
  candidates
- `semantic_fallback_reason: <reason>` whenever the runtime does not complete
  on the preferred vector path

These fields must remain available on the primary session-native path. Any
compatibility-route use remains explicitly secondary.

## Inspect Surface Requirements

`inspect` must make it possible to answer all of the following without reading
source code:

- whether semantic retrieval ran on the disabled path, the JSON fallback path,
  or the real `sqlite-vec` path
- whether vector capability was ready, missing, unsupported, degraded, or
  corrupt
- how many vector queries ran and how many chunk candidates they returned
- whether a selected source was expanded or reranked through the semantic path
- why a source or chunk candidate was rejected, downgraded, skipped, or
  collapsed away
- whether the runtime returned to the baseline path because of missing
  capability, stale or incompatible state, dimension mismatch, or corruption

Minimum detailed fields, or equivalent structured output:

- `semantic_policy_state`
- `semantic_capability_state`
- `semantic_engine`
- `hybrid_outcome`
- `vector_query_count`
- `vector_candidates_returned`
- `semantic_fallback_reason`
- `selected_evidence[*].match_origin`
- `selected_evidence[*].source_ref`
- `selected_evidence[*].source_kind`
- `selected_evidence[*].authority_rank`
- `selected_evidence[*].selection_reason`
- `selected_evidence[*].semantic_score` when available
- `selected_evidence[*].collapsed_from_chunk_count` when multiple chunks map to
  the same source
- `selected_evidence[*].canon_semantic_contract_line` when the source is
  Canon-backed
- `selected_evidence[*].canon_semantic_provenance_ref` when the source is
  Canon-backed
- `rejected_candidates[*].match_origin`
- `rejected_candidates[*].selection_reason`
- `rejected_candidates[*].compatibility_state`

## Trace Requirements

Trace output must preserve step-by-step vector lifecycle and retrieval events.
Minimum trace records, or equivalent typed events:

- vector capability evaluated
- trusted extension load attempted
- vector capability downgraded or failed
- manifest marked stale or incompatible
- incremental refresh upserted changed chunks
- disappeared source deleted chunk and vector rows
- vector query executed
- vector query returned chunk candidates
- semantic candidate expanded the baseline set
- semantic candidate reranked the baseline set
- semantic candidate rejected or skipped
- runtime fell back to `baseline_json` or disabled semantic execution

Each trace record must preserve enough context to connect the event back to one
retrieval query or one lifecycle operation.

## Failure Projection Rules

When vector-backed retrieval does not produce the preferred outcome, the runtime
must make the reason explicit:

- `disabled`: semantic retrieval was not requested
- `baseline_json`: semantic retrieval continued on the JSON fallback path
- `fallback`: vector retrieval was requested but the runtime returned to the
  baseline path
- `degraded`: vector retrieval ran with visible caveats
- `missing`: the index or vector capability was not present
- `incompatible`: schema, fingerprint, or dimension mismatch prevented safe use
- `corrupt`: the DB, manifest, or vector table could not be trusted

The projection must never imply that the `sqlite-vec` path succeeded when the
runtime actually used a fallback or stayed on the baseline path.

## Explicit Exclusions

This contract does not require:

- a new UI or dashboard outside the existing CLI and trace surfaces
- default chunk-level user-facing output in normal status views
- hidden scoring or ranking heuristics without surfaced rationale
- Canon-owned control over vector ranking, fallback thresholds, or stop policy