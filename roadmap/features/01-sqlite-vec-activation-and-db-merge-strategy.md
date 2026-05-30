# TD-002: Real sqlite-vec Activation And DB Merge Strategy

## Status

Proposed - Ready For Prioritization

## Type

Tech Debt / Architecture / Specification

---

# 1. Background

Boundline already persists advanced-context retrieval state in the workspace-local
SQLite file `.boundline/context-intelligence/retrieval-index.sqlite3`.

The current semantic-acceleration baseline delivers:

- a local SQLite document store plus FTS5 baseline
- semantic metadata persisted beside the baseline retrieval index
- optional local semantic policy and operator-visible fallback states
- hybrid explainability in `plan`, `status`, `next`, and `inspect`

However, the runtime does not yet use `sqlite-vec` as the semantic query engine.
Today the semantic path stores embeddings as JSON payloads in `semantic_chunks`
and computes cosine similarity in Rust after reading rows back out of SQLite.

The root workspace already reserves the feature flags `semantic-acceleration` and
`sqlite-vec`. This specification provides the runtime closure needed to make the
shipped semantic-acceleration surface use real vector tables and SQL-side
nearest-neighbor selection, while preserving the current bounded hybrid semantics.

Because this database is workspace-local and derived from retrievable workspace evidence,
the schema-evolution strategy minimizes merge pain by avoiding fragile in-place
migrations where a rebuild is cheaper and safer.

---

# 2. Non-Goals

To prevent scope creep, the following are explicitly **out of scope** for this specification:

- remote embeddings (this remains 100% local)
- graph DB integrations
- durable DB migration frameworks for operator data
- automatic full DB rebuilds inside Git hooks
- chunk-level user-facing output (unless strictly for diagnostics)
- altering the V1 FTS authority model

---

# 3. Current State Snapshot

| Area | Current State | Gap To Close |
|---|---|---|
| Semantic storage | `semantic_chunks` stores `chunk_text` and `embedding_payload_json` | No `vec0` virtual table yet |
| Capability detection | Runtime inspects `PRAGMA module_list` for `vec0` and `vec_each` | Detection exists, but no extension-backed query path |
| Query execution | Semantic matches are read row-by-row and scored in Rust | No SQL-side top-k similarity query |
| Refresh behavior | Semantic refresh deletes all semantic rows and rebuilds them | No incremental upsert/delete path |
| Chunking | One semantic chunk per `source_ref`, truncated to 8 KiB | No multi-chunk granularity for large files |
| Hybrid semantics | FTS baseline plus semantic expand/rerank annotations | Must be preserved during engine swap |

---

# 4. Delivery Goal

Deliver a complete `sqlite-vec`-backed semantic path that:

- keeps the same workspace-local SQLite file and operator-facing projection
- preserves the V1 FTS and structured fallback as the authoritative baseline
- performs semantic nearest-neighbor selection inside SQLite rather than in Rust
- remains optional, local-only, and explicitly degradable
- treats the retrieval index as a derived cache, avoiding migration-heavy debt

---

# 5. Activation And Implementation Plan

## 5.1 Capability Boundary And Extension Loading

Create a dedicated vector-capability boundary separating feature gating, extension loading,
and runtime capability reporting (`ready`, `missing`, `unsupported`, `degraded`).

**Extension Loading Strategy:**
- The extension loading mechanism relies on dynamic loading (`load_extension`) wrapped
  in a safe detection boundary, leaning on `rusqlite`'s existing dynamic extension support.
- If dynamic loading fails, or if `sqlite-vec` is not compiled into the host environment,
  the capability gracefully downgrades to `missing` or `unsupported`.

**Acceptance Signal:**
- `status` and `inspect` surface the correct semantic policy and capability state.

## 5.2 Vector Table Target Schema

`semantic_chunks` remains the explainable metadata table. A new dedicated `vec0` virtual
table must be created for the actual embedding vectors.

**Illustrative Target Schema:**
```sql
CREATE VIRTUAL TABLE IF NOT EXISTS semantic_chunk_vectors USING vec0(
    chunk_id TEXT PRIMARY KEY,
    embedding FLOAT[1536] -- Or dynamically bound to configured dimension
);
```

**Dual-Write and JSON Fallback Conditions:**
1. **Create `semantic_chunk_vectors`** beside the existing JSON payload column.
2. **Dual-write** both formats during the refresh phase.
3. The `embedding_payload_json` column is retained as a temporary rollback path.
4. **Exit condition**: The JSON column can only be removed after the vector-read path
   is proven stable across a full release, and once tests comprehensively cover scenarios
   where the vector table is corrupt, stale, or unavailable. Operator output must remain
   identical or explicitly versioned during this transition.

