# Data Model: Real sqlite-vec Activation And DB Merge Strategy

This slice extends the existing advanced-context and semantic-acceleration
models from `058-advanced-context-intelligence` and
`059-semantic-acceleration`. The entities below describe the additive lifecycle
state needed to activate real `sqlite-vec` queries, keep the derived index
incremental, and preserve explicit operator recovery paths.

## 1. DerivedIndexManifest

**Purpose**: Represents the lightweight companion manifest for the workspace-
local derived retrieval index.

**Key Fields**:

- `schema_version`: stable retrieval-index compatibility line such as
  `retrieval-index-v3`
- `workspace_root`: canonical workspace path the manifest belongs to
- `git_branch`: branch last seen by the index
- `git_head`: HEAD commit last used to refresh the index
- `last_seen_head`: most recent HEAD observed during a cheap status probe
- `index_status`: `ready`, `missing`, `stale`, `incompatible`, `degraded`,
  `corrupt`, or `semantic_unavailable`
- `last_refresh_at`: timestamp of the most recent successful refresh or rebuild
- `last_refresh_reason`: `manual_refresh`, `rebuild`, `schema_change`,
  `branch_change`, `config_change`, `chunker_change`, `capability_change`, or
  `doctor_repair`
- `file_count`: number of indexed sources
- `chunk_count`: number of indexed chunks
- `fts5_state`: `ready`, `missing`, or `corrupt`
- `sqlite_vec_state`: `ready`, `missing`, `unsupported`, `degraded`, or
  `corrupt`
- `semantic_engine`: `disabled`, `baseline_json`, or `sqlite_vec`
- `workspace_fingerprint`: digest of the eligible workspace evidence set
- `config_fingerprint`: digest of relevant Boundline config
- `chunker_fingerprint`: digest of the active chunking algorithm and limits
- `embedding_model_fingerprint`: digest or stable label for the embedding shape

**Relationships**:

- owns many `SourceDigestRecord` records
- constrains many `SemanticChunkRecord` and `SemanticVectorRecord` records
- is read and updated by many `IndexMaintenanceOperation` records

**Validation Rules**:

- `workspace_root` must match the active workspace using the index
- `index_status = ready` requires compatible schema, matching fingerprints, and
  non-corrupt FTS and semantic state
- `semantic_engine = sqlite_vec` requires `sqlite_vec_state = ready`
- any schema, chunker, or embedding fingerprint mismatch must resolve to
  `incompatible` rather than silent reuse

## 2. SourceDigestRecord

**Purpose**: Represents the current indexed snapshot for one eligible source.

**Key Fields**:

- `source_ref`: stable workspace-relative or Canon-derived reference
- `source_kind`: `workspace_file`, `trace`, `review_finding`,
  `verification_evidence`, `project_memory`, or `canon_artifact`
- `content_hash`: digest of the source content used to detect refresh needs
- `compatibility_state`: `compatible`, `excluded`, `unsupported`, or `blocked`
- `authority_rank`: preserved baseline authority rank for the source
- `last_indexed_at`: most recent timestamp this source participated in refresh
- `chunk_count`: number of active chunks currently derived from the source
- `source_presence_state`: `present`, `deleted`, or `skipped`

**Relationships**:

- belongs to one `DerivedIndexManifest`
- owns many `SemanticChunkRecord` records
- may be referenced by one `AdvancedContextVectorProjection`

**Validation Rules**:

- `source_ref` must remain stable across refreshes for unchanged content
- `chunk_count` must equal the number of non-deleted chunks derived from the
  source
- `source_presence_state = deleted` must eventually remove all related active
  chunk and vector rows during refresh

## 3. SemanticChunkRecord

**Purpose**: Represents one stable chunk of a source that can be refreshed,
deleted, or collapsed back to source-level evidence.

**Key Fields**:

- `chunk_id`: stable identifier derived from `source_ref` plus `chunk_ordinal`
- `source_ref`: owning source reference
- `chunk_ordinal`: stable zero-based chunk position within the source
- `chunk_range`: source-relative boundary used for provenance and debugging
- `provenance_boundary`: boundary label preserved in operator output
- `provenance_ref`: persisted pointer to the chunk or collapsed source proof
- `content_hash`: digest of the chunk text
- `chunk_state`: `ready`, `stale`, `blocked`, `deleted`, or `missing_vector`
- `embedding_dimensions`: declared embedding width for the chunk
- `canon_semantic_contract_line`: optional Canon semantic line for Canon-backed
  chunks
- `semantic_labels`: additive labels preserved for explainability

**Relationships**:

- belongs to one `SourceDigestRecord`
- may own one `SemanticVectorRecord`
- may be referenced by many `AdvancedContextVectorProjection` decisions

**Validation Rules**:

- `chunk_id` must remain stable for unchanged content under the same chunking
  algorithm
