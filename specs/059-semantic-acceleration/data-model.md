# Data Model: Advanced Context Intelligence Semantic Acceleration

This slice extends the S5 V1 advanced-context model from
`058-advanced-context-intelligence` rather than replacing it. The entities
below describe the additive semantic layer that must remain explainable,
bounded, and compatible with the existing projection and Canon consumer
boundary. The semantic layer is controlled by a new `semantic_acceleration`
config surface that resolves alongside, and does not redefine, the V1
`AdvancedContextConfig` baseline.

## 1. SemanticAccelerationPolicy

**Purpose**: Represents the effective configuration and runtime capability state
for the optional semantic layer.

**Key Fields**:

- `config_key`: stable config key, `semantic_acceleration`
- `policy_state`: `disabled` or `local`
- `value_source`: `workspace`, `cluster`, `global`, or `default`
- `capability_state`: `ready`, `unavailable`, `unsupported`, or `degraded`
- `fallback_reason`: surfaced reason when the requested semantic mode could not
  run
- `embedding_scope`: eligible source classes that may receive local embeddings
- `budget_snapshot`: bounded semantic expansion and rerank limits applied to
  the active query

**Relationships**:

- resolves into one `HybridRetrievalQuery`
- is resolved through the dedicated `semantic_acceleration` config surface
  alongside the existing `AdvancedContextConfig` baseline
- constrains one `SemanticIndexManifest`

**Validation Rules**:

- `policy_state = disabled` is the safe default when no explicit opt-in exists
- `policy_state = local` requires either `capability_state = ready` or an
  explicit surfaced fallback reason
- semantic acceleration must not silently reinterpret
  `AdvancedContextConfig.retrieval_mode`
- local policy cannot imply remote transmission permission in this slice

## 2. SemanticIndexManifest

**Purpose**: Represents the semantic extension of the existing workspace-local
retrieval index.

**Key Fields**:

- `manifest_id`: stable semantic-manifest identifier
- `workspace_root`: canonical workspace path
- `retrieval_schema_line`: compatible V1 retrieval schema line
- `semantic_schema_line`: semantic-extension schema line
- `vector_extension_state`: `ready`, `missing`, `unsupported`, or `stale`
- `embedding_schema_line`: local embedding payload compatibility line
- `eligible_source_kinds`: source classes indexed for semantic matching
- `last_semantic_refresh_trace_id`: latest trace that materially refreshed
  semantic content
- `last_semantic_refresh_reason`: why semantic content was refreshed or marked
  stale

**Relationships**:

- extends one V1 `RetrievalIndexManifest`
- owns many `SemanticChunkRecord` records
- is referenced by many `HybridRetrievalQuery` records

**Validation Rules**:

- the semantic manifest must remain attached to the same workspace as the V1
  retrieval index
- `vector_extension_state = ready` requires a compatible semantic schema line
- `eligible_source_kinds` must be a subset of V1-explainable retrieval sources

## 3. SemanticChunkRecord

**Purpose**: Represents one locally embedded, provenance-preserving fragment of
eligible retrieval content.

**Key Fields**:

- `chunk_id`: stable semantic chunk identifier
- `source_kind`: `workspace_file`, `trace`, `review_finding`,
  `verification_evidence`, `project_memory`, or `canon_artifact`
- `source_ref`: stable local path or canonical artifact reference
- `provenance_boundary`: local section or Canon semantic boundary the chunk
  belongs to
- `provenance_ref`: persisted provenance pointer surfaced to operators
- `content_hash`: digest used to detect staleness
- `embedding_state`: `pending`, `ready`, `stale`, or `blocked`
- `embedding_dimensions`: declared local embedding width
- `canon_semantic_contract_line`: optional Canon semantic contract line for
  Canon-backed chunks
- `semantic_labels`: additive optional labels carried from producer metadata or
  local derivation

**Relationships**:

- belongs to one `SemanticIndexManifest`
- may back one or more `SemanticMatchDecision` records
- may decorate one existing V1 `RetrievedEvidenceCandidate`

**Validation Rules**:

- every semantic chunk must preserve a stable `source_ref` and `provenance_ref`
- Canon-backed chunks require `canon_semantic_contract_line` and a Canon
  semantic provenance reference
- `embedding_state = ready` requires a non-zero embedding width and compatible
  manifest state

**State Transitions**:

- `pending -> ready`
- `pending -> blocked`
- `ready -> stale`
- `stale -> ready`
- `stale -> blocked`

## 4. HybridRetrievalQuery

