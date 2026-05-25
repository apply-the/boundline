# Research: Native Canon CLI Surface

**Feature**: 042-native-canon-cli
**Date**: 2026-05-05

## R1: Canon Capability Detection — How To Verify The Real Governance Surface

**Decision**: Extend the existing `query_canon_capabilities()` call and
`CanonCapabilitySnapshot` to serve as the authority for install diagnostics and
runtime gating, rather than introducing a separate discovery mechanism.

**Rationale**: The codebase already invokes `canon governance capabilities --json`
and parses the response into `CanonCapabilitySnapshot` with `supported_modes`,
`operations`, and schema-level fields.  The existing `evaluate_canon_install()`
only checks version via `canon --version`.  The plan adds a second diagnostic
step that calls `query_canon_capabilities()` after version passes and verifies
that: (a) `governance start` and `governance refresh` are in the operations
list, (b) all 15 canonical modes are in `supported_modes`, and (c) the
capability snapshot is cached for the workspace session.

**Alternatives considered**:
- **Probe each subcommand individually** (`canon governance start --help`): Too
  slow, fragile across Canon versions, and not machine-readable.
- **Ship a pinned Canon binary only**: Does not address PATH-shadowed binaries
  or operator-managed Canon installs, which the spec explicitly requires.

**Key types**:
- `CanonCapabilitySnapshot` at `src/domain/governance.rs:398-412` — already has
  `supported_modes: Vec<CanonMode>` and `operations: Vec<String>`
- `query_canon_capabilities()` at `src/adapters/governance_runtime.rs:402-431`
- `evaluate_canon_install()` at `src/domain/distribution.rs:79-88`
- `CanonInstallStatus` at `src/domain/distribution.rs:52-61`

## R2: Workspace Resolution Strategy

**Decision**: Implement a shared `resolve_workspace()` function in `src/cli/`
that searches upward for `.boundline/`, then git root, then CWD, replacing the
duplicated per-module pattern.

**Rationale**: The spec requires a specific resolution order:
1. Explicit `--workspace <path>` when supplied
2. Upward search from CWD for an existing `.boundline/` directory; use its parent
3. Nearest git root
4. Current working directory

The existing `resolve_workspace()` implementations in `src/cli/session.rs` and
`src/cli/checkpoint.rs` only handle the `--workspace` flag and CWD fallback.
A shared helper avoids duplication and ensures all commands use the same
resolution semantics.

**Alternatives considered**:
- **Keep the simple CWD fallback**: Does not satisfy FR-005c, which requires
  `.boundline/` parent detection and git root fallback.
- **Move resolution into the orchestrator layer**: Resolution is a CLI-surface
  concern (the orchestrator receives an already-resolved workspace path).

## R3: Config Schema Extension For Canon Preferences

**Decision**: Add a new `canon` section to `ConfigFile` alongside the existing
`routing` section, containing `mode_selection: CanonModeSelectionPreference` and
optionally a `default_risk`, `default_zone`, and `default_owner` for workspace
defaults.

**Rationale**: The existing `ConfigFile` has only `version` and `routing`.
Canon mode-selection preference (`manual`, `auto-confirm`, `auto`) is a
workspace governance choice, not a routing decision, so it belongs in a separate
section.  Governance field defaults (`risk`, `zone`, `owner`) already exist on
`CanonRuntimeConfig` in the execution profile, but the spec requires
workspace-level defaults set during `init` without requiring an execution
manifest.

**Alternatives considered**:
- **Put mode-selection inside `RoutingConfig`**: Conflates governance
  configuration with model routing.  Separating them keeps each section focused.
- **Use a separate `governance.toml` file**: Adds a new persistence surface
  that the spec does not require.  All workspace-local config stays in
  `config.toml` per the existing pattern.

**Key types to extend**:
- `ConfigFile` at `src/domain/configuration.rs:493-505`
- New: `CanonPreferences { mode_selection, default_risk, default_zone, default_owner }`
- New: `CanonModeSelectionPreference` enum: `Manual`, `AutoConfirm`, `Auto`

## R4: Mode-Selection Inference And Confirmation Pattern

**Decision**: Extend the existing `build_autopilot_decision()` function and
`resolved_canon_mode()` helpers to respect the workspace's
`CanonModeSelectionPreference` when choosing how to select the Canon mode for a
governed stage.

**Rationale**: The orchestrator already has a mode-resolution pipeline:
`candidate_canon_modes()` → `resolved_canon_mode()` → autopilot decision.  The
mode-selection preference adds a gate:
- `manual`: return `PendingSelection` if no explicit `--mode` was provided
- `auto-confirm`: infer mode, surface it as a confirmation prompt, wait for
  operator response
- `auto`: infer mode and proceed if confidence is high; fall back to
  confirmation on ambiguity

The `AutopilotDecisionRecord` already captures `candidate_modes` and
`selected_mode`, so the decision record naturally extends to include the
selection preference and confidence level.

**Alternatives considered**:
- **New mode-selection module**: Unnecessary indirection; the existing
  governance orchestrator already handles mode resolution.
