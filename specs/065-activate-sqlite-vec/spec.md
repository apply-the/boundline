# Feature Specification: Real sqlite-vec Activation And DB Merge Strategy

**Feature Branch**: `065-activate-sqlite-vec`  
**Created**: 2026-05-30  
**Status**: Draft  
**Input**: User description: "Create a feature spec from feat-sqlite-vec-activation-and-db-merge-strategy.md"

## Clarifications

### Session 2026-05-30

- Q: What explicit policy governs branch switch, merge, and rebase handling for the derived retrieval index? → A: The derived retrieval index is local, disposable, rebuildable, never a Git-merge artifact, and Git freshness events only mark stale or request an explicitly configured bounded lightweight refresh; full rebuilds remain explicit operator actions.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Surface Semantic Evidence Reliably (Priority: P1)

As a Boundline operator using the session-native workflow, I want local semantic
retrieval to use the packaged `sqlite-vec` capability against the existing
derived retrieval index so Boundline can surface relevant evidence beyond
keywords without weakening the authoritative baseline.

**Why this priority**: This is the primary delivery value of the feature. If
semantic retrieval does not improve evidence discovery while preserving trust in
the baseline path, the slice does not justify its added complexity.

**Independent Test**: In a workspace where the best supporting evidence has
weak lexical overlap with the task, run the same bounded workflow once with
vector capability ready and once with the capability unavailable, then verify
that the ready path surfaces additional relevant evidence while the unavailable
path falls back explicitly without losing the baseline result.

**Acceptance Scenarios**:

1. **Given** vector capability is ready and the derived retrieval index is
   current, **When** Boundline assembles advanced context for a bounded task
   with weak keyword overlap, **Then** it surfaces semantically related local
   evidence while preserving the established authority order of the baseline
   retrieval path.
2. **Given** vector capability is missing, unsupported, or fails to initialize,
   **When** the same task runs, **Then** Boundline completes on the baseline
   retrieval path and reports the degraded reason in normal operator output.
3. **Given** semantic expansion finds a candidate outside the active authority
   boundary, **When** the result is projected to the operator, **Then** that
   candidate remains rejected and its rejection reason stays inspectable.

---

### User Story 2 - Maintain The Derived Index Safely (Priority: P1)

As a maintainer switching branches or editing a small subset of files, I want
the derived retrieval index to refresh incrementally and rebuild only when
compatibility is broken so normal work stays fast and Git never treats the
index as mergeable source-of-truth data.

**Why this priority**: Incremental maintenance and safe rebuild boundaries are
the only sustainable way to operate a workspace-local derived index across day-
to-day development workflows.

**Independent Test**: Change a small subset of indexed sources, refresh the
index, and verify that only changed or removed evidence units are updated; then
introduce an incompatible compatibility change and verify that Boundline marks
the index for rebuild instead of attempting an in-place merge.

**Acceptance Scenarios**:

1. **Given** only a subset of indexed sources changed, **When** the operator
   refreshes the index, **Then** Boundline updates only changed or removed
   evidence units and keeps unchanged units available.
2. **Given** index compatibility has been broken by a schema, chunking, or
   embedding-shape change, **When** the operator checks or refreshes the index,
   **Then** Boundline marks the index incompatible and directs the operator to
   rebuild instead of attempting fragile in-place repair.
3. **Given** a branch checkout or merge invalidates the current index, **When**
   the next operator check occurs, **Then** the index is marked stale and the
   operator receives a safe next action without the source-control command
   being blocked by a heavy rebuild.

---

### User Story 3 - Diagnose Stale, Corrupt, Or Tracked Index State (Priority: P2)

As a maintainer or reviewer, I want clear status and diagnostic outputs for
index health, Git hygiene, and fallback routing so I can recover from
corruption, stale state, or accidentally tracked artifacts without guessing.

**Why this priority**: This feature introduces more local state. Without clear
operator diagnostics, the maintenance burden would outweigh the retrieval gain.

**Independent Test**: Exercise stale, corrupt, and accidentally tracked index
states, then verify that normal operator outputs identify the state, explain
the fallback behavior, and point to the correct recovery action.

**Acceptance Scenarios**:

1. **Given** the derived index is corrupt, partially populated, or missing
   semantic data, **When** the operator inspects index health, **Then**
   Boundline reports the state, fallback behavior, and recommended recovery
   action.
2. **Given** derived index artifacts are accidentally tracked by Git or ignore
   rules are missing, **When** the operator runs initialization or diagnostics,
   **Then** Boundline flags the issue and shows the safe remediation without
   hiding authoritative workspace artifacts.