**Purpose**: Represents one bounded advanced-context query that starts from the
V1 candidate set and may expand or rerank it through semantic similarity.

**Key Fields**:

- `query_id`: stable query identifier
- `base_query_id`: reference to the V1 retrieval query or projection context
- `policy_state`: effective semantic policy for the query
- `capability_state`: runtime semantic capability observed for this query
- `hybrid_outcome`: `baseline_only`, `expanded`, `reranked`, `skipped`, or
  `fallback`
- `semantic_candidate_budget`: maximum semantic candidates considered
- `semantic_rerank_budget`: maximum semantic promotions applied
- `terminal_state`: `selected`, `degraded`, `insufficient`, `exhausted`, or
  `unavailable`
- `terminal_reason`: explicit operator-facing outcome reason

**Relationships**:

- reads one `SemanticAccelerationPolicy`
- reads one `SemanticIndexManifest`
- owns many `SemanticMatchDecision` records
- updates one existing `AdvancedContextProjection`

**Validation Rules**:

- `hybrid_outcome = expanded` or `reranked` is invalid when policy is disabled
- `hybrid_outcome = reranked` requires at least one existing V1 candidate
- `hybrid_outcome = fallback` requires a non-empty `terminal_reason`

**State Transitions**:

- `pending -> baseline_only`
- `pending -> expanded`
- `pending -> reranked`
- `pending -> fallback`
- `pending -> skipped`

## 5. SemanticMatchDecision

**Purpose**: Represents the explainable semantic decision applied to one
candidate during hybrid retrieval.

**Key Fields**:

- `decision_id`: stable identifier
- `candidate_ref`: selected or rejected evidence reference
- `match_origin`: `fts`, `semantic_expand`, `semantic_rerank`, or `structured_fallback`
- `lexical_score`: optional baseline lexical score
- `semantic_score`: optional local similarity score
- `decision_state`: `selected`, `downgraded`, `rejected`, or `skipped`
- `decision_reason`: explicit explanation for the final decision
- `final_rank`: bounded output order after hybrid evaluation
- `authority_rank`: authoritative rank preserved from the existing candidate

**Relationships**:

- belongs to one `HybridRetrievalQuery`
- annotates one existing `RetrievedEvidenceCandidate`
- may reference one `CanonSemanticArtifactView`

**Validation Rules**:

- `match_origin = semantic_expand` or `semantic_rerank` requires a
  `semantic_score`
- `decision_state = selected`, `downgraded`, `rejected`, or `skipped` requires
  a non-empty `decision_reason`
- semantic decisions may not override conflicting structured runtime context or
  compatible Canon authority inputs

## 6. CanonSemanticArtifactView

**Purpose**: Represents the consumer-side semantic compatibility view for one
Canon artifact.

**Key Fields**:

- `artifact_class`: Canon producer artifact class
- `indexing_contract_line`: Canon indexing contract line
- `semantic_contract_line`: Canon semantic contract line
- `semantic_eligibility`: `eligible`, `excluded`, or `unsupported`
- `semantic_provenance_boundary`: Canon-defined boundary the consumer must
  preserve
- `semantic_provenance_ref`: Canon provenance pointer surfaced in runtime
  output
- `compatibility_state`: `compatible`, `unsupported_contract`,
  `missing_metadata`, or `policy_blocked`

**Relationships**:

- may be referenced by one or more `SemanticChunkRecord` records
- may be surfaced by one or more `SemanticMatchDecision` records

**Validation Rules**:

- `semantic_eligibility = eligible` requires both semantic contract line and
  semantic provenance reference
- excluded or unsupported artifacts must still preserve an explicit skip reason
- compatibility state must never be inferred from prose-only Canon content

## Cross-Entity Invariants

- The V1 advanced-context baseline remains executable when `SemanticAccelerationPolicy`
  resolves to `disabled` or when semantic capability is unavailable.
- The dedicated `semantic_acceleration` policy must resolve independently from
  the V1 `advanced_context` retrieval mode so V1 defaults keep their original
  meaning.
- Every semantic chunk and match decision must map back to a V1 `source_ref` or
  Canon `semantic_provenance_ref`.
- `SemanticMatchDecision` records may enrich recall and ranking, but they must
  never override conflicting structured runtime context.
- Canon semantic compatibility may admit or reject candidates, but it must not
  delegate runtime ranking or fallback control to Canon.
- Every `HybridRetrievalQuery` must end in an explicit operator-facing outcome
  and preserve the reason when it falls back to the V1 path.
