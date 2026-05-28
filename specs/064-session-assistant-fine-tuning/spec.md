# Feature Specification: Provider Auth, Probe Readiness, and Assistant Handoff Fine-Tuning

**Feature Branch**: `064-session-assistant-fine-tuning`
**Created**: 2026-05-25
**Updated**: 2026-05-28
**Status**: Implemented (Retrospective Updated)
**Input**: Retrospective update for commits `cad1675`, `9ba0b21`, and `6182711`

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Provider Authentication Lifecycle (Priority: P1)

As an operator using provider-backed Boundline routes, I want device-flow login, status, and removal commands for stored model-provider credentials, so I can authenticate GitHub Copilot once and reuse that access across CLI and assistant-host flows.

**Why this priority**: Provider authentication is a hard prerequisite for Copilot-backed runtime work; without it, later assistant and execution surfaces degrade immediately.

**Independent Test**: Run the `boundline models auth` lifecycle for `github-copilot` and verify login stores credentials, status reports the provider without exposing secrets, and remove clears the stored entry.

**Acceptance Scenarios**:

1. **Given** no stored provider auth, **When** `boundline models auth login --provider github-copilot` completes device flow, **Then** Boundline persists a token-backed auth profile and reports the profile storage path.
2. **Given** one or more stored provider entries, **When** `boundline models auth status` runs, **Then** it lists authenticated providers and the auth profile path without printing token or API key values.
3. **Given** a stored provider entry, **When** `boundline models auth remove --provider github-copilot` runs, **Then** the provider is deleted from the auth profile store and later status output no longer lists it.

---

### User Story 2 - Planning Gates and Assistant-Safe Handoffs (Priority: P1)

As an assistant host or operator reading Boundline session output, I want goal, plan, backlog, and planning-analysis gate state surfaced together with assistant-safe follow-up commands, so I can stop on real runtime gates instead of improvising the next step from chat context.

**Why this priority**: The runtime already owns planning readiness; exposing those gates and handoffs accurately prevents hosts from continuing into invalid execution states.

**Independent Test**: Run the planning gate and host output contract tests and verify `goal_quality_state`, `plan_quality_state`, `backlog_quality_state`, `planning_analysis_state`, `assistant_resume_command`, and `assistant_next_command` are preserved where applicable.

**Acceptance Scenarios**:

1. **Given** a goal or plan that still needs clarification, **When** session or host JSON output is rendered, **Then** the relevant quality state is surfaced and the reported assistant-safe continuation overrides a default next step.
2. **Given** a governed or bounded session blocked on backlog or planning analysis, **When** `run`, `status`, or `next` is evaluated, **Then** Boundline reports the blocking state and does not continue into execution.
3. **Given** a structured `phase_request`, **When** assistant-facing output is emitted, **Then** the response preserves `phase_request`, `assistant_resume_command`, and `assistant_next_command` semantics instead of flattening them into plain CLI advice.

---

### User Story 3 - Probe Preflight Readiness (Priority: P1)

As an assistant host deciding whether to initialize, doctor, or continue a workspace, I want a read-only `boundline probe` command, so I can detect bootstrap, provider, and session readiness before running orchestration.

**Why this priority**: Hosts need a cheap readiness check that does not mutate workspace state and does not invent repo-local handoffs when only global bootstrap is valid.

**Independent Test**: Run the probe command contract tests and verify bootstrap, doctor, current-workspace resolution, and host-envelope JSON cases all route correctly.

**Acceptance Scenarios**:

1. **Given** an uninitialized workspace, **When** `boundline probe` runs, **Then** it recommends `boundline init`, omits any assistant handoff, and returns no repo-local recovery route.
2. **Given** an initialized workspace with missing provider credentials, **When** `boundline probe` runs, **Then** it recommends `boundline doctor` and surfaces `/boundline-doctor` as the assistant-safe handoff.
3. **Given** an initialized workspace with healthy provider credentials but no active session, **When** `boundline probe` runs, **Then** it recommends `boundline goal` and surfaces `/boundline-goal` as the assistant-safe handoff.
4. **Given** `boundline probe --json`, **When** the command succeeds, **Then** it emits the standard host envelope and includes the rendered probe report JSON in `rendered_output`.

