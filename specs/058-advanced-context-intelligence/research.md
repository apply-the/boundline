# Research: Advanced Context Intelligence

## Decision 1: Use one workspace-local SQLite + FTS5 retrieval index as the required storage baseline

**Rationale**: The feature must remain local-first, deterministic, and easy to
package inside the existing Boundline CLI workspace. A single workspace-local
database under `.boundline/index/` can hold searchable document chunks,
structured metadata, Canon artifact references, and projected relationships
without introducing a service dependency or a second operational runtime.
SQLite with FTS5 aligns with the S5 addendum and gives explainable keyword and
phrase retrieval even when vector or remote expansion is unavailable.

**Alternatives considered**:

- Plain file-only indexes in JSON or Markdown: rejected because update,
  ranking, and query behavior would become harder to keep bounded and
  inspectable as the corpus grows.
- LanceDB as the default store: rejected for the first slice because it adds
  packaging and operational complexity that the local-first runtime does not
  need yet.
- Qdrant or other service-backed retrieval stores: rejected because the core
  feature must remain usable offline and without mandatory infrastructure.

## Decision 2: Model relationship and impact reasoning as typed local projections, not as an external graph runtime

**Rationale**: S5 needs explainable relationships among systems, domains,
invariants, tests, contracts, reviewers, and evidence, but the constitution
requires the smallest viable slice and a sequential-first execution model.
Typed local relationship projections stored alongside retrieved evidence are
enough to support impact analysis, missing-test detection, and reviewer-gap
inference without introducing a graph server or an entirely new reasoning
subsystem.

**Alternatives considered**:

- Kuzu as an immediate embedded graph dependency: deferred because the first
  slice can get most of the value from structured relationship tables and typed
  projections without paying for graph-specific complexity.
- Neo4j as the primary graph layer: rejected because it violates the local-
  first and lightweight-distribution goals.
- SurrealDB as a document-graph hybrid: rejected because it broadens the
  architecture far beyond the delivery need of the first slice.

## Decision 3: Consume Canon through the existing artifact-indexing baseline, with no new Canon feature slice

**Rationale**: Canon already owns the producer-side artifact indexing contract
through `051-artifact-indexing-contract` and the stable promotion-contract
documentation. Boundline only needs a consumer-side contract that says which
Canon metadata it may index, when incompatible artifacts must be ignored, and
how Canon provenance remains visible in retrieved evidence. This preserves the
boundary that Canon owns semantic metadata while Boundline owns runtime
retrieval, ranking, and impact decisions.

**Alternatives considered**:

- Creating a parallel Canon S5 spec: rejected because it would duplicate an
  already-stable producer surface without adding delivery value.
- Parsing Canon prose without a contract: rejected because it would make the
  retrieval path fragile and under-specified.

## Decision 4: Keep remote semantic expansion opt-in and non-authoritative

**Rationale**: Source code and Canon-backed artifacts may be sensitive, and the
core feature must remain usable in offline or policy-restricted environments.
Remote semantic expansion therefore stays disabled by default, must be
explicitly enabled by the workspace, and can only enrich local results. It may
never override structured runtime context, Canon contract semantics, or
workspace configuration.

**Alternatives considered**:

- Remote retrieval enabled by default: rejected because it breaks local-first
  expectations and can leak code or governed content.
- Separate remote-only retrieval mode: rejected because the feature must remain
  correct even when no network or provider is available.

## Decision 5: Treat vector search and richer provider integrations as additive accelerators, not as correctness prerequisites

**Rationale**: The first delivery slice must already be useful with structured
runtime context, FTS-backed retrieval, explainable impact projection, and
explicit degraded states. Local vector similarity and richer provider adapters
may improve ranking and discovery later, but they should plug into the same
retrieval-mode, provenance, and authority-ordering model rather than defining a
second retrieval architecture.

**Alternatives considered**:

- Designing around vector search first: rejected because it would make
  explainability and local-first operation harder to guarantee.
- Deferring all semantic behavior until a full vector stack exists: rejected
  because the feature still delivers immediate value through hybrid retrieval,
  relationship projection, and bounded impact analysis in the first slice.