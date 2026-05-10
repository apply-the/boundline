# Feature Specification: Guided Init TUI and Runtime Catalog

**Feature Branch**: `046-guided-init-tui`  
**Created**: 2026-05-09  
**Status**: Draft  
**Input**: User description: "Transform `boundline init` into a guided terminal UX with select and multi-select prompts, per-slot model route editing, visible defaults, bundled runtime/model catalog metadata, custom model fallback, non-interactive automation flags, CLI spinners for time-consuming operations, and global `--version` support. The interactive flow must keep users in the current step on validation errors, show a final summary before writing config, and avoid raw escape sequences or fragile comma-separated freeform input."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bootstrap a Workspace Without Memorizing Syntax (Priority: P1)

An operator can run `boundline init` in a terminal and complete workspace
bootstrap through guided selections, defaults, and per-slot editing instead of
typing fragile comma-separated or `SLOT=RUNTIME:MODEL` strings manually.

**Why this priority**: `init` is the first delivery-facing experience for a new
workspace. If the bootstrap path is brittle, developers fail before Boundline
can demonstrate value on any real task.

**Independent Test**: In an empty workspace, run guided `boundline init`,
configure Canon approval, assistant surfaces, and model routes using only
navigation keys, selection keys, and confirmation prompts, then confirm the
summary and verify that the expected Boundline files are written.

**Acceptance Scenarios**:

1. **Given** an empty workspace and an operator starting `boundline init`,
   **When** the operator moves through guided prompts for Canon approval,
   assistant surfaces, and route defaults, **Then** Boundline shows visible
   defaults, allows slot-by-slot route editing, and writes the resulting
   configuration without requiring manual route syntax entry.
2. **Given** an operator editing one route slot during guided init, **When**
   the operator changes that slot and keeps other defaults, **Then** Boundline
   preserves the untouched defaults and summarizes the final route table before
   writing files.
3. **Given** an operator enters an invalid custom model identifier or attempts
   to confirm an invalid selection, **When** validation fails, **Then**
   Boundline keeps the operator in the current step, shows a contextual error,
   and does not abort the full init session.

---

### User Story 2 - Choose Routes From an Honest Catalog (Priority: P2)

An operator can understand what runtime and model choices Boundline knows about,
where those choices come from, and when a choice is custom rather than bundled,
so routing decisions are inspectable instead of guessed.

**Why this priority**: Guided prompts still fail if the catalog is opaque.
Developers need to know the default routes, the bundled choices, and when they
are stepping outside verified presets.

**Independent Test**: In guided init, select assistant surfaces, inspect the
proposed route table, open at least one slot editor, choose a bundled model for
one slot and a custom model for another, then verify the summary marks the
catalog source and the custom route status clearly.

**Acceptance Scenarios**:

1. **Given** selected assistant surfaces with bundled route defaults,
   **When** the operator reaches the route review step, **Then** Boundline
   shows the proposed route table, identifies the bundled catalog as the source,
   and lets the operator accept defaults, edit one slot, or clear routes.
2. **Given** a route slot whose desired model is not present in the bundled
   catalog, **When** the operator chooses a custom model identifier, **Then**
   Boundline accepts the custom value, marks it as unverified in the summary,
   and keeps the runtime choice explicit.
3. **Given** no selected assistant provides a bundled default for a required
   slot, **When** the operator reviews routes, **Then** Boundline surfaces that
   slot as unset and asks for an explicit operator decision instead of silently
   inventing a route.

---

### User Story 3 - Automate and Observe Long Init Work (Priority: P3)

An operator can use `boundline init` both interactively and in automation, with
clear version output and visible progress for time-consuming work, so bootstrap
remains credible in terminals, scripts, and CI-like flows.

**Why this priority**: A better interactive wizard is incomplete if automation
breaks or long-running work appears frozen. Progress and non-interactive parity
make the bootstrap surface operationally trustworthy.

**Independent Test**: Run `boundline --version`, then execute a non-interactive
`boundline init` with explicit flags for approval, assistants, and routes,
using a scenario that performs enough file or asset work to trigger progress
feedback. Verify the command completes without prompts and without emitting raw
interactive control sequences to a non-interactive output stream.

**Acceptance Scenarios**:

1. **Given** an operator invoking `boundline --version` or `boundline -V`,
   **When** the command is executed, **Then** Boundline prints the current
   version and exits successfully without requiring a subcommand.
2. **Given** an operator invoking `boundline init --non-interactive` with all
   required flags, **When** the command runs in a non-interactive environment,
   **Then** Boundline applies the same routing and approval decisions that the
   guided flow would produce and finishes without interactive prompts.
3. **Given** an init operation includes a time-consuming step, **When** the
   command runs in an interactive terminal, **Then** Boundline shows visible
   progress feedback until the step completes, and **When** the same command
   runs in a non-interactive context, **Then** Boundline emits stable text
   progress without raw escape sequences or spinner artifacts.

### Edge Cases

- What happens when `boundline init` starts on a terminal that does not support
  interactive navigation or when stdin/stdout are not attached to a TTY?
- What happens when the operator cancels during route editing or summary
  confirmation after Boundline has gathered answers but before any files are
  written?
- How does the wizard behave when no assistant surfaces are selected and no
  bundled defaults are available for required route slots?
- How does the system handle a custom model identifier that is syntactically
  malformed, blank, or duplicated across conflicting slot edits?
