# Feature Specification: Large Codebase Context Substrate

**Feature Branch**: `070-large-codebase-context-substrate`

**Created**: 2026-06-05

**Status**: Draft

**Input**: User description: "Create the Boundline feature specification for the next roadmap item: large codebase context substrate. The feature must define a local large-codebase context substrate with context fidelity tiers, inclusion modes, omitted-context reasons, critical-context blocking behavior, search-before-read, symbol-aware indexing, repository map navigation, hybrid ranking, lazy hash references, patch-safe editing, and the boundary for a derived persistent context snapshot cache that is explicitly not memory. Copy roadmap/features/06-large-codebase-context-substrate.md into the new spec folder as feat-large-codebase-context-substrate.md, remove the roadmap feature file, and update roadmap folder files accordingly." More input defined in `./feat-large-codebase-context-substrate.md`

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Protect Critical Context In Large Repositories (Priority: P1)

An operator working in a large repository wants Boundline to refuse unsafe
context loading behavior, keep critical planning and execution context at high
fidelity, and stop the runtime when critical context would otherwise be silently
omitted or reduced to a lossy summary.

**Why this priority**: This is the first user-visible safety boundary. If
Boundline still performs unsafe huge reads or silently drops critical context,
the feature fails its core purpose.

**Independent Test**: Can be fully tested by running context selection against a
fixture repository containing oversized files, active planning artifacts, and
critical execution inputs, then confirming that unsafe full reads are refused,
critical context is preserved or explicitly blocked, and the runtime does not
continue on a silently lossy context pack.

**Acceptance Scenarios**:

1. **Given** an active goal, active spec, active plan, and failing tests in a
   large repository, **When** Boundline builds a context pack, **Then** those
   critical items remain directly included or are retrieved through a mandatory
   high-fidelity path rather than being silently summarized away.
2. **Given** a requested full read of an oversized file without an explicit
   operator allowance, **When** Boundline prepares planning or execution
   context, **Then** it refuses the unsafe full read and records why the file
   was not loaded in full.
3. **Given** a critical context item cannot be included at required fidelity,
   **When** Boundline evaluates context readiness, **Then** it produces a
   blocking outcome instead of continuing with a silently degraded pack.

---

### User Story 2 - Explain Context Inclusion And Omission (Priority: P2)

An operator wants inspectable context-pack output that explains what Boundline
selected, what it omitted, the fidelity tier and inclusion mode for each item,
and why those decisions were made before planning or execution continues.

**Why this priority**: Without inspectable selection and omission reasoning, the
runtime becomes harder to trust and harder to repair when context is incomplete
or unexpectedly narrow.

**Independent Test**: Can be fully tested by building a context pack for a
large repository fixture and confirming that Boundline surfaces selected items,
omitted items, fidelity tiers, inclusion modes, reasons, authority, and search
or ranking rationale through inspectable runtime output.

**Acceptance Scenarios**:

1. **Given** a context pack that mixes critical, supporting, ambient, and
   archived candidates, **When** the operator inspects the session, **Then**
   Boundline shows each included or omitted item with its fidelity tier,
   inclusion mode, and omission or inclusion reason.
2. **Given** a large repository where only excerpts, digests, or signatures are
   used for some artifacts, **When** the operator inspects context selection,
   **Then** Boundline shows that those artifacts were compacted intentionally
   and how the full source can be resolved on demand.

---

### User Story 3 - Keep Derived Cache Separate From Memory (Priority: P3)

An operator wants Boundline to reuse a local derived context snapshot cache when
it is fresh, invalidate it when repository or runtime conditions change, and
keep that cache explicitly separate from memory, truth, or governed knowledge.

**Why this priority**: The cache boundary prevents the context substrate from
quietly becoming an authority surface or an implicit memory system.

**Independent Test**: Can be fully tested by creating a reusable local snapshot,
triggering freshness events such as branch or config changes, and confirming
that Boundline invalidates or rebuilds the cache while continuing to describe it
as derived, local, disposable, and non-authoritative.

**Acceptance Scenarios**:

1. **Given** a previously derived local context snapshot, **When** a freshness
   event such as a branch switch, merge, schema change, adapter change, or
   Canon packet change occurs, **Then** Boundline marks the snapshot stale
   before it can be treated as a reusable planning context.
2. **Given** a workspace where cache files have been accidentally tracked,
   **When** the operator runs workspace diagnostics, **Then** Boundline reports
   that tracked-cache state as a repairable problem rather than silently
   accepting it.

### Edge Cases

- A critical context item is large enough to exceed normal read limits but is
  still required for a safe planning or execution decision.
- Search-before-read returns multiple plausible sources and no single candidate
  is clearly authoritative.
- The repository map is stale or unavailable when a planning gate expects
  source-aware context selection.
- A large log or diff is compacted by digest, but later becomes directly
  relevant to a blocked runtime decision.