---

### User Story 4 - Cross-Host Assistant Contract Parity (Priority: P2)

As a maintainer shipping assistant assets across Copilot, Claude, Codex, and Antigravity, I want prompt sections, next-step routing, and host-native action syntax kept consistent, so contract tests stay green and each host preserves the same runtime authority.

**Why this priority**: Assistant assets drift easily. Cross-host parity and explicit contract coverage keep prompt guidance from diverging from runtime behavior or from each other.

**Independent Test**: Run the assistant command pack and definition contract suites and verify the touched assets satisfy required sections, routing semantics, probe preflight guidance, and host-specific action syntax.

**Acceptance Scenarios**:

1. **Given** a Copilot action prompt, **When** it renders an assistant-safe next step, **Then** it uses `command:github.copilot.chat.execute` links instead of plain text shell recommendations.
2. **Given** a Claude, Codex, or Antigravity command asset, **When** it describes the next step, **Then** it preserves host-native `/boundline:*` routing instead of Copilot-specific command URIs.
3. **Given** readiness-sensitive commands such as goal, plan, status, or recover, **When** the user starts from an uncertain workspace, **Then** the assistant guidance instructs the host to run `boundline probe --workspace <workspace> --json` and to respect bootstrap-only or doctor-only outcomes.

## Edge Cases

- Unsupported providers must fail `models auth login` with a clear unsupported-provider error rather than silently falling back.
- `models auth status` must never print stored token or API key values.
- Removing a provider that is not present must return a non-success report without corrupting the auth profile store.
- Probe must not invent an assistant route for bootstrap-only work; uninitialized workspaces must stay on `boundline init --assistant <host>` or the host-global bootstrap surface.
- Probe path reporting must remain stable across macOS temp-directory normalization differences such as `/var` versus `/private/var`.
- Planning-gate precedence must remain deterministic: plan-quality and backlog-quality stops must surface before execution continues, and blocked planning analysis must prevent `run` from advancing.
- Copilot prompts must not use host-native `/boundline:*` syntax as their only action mechanism, and non-Copilot assets must not emit Copilot-specific command URIs.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST expose `boundline models auth login`, `boundline models auth status`, and `boundline models auth remove` subcommands.
- **FR-002**: `boundline models auth login` MUST support device-flow authentication for `github-copilot`.
- **FR-003**: Stored provider authentication MUST use a versioned auth profile store with typed token or API-key entries and persisted token acquisition timestamps.
- **FR-004**: `boundline models auth status` MUST list authenticated providers and the auth profile storage path without exposing secret values.
- **FR-005**: `boundline models auth remove` MUST delete stored auth for the requested provider and return a non-success report when no stored auth exists.
- **FR-006**: Provider runtime authentication resolution MUST consult stored auth profiles alongside existing environment-based credentials where the touched adapters support them.
- **FR-007**: Goal, plan, run, status, and next session outputs MUST surface runtime gate projections such as `goal_quality_state`, `plan_quality_state`, `backlog_quality_state`, and `planning_analysis_state` when available.
- **FR-008**: Boundline MUST stop execution handoff on clarification-required or blocked planning gates instead of advancing into run-time execution.
- **FR-009**: Assistant-facing session flows MUST preserve `phase_request`, `assistant_resume_command`, and `assistant_next_command` semantics without flattening them to generic text.
- **FR-010**: The CLI MUST expose a read-only `boundline probe` command with optional workspace resolution from the current directory.
- **FR-011**: Probe output MUST report workspace initialization, config presence, execution profile presence, session state, provider health, Canon readiness, runtime capabilities, `recommended_next`, and optional `recommended_handoffs`.
- **FR-012**: Probe MUST omit repo-local assistant routing when bootstrap is the only valid next step; uninitialized workspaces MUST recommend `boundline init` without assistant handoffs.
- **FR-013**: `boundline probe --json` MUST emit the standard host envelope with `command_name = probe`, `exit_status`, and the rendered probe report JSON in `rendered_output`.
- **FR-014**: Readiness-sensitive assistant assets for goal, plan, status, and recover MUST use probe as the documented preflight readiness check and MUST route bootstrap outcomes to `boundline init --assistant <host>` or the host-global bootstrap surface.
- **FR-015**: Copilot action prompts MUST render assistant-safe next steps as `command:github.copilot.chat.execute` links.
- **FR-016**: Claude, Codex, and Antigravity assets MUST preserve host-native `/boundline:*` routing and MUST NOT emit Copilot-specific command URIs.
- **FR-017**: Assistant prompt assets across Copilot, Claude, Codex, and Antigravity MUST include required sections, `Next-Step Routing`, and runtime precedence rules validated by contract tests.
- **FR-018**: Goal, plan, run, status, inspect, and follow-up assistant assets MUST document relevant goal-quality, plan-quality, backlog-quality, planning-analysis, and follow-through stop conditions where those runtime projections are part of the touched flow.
- **FR-019**: Release-facing documentation MUST describe probe as a helper surface for assistant hosts rather than as a repo-local `/boundline:*` command.

