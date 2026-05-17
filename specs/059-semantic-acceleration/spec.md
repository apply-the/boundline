# Feature Specification: Advanced Context Intelligence Semantic Acceleration

**Feature Branch**: `059-semantic-acceleration`  
**Created**: 2026-05-17  
**Status**: Draft  
**Input**: User description: "procediamo con S5v2, prossima spec. Se serve modificare canon, crea una spec anche in /Users/rt/workspace/apply-the/canon e allinea le due spec con un contract"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Recover Relevant Evidence Beyond Keywords (Priority: P1)

As a Boundline operator using the session-native workflow, I want Boundline to
augment the S5 V1 retrieval baseline with optional local semantic acceleration
so semantically relevant code, docs, traces, and compatible Canon artifacts can
be found even when keyword overlap is weak.

**Why this priority**: S5.v2 only matters if it improves real delivery context
assembly without weakening the proven S5 V1 baseline.

**Independent Test**: In a workspace where the best supporting evidence is
semantically related but not a strong lexical match, run the same bounded task
with semantic acceleration disabled and enabled and verify that the enabled path
surfaces additional relevant local evidence while preserving the existing
authority order.

**Acceptance Scenarios**:

1. **Given** local semantic acceleration is enabled and a bounded goal has weak
   keyword overlap with the most relevant local evidence, **When** Boundline
   assembles advanced context, **Then** it surfaces that semantically related
   evidence without displacing structured runtime context as the primary input.
2. **Given** semantic acceleration is unavailable, unsupported, or degraded in
   the active workspace, **When** Boundline builds advanced context, **Then** it
   falls back explicitly to the S5 V1 retrieval path and records why the
   accelerator did not participate.
3. **Given** the same task is routed through a compatibility workflow,
   **When** semantic acceleration is surfaced, **Then** the runtime labels the
   compatibility route explicitly as secondary to the primary session-native
   path.

---

### User Story 2 - Explain Hybrid Ranking And Rejection (Priority: P1)

As a maintainer or reviewer, I want `status`, `inspect`, and trace surfaces to
show whether semantic similarity expanded the V1 candidate set or reranked it,
so I can trust why retrieval changed.

**Why this priority**: A stronger retrieval path is not acceptable unless it
remains explainable enough for delivery and review decisions.

**Independent Test**: Run a bounded task that produces both lexically matched
and semantically matched evidence, then inspect the resulting projection and
verify that it explains which candidates were expanded, reranked, downgraded,
or rejected and why.

**Acceptance Scenarios**:

1. **Given** a candidate is selected through semantic similarity, **When** an
   operator inspects the result, **Then** the runtime shows the source artifact,
   provenance, selection rationale, and whether the candidate expanded the V1
   set or reranked it.
2. **Given** a semantic candidate is downgraded or rejected, **When** the
   operator uses `inspect` or trace output, **Then** the runtime surfaces the
   rejection reason instead of hiding the candidate.
3. **Given** semantic evidence is stale, weak, or contradictory, **When**
   relationship and impact projection continue, **Then** Boundline degrades
   explicitly rather than implying unwarranted confidence.

---

### User Story 3 - Respect Canon And Workspace Boundaries (Priority: P2)

As an operator in a governed repository, I want Boundline to consume Canon
semantic metadata only through a documented contract and obey workspace policy,
so semantic acceleration stays local, bounded, and optional.

**Why this priority**: Cross-repo enrichment is only safe if producer
boundaries remain explicit and local policy can disable the accelerator without
breaking delivery.

**Independent Test**: Run a bounded task in a workspace that contains
compatible and incompatible Canon artifacts, then verify that Boundline accepts
only contract-compatible artifacts, surfaces skip reasons for the rest, and
continues to operate correctly when semantic acceleration is disabled.

**Acceptance Scenarios**:

1. **Given** a Canon artifact exposes compatible semantic eligibility metadata,
   **When** Boundline includes it in semantic retrieval, **Then** the runtime
   preserves the Canon artifact class, semantic contract line, and provenance
   reference in the projection.
2. **Given** a Canon artifact is missing required semantic metadata or uses an
   unsupported contract line, **When** Boundline evaluates it, **Then** the
   runtime skips the artifact and surfaces the compatibility reason.
3. **Given** semantic acceleration is disabled by workspace policy, **When**
   the same bounded task runs, **Then** delivery still succeeds on the S5 V1
   baseline without any hidden dependency on semantic acceleration.

### Edge Cases

- The semantic accelerator is enabled but the workspace lacks the required
  local capability, so Boundline must surface an explicit fallback to the S5 V1
  path instead of silently degrading.
- Semantic similarity finds a promising candidate that conflicts with stronger
  structured runtime evidence, so the semantic candidate must remain
  non-authoritative.
- A Canon artifact is indexable under the V1 indexing contract but excluded or
  unsupported under the semantic contract, so Boundline must explain why it was
  not used for semantic expansion.
- The semantic candidate budget is exhausted before a confident candidate is
  selected, so the runtime must surface a bounded degraded or exhausted state.
- The same semantic evidence appears on both the session-native and
  compatibility paths, so the runtime must preserve route labeling and avoid
  implying the compatibility path is primary.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST treat semantic acceleration as an optional
  augmentation layer on top of the S5 V1 advanced-context baseline rather than
  a replacement for that baseline.
- **FR-002**: Boundline MUST preserve retrieval authority order of structured
  runtime context first, Canon-governed memory second, workspace overrides
  third, semantic retrieval fourth, and similarity expansion after those
  authoritative inputs.
