# Feature Specification: Contextual Help And Documentation Architecture (Boundline)

**Feature Branch**: `073-contextual-help-docs`

**Created**: 2026-06-06

**Status**: Draft

**Input**: User description: "Boundline help-next — operator guidance surface for uninitialized, initialized, active-session, blocked, and failed runtime states with documentation link projection."

## Clarifications

### Session 2026-06-06

- Q: Should this feature be one monolithic cross-repo spec? → A: No. Boundline owns `boundline help-next` and runtime/session diagnostics. Canon owns `canon help-next` and mode/documentation diagnostics. Two separate specs, coordinated but independently owned.
- Q: Should `help-next` emit a structured runtime event for observability? → A: Yes. Emit `boundline.help_next.requested` as a structured append-only event recording the diagnosed state, diagnostics count, recommended action, command, docs link, and output format (human or JSON). Must not include secrets, raw prompts, or raw traces. Keeps `help-next` read-only while making operator friction observable.
- Q: How should documentation links be resolved and kept current across releases? → A: Use a versioned `.boundline/help-links.toml` link map file mapping diagnostic keys to relative wiki paths or stable URLs. Loaded at runtime, committed in repo. Missing keys produce a non-blocking warning with a generic troubleshooting fallback. Links are not hardcoded in Rust source.
- Q: When multiple issues exist, should help-next report one or all? → A: Default shows the single highest-priority blocking issue plus additional-issue count. `--all` lists all issues ordered by priority. `--json` includes both `primary_issue` and `additional_issues` arrays.
- Q: What should help-next output when the workspace is healthy with no blockers? → A: Report state=ready, confirm no blockers found, and recommend the next logical command for the current lifecycle phase (e.g., `boundline run`). For `--json`, include `state: "ready"`, `blockers_found: false`, `primary_issue: null`, `additional_issues: []`, plus recommendation fields.

## User Scenarios & Testing

### User Story 1 - Discover Next Action From Any Runtime State (Priority: P1)

An operator at any stage of the Boundline lifecycle — uninitialized workspace, initialized but no session, active session, blocked planning/execution, failed state, or healthy ready state — runs `boundline help-next` and receives the current state, the recommended next action, the exact command to run, the reason why, and a link to relevant documentation.

**Why this priority**: Without a discoverable next step, operators get stuck and abandon the tool. This is the single highest-impact adoption feature.

**Independent Test**: Run `boundline help-next` in an uninitialized temp workspace, confirm it suggests `boundline init`, then initialize, run again, confirm it suggests creating a goal.

**Acceptance Scenarios**:

1. **Given** an uninitialized workspace, **When** the operator runs `boundline help-next`, **Then** the output reports state=uninitialized, recommends `boundline init`, and links to the installation guide.
2. **Given** an initialized workspace with no active session, **When** the operator runs `boundline help-next`, **Then** the output reports state=initialized, recommends creating a goal, and shows the exact `boundline goal` command.
3. **Given** an active session with a blocked planning-analysis gate, **When** the operator runs `boundline help-next`, **Then** the output reports the blocked state, the blocking findings, the recommended repair action, and the `boundline plan` continuation command.
4. **Given** a failed execution state, **When** the operator runs `boundline help-next`, **Then** the output reports the failure reason, available recovery paths, and the `boundline run` retry command.
5. **Given** a healthy active session with no blockers, **When** the operator runs `boundline help-next`, **Then** the output reports `state=ready`, confirms no blockers found, and recommends the next logical command for the current phase.

---

### User Story 2 - Diagnose Missing Configuration Or Provider Readiness (Priority: P2)

An operator wants to know why Boundline cannot proceed due to missing configuration, unregistered providers, or missing context packs.

**Why this priority**: Configuration gaps are the most common onboarding friction. Surfacing them before execution saves debugging cycles.

**Independent Test**: Remove a required config key, run `boundline help-next`, confirm the output identifies the missing key and the config file path.

