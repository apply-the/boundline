# TD-002: Real sqlite-vec Activation And DB Merge Strategy

## Status

Proposed - Ready For Prioritization

## Type

Tech Debt / Architecture

---

# 1. Background

Boundline already persists advanced-context retrieval state in the workspace-local
SQLite file `.boundline/context-intelligence/retrieval-index.sqlite3`.

The current semantic-acceleration slice delivers:

- a local SQLite document store plus FTS5 baseline
- semantic metadata persisted beside the baseline retrieval index
- optional local semantic policy and operator-visible fallback states
- hybrid explainability in `plan`, `status`, `next`, and `inspect`

However, the runtime does not yet use `sqlite-vec` as the semantic query engine.
Today the semantic path stores embeddings as JSON payloads in `semantic_chunks`
and computes cosine similarity in Rust after reading rows back out of SQLite.

The root workspace already reserves the feature flags `semantic-acceleration` and
`sqlite-vec`, so the roadmap item is not conceptual discovery. It is the runtime
closure needed to make the shipped semantic-acceleration surface use real vector
tables and SQL-side nearest-neighbor selection while preserving the current
bounded hybrid semantics.

Because this database is workspace-local and currently derived from retrievable
workspace evidence, the schema-evolution strategy should minimize merge pain and
avoid introducing fragile in-place migrations where rebuild is cheaper and safer.

Last verified: 2026-05-26 against the Boundline `0.64.0` worktree.

---

# 2. Current State Snapshot

| Area | Current State | Gap To Close |
|---|---|---|
| Semantic storage | `semantic_chunks` stores `chunk_text` and `embedding_payload_json` | No `vec0` virtual table yet |
| Capability detection | Runtime inspects `PRAGMA module_list` for `vec0` and `vec_each` | Detection exists, but no extension-backed query path |
| Query execution | Semantic matches are read row-by-row and scored in Rust | No SQL-side top-k similarity query |
| Refresh behavior | Semantic refresh deletes all semantic rows and rebuilds them | No incremental upsert/delete path |
| Chunking | One semantic chunk per `source_ref`, truncated to 8 KiB | No multi-chunk granularity for large files |
| Hybrid semantics | FTS baseline plus semantic expand/rerank annotations already exist | Must be preserved during engine swap |

The current state is useful and explainable, but it is still a vector-backed
cache rather than a true vector-search runtime.

---

# 3. Delivery Goal

Deliver a real `sqlite-vec`-backed semantic path that:

- keeps the same workspace-local SQLite file and operator-facing projection
- preserves the V1 FTS and structured fallback as the authoritative baseline
- performs semantic nearest-neighbor selection inside SQLite rather than in Rust
- remains optional, local-only, and explicitly degradable
- avoids turning the retrieval index into a merge-heavy system of record

---

# 4. Activation Plan

## 4.1 Capability Boundary And Extension Loading

Create a dedicated vector-capability boundary that separates:

- feature gating (`semantic-acceleration`, `sqlite-vec`)
- extension loading or static binding decisions
- runtime capability reporting: `ready`, `missing`, `unsupported`, `stale`

Implementation notes:

- keep the existing manifest-level capability states because operator output and
  tests already depend on them
- add one explicit loader path rather than scattering `sqlite-vec` handling
  across query code
- record the effective vector capability in the semantic manifest every time the
  index is initialized or refreshed

Acceptance signal:

- `status` and `inspect` still surface the same semantic policy and capability
  state even when the actual engine becomes `sqlite-vec`

## 4.2 Split Metadata From Vector Storage

Keep `semantic_chunks` as the explainable metadata table, but add a dedicated
vector table for the embedding payload.

Target direction:

- `semantic_chunks`: chunk identity, provenance, labels, schema line, chunk text,
  compatibility metadata
- `semantic_chunk_vectors` or equivalent `vec0` virtual table: `chunk_id` plus
  the actual embedding vector in the format expected by `sqlite-vec`

Transition strategy:

- stage 1: create the vector table beside the existing JSON payload column and
  dual-write both formats
- stage 2: read semantic matches from the vector table while still retaining the
  JSON payload as a temporary rollback path
- stage 3: remove `embedding_payload_json` only after the vector path is proven
  stable and downgrade behavior is covered

Why this matters:

- it preserves explainability without forcing the vector table to carry all
  operator-facing metadata
