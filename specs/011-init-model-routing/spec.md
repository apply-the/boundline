# Feature Specification: Human-Friendly Init and Model Routing

**Feature Branch**: `011-init-model-routing`  
**Created**: 2026-04-28  
**Status**: Draft  
**Input**: User description: "Add a feature-complete human-friendly Boundline init flow with provider and model routing, editable global and workspace configuration precedence, assistant runtime setup for Claude Codex Copilot and Gemini CLI, and distinct defaults for planning implementation verification review voting and adjudication with operator-first usability."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Initialize a Workspace Without Hand-Written JSON (Priority: P1)

As a developer using Boundline for the first time in a repository, I want a guided
`boundline init` flow that prepares the workspace for bounded delivery work so that
I do not need to understand or hand-author internal JSON files before I can
start.

**Why this priority**: If first-run setup still requires manual JSON authoring,
Boundline remains human-hostile at the exact moment a new operator decides whether
to trust it.

**Independent Test**: In a repository that does not already contain Boundline
workspace files, run `boundline init`, choose a delivery template, confirm the
setup summary, and verify that the workspace is ready for `doctor`, `capture`,
`plan`, and `run` without the user manually editing internal configuration.

**Acceptance Scenarios**:

1. **Given** a repository with no `.boundline` workspace files, **When** the
  developer runs `boundline init` and selects a bounded template such as bug-fix,
  change, or delivery, **Then** Boundline creates the required workspace files,
  explains what was created, and leaves the repository ready for the normal
  session flow.
2. **Given** a repository that already contains Boundline workspace files,
  **When** the developer reruns `boundline init`, **Then** Boundline shows what already
  exists, offers a safe update path, and does not silently overwrite existing
  configuration.
3. **Given** the selected setup cannot continue because the repository lacks a
  required runtime or the developer rejects the proposed changes, **When** init
  reaches that point, **Then** Boundline stops explicitly, preserves the current
  repository state, and tells the developer what to do next.

---

### User Story 2 - Configure and Understand Effective Routing Defaults (Priority: P2)

As a developer installing Boundline globally or across multiple repositories, I
want editable global defaults plus workspace-local overrides for runtimes and
models so that I can keep one preferred baseline while adapting individual
repositories without editing opaque files by hand.

**Why this priority**: Usability breaks again if operators can initialize a
workspace once but cannot later understand or change the model-routing behavior
without digging through machine-shaped config.

**Independent Test**: Save one global runtime/model configuration, override a
subset of values in one workspace, inspect the effective resolved settings, and
modify them later through CLI commands without manually editing configuration
files.

**Acceptance Scenarios**:

1. **Given** a developer has global Boundline defaults and no workspace override,
  **When** Boundline resolves routing for a supported step, **Then** it uses the
  global value and shows that the value came from global configuration.
2. **Given** a workspace overrides only some routing values, **When** the
  developer inspects the effective configuration, **Then** Boundline shows the
  resolved runtime/model per supported step and identifies whether each value
  came from CLI input, workspace config, global config, or built-in defaults.
3. **Given** the developer tries to save an invalid runtime/model combination
  or a configuration that references an unavailable runtime, **When** Boundline
  validates the change, **Then** it refuses to save or apply the invalid value
  and explains the problem in user-facing terms.

---

### User Story 3 - Route Different Models for Delivery and Review Roles (Priority: P3)

As a developer relying on Boundline review and governance flows, I want different
default runtime and model assignments for planning, implementation,
verification, review roles, voting councils, and adjudication so that review
does not collapse into one homogeneous model profile.

**Why this priority**: Feature-complete routing is not credible if every step
and every reviewer role defaults to the same model choice regardless of its job.

**Independent Test**: Configure one runtime/model profile for planning and
implementation, different reviewer profiles for a voting council, and a distinct
adjudicator profile; then verify that Boundline reports and uses the correct
effective routing for each role.

**Acceptance Scenarios**:

1. **Given** a developer configures different defaults for planning,
  implementation, verification, review, and adjudication, **When** Boundline
  resolves a task through those stages, **Then** each stage uses the correct
  effective routing instead of inheriting one generic model profile.
2. **Given** a voting council contains multiple reviewer roles, **When** the
  developer configures review routing, **Then** Boundline allows reviewer roles to
  use different runtime/model defaults from each other and from the delivery
  stages.
3. **Given** the developer selects a runtime that needs repository-local prompt
  or command-pack setup, **When** `boundline init` or a later config command applies
  that choice, **Then** Boundline offers to create or refresh the repository-local
  support files and reports exactly what changed.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- If no supported runtime is available on the machine, Boundline must still let the
  operator initialize bounded workspace files, but it must clearly mark the
  missing runtime capability before execution starts.
- If the developer selects Gemini, Boundline must treat it as a CLI-only runtime in
  the first slice and must not imply that a richer native client integration is
  available.
- If a workspace override deletes or conflicts with a global value, Boundline must
  resolve the effective configuration deterministically and explain which value
  won.
- If `boundline init` encounters existing config files that differ from the chosen
  template, Boundline must preview the change and require an explicit confirmation
  before overwriting or merging.
- If a review council is configured with duplicated reviewer identities or a
  missing adjudicator route, Boundline must stop with actionable guidance rather
  than silently collapsing to one default model.
