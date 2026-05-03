# Feature Specification: Human-Facing Brief Ingestion

**Feature Branch**: `010-human-brief-ingestion`  
**Created**: 2026-04-27  
**Status**: Draft  
**Input**: User description: "Make Boundline human-facing by accepting chat text, one or more Markdown briefs, and chat text that references existing Markdown files as external input, so Copilot users can run bug-fix, change, or delivery work without authoring internal JSON manifests."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Start Work From Human Input (Priority: P1)

As a developer using Boundline from Copilot or the terminal, I want to start a
bounded bug-fix, change, or delivery task from plain text or one authored brief
so that I can use Boundline without learning or authoring internal manifests.

**Why this priority**: If Boundline cannot start from human-authored input, the
product leaks its internal state model and fails at the first operator touchpoint.

**Independent Test**: Start a new task from either plain text or one Markdown
brief, verify that Boundline captures the request, derives a bounded task draft, and
either plans the work or asks for one explicit clarification without requiring
the user to edit internal files.

**Acceptance Scenarios**:

1. **Given** a developer provides a plain-language task request in chat or at
  the CLI, **When** Boundline starts the task, **Then** it captures that request as
  the external brief, derives a bounded delivery task, and proceeds without
  requiring a user-authored JSON manifest.
2. **Given** a developer provides one Markdown brief, **When** Boundline starts the
  task, **Then** it treats that brief as authored input, records that source in
  inspectable state, and plans or runs from it without asking the user to
  translate it into an internal structure.
3. **Given** the provided text or brief is too vague, too broad, or missing a
  critical business constraint, **When** Boundline cannot derive a credible bounded
  task, **Then** it asks a targeted clarification or stops explicitly before
  planning instead of inventing missing structure or requesting internal field
  edits.

---

### User Story 2 - Reuse Multiple Authored Sources (Priority: P2)

As a developer working in a legacy repository, I want to provide multiple
Markdown documents or text that refers to existing repository documents so that
I can reuse authored material instead of rewriting it into one synthetic brief.

**Why this priority**: Real repository work rarely starts from one pristine
brief; value appears only when Boundline can ingest the documents humans already
have.

**Independent Test**: Start a task from multiple Markdown files and a text note
that references existing repository documents, verify that Boundline resolves them
into one bounded brief bundle with visible provenance, and verify that missing or
conflicting sources stop or clarify explicitly.

**Acceptance Scenarios**:

1. **Given** a developer provides more than one Markdown input, **When** Boundline
  starts the task, **Then** it ingests them as one bounded authored bundle,
  preserves their source order or declared precedence, and exposes the resolved
  input set for later inspection.
2. **Given** a developer provides text that names existing Markdown files in the
  repository, **When** Boundline starts the task, **Then** it resolves those files
  as additional authored input without requiring the user to copy their content
  into a second format.
3. **Given** one referenced document is missing, unreadable, duplicated in a
  conflicting way, or contradicts another authored source, **When** Boundline starts
  the task, **Then** it surfaces an explicit clarification or failure that names
  the offending source and does not silently omit or merge it.

---

### User Story 3 - Govern Human-Facing Runs Without Internal Wiring (Priority: P3)

As a developer using Copilot in a governed environment, I want to declare
high-level governance intent alongside my brief so that Boundline can use Canon
without asking me for stage mappings, runtime wiring, or other internal nouns.

**Why this priority**: Governance only adds product value if it can be requested
in human terms; otherwise Boundline still leaks its internal model even when the
input brief is authored correctly.

**Independent Test**: Start a governed task from human-authored input plus
business-level governance attributes, verify that Boundline maps them into internal
governance behavior, and confirm that blocked or approval-gated states are
reported without asking the user to edit internal configuration.

**Acceptance Scenarios**:

1. **Given** a developer provides human-authored task input plus governance
  intent such as using Canon with explicit risk, zone, and owner values,
  **When** Boundline starts the task, **Then** it applies governance internally and
  exposes the selected governed state through normal run, status, next, or
  inspect surfaces.
2. **Given** a governed task is missing one required governance attribute,
  **When** Boundline cannot continue credibly, **Then** it asks only for the missing
  external business value and does not ask the user for stage identifiers,
  packet bindings, Canon modes, or manifest fields.
3. **Given** Canon blocks the task or requires approval, **When** the developer
  continues through Boundline, **Then** Boundline remains in an explicit blocked or
  awaiting-approval state and reports the next action without asking the user to
  repair internal files by hand.

---

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- If the developer provides both plain text and one or more Markdown sources,
  Boundline must preserve which source overrides or supplements another instead of
  blending them silently.
- If the same Markdown document is provided directly and also referenced from
  text, Boundline must deduplicate it deterministically and expose the resolved
  provenance.
- If a referenced document path points outside the workspace boundary or to a
  non-text artifact that Boundline cannot credibly ingest in the first slice, Boundline
  must stop or clarify explicitly.
- If Boundline cannot derive a credible flow or bounded validation path from the
  external input within configured limits, it must stop before planning or
  execution with an explicit reason.
- If the developer resumes the same session from Copilot after one or more
  clarifications, Boundline must preserve the accepted authored inputs and ask only
  for still-missing external information.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: Boundline MUST accept external task input as direct text, one Markdown
  document, multiple Markdown documents, or direct text that refers to existing
  Markdown documents in the workspace.