3. **Given** semantic matching expands or reranks candidates, **When** a
   reviewer uses status or inspect surfaces, **Then** the output shows whether
   retrieval expanded or reranked the baseline set and how rejected candidates
   were handled.

### Edge Cases

- Local vector capability is available on one machine but absent on another in
  the same repository; Boundline must degrade explicitly without invalidating
  baseline retrieval.
- The derived index metadata says semantic data is ready but vector-backed
  content is empty or missing; Boundline must report a degraded or stale state
  instead of returning misleading semantic results.
- The embedding shape or chunking policy changes between refreshes; Boundline
  must stop incremental updates and require a rebuild.
- The derived database or its manifest is corrupt or missing; Boundline must
  preserve baseline retrieval and guide the operator to clean or rebuild.
- Git hooks or branch changes mark the index stale during normal source-control
  operations; Boundline must not block those operations with an automatic heavy
  rebuild.
- Evidence is indexed in multiple chunks for one source; source-level
  provenance and rejection details must remain understandable after results are
  collapsed back to the source artifact.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST keep advanced-context retrieval correct when local
  vector capability is unavailable by preserving the existing authoritative
  baseline as the default completion path.
- **FR-002**: Boundline MUST treat vector-backed semantic retrieval as a
  workspace-local, optional augmentation that operators can disable or recover
  from without losing core delivery behavior.
- **FR-003**: Boundline MUST use the existing derived retrieval index as the
  single local store for baseline and semantic retrieval so the semantic path
  does not introduce a separate mergeable source of truth.
- **FR-004**: Boundline MUST surface a distinct capability or health state for
  ready, missing, unsupported, stale, incompatible, degraded, and corrupt index
  conditions.
- **FR-005**: Boundline MUST perform semantic nearest-neighbor selection inside
  the local retrieval store when vector capability is ready.
- **FR-006**: Boundline MUST allow semantic retrieval only to expand or rerank
  candidates within the current authority boundaries and configured expansion
  limits.
- **FR-007**: Boundline MUST preserve authoritative baseline candidates even
  when semantic reranking is active.
- **FR-008**: Boundline MUST preserve source provenance, Canon labels, and
  rejection details when semantic matches are derived from chunked evidence and
  presented at source level.
- **FR-009**: Boundline MUST split large sources into stable, repeatable
  evidence units so incremental refresh can detect changed, unchanged, and
  deleted content reliably.
- **FR-010**: Boundline MUST refresh the derived index incrementally during
  normal maintenance by updating only changed evidence units and deleting units
  whose sources no longer exist.
- **FR-011**: Boundline MUST keep enough manifest metadata to determine refresh
  reason, workspace freshness, and whether incremental refresh remains safe.
- **FR-012**: Boundline MUST require a full rebuild, rather than an in-place
  merge, when index compatibility changes or corruption makes incremental
  refresh unsafe.
- **FR-013**: Boundline MUST provide explicit operator actions to inspect
  status, refresh incrementally, rebuild fully, remove the derived cache, and
  run diagnostics on index health and Git hygiene.
- **FR-014**: Boundline MUST protect repositories from derived index artifacts
  by managing ignore rules for disposable database files and by warning when
  those artifacts, including the derived SQLite DB, manifest sidecars, and
  SQLite WAL or SHM files, are accidentally tracked and by recommending safe
  untracking.
- **FR-015**: Boundline MUST treat branch checkout, merge, pull-with-merge,
  rebase, and post-rewrite events as freshness events that mark the index
  stale or request an explicitly configured bounded lightweight refresh,
  without automatically launching a heavy rebuild inside the source-control
  operation.
- **FR-016**: Boundline MUST surface minimal routing metrics and fallback
  reasons in normal status and inspect outputs so operators can tell which
  retrieval path ran and why.
- **FR-017**: Boundline MUST deterministically handle missing capability,
  vector-store initialization failure, empty semantic content, metadata and
  vector divergence, dimension mismatch, and database corruption without silent
  success.

### Scope Boundaries *(mandatory)*

- **In Scope**: activation of local vector-backed semantic retrieval within the
  existing derived index; incremental refresh and rebuild policy; operator
  status, refresh, rebuild, clean, and diagnostic actions; stale-state marking
  after branch changes; Git ignore hygiene; and source-level explainability for
  expanded or reranked evidence.
- **Out of Scope**: remote embeddings; external vector stores; graph databases;
  durable operator-owned data in the derived index; automatic full rebuilds
  during Git hooks; chunk-level user-facing output outside diagnostics; and any
  change to the V1 authority order.

