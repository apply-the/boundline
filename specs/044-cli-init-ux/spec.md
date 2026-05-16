# Feature Specification: Guided CLI UX And Clearer Messaging

**Feature Branch**: `044-cli-init-ux`  
**Created**: 2026-05-07  
**Status**: Draft  
**Input**: User description: "Improve the Boundline CLI onboarding and interactive init UX so prompts are more discoverable and more human-readable, especially model route selection, with better examples, clearer recovery messages, and more expressive output aligned with CLI UX best practices. Operators should not need to guess where valid `SLOT=RUNTIME:MODEL` values come from."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Finish Init Without Leaving The Terminal (Priority: P1)

An operator in a new repository can complete guided `boundline init` without
opening external documentation or memorizing route syntax, because each prompt
explains defaults, optional fields, valid choices, and where the chosen values
can be inspected or changed later.

**Why this priority**: `init` is the first value moment for a new operator. If
the first-run path depends on tribal knowledge such as route slot names or
model syntax, Boundline feels opaque before any bounded work begins.

**Independent Test**: Run guided `boundline init` in a fresh Git
repository with a user who has not read the docs. The user must be able to
finish initialization using only the interactive prompt text and resulting
summary output.

**Acceptance Scenarios**:

1. **Given** a fresh repository and an operator starts guided init, **When**
   the command asks for assistant and route choices, **Then** it lists the
   supported assistant values, supported route slots, that route input is
   optional when defaults exist, and at least one valid route example.
2. **Given** an operator leaves the route answer blank, **When** init
   completes, **Then** the closing summary states which routes were seeded by
   default, why blank input was accepted, and where the operator can inspect or
   override the effective routes afterward.
3. **Given** an operator uses the help surface instead of guided mode, **When**
   they inspect the init help output, **Then** the route syntax, valid examples,
   and blank-input/default behavior are consistent with the guided prompt.

---

### User Story 2 - Recover From Bad Input Quickly (Priority: P2)

An operator who mistypes an assistant, route slot, or route format receives a
human-readable explanation and immediate recovery guidance instead of a terse
parser error, so a first-run attempt does not stall on unclear syntax or hidden
constraints.

**Why this priority**: Better onboarding fails if one typo turns the CLI back
into trial-and-error. Assisted recovery is the fastest way to keep people in
the flow after a mistake.

**Independent Test**: Exercise guided and flag-based init with malformed route
values, unsupported assistant values, unavailable defaults, and overwrite
conflicts. Each case must fail explicitly with an actionable correction and no
silent mutation of workspace state.

**Acceptance Scenarios**:

1. **Given** an operator enters malformed route input, **When** validation
   fails, **Then** the error names the offending value, explains the expected
   shape in plain language, and shows at least one valid example.
2. **Given** an operator enters an unsupported assistant or a route that cannot
   be satisfied credibly, **When** init validates or seeds defaults, **Then**
   the output distinguishes unsupported input from unavailable local capability
   and gives a precise next action or retry command.
3. **Given** guided init runs in a non-interactive environment or on a
   workspace that would require overwrite confirmation, **When** interaction
   cannot continue safely, **Then** Boundline stops explicitly with the reason
   and the non-interactive command or flag combination needed to proceed.

---

### User Story 3 - Read The Outcome At A Glance (Priority: P3)

An operator can scan init and doctor output quickly because success, warnings,
defaults, and next actions are grouped semantically, and richer presentation is
used only when the terminal supports it.

**Why this priority**: Discoverability is not only about prompts. Operators
also need confidence that they understood the result of a command without
parsing dense walls of text.

**Independent Test**: Compare interactive TTY and plain-text or redirected runs
of the same init and doctor flows. Both modes must communicate the same meaning
clearly while preserving scriptability and automation behavior.

**Acceptance Scenarios**:

1. **Given** an interactive terminal with rich display support, **When** init
   or doctor reports onboarding or readiness results, **Then** success,
   warnings, defaults, and next steps appear in clearly separated sections with
   semantic emphasis.
2. **Given** plain output, redirection, or a terminal without rich formatting
   support, **When** the same commands run, **Then** the output falls back to
   readable plain text with identical meaning and no hidden instructions.
3. **Given** a successful or failed first-run command, **When** the output
   ends, **Then** the operator can identify what happened and the next concrete
   action from one short closing summary.

### Edge Cases

- What happens when the operator presses Enter on optional model routes after
  selecting one assistant and expects defaults to be applied?
- How does the system behave when only some route slots are entered explicitly
  and the remaining slots must still be seeded from defaults?
- What happens when the operator pastes comma-separated routes with extra
  whitespace, duplicate assignments, or unknown slots?
- How does the system surface route guidance when a selected assistant has
  credible defaults but the operator wants to override only one slot?
