# Feature Specification: Expert Pack Selection

**Feature Branch**: `053-expert-pack-selection`  
**Created**: 2026-05-14  
**Status**: Draft  
**Input**: User description: "Starting from roadmap S2, implement a bounded cross-repo slice where Boundline owns built-in expert-pack selection and runtime role recommendation while Canon provides governed expertise inputs only when available."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Select Built-In Experts Before Planning (Priority: P1)

As a Boundline operator using the primary session-native workflow, I want
Boundline to turn workspace and target cues into an explicit expert-pack
selection before planning continues, so planning and reviewer guidance begin
from declared expertise instead of ad hoc manual role picking.

**Why this priority**: If the system cannot explain which expert packs it is
using for the current workspace, later planning and review behavior becomes
harder to trust and harder to reproduce.

**Independent Test**: In a workspace with configured domain templates and
reviewer roles, run goal planning and verify that Boundline records the
selected built-in expert packs, suggested runtime roles, and provenance without
requiring Canon.

**Acceptance Scenarios**:

1. **Given** a workspace whose effective domain templates match the selected
   target, **When** Boundline assembles planning context, **Then** it produces
   a deterministic expert-pack selection outcome that names the selected expert
   packs and the runtime roles they recommend.
2. **Given** the same workspace, goal, and bounded target, **When** Boundline
   repeats expert-pack selection without relevant repo or config changes,
   **Then** the ordered selection outcome is materially the same apart from
   generated ids, timestamps, and trace-write metadata.
3. **Given** Canon is absent or unusable, **When** Boundline performs
   expert-pack selection, **Then** the local-only selection path remains
   functional and explicit about the lack of Canon input.

---

### User Story 2 - Apply Effective Overrides And Governed Expertise Inputs (Priority: P1)

As a maintainer, I want existing Boundline override precedence and optional
Canon expertise inputs to refine expert-pack selection without taking ownership
away from Boundline, so workspace-specific knowledge can sharpen choices while
keeping runtime selection local and inspectable.

**Why this priority**: The roadmap value is not just detecting a generic domain;
it is producing a bounded, inspectable selection that respects local delivery
configuration first and treats Canon as optional governed input.

**Independent Test**: Configure conflicting global, cluster, and local routing
or domain-template inputs, add an optional Canon expertise input, and verify
that Boundline keeps its existing precedence rules while surfacing which inputs
influenced or were rejected from the final selection.

**Acceptance Scenarios**:

1. **Given** a workspace where local configuration disables or narrows a
   candidate expert pack that would otherwise match, **When** selection runs,
   **Then** Boundline honors the effective local precedence and records the
   overridden candidate as rejected or suppressed with an explicit reason.
2. **Given** a Canon expertise input that supports one selected domain but does
   not match another, **When** Boundline evaluates expert-pack candidates,
   **Then** it uses the Canon input only as supporting evidence for compatible
   candidates and does not let Canon choose the runtime role directly.
3. **Given** a Canon expertise input with an unsupported contract line or
   unknown expertise kind, **When** Boundline evaluates it, **Then** it ignores
   that input safely, keeps the local-only path active, and exposes the reason
   in the selection trace.

---

### User Story 3 - Inspect Selected And Rejected Candidates (Priority: P2)

As an operator or reviewer, I want `status`, `next`, and `inspect` to show why
expert packs and runtime roles were selected or rejected, so I can audit the
selection path without reading internal code.

**Why this priority**: Inspectability is the trust boundary for a system that
claims bounded role selection rather than hidden runtime magic.

**Independent Test**: Run a session that selects expert packs for a target,
then inspect the session surfaces and verify that the selected candidates,
rejected candidates, supporting inputs, and Canon involvement are visible.

**Acceptance Scenarios**:

1. **Given** a successful expert-pack selection outcome, **When** a user
   inspects session state, **Then** the runtime surfaces selected packs,
   recommended runtime roles, and the evidence that supported them.
2. **Given** an incompatible or unroutable candidate, **When** a user inspects
   the session, **Then** the runtime shows that the candidate was rejected and
   why it could not be used.
3. **Given** a session that used no Canon input, **When** a user inspects the
   trace, **Then** the runtime distinguishes local expert-pack selection from
   optional governed expertise input.

### Edge Cases

- Multiple domain families match the same bounded target and more than one
  expert pack is credible, so Boundline must keep a deterministic order instead
  of silently picking one.
- A reviewer role is suggested by an expert pack but has no effective route,
  so the role must be rejected explicitly rather than appearing implicitly
  available.
- All matching expert packs are disabled or incompatible after effective
  precedence is applied, so Boundline must emit an explicit `none-selected`
  outcome instead of pretending a default expert exists.
- Canon input is blocked or published under an unsupported contract
  line, so Boundline must continue locally and preserve the rejection reason.
- The workspace-level selection and the target-specific selection differ, so
  the trace must say which target the current outcome belongs to.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST define a stable built-in expert-pack catalog whose
  entries include a stable identifier, supported domain families, recommended
  runtime roles, and the cues required to select that pack credibly.
- **FR-002**: Boundline MUST compute an explicit expert-pack selection outcome
  before planning continues whenever domain-template resolution is available for
  the current workspace or bounded target.
- **FR-003**: Expert-pack selection MUST consider the bounded target,
  detected domain families, effective domain-template configuration, effective
  reviewer-role routing, and optional Canon-governed expertise inputs,
  including Canon `expertise_input.expertise_kind` and
  `expertise_input.domain_families` metadata from compatible `v1` inputs.
