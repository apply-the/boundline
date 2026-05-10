# Feature Specification: Catalog Freshness, Independent Voting, and File-Backed Inputs

**Feature Branch**: `047-catalog-voting-inputs`  
**Created**: 2026-05-10  
**Status**: Draft  
**Input**: User description: "Refresh the bundled provider model catalog from current public web docs, require independent reviewer voting so review councils do not collapse onto the same effective runtime/model, and extend prompted authored-input fields to accept inline text, a single workspace file path, or an array of workspace file paths such as [./docs/prd.md, ./docs/adr.md]."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.
  When both a session-native workflow and a compatibility workflow exist, the spec MUST name which path is primary and keep compatibility behavior explicit rather than implicit.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Choose From A Current Catalog (Priority: P1)

As an operator configuring Boundline routes, I want the bundled runtime and model
catalog to reflect the currently documented route-capable models for Copilot,
Claude, Codex, and Gemini so that I can choose known models without guessing or
falling back to custom identifiers for mainstream options.

**Why this priority**: If the bundled catalog is stale, `init`, config editing,
and assistant defaults immediately lose credibility at the first operator touchpoint.

**Independent Test**: Open guided route selection or inspect the bundled route
choices after a catalog refresh, verify that currently documented mainstream
models are available as bundled options, and confirm that stale or no-change
catalog outcomes are recorded explicitly.

**Acceptance Scenarios**:

1. **Given** current public provider documentation includes mainstream models
  such as GPT-5.5, Claude Opus 4.7, and Gemini 3.1 Pro Preview, **When** an
  operator edits a route, **Then** those models appear as bundled selections
  instead of requiring custom model entry.
2. **Given** a catalog refresh finds no change from the currently bundled model
  set, **When** the refresh completes, **Then** Boundline records an explicit
  no-change result instead of silently skipping catalog validation.
3. **Given** a provider publishes route-adjacent models that are not credible
  for Boundline planning, implementation, verification, review, or adjudication,
  **When** the catalog is refreshed, **Then** Boundline excludes those models
  explicitly instead of mixing unrelated audio, image, or media-only options
  into the standard route picker.

---

### User Story 2 - Start From File-Backed Authored Input (Priority: P2)

As an operator capturing a bounded delivery task, I want prompted authored input
to accept inline text, one workspace file path, or an ordered array of workspace
file paths such as `[./docs/prd.md, ./docs/adr.md]` so that I can reuse existing
repository documents without pasting them into one fragile prompt.

**Why this priority**: Boundline already depends on authored context. If the
prompt surface only accepts inline text, real delivery work still starts with
copy-paste friction and loses source provenance.

**Independent Test**: Start capture or run from prompted input using inline text,
one workspace document, and an ordered array of workspace documents, then verify
that Boundline preserves source order, provenance, and explicit failure behavior
for invalid paths.

**Acceptance Scenarios**:

1. **Given** an operator chooses file-backed input instead of inline text,
  **When** they provide one valid workspace path, **Then** Boundline captures
  that document as authored input and exposes the source path in inspectable
  state.
2. **Given** an operator provides an ordered array of workspace paths,
  **When** Boundline resolves the request, **Then** it preserves the declared
  order, deduplicates repeated entries deterministically, and records the full
  source list for later inspection.
3. **Given** a provided path is missing, unreadable, outside the workspace
  boundary, or not a credible text source, **When** Boundline tries to resolve
  it, **Then** Boundline stops or asks for correction explicitly instead of
  silently dropping that source.

---

### User Story 3 - Keep Review Voting Independent (Priority: P3)

As a developer relying on review voting, I want Boundline to count votes as
independent only when each counted reviewer resolves to a distinct effective
review route so that a nominal multi-review council cannot silently collapse
into the same reviewer repeated multiple times.

**Why this priority**: Voting only adds trust if the reviewers are actually
independent at the effective runtime and model level. Otherwise the system
inflates confidence without adding new evidence.

**Independent Test**: Run one review council whose members resolve to distinct
effective review routes and one council that collapses onto the same effective
route, then verify that Boundline accepts only the distinct council as an
independent vote and surfaces the collapsed council explicitly.

**Acceptance Scenarios**:

1. **Given** a review council resolves to distinct effective review routes,
  **When** the review phase runs, **Then** Boundline records each reviewer,
  counts the vote normally, and exposes the route behind each counted review.
2. **Given** two or more configured reviewers resolve to the same effective
  runtime and model pair, **When** Boundline evaluates council independence,
  **Then** it surfaces the council as non-independent before counting the vote.
3. **Given** Boundline cannot assemble a distinct reviewer set within the
  configured limits, **When** review is still required, **Then** Boundline stops,
  escalates, or degrades explicitly instead of presenting a misleading multi-vote
  outcome.

### Edge Cases

- If provider documentation introduces a renamed or preview-only model between
  refreshes, Boundline must either include it with explicit preview labeling or
  record why it was excluded from the bundled route-capable catalog.
- If a prompted file array contains the same path twice, mixes inline text with
  file paths, or includes documents whose ordering matters, Boundline must keep
  the resolved precedence visible rather than flattening the sources silently.
- If every supplied file path is invalid or outside the workspace, Boundline
  must stop before planning or execution with one explicit input-resolution
  failure.
- If a review council looks distinct by role name but resolves to the same
  effective runtime and model pair, Boundline must treat that as non-independent
  even when the configured reviewer roles are different.