## 5.3 Replace Full Rebuild With Incremental Refresh

Move to an incremental refresh flow to avoid dropping the entire semantic table on minor edits.

- Derive stable chunk ids from `source_ref` plus a chunk ordinal.
- Compare content hashes before rewriting vectors.
- Upsert only changed chunks and delete chunks whose source disappeared.
- Maintain a manifest-level refresh reason and schema line for diagnostics.

## 5.4 Testable Invariants for Hybrid Query Execution

Replace the Rust-side full scan with a SQL-side top-k vector query.
The assertion that "semantic may only expand or rerank within existing authority"
is translated into the following **testable invariants**:

1. Semantic queries MUST NOT promote evidence that is explicitly incompatible with the current boundary.
2. Semantic expansion MUST NOT exceed the configured `expansion_limit`.
3. Semantic reranking MUST NOT eliminate authoritative candidates discovered by the FTS baseline.
4. Rejected candidates MUST preserve the `rejected_candidate:` line or equivalent structure.
5. Provenance and Canon labels MUST survive the collapse from `chunk` back to `source_ref`.

## 5.5 Improve Chunking

Split large documents into stable chunk windows, preserving `source_ref` plus a stable chunk
suffix (e.g., `#chunk-N`). Keep provenance and Canon boundary information strictly per chunk.

## 5.6 Concrete Failure Modes to Handle

The runtime must deterministically handle the following states:
- `vec0` extension missing from the host environment.
- `vec0` extension present, but virtual table creation fails.
- Vector table is empty, but `semantic_chunks` metadata is populated.
- Dimension mismatch (e.g., table initialized with 1536, but model returns 768).
- Database file is corrupt.
- Index manifest is stale or missing.
- JSON fallback payload is present, but the vector table is unexpectedly absent.

## 5.7 Minimum Observability

To debug the hybrid semantic flow, the manifest, status, and inspect outputs
must surface minimal but explicit routing metrics:

- `semantic_engine`: `baseline_json` | `sqlite_vec` | `disabled`
- `vector_query_count`: Integer
- `vector_candidates_returned`: Integer
- `semantic_fallback_reason`: Enum/String explaining degradation (if any)

---

# 6. Branch And Source-Control Policy

1. `retrieval-index.sqlite3` is a local derived cache.
2. It MUST NOT be treated as a mergeable source-of-truth.
3. It SHOULD NOT be committed.
4. If committed accidentally, or if schema/content conflicts occur, delete and rebuild.
5. Branch checkout MAY invalidate the index when workspace digest, schema line, or manifest version changes.
6. WAL/SHM sidecar files MUST NOT be committed.
7. Durable operator-owned data MUST NOT be added to this DB unless the feature also introduces versioned migrations and a strict merge policy.

---

# 7. Rebuild Policy For Schema and Dimension Changes

Because the database is a derived cache, destructive schema changes trigger rebuilds rather than complex migrations.

- **Change in `SEMANTIC_EMBEDDING_DIMENSIONS`**: Incompatible rebuild required.
- **Change in Chunking Algorithm**: Incompatible rebuild required.
- **Change in Vector Table Schema**: Incompatible rebuild required.
- **Change in Explainability Metadata (compatible)**: Expand-contract dual-write.

---

# 8. Derived Index Lifecycle And Git Hygiene

Boundline must actively protect Git from derived files, provide explicit index lifecycle
commands, and offer optional lightweight Git hooks that manage freshness without executing
heavy, blocking rebuilds.

## 8.1 Architectural Decision

The source of truth remains:
```text
workspace files
.boundline/session / traces / config (authoritative)
Canon project memory (if present)
```

The database `.boundline/context-intelligence/retrieval-index.sqlite3` is:
```text
local
derived
disposable
rebuildable
not mergeable
not authoritative
```
Boundline treats it strictly as a technical cache.

## 8.2 `boundline init` Updates `.gitignore`

During `boundline init`, Boundline appends/updates a managed block to `.gitignore`:

```gitignore
# BEGIN boundline derived indexes
.boundline/context-intelligence/*.sqlite3
.boundline/context-intelligence/*.sqlite3-wal
.boundline/context-intelligence/*.sqlite3-shm
.boundline/context-intelligence/*.sqlite3-journal
.boundline/context-intelligence/*.lock
.boundline/context-intelligence/tmp/
# END boundline derived indexes
```

**Behavior:**
- If `.gitignore` exists: append or update the managed block using markers.
- If `.gitignore` does not exist: create it.
- If equivalent rules already exist: do not duplicate.
- If the repository is not Git-tracked: skip with an informational message.
- If `.gitignore` is read-only: emit a warning with manual instructions.
- Boundline DOES NOT blanket-ignore the entire `.boundline/` directory, as config and artifacts are intentionally versionable.