- If a repository-local assistant setup step would modify files outside the
  active repository root, Boundline must refuse that change.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: Boundline MUST provide a `boundline init` workflow that can prepare a
  repository for bounded delivery work without requiring the operator to
  hand-author internal JSON files.
- **FR-002**: Boundline MUST let the operator choose an initial bounded workspace
  template appropriate for bug-fix, change, or delivery work during init.
- **FR-003**: Boundline MUST present setup choices and explanations in user-facing
  terms rather than exposing internal nouns that only make sense to the
  implementation.
- **FR-004**: Boundline MUST support Claude, Codex, Copilot, and Gemini as runtime
  choices in the first slice, with Gemini explicitly treated as CLI-only.
- **FR-005**: Boundline MUST detect which supported runtimes are currently usable
  on the machine and MUST surface that capability information before the user
  commits to a routing choice.
- **FR-006**: If a selected runtime needs repository-local support files,
  command packs, or prompts, Boundline MUST offer to scaffold or refresh those
  files as part of init or a later setup command.
- **FR-007**: Boundline MUST persist user-scoped default configuration separately
  from workspace-local configuration so a global installation can supply
  defaults while repositories can override them.
- **FR-008**: Boundline MUST resolve effective configuration using this precedence:
  explicit CLI input first, then workspace-local config, then user-scoped global
  config, then built-in defaults.
- **FR-009**: Boundline MUST let operators inspect the effective resolved config,
  including the source of each resolved value.
- **FR-010**: Boundline MUST let operators change saved configuration later through
  CLI commands instead of requiring manual file editing.
- **FR-011**: Boundline MUST support separate runtime/model defaults for at least
  planning, implementation, verification, review, and adjudication stages.
- **FR-012**: Boundline MUST support review-role-specific routing so multiple
  reviewer roles in a voting council can intentionally use different runtime or
  model defaults from one another.
- **FR-013**: Boundline MUST allow the adjudicator to use a distinct routing profile
  from the main review council.
- **FR-014**: Boundline MUST validate provider/model assignments before saving or
  applying them and MUST refuse invalid or unavailable combinations with clear,
  actionable feedback.
- **FR-015**: Boundline MUST show a preview or summary of the files and config it is
  about to create, update, or overwrite during init and MUST require explicit
  confirmation before destructive changes.
- **FR-016**: Rerunning init on an already-configured repository MUST preserve
  existing usable settings unless the operator explicitly chooses to replace or
  merge them.
- **FR-017**: Boundline MUST keep the existing advanced manifest-driven path
  available for automation and expert use, while making guided init the normal
  human-facing entry point.
- **FR-018**: Boundline MUST emit inspectable status or trace output for init and
  routing decisions so developers can understand what setup occurred and why a
  value was chosen.
- **FR-019**: Boundline MUST stop in an explicit terminal state when setup cannot
  continue because required runtime capabilities are missing, a config value is
  invalid, or the operator declines a required destructive change.
- **FR-020**: Boundline MUST document the feature across the main README, workflow
  guides, assistant-facing docs, and review documentation so the documented user
  path matches the shipped behavior.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: guided repository initialization, runtime capability discovery,
  assistant runtime setup for supported surfaces, global and workspace config
  precedence, CLI-driven config inspection and mutation, differentiated routing
  across delivery and review roles, and user-facing documentation updates.
- **Out of Scope**: benchmarking models automatically, selecting providers based
  on hidden heuristics, cloud account provisioning, non-CLI desktop UI,
  distributed execution, secret-management systems beyond existing local
  expectations, and rich native Gemini client integration in the first slice.

### Key Entities *(include if feature involves data)*

- **Workspace Init Profile**: The user-facing setup choice that defines the
  bounded workspace template, optional assistant scaffolding, and initial local
  Boundline files for one repository.
- **Runtime Capability**: The detected availability and setup readiness of a
  supported runtime such as Claude, Codex, Copilot, or Gemini CLI on the current
  machine.
- **Model Routing Configuration**: The saved user intent that maps supported
  delivery and review steps to runtime/model defaults.
- **Config Value Source**: The origin of a resolved value, such as CLI input,
  workspace-local config, global config, or built-in default.
- **Review Role Assignment**: The routing choice for one review participant,
  including reviewer roles in a council and the adjudicator role when present.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative first-run repositories, developers can reach a
  usable Boundline workspace through `boundline init` in under 5 minutes without
  manually editing internal JSON files in at least 90% of observed runs.
- **SC-002**: In representative configuration scenarios, developers can inspect
  the effective runtime/model choice and its source for any supported step in
  under 60 seconds.
- **SC-003**: In representative review setups, developers can configure at
  least two distinct reviewer role profiles plus a separate adjudicator profile
  without manual file editing in 100% of observed runs.
- **SC-004**: In representative rerun and failure scenarios, Boundline reaches an
  explicit non-destructive success, blocked, or aborted outcome with actionable
  guidance in 100% of observed init or config flows.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Boundline continues to operate as a bounded CLI-first workflow and does not gain a
  separate GUI in this feature.
- Supported runtimes may differ in how they are invoked, but Boundline can still
  describe them through one user-facing routing model.
- Operators may install Boundline globally, but repository-local overrides remain
  necessary because team and project constraints differ.
- Existing manifest-driven automation and tests must continue working after this
  feature ships.
