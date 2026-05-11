# Feature Specification: Assistant Plugin Packages

**Feature Branch**: `048-assistant-plugin-packages`  
**Created**: 2026-05-11  
**Status**: Draft  
**Input**: User description: "Add host-specific assistant plugin packaging for Boundline so developers can use Boundline from chat surfaces such as Claude Code, Codex, Cursor, Copilot, and future assistant hosts without treating Boundline as just a raw CLI. Boundline remains the local delivery orchestrator for bounded engineering work. Add plugin manifests, command/skill bindings, validation, docs, README guidance, version upgrade first, and final proof of 95% coverage on created or modified Rust files plus fmt, clippy, and green tests."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install Boundline From A Chat Host (Priority: P1)

As a Boundline developer, I want a host-specific package for my assistant surface so I can discover Boundline as a session-native delivery runtime instead of guessing how raw CLI commands map into chat.

**Why this priority**: Install and discovery are the first usable outcome. Without host packages, the rest of the command and validation work cannot be exercised by a chat-surface user.

**Independent Test**: Inspect the Claude Code, Codex, Cursor, and Copilot package surfaces and confirm each supported host exposes Boundline identity, version, installable metadata or documented prompt-pack boundaries, starter prompts, and links to the shared command behavior.

**Acceptance Scenarios**:

1. **Given** a developer uses Claude Code, Codex, or Cursor, **When** they inspect the repository package folders, **Then** they can identify the matching host package, Boundline display metadata, supported capabilities, command or skill paths, starter prompts, and installation documentation.
2. **Given** a developer uses Copilot, **When** they inspect the assistant package documentation, **Then** they see prompt or command-pack support represented honestly without a fabricated universal Copilot plugin format.

---

### User Story 2 - Drive The Session-Native Loop From Chat (Priority: P2)

As a chat-surface user, I want namespaced Boundline commands to start, capture, plan, run, inspect, recover, and optionally govern a session so the assistant works from `.boundline/session.json` and the runtime's current state rather than chat memory alone.

**Why this priority**: Package metadata is only valuable if it leads users into the real Boundline loop and preserves the CLI/runtime as the source of truth.

**Independent Test**: Review command bindings and starter prompts to confirm `/boundline:start`, `/boundline:capture`, `/boundline:plan`, `/boundline:run`, `/boundline:status`, `/boundline:inspect`, `/boundline:recover`, and conditional `/boundline:govern` all guide or call the real runtime, preserve session-native state, and expose blocked, clarification-required, failed, exhausted, and terminal states explicitly.

**Acceptance Scenarios**:

1. **Given** no active Boundline session exists, **When** the user invokes `/boundline:start`, **Then** the command guides Boundline startup through the real runtime and establishes `.boundline/session.json` as the authoritative state.
2. **Given** a Boundline session is blocked, clarification-required, failed, exhausted, or terminal, **When** the user invokes status, inspect, recover, or run commands from a host package, **Then** the package instructions surface the runtime state and next action without inferring success from chat history.
3. **Given** Canon governance is not configured, **When** the user reviews available Boundline chat commands, **Then** `/boundline:govern` is documented as conditional and Canon is not made visible as normal delivery flow.

---

### User Story 3 - Prevent Host Package Drift (Priority: P3)

As a Boundline maintainer, I want automated validation for host package manifests, shared metadata, referenced paths, command coverage, and version alignment so host packages remain coherent as Boundline evolves.

**Why this priority**: Multi-host packaging becomes unsafe when version values, command declarations, or path references drift from the runtime and documentation.

**Independent Test**: Run the plugin package validation command and confirm it checks JSON parseability, required fields, referenced paths, required commands, version alignment, and prohibited positioning language.

**Acceptance Scenarios**:

1. **Given** a JSON manifest has invalid syntax or missing required metadata, **When** validation runs, **Then** validation fails and identifies the host package and field.
2. **Given** a host package references a missing command, skill, prompt, hook, or asset path, **When** validation runs, **Then** validation fails with the missing path.
3. **Given** a package version differs from the Boundline workspace version, **When** validation runs, **Then** validation fails with the mismatched version.
4. **Given** plugin metadata describes Boundline as a generic agent framework, debate CLI, prompt library, governance runtime, Canon replacement, or raw CLI wrapper, **When** validation runs, **Then** validation fails with the prohibited wording.

### Edge Cases