- **FR-002**: Boundline MUST let developers initiate capture, planning, or run flows
  from that external input without requiring them to author `.boundline/execution.json`,
  `.boundline/fixture.json`, or any equivalent internal manifest by hand.
- **FR-003**: Boundline MUST normalize all accepted external inputs into one bounded,
  inspectable task-input record before planning or execution begins.
- **FR-004**: Boundline MUST preserve the provenance of each accepted input source,
  including whether it came from direct text, an attached Markdown document, or
  a referenced workspace document.
- **FR-005**: Boundline MUST expose the captured input summary and resolved source set
  through inspectable session or trace surfaces.
- **FR-006**: Boundline MUST resolve referenced Markdown documents only from within
  the active workspace boundary and MUST fail or clarify explicitly when a
  referenced source is missing, unreadable, unsupported, or outside that boundary.
- **FR-007**: Boundline MUST deduplicate repeated input sources deterministically and
  preserve one explicit precedence order when direct text, attached documents,
  and referenced documents overlap.
- **FR-008**: Boundline MUST ask targeted clarification questions only when the
  provided human-facing input is insufficient, contradictory, or too broad to
  derive a credible bounded task.
- **FR-009**: When clarification is required, Boundline MUST ask only for missing
  external business context and MUST NOT require the user to provide internal
  fields, stage identifiers, Canon modes, packet references, or manifest wiring
  unless the user explicitly requests an advanced path.
- **FR-010**: Boundline MUST support the same external input model for both direct
  CLI usage and assistant-driven usage such as Copilot.
- **FR-011**: Boundline MUST allow users to provide governance intent in external
  business terms, including whether governed execution is requested and any
  required risk, zone, or owner values.
- **FR-012**: Boundline MUST translate that governance intent into internal
  stage-scoped behavior without requiring the user to author stage mappings or
  runtime-selection configuration.
- **FR-013**: When governed execution becomes blocked or approval-gated, Boundline
  MUST preserve the explicit blocked or awaiting-approval state and expose the
  next user-facing action through normal status surfaces.
- **FR-014**: Boundline MUST remain compatible with existing manifest-driven flows as
  an advanced or automation path while making the human-facing input model the
  default external interaction surface.
- **FR-015**: Boundline MUST derive or request minimal clarification for the intended
  flow when the developer's request implies bug-fix, change, or delivery work but
  does not name the flow explicitly.
- **FR-016**: Boundline MUST stop before planning or execution when no credible
  bounded task can be derived from the accepted human-facing input within the
  configured limits and MUST report why.
- **FR-017**: Boundline MUST preserve accepted authored inputs across later status,
  next, inspect, and resumed run interactions so the developer does not need to
  restate the original brief after the task has started.

### Scope Boundaries *(mandatory)*

- **In Scope**: human-facing ingestion of direct text and Markdown-based authored
  briefs, bounded source resolution from the workspace, explicit clarification for
  missing context, assistant-compatible continuity, and governance intent stated
  in human business terms.
- **Out of Scope**: rich chat UI design, multimodal binary attachments,
  long-term memory across repositories, distributed orchestration, provider
  routing complexity, Canon internals exposure, and removal of the existing
  advanced manifest path for automation or tests.

### Key Entities *(include if feature involves data)*

- **External Task Input**: The user-authored task brief presented to Boundline,
  containing direct text, one or more Markdown sources, or text that refers to
  repository documents.
- **Input Source Reference**: A resolved pointer to one authored source used by
  the task, including its origin type, workspace path when applicable, and its
  precedence within the bounded brief bundle.
- **Clarification Record**: The explicit missing or conflicting business context
  that Boundline must obtain before it can derive a credible bounded task.
- **Governance Intent**: The human-facing declaration that a task should use
  governed execution, including only business-level governance attributes the
  operator can reasonably provide.
- **Derived Task Draft**: The inspectable internal task representation Boundline
  builds from external input before planning or execution proceeds.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative bug-fix, change, and delivery scenarios,
  developers can start a bounded Boundline task from plain text or authored Markdown
  input without manually authoring an internal manifest in 100% of observed runs.
- **SC-002**: In representative multi-source scenarios, Boundline resolves the
  accepted input set with explicit provenance and surfaces any missing or
  conflicting source before planning in 100% of observed runs.
- **SC-003**: In representative clarification scenarios, developers can resolve
  the missing business context and continue planning within at most two focused
  clarification turns in at least 90% of observed runs.
- **SC-004**: Developers can identify what human-authored inputs Boundline used,
  whether governance is active, and what next action is required from standard
  status or inspection output in under 60 seconds.

## Assumptions

- Boundline continues to execute one bounded active session per workspace in the
  initial slice of this feature.
- The first human-facing input slice targets text and Markdown only; binary or
  richly structured attachments remain out of scope.
- Existing manifest-driven configuration may still be used internally or by
  automation, but human operators should not need to author or understand it for
  normal Copilot-driven use.
- Developers using this feature already operate inside a repository that Boundline
  can inspect and where referenced Markdown files are available locally.
- Governance remains constrained by the currently supported Canon compatibility
  surface, but the operator-facing request model must stay at the business level
  rather than leaking stage-scoped runtime details.