- If the primary path remains session-native but an advanced compatibility path
  still accepts direct model routes or manifest-backed inputs, Boundline must
  expose which path is active instead of blending them.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: Boundline MUST keep a bundled model catalog for route selection
  that is refreshed against current public provider documentation.
- **FR-002**: Boundline MUST expose current mainstream route-capable models for
  Copilot, Claude, Codex, and Gemini as bundled options when those models are
  documented by their providers.
- **FR-003**: Boundline MUST keep the standard route catalog limited to models
  that are credible for planning, implementation, verification, review, or
  adjudication, rather than mixing unrelated audio, image, video, or other
  non-route product families into the default picker.
- **FR-004**: Boundline MUST record the outcome of each catalog refresh as an
  explicit delta or an explicit no-change result.
- **FR-005**: Boundline MUST allow prompted authored input to be supplied as
  inline text, one workspace file path, or an ordered array of workspace file
  paths.
- **FR-006**: Boundline MUST resolve prompted file paths only from within the
  active workspace boundary and MUST stop or clarify explicitly when a provided
  path is missing, unreadable, unsupported, or outside that boundary.
- **FR-007**: Boundline MUST preserve the provenance and declared order of every
  accepted input source across later status, inspect, and resumed task flows.
- **FR-008**: Boundline MUST deduplicate repeated prompted file inputs
  deterministically without silently changing the visible source set.
- **FR-009**: Boundline MUST support the same prompted input choices on the
  primary session-native path and any explicit compatibility path that accepts
  authored task input.
- **FR-010**: Boundline MUST treat a review council as independent only when
  each counted reviewer resolves to a distinct effective review route.
- **FR-011**: Boundline MUST expose the effective route behind each counted
  reviewer through inspectable review evidence.
- **FR-012**: Boundline MUST block, degrade, or escalate explicitly when a
  configured multi-review council collapses onto non-distinct effective routes.
- **FR-013**: Boundline MUST NOT present a collapsed council as a normal multi-
  reviewer vote outcome.
- **FR-014**: Boundline MUST preserve explicit failure or recovery evidence for
  catalog refresh problems, prompted input-resolution failures, and non-independent
  review councils.

### Scope Boundaries *(mandatory)*

- **In Scope**: refreshing the bundled route-capable model catalog from current
  public provider docs; exposing newly documented bundled models in operator
  route selection; prompted authored input as inline text, one workspace path,
  or an ordered array of workspace paths; and explicit review-voting independence
  checks for bounded review councils.
- **Out of Scope**: live network discovery during everyday `init` runs; support
  for every provider product family such as media, voice, or embedding models in
  the standard route picker; binary or out-of-workspace attachments; open-ended
  debate systems; unbounded reviewer orchestration; and general UI redesign.

### Key Entities *(include if feature involves data)*

- **Bundled Catalog Entry**: One curated route-capable model option, including
  runtime, operator-visible label, model identifier, and whether the choice is
  stable, preview, or otherwise specially labeled.
- **Catalog Refresh Result**: The explicit record of a provider-doc check,
  including reviewed sources, applied catalog delta, or a no-change outcome.
- **Prompted Input Source Set**: The resolved authored input bundle built from
  inline text, one workspace path, or an ordered array of workspace paths.
- **Review Council Independence State**: The bounded record showing whether a
  configured review council resolved to distinct effective reviewers or collapsed
  into non-independent routes.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative route-editing scenarios, operators can select
  currently documented mainstream models such as GPT-5.5, Claude Opus 4.7, and
  Gemini 3.1 Pro Preview from the bundled catalog without entering a custom model id.
- **SC-002**: Every catalog refresh run ends with either a visible bundled-model
  delta or an explicit no-change result.
- **SC-003**: In representative prompted-input scenarios, 100% of accepted single-
  file and multi-file authored inputs preserve visible source provenance and order.
- **SC-004**: In representative collapsed-council scenarios, 100% of non-distinct
  multi-review configurations are surfaced before a misleading vote result is recorded.
- **SC-005**: No invalid prompted file path or non-independent review council is
  silently ignored or converted into implicit success.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**:
  - OpenAI model catalog: https://developers.openai.com/api/docs/models
  - Anthropic model overview: https://platform.claude.com/docs/en/docs/about-claude/models/overview
  - Google Gemini model catalog: https://ai.google.dev/gemini-api/docs/models
- **Catalog Delta**: Add the currently documented mainstream route-capable model
  options missing from the bundled catalog, including GPT-5.5 and GPT-5.4 Nano,
  Claude Opus 4.7 and Sonnet 4.6, and Gemini 3.1 Pro Preview plus the currently
  documented Gemini 3.x route-capable Flash or Flash-Lite variants that belong in
  the standard picker.
- **No-Change Rationale**: Not applicable for this feature; the web review found
  a real bundled-catalog gap that must be corrected.

## Assumptions

- The bundled catalog remains curated to route-capable general-purpose models
  rather than every model family published by each provider.
- Existing multi-review flows remain bounded and continue to rely on the current
  review and adjudication lifecycle instead of introducing open-ended councils.
- Prompted file-backed authored input is limited to workspace-local text or
  Markdown-like sources in the first slice.
- The primary operator path remains session-native, while any manifest-backed or
  direct route-entry behavior remains an explicit compatibility path.
