# Feature Specification: Context Selection Hardening

**Feature Branch**: `040-context-selection-hardening`  
**Created**: 2026-05-03  
**Status**: Draft  
**Input**: User description: "Procedi con speckit specify, plan, tasks e implements per l'ultima feature. Non fare slicing, voglio feature complete. Al solito, un task per fare bump della versione e uno per aggiornare tutte le docs impattate e il changelog. Infine coverage dei file rust modificati o creati e soluzione di problemi su clippy e esecuzione di cargo fmt. Assicurati che la coverage dei file rust modificati sia sopra il 95%. aggiorna la roadmap togliendo quanto fatto. Infine stampa un commit message descrittivo. Ti chiedo anche di migliorare le docs considerando questo feedback: Synod = runtime operativo che decide, esegue, valida, tiene stato; Canon = runtime governato che registra, struttura, approva, pubblica; README troppo denso; servono quick path brutale e advanced architecture separati."

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

### User Story 1 - Select Bounded Context From Explicit Evidence (Priority: P1)

An operator can capture a bounded goal and have Boundline build the planning
context from explicit evidence such as failing tests, validation output,
authored brief references, recent workspace mutations, workflow-owned targets,
and reusable Canon artifacts instead of relying mainly on keyword scoring over
repository paths.

**Why this priority**: This is the behavioral core of the feature. Without it,
Boundline still looks inspectable on the surface while choosing context from
weak heuristics that are hard to trust.

**Independent Test**: Run the session-native `start -> capture -> plan` path on
representative workspaces that contain failing-test evidence, authored brief
input, or recent mutation evidence, and verify that the resulting context pack
selects files and artifacts from those evidence anchors with explicit reasons.

**Acceptance Scenarios**:

1. **Given** a workspace whose latest validation output points to a failing test
  target and one related source file, **When** the operator runs `plan`,
  **Then** Boundline selects the test target and related source input because
  of that evidence and records why each one was chosen.
2. **Given** a captured goal whose wording is generic but whose authored brief
  names specific files or modules, **When** planning runs, **Then** Boundline
  uses the authored brief references as primary context evidence instead of
  falling back to broad path matching.

---

### User Story 2 - Inspect Why Each Input Was Selected (Priority: P2)

An operator can use the normal Boundline surfaces and see, at file or artifact
level, why a context input was selected, which evidence anchored it, whether it
is primary, and why the current context is credible, stale, or insufficient.

**Why this priority**: Context hardening only improves operator trust if the
selection story is visible on the same `plan`, `run`, `status`, `next`, and
`inspect` surfaces already used to understand delivery state.

**Independent Test**: Generate a plan on the primary session-native path and
verify that `status`, `next`, and `inspect` expose file-level provenance labels,
evidence anchors, and stop reasons without requiring the operator to read the
raw trace JSON.

**Acceptance Scenarios**:

1. **Given** a confirmed goal plan with multiple selected context inputs,
  **When** the operator runs `status` or `inspect`, **Then** the output names
  the selected inputs together with the reason each one entered the context.
2. **Given** an explicit compatibility follow-up trace, **When** the operator
  runs `inspect`, **Then** Boundline preserves compatibility authority while
  reusing the same context-provenance vocabulary used on the primary path.

---

### User Story 3 - Stop Explicitly When Context Is Not Credible (Priority: P3)

An operator gets an explicit non-success planning outcome when the only support
for a context pack is ambient repository shape, contradictory evidence, or an
unjustified cross-workspace jump, instead of a plan that silently pretends the
context is good enough.

**Why this priority**: The feature is incomplete if Boundline can explain good
context but still degrades silently when the evidence is weak or conflicting.

**Independent Test**: Use representative workspaces where only weak keyword
matches exist, where evidence conflicts, or where a cluster member file would
otherwise be selected without a causal anchor, then verify that planning stops
explicitly with bounded recovery guidance.

**Acceptance Scenarios**:

1. **Given** a workspace where the goal text loosely matches several files but
  no authored, validation, workflow, trace, or Canon evidence supports any of
  them, **When** the operator runs `plan`, **Then** Boundline stops with an
  insufficient-context result rather than claiming a credible selection.
2. **Given** a clustered workspace where a file in another member repository is
  not referenced by any direct evidence, **When** planning runs, **Then** that
  file is excluded from the active context and the operator can see why it was
  not admitted.

---

### User Story 4 - Ship A Coherent 0.40.0 Surface (Priority: P4)

A maintainer can ship `0.40.0` with the hardened context-selection behavior,
version bump, roadmap cleanup, changelog, coverage evidence, and documentation
updates all telling the same story, including a clearer README split between a
brutal quick path and deeper architecture guidance.

**Why this priority**: The runtime change is only feature-complete when the
release surface, docs, and validation evidence match the new operating model.

**Independent Test**: Follow the updated README and documentation on a
representative workspace, run the release validation commands, and confirm that
the version, roadmap, changelog, docs, and touched Rust coverage all align with
the shipped feature.

**Acceptance Scenarios**:

1. **Given** the `0.40.0` release artifacts, **When** a maintainer follows the
  README quick path, **Then** they can reach the first successful bounded
  session flow without reading the advanced architecture section.
2. **Given** modified or newly created Rust files for this feature, **When**
  the maintainer runs release validation, **Then** touched Rust coverage stays
  above 95%, clippy issues introduced by the slice are resolved, and formatting
  completes successfully.

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when several files share one evidence anchor but Boundline still
  needs to keep the context pack bounded?
- How does the system handle evidence that was once valid but is now stale,
  contradicted by newer validation output, or points outside the active
  workspace scope?
- How does the system surface the primary session-native route versus any
  explicit compatibility route when both can project context provenance?
- What happens when an authored brief mentions a file that no longer exists or
  when a cluster member path is implied but not directly evidenced?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST build the planning context pack from explicit bounded
  evidence classes such as authored brief references, validation output,
  compiler or linter paths, failing test targets, recent workspace mutations,
  workflow-owned targets, recent traces, and reusable Canon artifacts when
  those inputs are available.
- **FR-002**: System MUST require each selected context input to carry an
  explicit rationale and evidence anchor so the operator can tell why that
  input entered the pack.
- **FR-003**: System MUST NOT treat keyword similarity or broad path scoring as
  sufficient by itself to make a context pack credible.
- **FR-004**: System MUST preserve the selected-input story through the active
  goal plan, authoritative session projection, and inspectable trace output.
- **FR-005**: System MUST surface file-level or artifact-level context
  provenance through `plan`, `run`, `status`, `next`, and `inspect` on the
  primary session-native path.
- **FR-006**: System MUST preserve explicit compatibility and cluster authority
  when projecting context provenance outside the primary session-native path.
- **FR-007**: System MUST stop planning explicitly when the available evidence
  is insufficient, stale, contradictory, or too broad to justify a bounded
  context selection.
- **FR-008**: System MUST keep cross-workspace and cluster-member context
  selection bounded by direct evidence instead of allowing implicit repository
  sprawl.
- **FR-009**: System MUST surface one explicit recovery cue when the current
  context pack is stale or insufficient rather than silently continuing with an
  ambient guess.
- **FR-010**: System MUST include unit, integration, and output-projection
  validation for explicit-evidence selection, inspectable provenance, and
  non-credible planning outcomes.
- **FR-011**: System MUST include an explicit task for the `0.40.0` version
  bump across versioned release surfaces.
- **FR-012**: System MUST include an explicit task for updating all impacted
  docs, the roadmap, assistant guidance if affected, and the changelog, with
  the README reorganized into a quick path and an advanced architecture layer.
- **FR-013**: System MUST finish with touched Rust coverage above 95%, clean
  formatting, and clippy free of slice-introduced issues.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: causal context selection from explicit evidence; file-level or
  artifact-level provenance; explicit insufficient or stale stop handling;
  clustered boundary enforcement for context selection; `0.40.0` release
  closeout including version bump, docs, roadmap, changelog, coverage, clippy,
  and formatting.
- **Out of Scope**: checkpoint and rewind; a full execution-engine refactor;
  generalized semantic indexing; background repository indexing; new UI
  surfaces; distributed execution; Canon-owned planning; remote persistence
  changes.

### Key Entities *(include if feature involves data)*

- **Context Evidence Anchor**: the specific piece of bounded evidence that
  justifies selecting a file or artifact, such as a failing test path, recent
  validation record, authored brief reference, workflow target, or reusable
  Canon artifact.
- **Context Input**: one selected file, symbol, brief reference, trace item, or
  governed artifact together with its rationale, source, evidence anchor, and
  whether it is a primary planning input.
- **Context Pack**: the bounded collection of selected inputs, credibility
  state, summary, selected targets, and any stop or staleness reason for one
  plan revision.
- **Context Projection**: the operator-facing rendering of the context pack
  across `plan`, `run`, `status`, `next`, and `inspect`, including provenance
  and recovery guidance.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative planning runs, operators can identify the
  selected primary context inputs and why they were selected from standard
  Boundline output in under 2 minutes.
- **SC-002**: 100% of representative planning runs end in an explicit credible,
  stale, or insufficient context state rather than silently proceeding on broad
  repository heuristics alone.
- **SC-003**: In representative first-run documentation checks, a new operator
  can reach the first successful bounded session flow from the README quick path
  without reading the advanced architecture section.
- **SC-004**: All modified or newly created Rust files in this slice finish the
  release validation suite above 95% line coverage with clean formatting and
  lint results.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- The primary delivery path remains the session-native CLI flow, and explicit
  compatibility execution remains subordinate and opt-in.
- Existing goal-plan, session, trace, and CLI projection models can be
  extended to carry richer provenance without introducing a second planning
  runtime.
- The release for this feature will be `0.40.0`, matching the sequential
  feature number assigned by the repository's Spec Kit workflow.
- README restructuring can improve information layering without changing the
  core CLI command surface or the Boundline-versus-Canon responsibility split.
