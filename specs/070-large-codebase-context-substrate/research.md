# Research: Large Codebase Context Substrate

## Provider Catalog Refresh

Public provider documentation was rechecked on 2026-06-05 as required by the
constitution:

- OpenAI's current models documentation still surfaces the GPT-5.5 and GPT-5.4
  family used by Boundline routing guidance, including `gpt-5.5`, `gpt-5.4`,
  and `gpt-5.4-mini` on the current API models page:
  [OpenAI models](https://platform.openai.com/docs/models).
- Anthropic's current Claude models overview still covers the modern Claude 4.x
  line relevant to Boundline assistant compatibility guidance, including the
  currently documented Opus, Sonnet, and Haiku families:
  [Claude models overview](https://docs.anthropic.com/en/docs/intro).
- Google's current Gemini models page still documents the Gemini 2.5 and 3.x
  families already represented in the bundled catalog, including
  `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite`, and the Gemini
  3.x lines:
  [Gemini models](https://ai.google.dev/gemini-api/docs/models).

Result: no feature-driven catalog change is required for this plan packet.
Boundline still ships the same provider families, assistant defaults, and route
surfaces after the 2026-06-05 check, so the bundled
`assistant/catalog/model-catalog.toml` only needs a release-line refresh to
`0.71.0`, not a catalog-shape change. Any future pruning of legacy vendor
aliases should be handled as dedicated catalog maintenance, not folded into
this substrate slice.

## Decision 1: Extend the existing local context-intelligence stack

**Decision**: Build the large-codebase substrate by extending existing
`context_intelligence`, `goal_planner`, `goal_plan`, and `project_index`
surfaces instead of creating a second indexing or retrieval engine.

**Rationale**: Boundline already owns local derived indexing, context-pack
assembly, and session-visible context projection. Reusing those surfaces keeps
state, traces, and CLI output aligned and avoids introducing another
authoritative context path.

**Alternatives considered**:

- Introduce a new standalone "large codebase runtime" module: rejected because
  it would duplicate existing local retrieval and context-pack ownership.
- Push the whole feature into external providers: rejected because the roadmap
  item is explicitly about a local substrate and derived local cache boundary.

## Decision 2: Make fidelity tiers and inclusion modes typed runtime state

**Decision**: Treat fidelity tier, inclusion mode, authority, omission reason,
and compaction state as typed Boundline-owned runtime data attached to context
candidates and persisted context-pack entries.

**Rationale**: The feature is only trustworthy if operators can inspect what
was loaded, what was compacted, and what was omitted. Typed state keeps the
selection path reproducible across `status`, `inspect`, traces, and assistant
surfaces.

**Alternatives considered**:

- Keep these concepts as prose-only trace lines: rejected because operators and
  tests would not have a stable contract to validate.
- Derive tiers dynamically at render time only: rejected because planning and
  blocking rules need a stable domain representation before rendering.

## Decision 3: Require search-before-read for oversized artifacts

**Decision**: Large-file or large-artifact handling must begin with search and
navigation signals, then only escalate to excerpt, digest, or bounded full-read
paths when explicit conditions justify it.

**Rationale**: Unsafe full reads are exactly the failure class this slice is
meant to prevent. Path matches, symbol matches, import/export relations, test
links, trace links, and Canon artifact relations are bounded local signals that
can narrow context without loading the whole artifact.

**Alternatives considered**:

- Always read the whole file and summarize afterward: rejected because the
  safety failure has already happened by the time summarization begins.
- Use keyword-only narrowing with no structural signals: rejected because it is
  too weak for large repositories with repeated or ambiguous names.

## Decision 4: Keep the repository map compact and derived

**Decision**: The repository navigation map should remain a compact derived
projection of files, symbols, relations, owner hints, and nearby tests rather
than a heavyweight authoritative semantic database.

**Rationale**: The slice needs enough structure to support search-before-read,
ranking, and omission explanation, but not a generalized code-knowledge
platform. A compact map supports the roadmap item without locking the project
into a much larger subsystem.

**Alternatives considered**:

- Build a complete multi-language semantic graph: rejected because it would
  exceed the minimal valuable slice.
- Avoid any repository map and rely on raw path search alone: rejected because
  it would not satisfy symbol-aware or relation-aware context selection.

## Decision 5: Use hybrid ranking, but keep it local and explainable

**Decision**: Rank context candidates using multiple local signals, including
lexical match, symbol match, dependency or relation distance, test relation,
changed-file proximity, Canon authority, guidance relevance, and fidelity tier,
with optional semantic acceleration only as an additive local input.

**Rationale**: Large-codebase selection quality depends on more than one score,
but the ranking still needs to be inspectable. Boundline already has a local
semantic-acceleration path; this slice should consume it only when available
and still explain the final selection without opaque weighting.

**Alternatives considered**:

- Single numeric rank from one subsystem: rejected because it hides why a
  candidate won and performs poorly on mixed artifact sets.
- Remote embedding or hosted search services: rejected because this slice is
  explicitly local and deterministic.

## Decision 6: Compact huge artifacts into digest-backed references

**Decision**: Very large logs, diffs, generated output, and similar artifacts
should be compacted into digest-backed references plus bounded summaries or
excerpts unless the active decision explicitly requires full content.

**Rationale**: Huge artifacts are often relevant, but rarely need to dominate
the active context window. Digest-backed references preserve provenance and
recoverability without forcing unsafe full inclusion.

**Alternatives considered**:

- Drop large artifacts entirely: rejected because the operator then loses
  evidence that may still matter for debugging or verification.
- Keep entire artifacts in normal planning context: rejected because it breaks
  bounded context-pack discipline.

## Decision 7: Keep patch-safe editing explicit for large files

**Decision**: Large-file editing should require anchored hunks, drift checks,
bounded scope, and post-apply verification before a change is treated as
accepted.

**Rationale**: The same repositories that trigger oversized-read problems also
make full-file rewrite strategies unsafe. Patch-safe editing belongs in the
substrate contract because context selection and edit safety are coupled.

**Alternatives considered**:

- Treat editing as out of scope for this slice: rejected because the seed
  feature explicitly calls for patch-safe behavior on huge files.
- Allow full-file rewrites once a file is "important enough": rejected because
  importance is not a safe substitute for bounded edit scope.

## Decision 8: Snapshot cache is derived state, not memory

**Decision**: Any persistent local snapshot cache must remain explicitly
derived, local, disposable, rebuildable, freshness-bound, and
non-authoritative.

**Rationale**: This feature must not silently turn into a memory system.
Reviewed memory belongs to later roadmap work; the substrate cache is only a
reusable acceleration and inspection aid.

**Alternatives considered**:

- Let the cache become the new planning truth when available: rejected because
  it collapses the line between derived state and reviewed memory.
- Avoid persistent cache state entirely: rejected because freshness-aware local
  reuse is part of the roadmap item's value.

## Decision 9: Freshness events must invalidate before reuse

**Decision**: Branch switches, merges, rebases, config changes, schema changes,
adapter changes, and Canon packet changes must mark snapshot-cache state stale
before it can influence a new planning context.

**Rationale**: A stale cache is worse than no cache if it silently replays old
repository shape into a new planning decision. Freshness is a first-class
correctness rule, not a later optimization.

**Alternatives considered**:

- Keep stale snapshots until an operator manually notices: rejected because the
  feature explicitly promises invalidation-before-reuse.
- Rebuild everything automatically in Git hooks: rejected because the seed
  explicitly says full rebuilds must not run automatically in hooks.
