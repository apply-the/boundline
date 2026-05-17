# Research: Advanced Context Intelligence Semantic Acceleration

## Provider Catalog Refresh

Checked the bundled assistant catalog against the public model-family sources
referenced in the feature spec on 2026-05-17. No bundled model additions or
removals were required for this slice, so the catalog remains a no-change
refresh and the semantic-acceleration work continues without a hosted-provider
dependency.

## Decision 1: Model semantic acceleration as a separate additive policy that defaults to disabled

**Rationale**: S5 V1 already ships a local advanced-context baseline whose
retrieval policy resolves through `AdvancedContextConfig`. S5.v2 must not change
that baseline's correctness contract or silently turn semantic similarity on.
The clean boundary is to keep V1 retrieval intact and add a separate semantic-
acceleration policy within the advanced-context surface that resolves to
`disabled` by default and `local` only through explicit workspace or cluster
opt-in. This preserves the roadmap rule that losing semantic capability falls
back to V1 without feature loss in the core delivery loop.

**Alternatives considered**:

- Reusing `retrieval_mode = local` as implicit semantic enablement: rejected
  because the V1 local baseline is already required and should not change
  meaning in-place.
- Defaulting semantic acceleration to local: rejected because the roadmap and
  addendum require semantic behavior to remain optional and local-only by
  explicit opt-in.
- Reintroducing remote semantic mode in the first slice: rejected because the
  spec confines the first S5.v2 implementation to local semantics.

## Decision 2: Extend the existing workspace-local SQLite retrieval index instead of adding a second semantic store

**Rationale**: The roadmap and S5 addendum explicitly require `sqlite-vec` to
layer onto the same workspace-local SQLite store used by V1. Reusing
`.boundline/context-intelligence/retrieval-index.sqlite3` keeps packaging,
refresh, provenance, and inspection aligned with the current runtime and avoids
splitting authority between an FTS database and a separate vector store.
Semantic tables should therefore be additive to the existing retrieval index,
not a parallel data source.

**Alternatives considered**:

- A second SQLite database for vectors only: rejected because it complicates
  refresh, compatibility, and inspectability without improving the minimal
  delivery slice.
- LanceDB as the default vector store: rejected because the addendum keeps it
  optional for larger repositories, not the default S5.v2 path.
- External vector infrastructure such as Qdrant: rejected because the slice
  must remain local-first, offline-capable, and service-free.

## Decision 3: Treat `sqlite-vec` as an optional local capability with explicit availability detection and fallback

**Rationale**: Public `sqlite-vec` documentation confirms it is available as a
Rust package and SQLite extension, but the project is still pre-v1. The plan
must therefore treat `sqlite-vec` as an optional capability that can be loaded
or detected per workspace rather than a hard requirement for basic correctness.
When vector capability is unavailable, unsupported, or cannot be initialized,
Boundline must surface that state and continue on the V1 FTS plus structured
fallback path.

**Alternatives considered**:

- Failing the whole advanced-context query when `sqlite-vec` is unavailable:
  rejected because S5.v2 must degrade to V1, not block delivery.
- Hiding vector-capability failure behind silent downgrade: rejected because
  the constitution requires explicit intelligence and visible fallback.
- Waiting for a later mature vector engine before planning S5.v2: rejected
  because the roadmap already scopes this slice as the next additive layer.

## Decision 4: Limit semantic indexing to V1-explainable artifact classes with provenance-preserving local embeddings

**Rationale**: Semantic acceleration only helps if every vector-backed match can
be traced back to the V1 evidence model and, for Canon-backed content, to the
producer semantic contract. The first slice should therefore embed only source
classes that already have an explainable provenance path: Markdown documents,
source-file chunks, review findings, verification evidence, runtime traces, and
compatible Canon artifacts. Every semantic chunk remains anchored to a stable
`source_ref` or Canon semantic provenance reference so V2 never manufactures an
opaque candidate.

**Alternatives considered**:

- Embedding every file in the repository indiscriminately: rejected because it
  widens scope, increases refresh cost, and weakens explainability.
- Using remote embeddings by default: rejected because code and Canon content
  may be sensitive and the slice must remain local-only.
- Deferring Canon artifacts until a later slice: rejected because the spec
  explicitly requires contract-compatible Canon semantic consumption now.

## Decision 5: Express semantic behavior as hybrid candidate annotations on the existing retrieval projection

**Rationale**: The current V1 runtime already persists `AdvancedContextProjection`
with selected evidence, rejected candidates, relationships, impact findings,
and terminal reasons. S5.v2 should extend that typed projection with hybrid
match metadata such as semantic capability state, match origin (`fts`,
`semantic_expand`, `semantic_rerank`), semantic selection or rejection reasons,
and fallback cause rather than creating a second semantic-only inspection
surface. This keeps `plan`, `status`, `next`, and `inspect` on the same
operator path.

**Alternatives considered**:

- A separate semantic report detached from advanced context: rejected because it
  would split observability across two runtime surfaces.
- Opaque scoring only in trace logs: rejected because the spec requires normal
  operator surfaces to explain expansion, reranking, and fallback behavior.
- Replacing V1 candidate models entirely: rejected because S5.v2 is additive,
  not a second retrieval architecture.

## Decision 6: Consume Canon semantic metadata only through the new producer contract plus the existing indexing baseline

**Rationale**: Earlier Canon contracts define stable indexing and promotion
semantics but do not define semantic eligibility, provenance boundary, or
compatibility for downstream vector consumers. Boundline should therefore depend
on Canon's new `056-semantic-artifact-contract` plus the existing indexing and
promotion contracts, while continuing to own runtime ranking, chunk derivation,
and fallback decisions locally.

**Alternatives considered**:

- Parsing Canon prose or inferred semantics without a contract: rejected
  because it would create ambiguous compatibility behavior.
- Letting Canon define Boundline ranking or fragment policy: rejected because
  it violates Boundline's independent runtime boundary.
- Omitting Canon from the first S5.v2 slice: rejected because the feature spec
  explicitly requires contract-compatible Canon participation and skip reasons.
