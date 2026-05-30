# Research: Real sqlite-vec Activation And DB Merge Strategy

## Provider Catalog Refresh

Public provider docs were rechecked on 2026-05-30 as required by the
constitution. OpenAI and Gemini require no bundled catalog change for this
slice. Anthropic's current public comparison set now leads with Claude Opus 4.8,
Claude Sonnet 4.6, and Claude Haiku 4.5, so the bundled catalog was refreshed
to add Opus 4.8 while retaining the older Opus entries as explicit
compatibility choices for hosts that still surface them. The retrieval slice
scope remains unchanged because this catalog update is limited to the bundled
assistant asset and does not alter runtime lifecycle behavior.

## Decision 1: Keep one derived retrieval database and add a real vector table plus companion manifest

**Decision**: Continue using the existing workspace-local retrieval database at
`.boundline/context-intelligence/retrieval-index.sqlite3`, add a dedicated
`vec0` virtual table for semantic vectors inside that database, and add a
companion `.boundline/context-intelligence/manifest.json` file for fast stale
and compatibility checks.

**Rationale**: The current runtime already keeps lexical and semantic metadata
in one derived store, and the roadmap explicitly calls for the same DB file to
remain the local semantic substrate. A second DB or service would create new
merge and ownership problems without improving the smallest valuable slice. The
manifest provides the missing fast path for stale, compatibility, and Git-hygiene
checks without forcing every command to open and inspect the SQLite file.

**Alternatives considered**:

- A second SQLite database for vectors only: rejected because it duplicates
  lifecycle and merge concerns.
- Keeping everything in SQLite without a manifest sidecar: rejected because
  `index status` and hook-triggered stale checks need a cheap non-heavy probe.
- An external vector store: rejected because the slice must remain local-only,
  offline-capable, and disposable.

## Decision 2: Use a trusted vector-capability boundary with explicit load states

**Decision**: Introduce a dedicated vector-capability boundary that owns
trusted extension loading, capability detection, and surfaced runtime states
such as `ready`, `missing`, `unsupported`, `degraded`, and `corrupt`.

**Rationale**: The current code only inspects `PRAGMA module_list`, which is
not enough to distinguish "extension exists", "extension loaded safely", and
"extension failed after request". `rusqlite` supports guarded dynamic extension
loading, but its safety model requires a narrow trusted boundary and no
untrusted SQL while extension loading is enabled. `sqlite-vec` also remains
pre-v1, so the design must assume host variation and explicit degradation.

**Alternatives considered**:

- Requiring `sqlite-vec` everywhere as a hard prerequisite: rejected because
  the existing advanced-context baseline must stay correct without it.
- Continuing with `PRAGMA module_list` checks only: rejected because it hides
  load failures and leaves too much ambiguity in operator output.
- Silently downgrading to baseline behavior: rejected because fallback and
  capability state must remain inspectable.

## Decision 3: Preserve a temporary dual-write rollback window during the vector cutover

**Decision**: Keep writing semantic metadata and JSON embedding payloads to the
existing `semantic_chunks` table while adding the real vector table, and treat
the JSON payload as a temporary rollback path until the vector read path proves
stable through one release cycle and focused corruption and stale-state tests.

**Rationale**: The current runtime already persists semantic metadata and JSON
payloads in `semantic_chunks`. Retaining that path temporarily reduces cutover
risk, simplifies validation of output parity, and supports operator recovery
when the vector table is missing, stale, or incompatible. Removing the JSON
column immediately would turn the first real vector rollout into an all-or-
nothing migration.

**Alternatives considered**:

- Immediate cutover to vectors only: rejected because it removes the simplest
  rollback path before stability is established.
- Keeping JSON as the long-term primary path: rejected because it leaves Rust-
  side full-scan similarity as the effective engine.
- Storing vectors only in the manifest: rejected because the manifest is for
  lifecycle metadata, not queryable nearest-neighbor state.

## Decision 4: Replace routine full rebuilds with chunk-level incremental refresh and explicit rebuild triggers

**Decision**: Move routine maintenance to chunk-level incremental refresh with
stable chunk identifiers, content hashes, and delete detection, while treating
schema-line changes, chunking changes, embedding-dimension changes, and DB
corruption as explicit rebuild triggers.

**Rationale**: The current `refresh_documents()` and `refresh_semantic_chunks()`
paths delete and repopulate entire tables. That keeps the scaffold simple, but
it is the main source of needless churn and merge pain once the index becomes a
real operator-visible cache. Stable chunk ids and manifest fingerprints provide
the smallest path to predictable refresh, while explicit rebuild boundaries are
safer than migration-heavy repair for disposable derived state.