- A cache snapshot exists for the workspace, but a branch or configuration
  change happened after it was created.
- An omitted context item becomes critical because the active phase or stage
  ownership changed after the first context-pack build.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST classify each context candidate into an explicit
  fidelity tier before the candidate can be included, summarized, digested, or
  omitted from runtime context.
- **FR-002**: The system MUST support explicit inclusion modes for context items
  and MUST record which mode was used for every included or omitted item in the
  final context-pack projection.
- **FR-003**: The system MUST treat critical context as a protected class that
  cannot be silently omitted or represented only by a lossy summary.
- **FR-004**: The system MUST block planning or execution admission when
  critical context required for the active runtime decision is unavailable at
  required fidelity.
- **FR-005**: The system MUST refuse unsafe large full-file reads unless an
  explicit operator-allowed path exists for that request.
- **FR-006**: The system MUST require a search-before-read path for large-file
  or large-artifact discovery, using one or more repository discovery signals
  before a full content read is attempted.
- **FR-007**: The system MUST maintain a compact repository navigation model
  that helps Boundline locate files, symbols, relationships, and nearby tests
  before loading large context bodies.
- **FR-008**: The system MUST rank context candidates using multiple local
  relevance signals rather than a single score, and the runtime MUST preserve
  enough explanation to show why selected items outranked omitted ones.
- **FR-009**: The system MUST compact very large logs, diffs, generated output,
  and similar artifacts into digest-backed references plus bounded summaries or
  excerpts unless full content is explicitly required.
- **FR-010**: The system MUST provide inspectable omission reasons for context
  that was excluded, downgraded, or compacted so operators can see what the
  runtime did not load and why.
- **FR-011**: The system MUST require patch-safe editing behavior for large-file
  changes, including anchored edit scopes, drift detection, and post-apply
  verification before the edit is treated as accepted.
- **FR-012**: The system MUST define a persistent local context snapshot cache
  boundary that is explicitly derived, local, disposable, rebuildable, and
  non-authoritative.
- **FR-013**: The system MUST invalidate or downgrade a cached context snapshot
  when freshness events affecting repository shape, runtime shape, or governed
  evidence occur.
- **FR-014**: The system MUST NOT treat the persistent context snapshot cache as
  memory, semantic truth, or reviewed knowledge.
- **FR-015**: The system MUST surface tracked-cache and stale-cache problems
  through repairable diagnostics rather than silently continuing as if the cache
  were valid.
- **FR-016**: The system MUST preserve source attribution for summarized,
  excerpted, compacted, or digest-backed context so an operator can resolve the
  original source on demand.
- **FR-017**: The system MUST keep archived or discardable context out of normal
  planning and execution admission unless an explicit inspect or archive lookup
  requests it.

### Key Entities *(include if feature involves data)*

- **Context Candidate**: A repository, trace, runtime, or governed artifact
  considered for inclusion in a context pack, including its source reference,
  authority, fidelity tier, and relevance rationale.
- **Context Pack Entry**: The persisted record of how a single context candidate
  was handled, including inclusion mode, reason, budget or cost indication, and
  omission or inclusion status.
- **Repository Navigation Map**: The local navigation projection that links
  files, symbols, relationships, tests, and other discovery hints used before
  large reads occur.
- **Context Omission Finding**: An inspectable runtime record explaining why a
  candidate was omitted, compacted, downgraded, or blocked from direct
  inclusion.
- **Context Snapshot Cache Entry**: A local derived snapshot of reusable context
  state that carries freshness and authority metadata while remaining
  explicitly non-memory.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In validated large-artifact regression cases, 100% of oversized
  full-read attempts without explicit allowance are refused, downgraded, or
  redirected before unsafe full content loading occurs.
- **SC-002**: In validated critical-context omission scenarios, Boundline blocks
  planning or execution admission in 100% of cases rather than continuing with
  a silently lossy context pack.
- **SC-003**: Operators can identify why a context item was included, omitted,
  compacted, or downgraded from inspectable runtime output in under 30 seconds
  for the maintained regression fixtures.
- **SC-004**: In the maintained large-repository fixture set, initial
  context-pack selection completes within 10 seconds for at least 95% of runs.
- **SC-005**: In validated freshness-event scenarios, stale snapshot cache
  entries are invalidated or marked non-reusable in 100% of cases before they
  can influence a new planning context.

## Assumptions

- The first slice focuses on local repository context selection, omission
  visibility, and derived cache boundaries rather than remote context services
  or provider-owned retrieval behavior.
- Provider-supplied context, Canon packet schema, and reviewed memory proposals
  remain separately owned surfaces that this feature may consume but does not
  redefine.
- A bounded operator override path for exceptional full reads may already exist
  or may be introduced later, but unsafe large full reads are refused by
  default in this slice.
- The initial repository navigation model targets the common repository shapes
  already covered by Boundline regression fixtures rather than every possible
  language ecosystem at once.