- `chunk_state = ready` requires a matching vector row or a surfaced fallback
  reason explaining why the runtime is in `baseline_json`
- Canon-backed chunks require preserved provenance and contract metadata

**State Transitions**:

- `ready -> stale`
- `stale -> ready`
- `ready -> blocked`
- `ready -> deleted`
- `missing_vector -> ready`

## 4. SemanticVectorRecord

**Purpose**: Represents the actual vector-backed row stored in the `vec0`
virtual table.

**Key Fields**:

- `chunk_id`: stable foreign key to the semantic chunk metadata row
- `vector_schema_line`: compatibility line for the vector table shape
- `embedding_dimensions`: concrete width stored for the vector row
- `write_generation`: monotonic generation number or timestamp for refresh
- `vector_state`: `ready`, `missing`, `stale`, `dimension_mismatch`, or
  `corrupt`

**Relationships**:

- belongs to one `SemanticChunkRecord`
- is constrained by one `DerivedIndexManifest`

**Validation Rules**:

- `embedding_dimensions` must match both the chunk metadata and manifest
  fingerprint
- `vector_state = ready` requires the row to exist in the vector table and be
  queryable under the active capability state
- any detected dimension mismatch must force `vector_state = dimension_mismatch`
  and an incompatible manifest state

## 5. IndexMaintenanceOperation

**Purpose**: Represents one bounded operator-visible lifecycle action for the
derived index.

**Key Fields**:

- `operation_id`: stable operation identifier
- `command_name`: `status`, `refresh`, `rebuild`, `clean`, or `doctor`
- `trigger`: `manual`, `post_checkout`, `post_merge`, `post_rewrite`,
  `schema_change`, `config_change`, or `capability_change`
- `pre_state`: manifest status before the operation
- `post_state`: manifest status after the operation
- `sources_scanned`: number of sources inspected
- `chunks_upserted`: number of chunk metadata rows updated or inserted
- `chunks_deleted`: number of chunk rows deleted
- `vector_rows_written`: number of vector rows inserted or replaced
- `fallback_reason`: explicit reason when the operation could not finish on the
  preferred path
- `recommended_action`: surfaced next step for the operator

**Relationships**:

- reads and may update one `DerivedIndexManifest`
- may update many `SourceDigestRecord`, `SemanticChunkRecord`, and
  `SemanticVectorRecord` records
- may emit one `AdvancedContextVectorProjection` fallback result when invoked
  indirectly during retrieval

**Validation Rules**:

- `status` must not trigger a heavy rebuild
- `refresh` may only mutate changed or deleted sources unless a rebuild trigger
  is detected
- `rebuild` must recreate both metadata and vector-backed semantic state from
  scratch
- `clean` must delete only disposable derived artifacts and leave authored
  workspace state intact

## 6. AdvancedContextVectorProjection

**Purpose**: Represents the operator-facing retrieval projection once real
vector-backed selection participates in advanced-context retrieval.

**Key Fields**:

- `query_id`: stable retrieval query identifier
- `semantic_engine`: `disabled`, `baseline_json`, or `sqlite_vec`
- `semantic_capability_state`: `ready`, `missing`, `unsupported`, `degraded`,
  or `corrupt`
- `vector_query_count`: number of vector queries executed for the decision point
- `vector_candidates_returned`: number of chunk candidates returned by vector
  search before source-level collapse
- `hybrid_outcome`: `baseline_only`, `expanded`, `reranked`, `fallback`, or
  `skipped`
- `semantic_fallback_reason`: explicit surfaced reason when vector search did
  not contribute a final selection
- `selected_source_refs`: chosen source-level evidence refs after collapse
- `rejected_source_refs`: rejected source-level evidence refs with reasons

**Relationships**:

- references one `DerivedIndexManifest`
- may reference many `SemanticChunkRecord` records before collapsing to source
  level
- extends the existing advanced-context projection visible to `plan`, `status`,
  `next`, and `inspect`

**Validation Rules**:

- `semantic_engine = sqlite_vec` requires `vector_query_count >= 1` whenever
  semantic retrieval is attempted
- `semantic_fallback_reason` is required when `hybrid_outcome = fallback` or
  the capability state is not `ready`
- chunk-level matches must collapse back to source-level evidence without losing
  provenance, Canon labels, or rejection explanations

## Cross-Entity Invariants

- The advanced-context baseline remains executable whenever the manifest is
  missing, stale, incompatible, degraded, or corrupt.
- Any change in schema line, chunker fingerprint, embedding dimensions, or
  trusted vector capability invalidates incremental reuse and forces an
  incompatible or rebuild-required state.
- Source deletion must remove both metadata and vector rows for all affected
  chunks during a refresh.
- Optional Git hooks may only mark stale state or request a bounded maintenance
  action; they must not trigger an unbounded rebuild inside the Git operation.
- Standard operator surfaces must remain source-oriented even when vector search
  operates at chunk granularity.