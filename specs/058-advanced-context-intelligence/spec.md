# Feature Specification: Advanced Context Intelligence

**Feature Branch**: `058-advanced-context-intelligence`
**Created**: 2026-05-16
**Status**: Reviewed
**Input**: User description: "Deliver the S5 V1 advanced-context baseline for Boundline using local SQLite + FTS5 + structured retrieval, explainable relationship and impact projection, Canon consumer compatibility, and explicit disabled or local retrieval policy without requiring semantic providers, graph infrastructure, or remote services."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Expand Context Without Losing Authority (Priority: P1)

As a Boundline operator using the session-native workflow, I want Boundline to
reuse bounded workspace evidence, traces, and compatible Canon artifacts through
one local retrieval pass so planning and status surfaces gain more context
without weakening the current authority model.

**Why this priority**: S5 V1 is valuable only if it improves real delivery
choices while keeping structured runtime evidence authoritative.

**Independent Test**: In a representative workspace with local files, tests,
prior traces, and compatible Canon metadata, run `plan`, `status`, and
`inspect` and verify that Boundline keeps structured context first, selects
local evidence through SQLite + FTS5 or explicit structured fallback, and
records visible rationale for the chosen evidence.

**Acceptance Scenarios**:

1. **Given** a workspace with bounded source and test files, **When** Boundline
   assembles advanced context, **Then** it preserves the authority order of
   structured runtime context first, Canon-governed memory second, workspace
   overrides third, and local retrieved evidence after those sources.
2. **Given** a workspace where SQLite FTS does not return a stronger match,
   **When** Boundline builds the advanced context projection, **Then** it falls
   back to structured bounded ordering and records why the fallback path was
   used.
3. **Given** a Canon artifact with compatible indexing metadata, **When**
   Boundline considers it for retrieval, **Then** the artifact remains semantic
   enrichment only and does not override explicit runtime or workspace state.

---

### User Story 2 - See Impact And Review Gaps Early (Priority: P1)

As a maintainer or reviewer, I want Boundline to derive explainable
relationships and impact findings from the selected evidence so I can see blast
radius, missing tests, required evidence, and related contract exposure before
execution or review continues.

**Why this priority**: Relationship and impact reasoning are the delivery value
added by S5 V1 beyond plain search.

**Independent Test**: Run a bounded change against a workspace containing
source files, tests, and Canon-backed memory, then verify that `status` and
`inspect` surface affected systems, missing tests, and evidence gaps with
explicit provenance.

**Acceptance Scenarios**:

1. **Given** selected evidence that includes source and test surfaces, **When**
   Boundline projects relationships, **Then** it surfaces explainable
   `exercises_test` and `requires_evidence` relationships where the local
   evidence supports them.
2. **Given** a selected source target without supporting test evidence,
   **When** Boundline performs impact analysis, **Then** it records a visible
   `missing_test` or `evidence_gap` finding instead of inferring confidence.
3. **Given** insufficient relationship evidence, **When** Boundline cannot make
   a credible impact claim, **Then** it reports the gap explicitly and avoids
   presenting tentative inference as settled fact.

---

### User Story 3 - Keep Retrieval Optional, Bounded, And Local-First (Priority: P2)

As an operator working in mixed trust environments, I want advanced retrieval
to remain optional, bounded, and local-first so Boundline stays usable offline
and does not depend on embeddings, hosted retrieval, or remote providers for
correctness.

**Why this priority**: The retrieval layer is acceptable only if the product
remains deterministic and local-first by default.

**Independent Test**: Run the same bounded workflow with retrieval disabled and
with local retrieval enabled, then verify that Boundline stays functional in
both modes, records the applied policy, and rejects unsupported remote settings
under the V1 contract.

**Acceptance Scenarios**:

1. **Given** a workspace with `retrieval_mode = "disabled"`, **When**
   Boundline builds advanced context, **Then** it records a terminal reason
   that retrieval was intentionally disabled and does not select evidence.
2. **Given** the default local policy, **When** Boundline builds advanced
   context, **Then** it uses only workspace-local SQLite + FTS5 indexing and
   marks remote transmission as blocked or local-only.
3. **Given** a config that requests remote retrieval or remote transmission,
   **When** Boundline validates the policy, **Then** it rejects that config as
   unsupported in the S5 V1 baseline instead of attempting an implicit remote
   path.

### Edge Cases

- SQLite retrieval finds no stronger match and Boundline must preserve the
  bounded target list through structured fallback.
- A Canon artifact exists but exposes an unsupported contract line or missing
  indexing metadata.
- Retrieved evidence becomes stale relative to the bounded session and the
  projection must degrade explicitly.
- The evidence set reaches configured depth, traversal, or selected-evidence
  limits and the projection must remain bounded.
- A projected relation or impact claim lacks sufficient local evidence and must
  remain tentative or be omitted.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST introduce advanced context intelligence as an
  optional augmentation layer on top of the existing structured runtime context
  rather than a replacement for that context.