- **FR-004**: Boundline MUST preserve deterministic ordering for selected and
  rejected expert-pack candidates when the same inputs are evaluated again.
- **FR-005**: Boundline MUST support a local-only selection path that remains
  functional when Canon expertise inputs are absent, blocked, or
  incompatible, and this slice only recognizes Canon `v1` expertise inputs of
  kind `domain-language` or `domain-model`.
- **FR-006**: Boundline MUST apply the existing effective precedence used for
  routing and domain-template configuration when resolving which expert-pack
  candidates are enabled, suppressed, or overridden.
- **FR-007**: Boundline MUST validate candidate compatibility against the
  bounded target, active domain families, required context availability, and
  routable runtime-role requirements, rejecting unsupported candidates with an
  explicit reason.
- **FR-008**: Boundline MUST persist enough expert-pack selection state and
  provenance for later planning, review, and inspection steps to reuse the same
  outcome without re-guessing why it was chosen.
- **FR-009**: Boundline MUST expose selected expert packs, rejected candidates,
  recommended runtime roles, and any Canon contribution through inspectable
  runtime surfaces such as status, next, and inspect.
- **FR-010**: Boundline MUST emit an explicit empty or `none-selected`
  selection outcome when no expert pack remains credible after validation.
- **FR-011**: Boundline MUST treat Canon expertise inputs as supporting
  evidence only and MUST NOT let Canon define runtime-role choice, model
  routing, or delivery control flow in this slice.
- **FR-012**: Boundline MUST NOT download, install, or manage external expert
  packs in this slice; external pack discovery and installation remain deferred.
- **FR-013**: Boundline MUST treat a Canon expertise input as applicable only
  when its `expertise_input.domain_families` intersects the current selected
  domain families and its publication outcome is explicitly considered usable by
  the Boundline-side contract for this slice.

### Scope Boundaries *(mandatory)*

- **In Scope**: a built-in expert-pack catalog, deterministic selection from
  workspace and target cues, existing override precedence, optional
  Canon-governed expertise inputs, explicit rejection reasons, and projection of
  selection state through runtime inspection surfaces.
- **Out of Scope**: external expert-pack discovery or installation, provider
  model catalog changes for routing behavior, council activation, voting,
  distributed execution, long-term memory, UI work, and deployment pipelines.

### Key Entities *(include if feature involves data)*

- **Expert Pack Definition**: A built-in Boundline expertise entry with a
  stable identifier, supported domain families, recommended runtime roles, and
  bounded selection cues.
- **Expert Pack Selection Outcome**: The persisted result that records selected
  packs, rejected packs, ordering, provenance, and the bounded target the
  outcome applies to.
- **Expertise Signal**: A local or Canon-derived cue such as a domain-template
  match, reviewer-role route, or governed expertise input that supports,
  suppresses, or rejects a candidate.
- **Rejected Candidate Reason**: The explicit operator-visible explanation for
  why a candidate pack or runtime role was not considered credible.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative workspaces with effective domain-template
  configuration, Boundline can produce an explicit expert-pack selection
  outcome before planning without requiring Canon.
- **SC-002**: 100% of incompatible, unroutable, or overridden expert-pack
  candidates produce a visible rejection or suppression reason rather than a
  silent drop.
- **SC-003**: A maintainer can identify the selected expert packs, recommended
  runtime roles, and Canon involvement for a session in under 5 minutes from
  runtime inspection surfaces.
- **SC-004**: Re-running expert-pack selection against unchanged workspace,
  target, and configuration inputs produces materially consistent ordered
  results apart from generated ids, timestamps, and trace-write metadata.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: OpenAI Models documentation at
  `https://developers.openai.com/api/docs/models`, Anthropic Models Overview at
  `https://platform.claude.com/docs/en/docs/about-claude/models`, and Google
  Gemini Models documentation at `https://ai.google.dev/gemini-api/docs/models`
  on 2026-05-14.
- **Catalog Delta**: No bundled catalog changes are required for this feature
  slice based on the spec-time audit.
- **No-Change Rationale**: The bundled catalog in
  `assistant/catalog/model-catalog.toml`, refreshed on 2026-05-13, already
  includes the currently documented coding-relevant model families needed by
  this slice: OpenAI `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.4-nano`;
  the Boundline adapter IDs for the current Anthropic families `opus-4.7`,
  `sonnet-4.6`, and `haiku-4.5`; and Google `gemini-2.5-pro`,
  `gemini-2.5-flash`, `gemini-2.5-flash-lite`, `gemini-3.1-pro-preview`, and
  `gemini-3.1-flash-lite`. Newly documented audio, media, and research-specific
  models do not change Boundline expert-pack selection semantics for this
  bounded slice.

## Assumptions

- The first slice ships only built-in expert-pack definitions; external pack
  import, installation, and marketplace behavior remain deferred.
- Existing domain-template detection and effective reviewer-role routing remain
  the authoritative local inputs for expert-pack selection.
- Canon-governed expertise inputs may be absent, blocked, or
  incompatible without invalidating Boundline's local selection path, and this
  slice only recognizes those inputs through compatible Canon publication and
  lineage surfaces rather than generic memory streams.
- When Canon input is present in this slice, Boundline treats Canon `v1`
  `domain-language` and `domain-model` expertise inputs as the only supported
  initial compatibility target.
- Expert-pack selection informs planning and inspection surfaces but does not
  itself choose concrete provider models or expand into council behavior.