- What happens when guided init runs in CI, in a non-TTY shell, or in a TTY
  without color support?
- How are existing workspace files and overwrite consequences explained before
  guided init mutates `.boundline` state?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST let a first-run operator complete guided init without
  consulting external documentation for supported assistant values, route slot
  names, or the meaning of blank optional route input.
- **FR-002**: System MUST make assistant selection, route selection, and
  blank-input behavior explicit at the moment each guided answer is requested.
- **FR-003**: System MUST surface the supported route slots and at least one
  valid `SLOT=RUNTIME:MODEL` example whenever route input is requested
  interactively or explained on help and recovery surfaces.
- **FR-004**: System MUST let operators skip route entry when credible defaults
  can be seeded and MUST say what will happen if the operator leaves the field
  blank.
- **FR-005**: System MUST summarize the effective assistant selection,
  auto-seeded routes, explicit overrides, and next commands immediately after
  init succeeds.
- **FR-006**: System MUST preserve a non-interactive init path that supports
  automation without requiring guided prompts or rich terminal presentation.
- **FR-007**: System MUST validate user-facing input before mutating workspace
  configuration and MUST report invalid assistant values, invalid slot names,
  malformed route shapes, unavailable defaults, and overwrite-state problems in
  human-readable language.
- **FR-008**: System MUST distinguish syntax errors, unsupported values,
  unavailable local capabilities, and workspace readiness or overwrite
  conflicts in separate actionable messages.
- **FR-009**: System MUST provide assisted recovery for user-facing failures
  with at least one of: a valid example, a closest supported value, or an exact
  retry command.
- **FR-010**: System MUST expose where operators can inspect or edit effective
  model routes after initialization without requiring repository code
  inspection.
- **FR-011**: System MUST present onboarding, warning, success, and next-step
  information in semantically grouped output on primary first-run surfaces,
  including init and doctor.
- **FR-012**: System MUST adapt decorative or color emphasis to terminal
  capabilities and MUST preserve full meaning when rich formatting is
  unavailable.
- **FR-013**: System MUST keep standard output, standard error, and exit
  behavior usable for scripting and automation.
- **FR-014**: System MUST keep help and guided surfaces consistent so examples,
  defaults, and recovery guidance do not contradict one another.
- **FR-015**: System MUST clarify partial route overrides by showing which
  slots remain auto-seeded and which are explicitly operator-owned.
- **FR-016**: System MUST explain overwrite consequences before replacing
  existing Boundline workspace files during guided init.

### Scope Boundaries *(mandatory)*

- **In Scope**: guided init discoverability; human-readable validation and
  recovery messages; consistent route syntax examples across prompt, help, and
  post-init summary surfaces; capability-aware semantic formatting for primary
  first-run output; clarity improvements on doctor and workspace-not-ready
  messaging; docs alignment where needed to mirror the CLI guidance.
- **Out of Scope**: command renames; a full-screen TUI; arbitrary theming or
  branding systems; expansion of supported assistant families; changes to the
  underlying default-route selection policy; long-running animated progress
  systems outside the first-run path; release-channel packaging work.

### Key Entities *(include if feature involves data)*

- **Guided Init Prompt Surface**: The ordered set of prompt text, defaults,
  examples, and inline explanations shown while Boundline collects init inputs
  from a human operator.
- **Recovery Guidance Message**: A user-facing failure message that explains
  what failed, why it failed, and the exact next action to recover.
- **Effective Route Summary**: The post-init summary that shows which routes are
  auto-seeded, which were explicitly provided, and where the resulting route
  state can be inspected or changed later.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At least 90% of representative first-run operators can complete
  guided init in a fresh repository without opening docs and without more than
  one retry on route entry.
- **SC-002**: 100% of invalid assistant and route input validation cases return
  an error that names the bad input, explains why it failed, and provides at
  least one corrective action.
- **SC-003**: 100% of successful guided init runs end with a summary that
  identifies the active assistant selection, the effective route outcome, and
  the next command to run.
- **SC-004**: Rich and plain-text terminal modes communicate the same success,
  warning, and next-step content for representative init and doctor scenarios.

## Assumptions

- `boundline init` remains the primary first-run onboarding surface for a new
  workspace.
- The supported assistant families and default-route catalog already exist for
  this slice; the feature improves discoverability and messaging rather than
  expanding provider breadth.
- Non-interactive automation is a hard requirement and cannot regress while the
  guided UX improves.
- Terminal capabilities vary across local shells, CI, and redirected output, so
  richer presentation must degrade cleanly to plain text.
- Secondary docs can mirror the improved guidance, but the CLI itself must be
  sufficient for a first-run operator to finish initialization.