- **FR-002**: Boundline MUST preserve explicit retrieval precedence of
  structured runtime context first, Canon-governed memory second, workspace
  overrides third, and retrieved local evidence after those authoritative
  inputs.
- **FR-003**: Boundline MUST support a retrieval-disabled mode that preserves
  the structured-only path without requiring SQLite retrieval to succeed.
- **FR-004**: Boundline MUST support bounded local retrieval across workspace
  files, tests, traces, and compatible Canon artifacts using one workspace-
  local SQLite + FTS5 index with structured fallback ordering.
- **FR-005**: Boundline MUST project explicit relationships and impact findings
  from selected evidence when sufficient local evidence exists to support the
  claim.
- **FR-006**: Boundline MUST explain why a document, relation, or impact
  finding was surfaced, including provenance and selection rationale.
- **FR-007**: Boundline MUST surface retrieval mode, index state, selected
  evidence, relationships, impact findings, and terminal or degraded reasons
  through `status`, `inspect`, and trace projections.
- **FR-008**: Boundline MUST keep retrieved evidence non-authoritative; it may
  enrich context but MUST NOT override explicit runtime manifests, Canon
  contract semantics, or workspace configuration.
- **FR-009**: Boundline MUST apply explicit budgets to refinement, refresh,
  traversal, expansion, depth, and selected-evidence count, and MUST record
  the resulting bounded state in the projection.
- **FR-010**: Boundline MUST consume Canon artifact indexing metadata only
  through the documented Canon producer contract and the Boundline-owned
  consumer interpretation of that contract.
- **FR-011**: Boundline MUST preserve local-first operation and MUST NOT
  require embeddings, vector search, graph databases, or remote services for
  S5 V1 correctness.
- **FR-012**: Boundline MUST reject unsupported S5.v2-or-later settings such
  as remote retrieval or remote transmission under the S5 V1 config contract.
- **FR-013**: Boundline MUST degrade explicitly when the local index is stale,
  retrieval is insufficient, Canon metadata is incompatible, or evidence is too
  weak to support a projection.
- **FR-014**: Boundline MUST keep any future semantic acceleration,
  sqlite-vec, or graph-projection work outside the S5 V1 baseline and document
  those features as deferred to later slices.
- **FR-015**: Boundline MUST NOT introduce hosted RAG, distributed search,
  remote embeddings, autonomous memory mutation, or Canon-owned runtime policy
  in this slice.

### Scope Boundaries *(mandatory)*

- **In Scope**: local SQLite + FTS5 retrieval, structured fallback ordering,
  Canon consumer compatibility checks, explainable relationship projection,
  impact findings, status and inspect visibility, trace projection, and typed
  disabled or local retrieval policy.
- **Out of Scope**: sqlite-vec similarity, embeddings, graph databases,
  hosted retrieval, remote providers, remote semantic modes, distributed
  indexes, autonomous memory mutation, Canon producer-side redesign, UI work,
  and deployment changes.

### Key Entities *(include if feature involves data)*

- **Retrieval Query**: A bounded request for additional local context derived
  from the active delivery state and selected targets.
- **Retrieved Evidence Candidate**: A workspace or Canon-backed artifact that
  may enrich the current context, including provenance, authority rank,
  selection rationale, and credibility status.
- **Relationship Projection**: An explainable link between selected evidence
  and delivery-relevant concepts such as tests, systems, domains, or evidence
  requirements.
- **Impact Analysis Finding**: A projected delivery implication showing what is
  affected, what evidence is missing, and what follow-up is warranted.
- **Retrieval Policy**: The typed local config that controls disabled or local
  retrieval mode, remote-policy disclosure, and bounded budgets.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative bounded-delivery workspaces, operators can see
  the selected evidence, authority order, and retrieval rationale from `status`
  or `inspect` within 5 minutes.
- **SC-002**: 100% of runs where retrieval is disabled, insufficient, or
  degraded end in an explicit terminal or degraded state rather than hidden
  failure.
- **SC-003**: For representative bounded changes, Boundline surfaces missing
  tests, evidence gaps, and explainable relationships before execution or
  review continues.
- **SC-004**: Boundline remains fully usable without semantic acceleration,
  graph infrastructure, or remote providers.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: OpenAI, Anthropic, and Google model catalogs on
  2026-05-16.
- **Catalog Delta**: No bundled catalog changes were required for S5 V1.
- **No-Change Rationale**: S5 V1 does not depend on remote model or embedding
  providers; the provider audit remains informative only and does not change
  the local-first implementation contract.

## Assumptions

- The primary product path remains the session-native workflow, and advanced
  context intelligence is evaluated there before any compatibility path.
- The S5 V1 baseline is the local SQLite + FTS5 retrieval layer described in
  the S5 addendum; semantic acceleration is deferred to S5.v2.
- Existing runtime intelligence and Canon artifact-indexing contracts provide
  enough authoritative inputs for this slice without inventing a second source
  of truth.