- it keeps future schema changes localized instead of mixing vector storage,
  provenance, and display concerns in one table

## 4.3 Replace Full Rebuild With Incremental Refresh

The current semantic refresh clears the entire semantic table and rebuilds from
scratch. That is acceptable for an early slice but becomes the wrong default once
vector tables are introduced.

Move to an incremental refresh flow:

- derive stable chunk ids from `source_ref` plus chunk ordinal
- compare content hashes before rewriting vectors
- upsert only changed chunks
- delete chunks whose source disappeared or whose chunk count shrank
- keep a manifest-level refresh reason and schema line for diagnostics

Acceptance signal:

- a single changed document should not force a full semantic rebuild of the
  workspace index

## 4.4 Move Semantic Query Execution Into SQLite

Replace the Rust-side full scan in `query_semantic_matches` with a SQL-side top-k
vector query against the `vec0` table.

Required properties:

- the FTS result set remains the primary V1 candidate set
- semantic acceleration may only expand or rerank within the existing authority
  and evidence budgets
- degraded, skipped, fallback, and baseline-only outcomes remain explicit
- Canon compatibility and provenance labels survive the engine change unchanged

Practical direction:

- issue one bounded query embedding per decision point
- retrieve top-k matching chunk ids from the vector table
- join them back to `semantic_chunks`
- collapse chunk-level hits into the existing source-level candidate model before
  the hybrid decision step

Acceptance signal:

- the selected and rejected evidence story exposed to operators does not regress
  even though the similarity engine moved into SQLite

## 4.5 Improve Chunking Before Chasing Recall Higher

The current model effectively stores one semantic chunk per document or source
ref. That limits recall on large files and makes a future vector table less
useful than it should be.

Introduce bounded multi-chunk indexing:

- split large documents or files into stable chunk windows
- preserve `source_ref` plus a stable chunk suffix such as `#chunk-N`
- keep provenance and Canon boundary information per chunk
- continue surfacing evidence at the source level unless a later slice decides
  chunk-level explainability is worth the extra output noise

Recommended order:

- do this before measuring `sqlite-vec` quality, otherwise the engine change will
  be blamed for recall limits that are actually caused by coarse chunking

## 4.6 Validation, Rollout, And Safety Gates

Validation should cover:

- schema init and dual-write behavior
- incremental refresh correctness
- vector-capability detection and downgrade behavior
- hybrid expansion and rerank invariants
- planner, status, next, and inspect output continuity
- compatibility with workspaces where `sqlite-vec` is unavailable

Rollout rule:

- the V1 FTS plus structured fallback path remains correct with semantic
  acceleration disabled or unavailable

---

# 5. Merge Strategies For Different Features That Touch The DB

This section covers three different problems that are often conflated:

- schema evolution across Git branches
- data merge across parallel feature work
- live multi-process access to the same SQLite file

They need different strategies.

## 5.1 Recommended Default For Boundline's Retrieval Index: Derived-Cache Rebuild

For the current retrieval index, the safest merge model is to treat the database
as derived state rather than as an authoritative store.

Recommendation:

- keep repo-visible truth in workspace files, traces, config, and session state
- store only rebuildable retrieval/index state in `retrieval-index.sqlite3`
- when an incompatible schema change lands, bump the semantic schema line or
  manifest version and rebuild affected tables instead of writing fragile branch-
  merge migrations

Why this is the best default here:

- SQLite supports only a limited native `ALTER TABLE` subset, and complex schema
  changes require the generalized table-copy procedure
- the retrieval index is a cache over local evidence, not a durable business
  system of record
- rebuild is cheaper than long-lived schema compatibility for this class of data

Use this for:

- FTS tables
- semantic chunk tables
- vector tables
- retrieval projections that can always be regenerated locally

Do not use this for:

- future durable user-authored state that cannot be reconstructed from repo or
  session artifacts

## 5.2 Append-Only Versioned Migrations For Durable Tables

If Boundline later stores durable operator-owned data in the same SQLite file,
use append-only versioned migrations.

Literature and tooling guidance agree on the core rules:

- all DB changes are migrations kept in version control with the application
- migrations must be small and integrated frequently
- each migration has a unique version identifier
- once applied to a downstream environment, a migration should not be edited;
  fixes roll forward in a new migration

Practical policy:

- prefer timestamp or timestamp-plus-suffix versions to reduce branch-number
  clashes
- maintain a migration history table with applied version and checksum
- require CI to build a fresh database from zero and apply all migrations on each
  merge to mainline

Good fit:

- durable state tables
- settings that outlive rebuilds
- local operator annotations or review artifacts if they become DB-backed

Poor fit:

- the current retrieval cache, where rebuild is simpler than migration debt

## 5.3 Expand-Contract For Destructive Schema Changes

For destructive or shape-changing work, use a transition phase instead of a
single-shot cutover.

Recommended sequence:

1. Add new columns or tables in a backwards-compatible form.
2. Backfill or dual-write.
3. Switch reads and higher-level logic.
4. Remove the old structure in a later follow-up.

This is especially important in SQLite because:

- destructive schema changes are trickier than in engines with richer online DDL
- generalized schema alteration often requires the documented copy-into-new-table
  procedure
- branch merges are easier when two features can coexist temporarily

Good fit:

- changing chunk identity shape
- moving from one-chunk-per-source to multi-chunk indexing
- replacing JSON embedding payloads with vector-table rows
- splitting one metadata table into metadata plus vector storage

## 5.4 SQLite Session Changesets For Data-Level Merge

If Boundline eventually needs to merge data edits from parallel local workspaces
or feature copies of the same database, SQLite's session extension is the most
direct built-in merge primitive.

What it provides:

- capture a changeset or patchset from one database handle
- apply it to another compatible database
- detect row-level conflicts during apply
- concatenate or invert changesets when needed

Important limitations:

- it requires declared primary keys
- it does not capture changes to virtual tables
- it is for data merge, not schema migration
- both sides need compatible starting schema and compatible baseline data

That makes it a plausible future fit for:

- durable operator annotations
- local findings or votes
- manually curated data that should merge like patches

It is not a good fit for:

- FTS virtual tables
- `vec0` virtual tables
- schema-evolution conflicts between branches

## 5.5 WAL Mode For Runtime Concurrency, Not For Git Merge

WAL is worth considering only as a live-access optimization.

Benefits:

- readers do not block writers
- writers do not block readers
- read and write workloads can overlap on the same host

Limits that matter here:

- SQLite still allows only one writer at a time
- WAL is same-host only in the normal shared-memory setup
- long-running readers can starve checkpoints and let the WAL grow
- WAL does not solve source-control merge or schema-compatibility problems

Operational rule if WAL is enabled for this index:

- keep readers short-lived
- define an explicit checkpoint strategy
- verify the bundled SQLite version includes the March 2026 WAL-reset fix

---

# 6. Recommended Combination For Boundline

For the current Boundline retrieval index, the best combination is:

- **Primary strategy**: derived-cache rebuild on incompatible semantic schema
  changes
- **Schema-change pattern**: expand-contract for destructive layout changes that
  need a transition release
- **Durable-state fallback**: append-only versioned migrations only if a future
  feature adds non-rebuildable tables to this database
- **Data-merge option**: SQLite session changesets only for future durable row-
  level user data, not for FTS or vector tables
- **Runtime access option**: WAL only if multi-process concurrency becomes a real
  bottleneck, and only with checkpoint discipline

In short:

- do not turn the retrieval index into a migration-heavy mini product unless the
  data becomes authoritative
- do not use the session extension for schema work
- do not confuse WAL with a branch-merge strategy

---

# 7. Sources

- Martin Fowler and Pramod Sadalage, *Evolutionary Database Design*:
  https://martinfowler.com/articles/evodb.html
- SQLite documentation, *Write-Ahead Logging*:
  https://www.sqlite.org/wal.html
- SQLite documentation, *The Session Extension*:
  https://www.sqlite.org/sessionintro.html
- SQLite documentation, *ALTER TABLE*:
  https://www.sqlite.org/lang_altertable.html
- Redgate Flyway documentation, *Versioned migrations*:
  https://documentation.red-gate.com/fd/versioned-migrations-273973333.html

---

# 8. Immediate Next Step

If this item is prioritized, the first implementation slice should be:

- add the vector-capability boundary and the `vec0` table beside the current
  semantic metadata table
- dual-write vectors during refresh
- keep the existing Rust-side semantic query path temporarily as the rollback
  fallback

That creates the smallest safe bridge from the current shipped behavior to a
real `sqlite-vec` runtime.