- **Defer `auto` until a later feature**: The spec explicitly requires all three
  preferences; implementing `auto` with confirmation fallback is
  straightforward and avoids a separate feature.

## R5: Input Assembly — From Operator Inputs To Canon Request Fields

**Decision**: Reuse and extend the existing `governance_input_documents()` and
`bounded_governance_context()` pipeline to assemble operator-provided goal text,
Markdown briefs, repository evidence, and clarification answers into Canon's
`input_documents` and `bounded_context` request fields.

**Rationale**: The pipeline already exists:
1. `AuthoredBriefBundle` captures goal + brief sources
2. `render_goal_text()` concatenates goal + sources with provenance headers
3. `governance_input_documents()` maps sources to `GovernanceInputDocument`
   with `kind` tags (`stage-brief`, `authored-brief`)
4. `bounded_governance_context()` builds `GovernanceBoundedContext` with
   `read_targets` and `reused_packets` from prior governed stages

The 042 feature extends this by:
- Adding clarification answers as additional input documents
- Mapping operator-level artifact types (PRD, C4, backlog, architecture) to
  Canon-expected input document kinds when Canon's capabilities response
  provides template hints
- Forwarding governed documents from prior stages automatically through the
  existing `reused_packets` field

**Alternatives considered**:
- **AI-generated Canon document authoring inside Boundline**: Out of scope.
  The spec says the active assistant may draft inputs in chat, but Boundline
  itself packages whatever the operator provides.  Document authoring is a
  chat-surface concern, not a Boundline orchestrator responsibility.
- **New input-assembly module**: The existing brief + governance context
  pipeline is sufficient; a separate module would duplicate logic.

## R6: Canon-Default Run Behavior

**Decision**: When the workspace is initialized for Canon (config.toml contains
`[canon]` section and diagnostics confirm the Canon surface), `boundline run`
defaults to Canon governance without requiring `--governance canon`.  Explicit
opt-out via `--governance local` or `--no-canon` overrides.

**Rationale**: The existing `execute_native_direct_run()` in `src/cli/run.rs`
chains `start` → `goal` → `plan` → `run`.  Currently, governance runtime
selection is explicit.  The change adds a resolution step after workspace
resolution:
1. Load workspace config
2. If `config.canon` is present and Canon diagnostics pass → default runtime =
   Canon
3. If `--governance local` or `--no-canon` → override to Local
4. Otherwise → Local (backward compatible)

The opt-out state is persisted in the session and projected through
`status`/`next`/`inspect`.

**Alternatives considered**:
- **Always default to Canon even without init**: Breaks backward compatibility
  and could fail on workspaces without Canon installed.
- **Require a global config to enable Canon-default**: Violates FR-005c
  (workspace-local only).

## R7: CanonMode Enum Expansion

**Decision**: Expand `CanonMode` from 9 variants to 15 by adding
`SystemShaping`, `Refactor`, `Review`, `Incident`, `SystemAssessment`,
`Migration`, and `SupplyChainAnalysis`.

**Rationale**: The spec lists 15 canonical modes.  The existing enum has:
`Requirements`, `Architecture`, `Backlog`, `Change`, `Discovery`,
`Implementation`, `Verification`, `SecurityAssessment`, `PrReview`.

Missing: `SystemShaping`, `Refactor`, `Review` (distinct from `PrReview`),
`Incident`, `SystemAssessment`, `Migration`, `SupplyChainAnalysis`.

Note: `PrReview` may need reconciliation with the new `Review` mode.  The spec
lists `review` as a canonical mode.  If Canon treats `review` and `pr-review`
as distinct modes, both should remain.  If Canon unifies them, `PrReview`
becomes an alias.  This will be resolved at implementation time based on the
actual Canon `capabilities` response.

**Key file**: `src/domain/governance.rs` — `CanonMode` enum + Display/FromStr
impls + `supported_canon_modes_for_stage()` mapping table.

## R8: Assistant Command Pack Alignment

**Decision**: Update all assistant command packs (Copilot, Codex, Claude, Gemini)
to expose the Canon-default workflow as the primary path, with new commands for
`/boundline-init`, `/boundline-doctor`, `/boundline-config-show`,
`/boundline-config-set-canon`, `/boundline-config-set`, and mode-specific
aliases (`/boundline-requirements`, etc.).

**Rationale**: The spec requires CLI/assistant parity (FR-008).  Existing
command packs in `assistant/copilot/prompts/` follow a consistent pattern:
YAML frontmatter → Intent → Required Context → Shell-Enabled Path → Chat-Only
Path → Output Interpretation → Next-Step Routing.  New commands follow the same
pattern.  Mode-specific aliases map directly to `boundline run --mode <mode>`.

**Alternatives considered**:
- **Single `/boundline-run` with mode argument only**: Loses the ergonomic
  benefit of mode-specific aliases in chat, which the spec explicitly allows.
- **Generate command packs programmatically**: Adds build complexity for a
  one-time authoring task.
