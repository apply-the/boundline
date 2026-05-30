# Quickstart: Real sqlite-vec Activation And DB Merge Strategy

## 1. Prepare A Temporary Workspace

Use a disposable workspace that already has the advanced-context baseline
available. Do not run this walkthrough against the Boundline repository root,
because this feature writes workspace-local derived state under `.boundline/`.

Expected result:

- the workspace can already run the baseline advanced-context flow
- the derived retrieval index is treated as disposable local state
- Git is available if you want to verify ignore rules and stale detection

## 2. Initialize Hygiene And Confirm Derived Files Stay Disposable

Run:

```bash
boundline init --workspace <workspace>
```

If you also want lightweight Git freshness hooks that mark the derived index
stale on checkout, merge, and rewrite events, rerun with:

```bash
boundline init --workspace <workspace> --semantic-index-hook-action mark-stale
```

Expected result:

- `init` creates or updates a managed `.gitignore` block for
  `.boundline/context-intelligence/*.sqlite3`, WAL, SHM, journal, lock, and
  temporary files
- authored `.boundline/` config remains versionable
- optional hook installation stays explicit through
  `--semantic-index-hook-action mark-stale`; the default init path still leaves
  hooks disabled

## 3. Inspect The Current Index Lifecycle State

Run:

```bash
boundline config set-semantic-acceleration --scope workspace --policy local
boundline index status --workspace <workspace>
boundline index status --workspace <workspace> --json
```

Expected result:

- the command reports `ready`, `missing`, `stale`, `incompatible`, `degraded`,
  `corrupt`, or `semantic_unavailable`
- the JSON output includes manifest details such as schema version, workspace
  fingerprint, semantic engine, vector capability, and recommended action
- no heavy refresh or rebuild is triggered just to answer status
- semantic acceleration remains opt-in until the workspace config explicitly
  sets `policy = "local"`

## 4. Refresh The Derived Index Incrementally

Run:

```bash
boundline index refresh --workspace <workspace>
boundline index status --workspace <workspace>
```

Expected result:

- only changed or new sources are rescanned and upserted
- disappeared sources delete their chunk and vector rows
- the manifest updates `last_refresh_at`, `last_refresh_reason`, `file_count`,
  and `chunk_count`
- if vector capability is unavailable, the runtime records an explicit fallback
  engine rather than pretending `sqlite-vec` succeeded

Observed during the 2026-05-30 temp-workspace walkthrough: refresh completed
successfully with `post_state = semantic_unavailable`,
`semantic_engine = baseline_json`, and `sqlite_vec_state = missing` on a
machine where `sqlite-vec` was not available, and `boundline index doctor`
still reported the manifest and ignore hygiene as passed.

## 5. Verify Real Vector Retrieval On Standard Runtime Surfaces

Run:

```bash
boundline plan --workspace <workspace>
boundline status --workspace <workspace>
boundline inspect --workspace <workspace>
```

Expected result:

- the standard runtime surfaces show `semantic_engine`,
  `semantic_capability_state`, `vector_query_count`,
  `vector_candidates_returned`, any `semantic_fallback_reason`, and
  `retrieval_recovery_guidance` when the local index or vector capability needs
  repair
- selected and rejected evidence remains source-oriented and explainable even
  when retrieval operated on chunks
- authoritative baseline candidates remain visible and are not silently removed

## 6. Change One Indexed Source And Confirm Incremental Refresh

Edit one eligible source in `<workspace>`, then run:

```bash
boundline index refresh --workspace <workspace>
boundline index status --workspace <workspace> --json
```

Expected result:

- the refresh updates only the affected source and its derived chunks
- unchanged sources remain intact
- JSON output reports non-zero changed counts without forcing a full rebuild

## 7. Force An Incompatible State And Rebuild Explicitly

Simulate a schema-line, chunking, or embedding-dimension incompatibility, then
run:

```bash
boundline index status --workspace <workspace>
boundline index rebuild --workspace <workspace>
boundline index status --workspace <workspace>
```

Expected result:

- status reports the index as incompatible before rebuild
- rebuild recreates lexical and semantic state from scratch
- post-rebuild status returns to `ready` or an explicit degraded state with a
  surfaced reason

## 8. Diagnose Git And Derived-Index Hygiene

Run:

```bash
boundline index doctor --workspace <workspace>
boundline index doctor --workspace <workspace> --json
```

Expected result:

- doctor reports whether derived DB artifacts are tracked by Git
- doctor flags missing ignore rules, stale sidecars, manifest mismatch, vector
  schema mismatch, or corruption
- the JSON output includes actionable remediation steps for each failure or
  warning

## 9. Verify Optional Hook Behavior Stays Lightweight

If optional index hooks were installed, switch branches or perform a merge in
the temporary workspace, then run:

```bash
boundline index status --workspace <workspace>
```

Expected result:

- the hook marks the index stale or requests a lightweight bounded refresh
- the hook entrypoint stays on `boundline index status --workspace <workspace>`
  rather than mutating the manifest directly
- the hook does not launch a heavy rebuild during the source-control operation
- the next operator-visible command shows the stale reason and recommended next
  action

Observed during the 2026-05-30 temp-workspace walkthrough: a branch switch set
`post_state = stale`, `stale_reason = branch_checkout`, and recommended
`boundline index refresh --workspace <workspace>` without changing the manifest
to a fake ready state.