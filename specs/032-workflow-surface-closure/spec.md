# Feature Specification: Product Unification And Surface Closure

**Feature Branch**: `032-workflow-surface-closure`  
**Created**: 2026-05-02  
**Status**: Draft  
**Input**: User description: "Feature 032 Product Unification And Surface Closure: make named workflow entry points first-class across Claude, Codex, Copilot, and Gemini guidance; unify workflow, session-native, and compatibility follow-through into one Boundline-owned product story; keep model and assistant binding inspectable without provider-specific command drift; update release docs, version bump, coverage, clippy, and fmt closeout."

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

### User Story 1 - Start And Continue Workflows Through Unified Assistant Surfaces (Priority: P1)

An operator can use Boundline from Claude, Codex, Copilot, or Gemini guidance to
discover, start, continue, and inspect a named workflow without dropping to raw
ad hoc CLI knowledge or learning a provider-specific product story.

**Why this priority**: The workflow layer is already a real entry point in the
CLI. As long as assistant guidance says "use raw workflow commands manually,"
Boundline still presents multiple overlapping products instead of one coherent
surface.

**Independent Test**: Register a representative workflow definition, then
verify that each shipped assistant surface can guide `workflow list`,
`workflow run`, `workflow status`, `workflow resume`, and `workflow inspect`
while preserving one bounded Boundline-owned follow-through story.

**Acceptance Scenarios**:

1. **Given** a workspace with `.boundline/workflows.toml`, **When** the operator
  asks an assistant how to begin, **Then** the assistant surface exposes the
  named workflow entry points as first-class Boundline commands instead of telling
  the operator to fall back to undocumented raw CLI usage.
2. **Given** an active named workflow that pauses for capture, clarification,
  governance, review, or completion follow-through, **When** the operator asks
  for status or the next step through an assistant surface, **Then** the
  surfaced guidance stays within the same workflow-aware Boundline command set and
  does not switch to a provider-specific or assistant-owned control plane.

---

### User Story 2 - Inspect Workflow Routing And Assistant Binding (Priority: P2)

An operator can inspect a named workflow and understand which Boundline route,
assistant family, and model binding are authoritative for the current step
without reconstructing that information from config files or provider-specific
documentation.

**Why this priority**: Product closure requires model and assistant choice to
stay inspectable even when the operator entered through a workflow rather than
the lower-level session commands.

**Independent Test**: Configure representative slot routes, run a workflow,
then verify that workflow-facing output and assistant guidance expose the same
effective routing, assistant binding, authority source, and workflow-owned next
action cues already expected on the session-native surfaces.

**Acceptance Scenarios**:

1. **Given** a workflow whose planning, implementation, or verification slot
  resolves to a configured route, **When** the operator runs workflow-aware
  status or inspect, **Then** the output exposes the effective route,
  assistant binding, and authority source needed to understand the current
  execution path.
2. **Given** a workflow whose active native route requires an assistant runtime
  outside the declared capability list, **When** the operator starts or resumes
  that workflow, **Then** Boundline stops explicitly with a bounded assistant-
  binding failure instead of silently switching assistant families or hiding
  the mismatch.

---

### User Story 3 - Keep One Primary Product Story Across Workflow And Compatibility Paths (Priority: P3)

An operator can tell that named workflows and direct session-native commands are
part of the same primary Boundline product, while explicit compatibility execution
remains available only as a subordinate and clearly marked route.

**Why this priority**: If workflow, session-native, and compatibility follow-up
all look equally primary, the product remains ambiguous even if the commands
work.

**Independent Test**: Compare one workflow-driven run, one direct session-native
run, and one explicit compatibility path, then verify that assistant guidance,
runtime output, and docs all present workflow plus session-native execution as
the main product story while keeping compatibility explicit and subordinate.

**Acceptance Scenarios**:

1. **Given** a named workflow run and a direct session-native run, **When** the
  operator reads status, next-step, or inspect guidance, **Then** both paths
  use the same Boundline-owned follow-through language and differ only in workflow
  identity or phase cues.
2. **Given** an explicit compatibility run, **When** the operator asks an
  assistant what to do next, **Then** the guidance keeps the compatibility
  authority explicit and does not imply that workflows or direct native runs are
  secretly compatibility-backed.

---

### User Story 4 - Ship Product Closure As 0.32.0 (Priority: P4)

A maintainer can ship `0.32.0` with runtime behavior, assistant guidance, docs,
roadmap, version metadata, changelog, and validation evidence all describing
the same final product identity: users use Boundline; workflows and direct runs are
primary surfaces; Canon stays visible but secondary inside bounded delivery.

**Why this priority**: The slice is not complete if the code changes land but
the release artifacts still describe Boundline as a collection of partially
overlapping assistant packs and side routes.

**Independent Test**: Follow the updated workflow-first operator guidance on a
representative workspace, then run the release validation suite and confirm the
version bump, docs, changelog, coverage, clippy, and formatting all align with
the shipped story.

**Acceptance Scenarios**:

1. **Given** the `0.32.0` release artifacts, **When** a maintainer follows the
  documented workflow-first path, **Then** the runtime behavior, assistant
  guidance, and release narrative all describe the same Boundline-owned product
  surface.
2. **Given** modified or newly created Rust files for this slice, **When** the
  maintainer runs the release validation suite, **Then** touched Rust coverage
  remains above 95%, clippy is clean for the slice, and formatting completes
  successfully.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when a workflow exists but no active session or recorded goal is
  yet available for the current phase?
- How does the system handle a workflow route that resolves to an assistant
  runtime outside the declared capability list?
- How does the system surface the primary workflow or session-native route
  versus any explicit compatibility route after traces already exist?
- What happens when a workflow command surface exists for chat assistants but
  Gemini remains CLI-first in the same release?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST expose named workflow discovery, start, status,
  resume, and inspect as first-class assistant guidance surfaces for the
  shipped assistant families instead of leaving workflow entry to raw CLI-only
  instructions.
- **FR-002**: System MUST keep named workflows on the same primary
  session-native product story as direct `goal -> plan -> run`
  execution rather than implying a separate workflow-owned runtime.
- **FR-003**: System MUST keep explicit compatibility execution available only
  as an explicit subordinate route and MUST NOT let workflow guidance or output
  imply that compatibility is the default execution path.
- **FR-004**: System MUST expose the effective route, assistant binding, model
  binding, and authority source needed to understand a workflow's current
  execution path on workflow-facing runtime or inspection surfaces.
- **FR-005**: System MUST preserve workflow identity, phase, execution
  condition, and bounded next-action cues alongside routing or binding context
  so operators can continue the same flow without reconstructing state from
  config files.
- **FR-006**: System MUST stop workflow-native execution explicitly when the
  active route requires an assistant runtime outside declared assistant
  capabilities instead of silently switching providers or continuing with an
  implicit fallback.
- **FR-007**: System MUST keep Gemini guidance aligned with the same Boundline-owned
  workflow and routing story even if Gemini remains CLI-first for this release.
- **FR-008**: System MUST keep assistant guidance, docs, roadmap, contributor
  guidance, and changelog aligned around the final product identity that users
  operate Boundline first and see Canon only as a bounded governed runtime inside
  that product.
- **FR-009**: System MUST include an explicit task for the `0.32.0` version
  bump across versioned release surfaces.
- **FR-010**: System MUST include an explicit task for updating every impacted
  doc plus the changelog for the release.
- **FR-011**: System MUST refresh focused validation for modified or newly
  created Rust files, keep touched-Rust coverage above 95%, resolve clippy
  issues introduced by the slice, and finish with repository formatting
  applied.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: workflow-aware assistant command surfaces for the shipped
  assistants; workflow runtime or inspect output that keeps routing, model, and
  assistant binding inspectable; explicit primary-versus-subordinate route
  guidance across workflow, direct native, and compatibility paths; `0.32.0`
  release closeout including version bump, docs, changelog, coverage, clippy,
  and formatting.
- **Out of Scope**: introducing a new provider gateway or assistant runtime;
  replacing the existing session engine; changing Canon's bounded governance
  role; building a GUI or non-CLI product surface; automatic provider
  selection based on hidden heuristics; full future-backend plugin work beyond
  keeping the current abstraction inspectable.

### Key Entities *(include if feature involves data)*

- **Workflow Assistant Surface**: the assistant-facing guidance surface for
  named workflow discovery and continuation, including supported commands,
  required context, chat-only fallback behavior, and bounded next-step rules.
- **Workflow Route Projection**: the inspectable summary of effective routing,
  model binding, assistant binding, and authority source that explains why a
  workflow step is using a particular execution path.
- **Product Identity Cue**: the wording and surfaced state that tells the
  operator whether they are on a primary Boundline workflow or session-native path
  versus an explicit subordinate compatibility path.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: Operators can complete a representative named workflow from
  discovery through bounded follow-through using any shipped assistant surface
  without needing undocumented raw CLI workflow commands.
- **SC-002**: In representative workflow runs, operators can identify the
  authoritative route, assistant binding, and model binding for the active step
  from workflow-facing output or guidance in under 2 minutes.
- **SC-003**: Workflow, direct native, and explicit compatibility follow-up
  guidance consistently indicates which path is primary versus subordinate in
  100% of release validation scenarios.
- **SC-004**: All modified or newly created Rust files in this slice finish
  the release validation suite above 95% line coverage with clean formatting
  and lint results.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Named workflows already reuse the persisted session and trace model rather
  than owning a separate runtime.
- Existing routing configuration and trace projection contain enough route and
  assistant-binding context to extend workflow surfaces without inventing a new
  provider control plane.
- Gemini remains CLI-first in this slice, but its guidance can still be aligned
  with the same workflow and routing vocabulary used by Claude, Codex, and
  Copilot.
- The smallest credible `0.32.0` improvement is to remove workflow-entry and
  product-identity ambiguity before considering broader future-backend work.