**Alternatives considered**:

- Keep full delete-and-rebuild refresh as the normal path: rejected because it
  penalizes routine edits and branch changes.
- Introduce a general DB migration framework: rejected because the database is
  explicitly disposable and should not gain durable-operator semantics.
- Refresh only at file granularity with one chunk per source: rejected because
  it does not solve large-file granularity or stable source-level collapse.

## Decision 5: Make `boundline index *` the source-of-truth lifecycle interface

**Decision**: Add a dedicated `boundline index` command family with `status`,
`refresh`, `rebuild`, `clean`, and `doctor` subcommands, and keep any skill or
assistant wrapper as a thin consumer of the CLI JSON output rather than a
second implementation surface.

**Rationale**: The roadmap explicitly separates CLI ownership from skill UX.
That matches the repo's current command patterns for `doctor`, `inspect`, and
`init`, and it keeps lifecycle behavior deterministic, testable, and available
outside any assistant surface. It also gives optional Git hooks and future
wrappers one stable JSON contract to consume.

**Alternatives considered**:

- Hide maintenance behind existing `plan` or `status` side effects: rejected
  because it obscures cost, state transitions, and recovery choices.
- Put lifecycle behavior in a skill only: rejected because the CLI must remain
  the cross-host source of truth.
- Extend `doctor` without a dedicated `index` family: rejected because status,
  refresh, rebuild, and clean are operational commands, not just diagnostics.

## Decision 6: Manage Git hygiene through a bounded ignore block and opt-in lightweight hooks

**Decision**: Extend `boundline init` hygiene management with a dedicated block
for derived index artifacts and support optional Git hooks that mark the index
stale or request a lightweight refresh, but never run a heavy rebuild by
default.

**Rationale**: Existing workspace hygiene logic already supports managed
pattern packs without overriding user-authored ignore rules. The derived index
is disposable and should never become a mergeable artifact, but the rest of the
`.boundline/` directory remains partly authoritative. Lightweight hooks preserve
freshness signals without turning normal source-control operations into long-
running maintenance commands.

**Alternatives considered**:

- Ignore the entire `.boundline/` directory: rejected because config and other
  authored artifacts are intentionally versionable.
- Install hooks automatically and run full rebuilds: rejected because hooks are
  invasive and heavy rebuilds would block normal Git operations.
- Rely on manual hygiene only: rejected because accidental tracking of DB and
  WAL files is a predictable failure mode.

### Branch switch and merge policy clarification

The derived retrieval index remains local, disposable, rebuildable, and never a
Git-merge artifact. Checkout, merge, pull-with-merge, rebase, and
post-rewrite are freshness events, not database merge events.

Optional Git hooks may mark the index stale or request a bounded lightweight
refresh when the operator explicitly enables that policy. Fetch requires no
action because it does not change the working tree. Commit hooks stay off by
default unless the operator explicitly enables lightweight refresh behavior.

Schema-line changes, chunker changes, embedding-shape changes, and comparable
compatibility breaks mark the derived index incompatible and require explicit
`boundline index rebuild`. Full rebuild must never run automatically inside Git
hooks. If the derived SQLite DB, WAL, SHM, or manifest sidecar files are
accidentally tracked by Git, `boundline index doctor` must surface the problem
and suggest safe untracking rather than treating those files as merge inputs.

## Decision 7: Extend the existing status, inspect, and trace surfaces instead of inventing a new vector-only report

**Decision**: Extend the current advanced-context projection with explicit
vector-engine and lifecycle fields such as `semantic_engine`,
`vector_query_count`, `vector_candidates_returned`, and
`semantic_fallback_reason`, while keeping `plan`, `status`, `next`, `inspect`,
and traces as the default observability surfaces.

**Rationale**: The repo already exposes `semantic_policy_state`,
`semantic_capability_state`, and `hybrid_outcome` on standard CLI surfaces.
Operators should not need to open a new report just to answer why semantic
retrieval did or did not run, whether the result used real vectors, or why the
runtime degraded back to the baseline path. Extending the existing projection
keeps this slice additive and inspectable.

**Alternatives considered**:

- A separate vector-debug report: rejected because it fragments routine
  observability.
- Trace-only detail with no standard CLI fields: rejected because the recovery
  path must be clear in normal operator use.
- Chunk-level default output: rejected because the user-facing surface should
  remain source-oriented except when diagnostics explicitly request more detail.