## 8.3 CLI Commands

A clear family of index lifecycle commands:

### `boundline index status`
Reads the manifest and DB without heavy processing.
Outputs state (`ready`, `missing`, `stale`, `incompatible_schema`, `degraded`, `corrupt`, `semantic_unavailable`) and detailed metadata (schema line, manifest version, workspace fingerprint, `fts_ready`, `sqlite_vec_ready`, fallback mode, recommended action).

### `boundline index refresh`
Performs an incremental refresh:
- scans the file manifest and compares content hashes
- updates changed files/chunks and deletes disappeared chunks
- updates FTS and vector rows
- writes the updated manifest

By default, this is a safe, incremental operation.

### `boundline index rebuild`
Performs a full rebuild. Used when schemas or embedding dimensions change, when the DB is corrupt, or upon explicit operator request.

### `boundline index clean`
Deletes the derived cache entirely. Subsequent status checks will report `missing`.

### `boundline index doctor`
Runs diagnostics: checks `.gitignore` hygiene, DB version compatibility, WAL/SHM presence, vector schema validity, and verifies if the DB file has accidentally been tracked by Git.

## 8.4 Git Hooks (Optional)

Git hooks are invasive and are NOT installed automatically without explicit consent (e.g., via `boundline index install-hooks` or a prompt during `init`).

**Recommended Hooks:**
- `post-checkout`: Marks the index stale. Optionally triggers a lightweight refresh of changed files.
- `post-merge`: Marks the index stale. Runs lightweight status.
- `post-rewrite`: Marks the index stale.

Hooks NEVER execute a full rebuild by default to prevent blocking Git operations for minutes.
The hook action (`mark-stale` vs `refresh-light` with a strict time budget) is configurable via `.boundline/config.toml`.

## 8.5 Manifest Index

A manifest resides alongside the database at `.boundline/context-intelligence/manifest.json`.

```json
{
  "schema_version": "retrieval-index-v3",
  "workspace_root": "/repo",
  "git": {
    "branch": "feature/foo",
    "head": "abc123",
    "last_seen_head": "abc123"
  },
  "index": {
    "status": "ready",
    "last_refresh_at": "2026-05-29T10:15:00Z",
    "last_refresh_reason": "manual-refresh",
    "file_count": 1234,
    "chunk_count": 8910
  },
  "capabilities": {
    "fts5": "ready",
    "sqlite_vec": "missing",
    "semantic": "degraded"
  },
  "fingerprints": {
    "workspace": "sha256:...",
    "config": "sha256:...",
    "chunker": "sha256:...",
    "embedding_model": "local:none"
  }
}
```
This allows Boundline to rapidly evaluate staleness without opening the SQLite file.

## 8.6 Stale Index Detection

The index is marked `stale` if any of the following change:
- Git HEAD or branch
- Tracked file digests
- `.boundline/config.toml`
- Schema version, chunking algorithm, or embedding dimension
- `sqlite-vec` host capability

## 8.7 Skill vs CLI Responsibilities

- **CLI (`boundline index *`)**: The source of truth. Handles deterministic, testable, cross-host execution logic.
- **Skill (`boundline-index-maintenance`)**: The UX wrapper. Calls `status`, parses JSON, explains the state to the operator, and asks for confirmation before executing potentially expensive rebuild operations. The skill MUST NOT reimplement index logic.

## 8.8 Recommended Operation Flow

- **Init**: Adds `.gitignore` rules, creates manifest, proposes hooks. Does NOT build a heavy semantic index automatically.
- **Branch Change**: `post-checkout` hook marks index stale.
- **Plan Execution**: `boundline plan` detects staleness. If stale but usable, it warns and continues with fallback. If missing, it drops down to FTS/structured-only.
- **Doctor Check**: Detects if the `.sqlite3` file is tracked by `git ls-files` and strongly recommends `git rm --cached`.

## 8.9 Acceptance Criteria

1. `init` adds a managed `.gitignore` block for derived index artifacts without ignoring authoritative configuration.
2. `index status` reports `missing`, `stale`, `ready`, `incompatible`, or `degraded`.
3. `index refresh` strictly updates only changed sources.
4. `index rebuild` completely recreates the DB from workspace evidence.
5. `index doctor` detects derived DB files incorrectly tracked by Git.
6. Optional git hooks mark the index stale after checkout, merge, or rewrite.
7. Hooks never run a full rebuild by default.
8. The assistant skill utilizes CLI JSON output and does not reimplement indexing logic.
9. `plan`, `status`, and `inspect` surfaces gracefully report stale/degraded index state.
10. Semantic execution proves adherence to expansion limits and authoritative FTS invariants.