- **FR-003**: Boundline MUST support explicit `disabled` and `local` semantic
  acceleration states, with `disabled` remaining the safe baseline until the
  workspace opts into local acceleration.
- **FR-004**: Boundline MUST evaluate semantic acceleration only from
  workspace-local state and locally available artifacts in this slice.
- **FR-005**: Boundline MUST keep the S5 V1 lexical retrieval and structured
  fallback path available even when semantic acceleration is enabled.
- **FR-006**: Boundline MUST allow semantic acceleration to expand or rerank a
  V1 candidate set without bypassing the V1 collection pass.
- **FR-007**: Boundline MUST apply explicit bounds to semantic candidate
  expansion, reranking, and selected-evidence counts.
- **FR-008**: Boundline MUST fall back explicitly to the S5 V1 path when
  semantic acceleration is disabled, unavailable, incompatible, exhausted, or
  otherwise unable to improve the current bounded query.
- **FR-009**: Boundline MUST preserve source provenance for every semantic
  candidate and, for Canon-backed candidates, MUST retain the artifact class,
  semantic contract line, and semantic provenance reference.
- **FR-010**: Boundline MUST consume Canon semantic eligibility only through
  the documented Canon producer contract for semantic artifacts plus the
  existing Canon indexing contract.
- **FR-011**: Boundline MUST skip Canon artifacts that are excluded,
  incompatible, or unsupported under the Canon semantic contract and MUST
  surface the skip reason.
- **FR-012**: Boundline MUST keep semantic candidates non-authoritative; they
  may enrich retrieval but MUST NOT override runtime manifests, workspace
  configuration, Canon promotion semantics, or explicit human-authored context.
- **FR-013**: Boundline MUST make `status`, `inspect`, and trace surfaces
  answer whether a semantic candidate expanded or reranked the V1 set and why
  it was selected, downgraded, rejected, or skipped.
- **FR-014**: Boundline MUST keep the session-native workflow as the primary
  surface for semantic acceleration and MUST label compatibility behavior as
  secondary when both paths exist.
- **FR-015**: Boundline MUST NOT require remote providers, external retrieval
  services, or Canon-owned runtime control for correctness in this slice.

### Scope Boundaries *(mandatory)*

- **In Scope**: optional local semantic acceleration layered on the S5 V1
  baseline, hybrid retrieval that can expand or rerank V1 candidates,
  explainable semantic selection and rejection surfaces, local fallback to the
  V1 path, Canon semantic-contract consumption, and bounded runtime visibility
  on session-native and explicit compatibility surfaces.
- **Out of Scope**: mandatory remote retrieval, external vector services,
  Canon-owned ranking policy, new review-council logic, S6 reasoning profiles,
  S7 assistant affordance work, hosted semantic providers, distributed indexes,
  autonomous memory mutation, and UI surfaces outside existing CLI and trace
  projections.

### Key Entities *(include if feature involves data)*

- **Semantic Acceleration Policy**: The workspace-controlled state that enables
  or disables local semantic acceleration and preserves explicit fallback to the
  S5 V1 baseline.
- **Hybrid Retrieval Query**: A bounded retrieval request that combines the S5
  V1 baseline with optional semantic expansion or reranking while preserving
  authority order.
- **Semantic Match Explanation**: The structured rationale that records whether
  a candidate expanded or reranked the V1 set, why it was accepted or rejected,
  and which source artifact it came from.
- **Canon Semantic Artifact View**: The consumer-side interpretation of a
  Canon artifact that carries contract-compatible semantic eligibility,
  provenance, and compatibility metadata.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative bounded-delivery scenarios where relevant local
  evidence exists but keyword overlap is weak, operators can surface at least
  one additional relevant evidence item in at least 80% of evaluation cases
  with semantic acceleration enabled.
- **SC-002**: 100% of runs where semantic acceleration is disabled,
  unavailable, incompatible, or exhausted end in an explicit fallback,
  degraded, or insufficient state rather than a hidden retrieval failure.
- **SC-003**: Reviewers can determine within 5 minutes whether a semantic
  candidate expanded or reranked the V1 set and why it was selected or rejected
  by using normal `status`, `inspect`, or trace output.
- **SC-004**: 100% of representative tasks that already succeed under the S5
  V1 baseline remain executable with semantic acceleration disabled.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: OpenAI model docs at
  `developers.openai.com/api/docs/models`, Anthropic Claude model overview at
  `platform.claude.com/docs/en/docs/about-claude/models`, and Google Gemini
  model docs at `ai.google.dev/gemini-api/docs/models`, reviewed on 2026-05-17.
- **Catalog Delta**: No bundled catalog changes were required during this spec
  pass.
- **No-Change Rationale**: The bundled catalog still matches the publicly
  documented assistant-facing OpenAI, Claude, and Gemini families relevant to
  Boundline routing, and the additional audio, image, embedding, and media
  models visible in public docs are outside the assistant-routing scope of this
  slice.

## Assumptions

- The S5 V1 advanced-context baseline and its current inspectable projection
  surfaces are already operational before this slice begins.
- The first S5.v2 slice keeps semantic acceleration local-only and optional.
- Canon will extend artifact metadata additively for semantic eligibility and
  provenance rather than replacing the existing V1 indexing contract.
- The bundled assistant catalog remains sufficient for this slice because S5.v2
  does not introduce a new required hosted provider dependency.