## Branch Switch, Merge, And Rebase Strategy

The derived retrieval index is local, disposable, and rebuildable. It is never
a Git-merge artifact.

Boundline treats branch checkout, merge, pull-with-merge, rebase, and
post-rewrite as freshness events for the derived index, not as database merge
events.

Expected behavior:

| Git event | Boundline behavior |
|---|---|
| branch checkout | mark index stale or run a bounded lightweight refresh only when explicitly configured |
| merge or pull with merge | mark index stale or run a bounded lightweight refresh only when explicitly configured |
| rebase or post-rewrite | mark index stale |
| fetch | no action, because the working tree does not change |
| commit | no action by default; an optional lightweight refresh may be configured |
| schema, chunker, or embedding-shape change | mark incompatible and require explicit rebuild |

Full rebuilds must never run automatically inside Git hooks.

The operator-visible flow is:

```text
git checkout / merge / rebase
-> optional hook marks index stale
-> next Boundline command reports stale reason
-> operator runs `boundline index refresh`
-> Boundline requires `boundline index rebuild` only when compatibility is broken
```

If the derived SQLite database or sidecar files are accidentally tracked by
Git, `boundline index doctor` must report the problem and suggest safe
untracking.

### Key Entities *(include if feature involves data)*

- **Derived Retrieval Index**: A disposable local cache that combines baseline
  and semantic retrieval data for a workspace, can be rebuilt from workspace
  evidence, and is never treated as authoritative.
- **Semantic Capability State**: The operator-visible health state describing
  whether vector-backed semantic retrieval is ready, missing, unsupported,
  stale, degraded, incompatible, or corrupt.
- **Evidence Chunk Record**: A stable indexed unit derived from a source
  artifact with repeatable identity, content fingerprint, provenance, and a
  clear path back to source-level evidence.
- **Index Manifest**: Companion metadata that describes freshness,
  compatibility, refresh reason, workspace fingerprint, and the recommended
  recovery path for the derived index.
- **Index Maintenance Action**: An operator-initiated action to inspect,
  refresh, rebuild, clean, or diagnose the derived index.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In at least 80% of representative bounded tasks where relevant
  local evidence has weak keyword overlap, operators surface at least one
  additional relevant evidence item without losing any authoritative baseline
  candidate.
- **SC-002**: In 100% of validation cases where only a subset of indexed
  sources changed, routine maintenance updates only the changed or deleted
  evidence units and does not require a full rebuild.
- **SC-003**: In 100% of validation cases where capability, freshness, or
  compatibility is broken, operators receive an explicit state and recommended
  recovery action during normal status or inspect use.
- **SC-004**: Reviewers can determine within 5 minutes whether semantic
  retrieval expanded or reranked the baseline set, and why any fallback or
  rejection occurred, using standard operator outputs alone.
- **SC-005**: Across the representative branch-switch and merge scenarios
  defined for this feature, 0 scenarios require manual merge resolution for
  derived index artifacts.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: OpenAI Models documentation at
  `https://developers.openai.com/api/docs/models`, Anthropic Models overview at
  `https://platform.claude.com/docs/en/docs/about-claude/models`, and Google
  Gemini Models documentation at `https://ai.google.dev/gemini-api/docs/models`,
  reviewed on 2026-05-30.
- **Catalog Delta**: The bundled catalog already covers the current OpenAI
  GPT-5.5 and GPT-5.4 families plus the current Gemini 2.5 and 3.x routing
  families, but Anthropic public docs now position Claude Opus 4.8, Claude
  Sonnet 4.6, and Claude Haiku 4.5 as the current top comparison set. The
  bundled catalog still tops out at Claude Opus 4.7, so a follow-up catalog
  refresh should add Opus 4.8 and review whether older Opus entries remain in
  the supported routing set.
- **No-Change Rationale**: Not applicable for Anthropic because a public-model
  delta was identified; OpenAI and Gemini required no catalog changes for this
  spec pass.

## Assumptions

- Workspaces already rely on the existing local retrieval baseline and must
  continue to operate when semantic acceleration is unavailable.
- The derived retrieval index can be rebuilt entirely from workspace evidence
  and current Boundline configuration, so it does not store operator-owned data
  that would make deletion unsafe.
- Operators accept explicit stale, degraded, and incompatible states in normal
  status and inspect outputs instead of silent self-repair.
- Optional Git hooks require explicit operator consent and are limited to
  lightweight stale-state handling rather than heavy rebuild work.
