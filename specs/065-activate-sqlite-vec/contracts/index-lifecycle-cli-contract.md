# Index Lifecycle CLI Contract

## Purpose

Define the minimum command and JSON contract for the derived retrieval-index
lifecycle surface introduced by feature 065. The CLI is the source of truth for
index health, refresh, rebuild, clean, and doctor behavior. Skills or other
wrappers may consume the CLI output, but they must not reimplement lifecycle
logic.

## Command Family

Boundline must expose a dedicated command family:

- `boundline index status`
- `boundline index refresh`
- `boundline index rebuild`
- `boundline index clean`
- `boundline index doctor`

Each command must support the same workspace-selection rules as other existing
Boundline developer commands and must offer structured JSON output suitable for
assistant or hook consumption.

## Common Output Requirements

All `boundline index *` commands must provide:

- a machine-readable JSON mode
- a human-readable terminal summary
- a stable top-level operation name
- a stable top-level workspace reference
- an explicit pre-state and post-state when mutation is possible
- a `recommended_action` field when the command surfaces a recoverable problem
- an explicit non-success reason when the preferred lifecycle path did not
  complete

Minimum shared JSON shape, or equivalent typed envelope:

- `command`
- `workspace_root`
- `operation_id`
- `pre_state`
- `post_state`
- `recommended_action`
- `warnings[]`
- `errors[]`

## `index status`

`boundline index status` must read lightweight lifecycle state without forcing a
refresh or rebuild.

Minimum JSON fields:

- `status`: `ready`, `missing`, `stale`, `incompatible`, `degraded`, `corrupt`,
  or `semantic_unavailable`
- `schema_version`
- `workspace_fingerprint`
- `git_branch`
- `git_head`
- `last_refresh_at`
- `last_refresh_reason`
- `file_count`
- `chunk_count`
- `fts5_state`
- `sqlite_vec_state`
- `semantic_engine`
- `fallback_reason`
- `recommended_action`

Behavior rules:

- `status` must not trigger a heavy rebuild
- `status` may mark the index stale when cheap freshness checks prove reuse is
  unsafe
- `status` must distinguish "missing index" from "semantic unavailable" and
  from "corrupt index"

## `index refresh`

`boundline index refresh` must perform a bounded incremental refresh by default.

Minimum JSON fields:

- `status_before`
- `status_after`
- `sources_scanned`
- `sources_changed`
- `sources_deleted`
- `chunks_upserted`
- `chunks_deleted`
- `vector_rows_written`
- `semantic_engine_after`
- `fallback_reason`
- `recommended_action`

Behavior rules:

- refresh must update only changed or deleted sources when compatibility
  remains valid
- refresh must stop and surface `incompatible` when schema, chunker,
  fingerprint, or dimension changes make incremental reuse unsafe
- refresh must surface explicit degradation when vector writes cannot complete
  but baseline metadata remains usable

## `index rebuild`

`boundline index rebuild` must discard incompatible or corrupt derived state and
recreate the index from workspace evidence.

Minimum JSON fields:

- `status_before`
- `status_after`
- `rebuild_reason`
- `sources_indexed`
- `chunks_indexed`
- `vector_rows_written`
- `warnings[]`

Behavior rules:

- rebuild must recreate both lexical and semantic state from scratch
- rebuild must not preserve stale vector rows from a prior incompatible schema
- rebuild must keep the resulting manifest and DB in a mutually consistent
  state or fail explicitly

## `index clean`

`boundline index clean` must delete only disposable derived artifacts.

Minimum JSON fields:

- `deleted_files[]`
- `status_after`
- `recommended_action`

Behavior rules:

- clean must remove the derived SQLite DB, manifest, and disposable sidecars
- clean must not delete authored config, traces, or other versionable
  workspace artifacts
- after clean, `index status` must report `missing`

## `index doctor`

`boundline index doctor` must report lifecycle and Git-hygiene problems without
hiding actionable remediation.

Minimum JSON fields:

- `status`
- `checks[]`
- `tracked_index_files[]`
- `missing_ignore_rules[]`
- `wal_sidecars_present`
- `manifest_consistency`
- `vector_schema_consistency`
- `recommended_action`

Each `checks[]` entry must include:

- `check_name`
- `result`: `pass`, `warn`, or `fail`
- `detail`
- `suggested_fix`

## Hook And Wrapper Integration Rules

- Optional Git hooks may call `boundline index status` or a bounded refresh
  flow, but they must not reimplement stale detection themselves.
- Assistant skills may call `boundline index status --json` and explain the
  result, but they must not become the primary lifecycle engine.
- Hook-triggered use must remain safe when the index is missing, corrupt, or
  the vector capability differs across machines.

## Explicit Exclusions

This contract does not require:

- automatic full rebuilds during checkout, merge, or rewrite operations
- a new UI surface outside the CLI and standard trace output
- hidden maintenance logic inside prompt templates or assistant skills
- durable operator-owned data inside the derived index