- What happens when a write, copy, or generated-asset step fails after the
  summary is confirmed?
- How does progress feedback avoid corrupting terminal input, copied logs, or
  redirected output streams?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support `boundline --version` and `boundline -V` as
  successful top-level version surfaces.
- **FR-002**: System MUST provide a guided interactive `init` mode that uses a
  terminal interaction layer with explicit selection, multi-selection,
  confirmation, defaults, cancellation, line editing, and validation behavior.
- **FR-003**: System MUST replace freeform comma-separated assistant selection
  and one-line route entry in guided init with structured steps that let the
  operator review and edit one route slot at a time.
- **FR-004**: System MUST show the default Canon approval mode, default
  assistant selections, and proposed route defaults before the operator commits
  the final configuration.
- **FR-005**: System MUST derive proposed route defaults from a bundled runtime
  and model catalog whose source is made explicit to the operator.
- **FR-006**: System MUST allow an operator to override a bundled route with a
  custom model identifier while marking the custom value as unverified until
  runtime.
- **FR-007**: System MUST keep the operator in the current step when validation
  fails and MUST surface a contextual correction message without aborting the
  overall init flow.
- **FR-008**: System MUST present a final summary of approval mode, assistant
  surfaces, route selections, and custom-route warnings before any config or
  assistant assets are written.
- **FR-009**: System MUST write no new or modified init outputs when the
  operator cancels before final confirmation.
- **FR-010**: System MUST provide a non-interactive init path that accepts
  explicit CLI flags for approval mode, assistant surfaces, and route
  selections and maps those inputs to the same stored configuration model used
  by guided init.
- **FR-011**: System MUST provide progress feedback for time-consuming init
  operations and MUST keep that feedback compatible with both interactive
  terminals and redirected or non-interactive output.
- **FR-012**: System MUST prevent raw terminal escape sequences from appearing
  in the user-visible interactive experience during guided init input or
  progress display.
- **FR-013**: System MUST surface write failures, validation failures, and
  canceled runs as explicit terminal outcomes with actionable next steps.
- **FR-014**: System MUST detect when guided terminal interaction is not
  available and MUST either complete through explicit non-interactive inputs or
  fail with guidance to rerun `init` with `--non-interactive`.
- **FR-015**: System MUST show all four route slots (`planning`,
  `implementation`, `verification`, `review`) during route review, including
  slots that are currently unset.
- **FR-016**: System MUST scaffold or refresh the existing repository-managed
  assistant packs for the selected assistant surfaces using explicit created,
  updated, or unchanged reporting grouped by surface.

### Scope Boundaries *(mandatory)*

- **In Scope**: guided terminal bootstrap for `boundline init`; visible
  defaults and slot-by-slot route editing; bundled runtime/model catalog
  metadata; custom model fallback with clear warnings; final configuration
  summary; non-interactive `init` flag parity; CLI progress feedback for
  time-consuming init work; top-level version output; explicit assistant-pack
  scaffolding status for selected surfaces.
- **Out of Scope**: remote provider model discovery; marketplace or network
  integrations for catalog refresh; redesign of everyday session-native
  execution commands outside `init`; host-chat runtime behavior; new governance
  modes beyond current approval semantics; broad terminal theming or full-screen
  dashboard work.

### Key Entities *(include if feature involves data)*

- **Init Interaction State**: The current guided step, visible defaults,
  validation state, and cancellation boundary for one `boundline init` run.
- **Bundled Model Catalog Entry**: A curated runtime/model option with a known
  display label, stable identifier, source metadata, and default-eligibility
  information used during route proposal.
- **Route Draft**: The in-progress selection for one route slot, including the
  chosen runtime, model identifier, whether the value is bundled or custom, and
  whether the slot is currently complete.
- **Init Summary**: The operator-facing review of approval mode, selected
  assistants, route decisions, warnings, and the pending write action.
- **Progress Activity**: A bounded long-running init step whose visible status
  remains attached to the terminal until the step succeeds, fails, or is
  canceled.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative interactive bootstrap scenarios, 100% of
  successful init runs complete without requiring the operator to type
  comma-separated assistant lists or `SLOT=RUNTIME:MODEL` route syntax.
- **SC-002**: In representative invalid-input scenarios, 100% of validation
  failures keep the operator in the current guided step and allow correction
  without restarting `boundline init`.
- **SC-003**: In representative route-review scenarios, 100% of bundled and
  custom route selections show whether the chosen value came from the bundled
  catalog or from an unverified custom identifier before config is written.
- **SC-004**: `boundline --version` and `boundline -V` succeed in 100% of
  validation runs.
- **SC-005**: In representative long-running init scenarios, every step lasting
  longer than one second shows progress feedback in interactive terminals and
  emits no raw spinner or cursor-control artifacts in non-interactive output.

## Assumptions

- The first slice uses a bundled, repository-managed model catalog rather than
  remote discovery, and the catalog can be revised in later releases.
- Interactive init remains a terminal surface rather than a full-screen TUI or
  host-editor UI.
- The existing Boundline config and assistant asset outputs remain the
  authoritative persisted result of `init`.
- The bundled catalog is curated in-repository and shipped with the CLI; it is
  not refreshed from providers at init time.
- Non-interactive automation must remain available for scripts, CI, and tests,
  even if guided init becomes the default human experience.
- Time-consuming init work is bounded to discrete steps that can expose visible
  progress without changing Boundline's broader execution model.