- A supported host has a stable JSON manifest shape while another host only supports prompt packs or command documentation; validation applies host-appropriate checks without claiming unsupported capabilities.
- Canon governance is configured downstream; `/boundline:govern` can become visible only as a conditional integration command that still reports Boundline session state first.
- The runtime reports blocked, clarification-required, failed, exhausted, or terminal state; host commands must stop or recover explicitly instead of continuing from chat-only assumptions.
- A future host package is added later; shared metadata and validation must be extensible without copying divergent command behavior.
- A host package wants additional host-specific glue; the package may add metadata or hooks, but Boundline behavior must remain shared and runtime-backed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST provide host package surfaces for Claude Code, Codex, and Cursor, plus a Copilot prompt or command-pack surface when a stable plugin manifest cannot be represented honestly.
- **FR-002**: Each supported host package MUST declare Boundline name, display name, version, description, author, homepage, repository, license, keywords, capabilities, and supported paths where the host surface supports those fields.
- **FR-003**: Host package language MUST position Boundline as a "Local delivery orchestrator for bounded engineering work", "Plan, act, verify, trace", "Turns bounded engineering goals into verified workspace changes", or "Session-native runtime for AI-assisted software delivery".
- **FR-004**: Host package language MUST NOT describe Boundline as a generic agent framework, debate CLI, prompt library, governance runtime, replacement for Canon, or merely a raw CLI wrapper.
- **FR-005**: Host packages MUST expose the required namespaced commands: `/boundline:start`, `/boundline:capture`, `/boundline:plan`, `/boundline:run`, `/boundline:status`, `/boundline:inspect`, and `/boundline:recover`.
- **FR-006**: Host packages MUST expose `/boundline:govern` only as conditional Canon-governance integration that is visible when governance is configured or explicitly documented as conditional.
- **FR-007**: Command and skill bindings MUST call or guide Boundline's real CLI/runtime and MUST NOT duplicate runtime logic in markdown.
- **FR-008**: Command and skill bindings MUST keep `.boundline/session.json` authoritative and MUST surface current runtime state and next action rather than relying only on chat history.
- **FR-009**: Command and skill bindings MUST handle blocked, clarification-required, failed, exhausted, and terminal states explicitly.
- **FR-010**: Shared command behavior and common plugin metadata MUST live in one source where possible, with host folders limited mostly to manifests, metadata, command bindings, assets, and host-specific glue.
- **FR-011**: Host packages MUST include starter prompts for turning an idea into a bounded implementation plan, fixing a failing test with Boundline, continuing an active session, and inspecting the latest trace for the next safe action.
- **FR-012**: Validation MUST check valid JSON where applicable, required metadata fields, referenced path existence, required command exposure, host-specific capability honesty, prohibited wording, and version alignment with the Boundline workspace package version.
- **FR-013**: Documentation MUST cover Claude Code, Codex, Cursor, and Copilot support boundaries, plus how chat commands map to Boundline CLI/runtime state.
- **FR-014**: README MUST include "Use Boundline from chat", "Use Boundline from CLI", and "How chat commands map to CLI/runtime state" sections.
- **FR-015**: The first implementation task MUST upgrade the Boundline version consistently for the feature release.
- **FR-016**: The final implementation task MUST prove at least 95% line coverage for every Rust source file created or modified by this slice and MUST run cargo fmt, clippy, and the test suite.

### Scope Boundaries *(mandatory)*

- **In Scope**: Host package manifests, shared metadata, command or skill bindings, conditional Copilot prompt-pack representation, validation, docs, README guidance, version upgrade, and tests for package validation.
- **Out of Scope**: Redesigning the Boundline runtime, making chat state authoritative, creating divergent host behavior, requiring users to edit manifests manually for normal operation, expanding Canon governance behavior, adding provider-routing complexity, UI work, deployment pipelines, or replacing the CLI.

### Key Entities *(include if feature involves data)*

- **Host Plugin Package**: A host-specific package folder or prompt-pack surface that declares Boundline identity, capabilities, commands, prompts, assets, and host-specific glue.
- **Shared Plugin Metadata**: Boundline-owned metadata that aligns common package fields such as version, description, author, repository, license, keywords, capabilities, and required commands.
- **Command Binding**: A host-facing declaration for a namespaced Boundline command that maps to the real session-native runtime loop and its current state.
- **Starter Prompt**: A short host-discoverable prompt that starts or resumes bounded delivery without requiring the user to edit manifests manually.
- **Package Validation Report**: Evidence that host packages have valid metadata, aligned versions, existing references, required commands, honest capabilities, and no prohibited positioning language.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can identify the right Boundline host package or prompt-pack boundary for Claude Code, Codex, Cursor, and Copilot in under five minutes using repository docs.
- **SC-002**: 100% of supported host packages declare required metadata and required session-native command surfaces where the host format supports those declarations.
- **SC-003**: 100% of JSON manifests in host package folders parse successfully during validation.
- **SC-004**: Validation fails for missing referenced paths, missing required metadata, missing required command surfaces, workspace-version drift, unsupported capability claims, or prohibited positioning wording.
- **SC-005**: Chat command documentation maps every required namespaced command to Boundline CLI/runtime state and identifies non-success states explicitly.
- **SC-006**: README and host docs state that `.boundline/session.json` remains authoritative and chat packages do not create a new runtime.
- **SC-007**: Final closeout includes fresh evidence for version upgrade, plugin validation, cargo fmt, clippy, tests, and at least 95% touched-Rust-file coverage.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**:
  - OpenAI model catalog: https://platform.openai.com/docs/models
  - Anthropic model overview and release notes: https://docs.anthropic.com/en/docs/about-claude/models/overview and https://docs.anthropic.com/release-notes/claude-apps
  - Google Gemini model catalog: https://ai.google.dev/gemini-api/docs/models
  - GitHub Copilot supported model catalog: https://docs.github.com/en/copilot/reference/ai-models/supported-models
- **Catalog Delta**: No model-entry change is required for this packaging slice because feature `047-catalog-voting-inputs` already refreshed the bundled route-capable catalog to the current mainstream Copilot, Codex/OpenAI, Claude, and Gemini entries used by Boundline. The implementation will align catalog metadata with the feature version only if the release bump requires it.
- **No-Change Rationale**: This feature does not add or change routing behavior. The current catalog already contains the model families needed by the chat packaging surfaces, and host packages only reference Boundline capabilities rather than introducing live provider discovery.

## Assumptions

- The next Boundline feature release after `0.48.0` is `0.49.0`.
- Claude Code, Codex, and Cursor can be represented by repository-local package folders with manifests or metadata plus command/path references.
- Copilot support remains a prompt or command pack unless the repository already has a stable Copilot plugin manifest convention.
- Existing assistant command packs under `assistant/` are the shared behavior source for host package bindings.
- Developers install or copy host package folders through documented host-specific steps; normal operation does not require editing manifest JSON by hand.