**Acceptance Scenarios**:

1. **Given** a missing required config key, **When** the operator runs `boundline help-next`, **Then** the output identifies the missing key, the config file path, and a link to the configuration reference.
2. **Given** a provider that is registered but not activated, **When** the operator runs `boundline help-next`, **Then** the output reports the provider state, the activation command, and setup requirements.

---

### Edge Cases

- What happens when `help-next` is run without a `.boundline/` directory at all?
- What happens when multiple issues exist simultaneously (e.g., missing config AND blocked planning)? → Resolved: Default reports the single highest-priority blocking issue with additional-issue count; `--all` lists all.
- How does the system handle a state where the session file is corrupt or unreadable? → Resolved: The system reports state=failed with a diagnostic indicating the session file is unreadable, exits with code 1, and recommends restoring from backup or re-initializing.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST provide a `boundline help-next` command that inspects the current workspace and session state without mutating any files.
- **FR-002**: The system MUST detect and report at minimum these six states: uninitialized, initialized (no session), active session (current phase, healthy), blocked (planning analysis or execution gate), failed, and ready (healthy with no blockers).
- **FR-003**: For each detected state, the system MUST output: current state label, next recommended action, exact CLI command, reason for the recommendation, and a link to relevant documentation.
- **FR-004**: The system MUST diagnose missing configuration keys, unregistered providers, missing provider activation, and missing context packs when they would block the next action.
- **FR-005**: The system MUST prioritize blocked/failed states over informational states when multiple conditions apply. By default, the system MUST report the single highest-priority blocking issue with a count of additional detected issues. The `--all` flag MUST list all detected issues ordered by priority.
- **FR-006**: The system MUST surface guardian findings and stop rules when they are the cause of a blocked state.
- **FR-007**: Documentation links MUST be resolved via a versioned link map file (`.boundline/help-links.toml`) that maps diagnostic keys to relative wiki paths or stable URLs. Links are loaded at runtime and committed in the repository. When a key is missing from the map, the system MUST still return the diagnostic and command but mark the link as unavailable with a non-blocking warning and a generic troubleshooting fallback.
- **FR-008**: The system MUST NOT mutate session state, configuration, or trace files during `help-next` execution.
- **FR-009**: The system MUST support a `--json` flag that produces the same diagnostic information as a stable structured JSON object suitable for CI, automation, and assistant integration.
- **FR-010**: The system MUST emit a `boundline.help_next.requested` structured runtime event (append-only, per the event vocabulary from spec 072) recording the diagnosed workspace/session state, lifecycle phase, blocked/failed/degraded category, diagnostics count, recommended action id, recommended command, docs link, and output format. The event MUST NOT include secrets, raw prompts, or raw traces.

### Key Entities

- **HelpNextState**: An enumeration of detectable workspace/runtime states (uninitialized, initialized, active, blocked, failed) with associated diagnostics.
- **HelpNextDiagnostic**: A single actionable finding (missing config, unregistered provider, blocked gate) with severity, source reference, and repair guidance.
- **HelpNextRecommendation**: The resolved next action including state label, command, reason, documentation link, primary issue, and when `--all` or `--json` is used, the full list of additional issues ordered by priority.

## Success Criteria

- **SC-001**: An operator in any of the five core states can identify the next recommended action within 10 seconds of running `boundline help-next`.
- **SC-002**: 100% of blocked-planning scenarios produce a help-next output that identifies the blocking finding and the repair command.
- **SC-003**: Missing configuration keys are identified by name and config file path in 100% of regression cases.

## Assumptions

- The existing readiness, probe, and session status surfaces provide the raw state data that `help-next` consumes.
- Documentation links reference the wiki structure defined in the joint documentation architecture but are maintained independently.
- The first slice implements help-next as a read-only diagnostic; interactive repair guidance is deferred.
- Canon `help-next` is a separate Canon-owned feature spec (canon/specs/073-contextual-help-docs/).