### Scope Boundaries *(mandatory)*

- **In Scope**: GitHub Copilot device-flow auth lifecycle; versioned auth profile storage; runtime auth-profile consumption in the touched provider adapters; planning-gate and assistant-safe handoff propagation; read-only `boundline probe`; readiness-sensitive prompt updates; cross-host prompt parity and contract closure; representative docs and tests for these surfaces.
- **Out of Scope**: additional provider-specific OAuth flows beyond the current `github-copilot` surface; a remote credential vault; replacement of environment credentials as a supported path; redesign of the broader assistant package architecture; new Canon contract surfaces.

### Key Entities *(include if feature involves data)*

- **AuthProfileStore**: Versioned global JSON store for persisted provider authentication entries.
- **ProviderAuthEntry**: Typed auth record containing provider identity plus either a token with `obtained_at` or an API key.
- **ModelsAuthReport**: CLI result model for auth login, status, and removal commands.
- **Planning Gate Projection**: Runtime-visible bundle of `goal_quality_state`, `plan_quality_state`, `backlog_quality_state`, and `planning_analysis_state` data used to decide whether execution can continue.
- **ProbeReport**: Read-only workspace readiness projection containing workspace, session, provider, Canon, capability, and next-step signals.
- **RecommendedNext**: Probe-level CLI and optional assistant-safe next action chosen from current readiness state.
- **RecommendedHandoff**: Probe-level assistant-host button or action recommendation.
- **Assistant Handoff Definition**: Prompt-side routing contract expressed through frontmatter, `Next-Step Routing`, and host-specific action syntax.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Representative provider-auth validation demonstrates successful `login`, `status`, and `remove` flows for `github-copilot` without exposing token values in status output.
- **SC-002**: Representative host command and planning-gate contract tests surface gate projections and block execution on invalid planning readiness states.
- **SC-003**: Probe contract coverage passes for bootstrap-only, doctor-required, goal-ready, current-workspace-resolution, and host-envelope JSON scenarios.
- **SC-004**: Assistant command pack and definition contract suites pass for the touched Copilot, Claude, Codex, and Antigravity assets.
- **SC-005**: Focused lint and behavioral validation for the touched slices complete cleanly without introducing regressions.
- **SC-006**: Readiness-sensitive assistant prompts consistently steer bootstrap to init, unhealthy providers to doctor, and session-ready workspaces to assistant-safe repo-local handoffs.

## Assumptions

- The global Boundline config directory remains the correct persistence root for `auth-profiles.json`.
- `github-copilot` is the only provider that needs device-flow login in this slice.
- Existing environment-based provider credentials remain supported; stored auth profiles are additive, not a replacement path.
- Prompt and command-pack edits remain bounded to the touched host assets and their contract coverage.
- This retrospective update should describe the behavior that actually landed in commits `cad1675`, `9ba0b21`, and `6182711`, even when earlier 064 draft content covered different fine-tuning work.
