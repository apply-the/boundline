# Feature Specification: Interactive Delivery Dashboard

**Feature Branch**: `064-interactive-delivery-dashboard`
**Created**: 2026-05-19
**Status**: Draft
**Input**: User description: "Create a complete interactive delivery dashboard for Boundline as an operator-facing terminal shell over existing runtime truth. It must ship as one complete feature, not a split roadmap slice. The dashboard lets operators inspect and act on existing sessions, goal plans, trace timelines, stop rules, guidance and guardian findings, checkpoints, degraded states, and read-only governed artifact references when present. It must preserve the CLI and session-native runtime as authoritative, avoid a second workflow engine, avoid a separate state store, avoid independent config/init/governance behavior, and require no Canon runtime changes. Use a separate dashboard workspace component, expose stable runtime snapshots or event projections from the existing Boundline-owned state, invoke the same runtime behavior for confirm, reject, replan, recover, and launch actions, include terminal-safe colored branding using a simple boundline ASCII wordmark only, include docs, changelog, roadmap cleanup, version bump, tests, formatting, lint, and modified Rust coverage closure, and do not mention roadmap code names in documents or file names."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Attach To Current Delivery State (Priority: P1)

An operator opens the dashboard inside a workspace and immediately sees the active Boundline delivery state without reconstructing it from linear command output. The dashboard shows the current session, goal, active stage, current step, next bounded action, stop posture, plan state, recent trace events, and whether the runtime is ready, waiting, blocked, failed, or complete. When multiple persisted session candidates or stale trace references are detectable, the dashboard resolves the current authoritative session explicitly instead of silently choosing an ambiguous state.

**Why this priority**: The primary value of the dashboard is visible runtime trust. If an operator cannot understand the current delivery state from the first screen, later controls are not credible.

**Independent Test**: Can be tested by preparing a workspace with an active session and trace history, opening the dashboard, and verifying that the displayed state matches the authoritative Boundline status and inspect surfaces.

**Acceptance Scenarios**:

1. **Given** a workspace with a confirmed plan and a running session, **When** the operator opens the dashboard, **Then** the dashboard shows the session summary, active stage, current step, next action, plan state, trace timeline, and stop posture from the same authoritative state used by Boundline's normal command surfaces.
2. **Given** a workspace with no active session, **When** the operator opens the dashboard, **Then** the dashboard clearly reports that no session is active and offers the same supported next commands that a terminal user would use to start or capture work.
3. **Given** a workspace whose latest execution is blocked by approval, missing context, failed validation, or exhaustion, **When** the operator opens the dashboard, **Then** the dashboard foregrounds the blocking reason and the allowed follow-up action instead of presenting the work as merely idle.
4. **Given** a workspace with multiple session candidates or trace references that do not match the current session revision, **When** the operator opens or refreshes the dashboard, **Then** the dashboard identifies the authoritative current session or reports the ambiguity with the valid normal command needed to resolve it.

---

### User Story 2 - Inspect Plans, Evidence, And Findings (Priority: P2)

An operator reviews the delivery context behind the current next action. The dashboard provides focused panels for the current goal plan, selected evidence, context-pack facts, trace timeline, stop rules, guidance and guardian outcomes, findings, checkpoints, dashboard diagnostics, and governed artifact references when they exist.

**Why this priority**: Operators need to trust why Boundline is about to act, stop, or ask. The dashboard must make the delivery rationale inspectable without giving external systems or decorative UI control over the runtime.

**Independent Test**: Can be tested by preparing sessions with goal plans, selected evidence, guidance results, guardian findings, checkpoints, and optional governed artifact references, then verifying that each panel presents the relevant facts and provenance while preserving read-only boundaries for governed references.

**Acceptance Scenarios**:

1. **Given** a session with selected workspace evidence and a current goal plan, **When** the operator opens the plan, context, and evidence views, **Then** the dashboard shows the plan revision, active target, evidence references, omitted or degraded context, available context-pack reason, source, budget cost, authority, and verification strategy needed to evaluate the next action.
2. **Given** a session with guidance, guardian findings, or review findings, **When** the operator opens the findings view, **Then** the dashboard shows severity, status, evidence references, and unresolved follow-up requirements in a scannable form.
3. **Given** governed artifacts or project-memory references are available, **When** the operator opens the governed reference view, **Then** the dashboard displays them read-only with readiness, provenance, and approval cues, and does not require changes to the governed runtime.
4. **Given** governed artifacts are absent, unreadable, or incompatible, **When** the operator opens the dashboard, **Then** the dashboard continues to work from Boundline runtime evidence and reports the governed reference state as unavailable or degraded.
5. **Given** workspace state or terminal capability is degraded, **When** the operator opens the dashboard diagnostics view, **Then** the dashboard reports workspace health, runtime command availability, terminal capability limits, state-readability status, and the valid fallback commands without changing runtime state.

---

### User Story 3 - Act Through Existing Runtime Boundaries (Priority: P3)

An operator can take allowed delivery actions from the dashboard, including confirming a plan, rejecting a proposed direction, requesting replanning, recovering from a stopped state, launching a new session path, or continuing a bounded run. Each action must produce the same behavior and traceable outcome as the equivalent normal Boundline command path.

**Why this priority**: A dashboard that only reads state helps inspection but does not complete the operator loop. Actions are valuable only if they preserve Boundline's existing execution boundaries instead of creating a second control plane.

**Independent Test**: Can be tested by executing each supported action from the dashboard and from the normal command surface against equivalent prepared sessions, then comparing resulting session state, traces, next actions, and terminal outcomes.

**Acceptance Scenarios**:

1. **Given** a plan is waiting for confirmation, **When** the operator confirms it from the dashboard, **Then** Boundline records the same confirmed plan state and follow-up action that the normal confirmation path would record.
2. **Given** a plan or next action is rejected from the dashboard, **When** the operator provides a bounded rejection reason, **Then** Boundline records the rejection, preserves the prior evidence, and moves to an explicit replan or stopped state without losing context.
3. **Given** a failed, blocked, or exhausted session, **When** the operator selects recovery or replanning from the dashboard, **Then** Boundline exposes only valid recovery choices and records the chosen path in the same inspectable session and trace state.
4. **Given** a dashboard action cannot be applied because the session changed, the workspace is invalid, or a stop rule forbids progression, **When** the operator attempts the action, **Then** the dashboard refuses the action, explains the current authoritative state, and offers the updated next valid action.

---

### User Story 4 - Ship As One Complete Release-Aligned Feature (Priority: P4)

A maintainer can release the dashboard as a complete Boundline capability with aligned version metadata, documentation, wiki pages, roadmap cleanup, changelog entry, validation evidence, and a simple terminal-safe brand treatment. The release must not depend on a later split feature to be useful.

**Why this priority**: The dashboard changes the primary operator experience for complex delivery. It must close as a coherent release with validation, docs, and product boundaries aligned.

**Independent Test**: Can be tested by following the updated docs on representative workspaces, running the validation suite, and confirming the dashboard remains useful across normal, blocked, degraded, and complete delivery states.

**Acceptance Scenarios**:

1. **Given** the feature is prepared for release, **When** a maintainer reviews repository docs, changelog, roadmap, version metadata, and assistant guidance, **Then** they all describe the shipped dashboard capability consistently without using roadmap code names.
2. **Given** the dashboard renders in a normal terminal, **When** the operator opens the first screen, **Then** it shows a simple colored `boundline` ASCII wordmark and does not rely on image files or wide ANSI banner art.
3. **Given** the dashboard dependency is unavailable or the terminal cannot support interactive rendering, **When** the operator attempts to use the dashboard, **Then** Boundline reports a clear degraded path and preserves the normal command surfaces as fully usable.

### Edge Cases

- The dashboard opens in a directory that is neither an initialized Boundline workspace nor a detectable repository root.
- The active session file is missing, partially written, or invalid while trace files still exist.
- The latest trace refers to a session revision that is older than the current session state.
- Multiple persisted sessions or trace candidates exist and only one can be treated as current.
- A plan is waiting for confirmation but a stop rule or governed approval has become authoritative since the dashboard was opened.
- A dashboard action is requested after another process has changed the session state.
- The dashboard is open while another process changes session, trace, checkpoint, finding, or governed-reference state.
- The terminal is too narrow to show full panels or the brand wordmark.
- Color support is disabled or unavailable.
- Governed artifact references are present but unreadable, stale, incompatible, or missing readiness metadata.
- The operator requests recovery when no recovery path is credible.
- The dashboard cannot access the runtime command path needed to apply an action.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The dashboard MUST treat Boundline's existing session, plan, trace, checkpoint, finding, and governed-reference state as authoritative and MUST NOT create a separate delivery state store.
- **FR-002**: The dashboard MUST show the active session summary, goal, route posture, current stage, current step, plan state, next bounded action, and latest terminal or non-terminal condition on the first operational screen.
- **FR-003**: The dashboard MUST show a trace timeline that includes recent runtime events, errors, retries, replanning events, confirmations, rejections, recovery events, and terminal outcomes when those facts are available.
- **FR-004**: The dashboard MUST provide inspectable views for goal plan details, selected evidence, context-pack facts, omitted or degraded context, stop rules, guidance results, guardian findings, review findings, checkpoints, dashboard diagnostics, and governed artifact references.
- **FR-005**: The dashboard MUST preserve the primary session-native route as the default product path and MUST label any explicit compatibility route as compatibility behavior rather than silently treating it as primary.
- **FR-006**: The dashboard MUST support allowed operator actions for confirming plans, rejecting proposed directions, requesting replanning, recovering from stopped states, launching a new session path, and continuing bounded execution.
- **FR-007**: Every dashboard action MUST produce the same authoritative state transition, trace evidence, next action, and terminal behavior as the equivalent normal Boundline command path.
- **FR-008**: The dashboard MUST refuse invalid or stale actions when the underlying session state changes, when the workspace is invalid, when required context is missing, or when stop rules forbid progression.
- **FR-009**: The dashboard MUST surface non-success states as first-class outcomes, including blocked, waiting, failed, exhausted, invalid, degraded, and missing-context states.
- **FR-010**: The dashboard MUST display governed artifact and project-memory references as read-only inputs when present and MUST remain fully usable when those references are absent.
- **FR-011**: The dashboard MUST NOT require Canon runtime changes, Canon-owned state migrations, or Canon-owned command behavior to deliver its core value.
- **FR-012**: The dashboard MUST NOT introduce a second workflow engine, independent configuration behavior, independent initialization behavior, independent governance behavior, hidden background progression, or separate orchestration semantics.
- **FR-013**: The dashboard MUST include terminal-safe colored branding using only a simple `boundline` ASCII wordmark, with a non-color fallback and no dependency on image assets or wide ANSI banner art.
- **FR-014**: The dashboard MUST provide a degraded mode when interactive rendering, terminal capabilities, workspace state, or runtime command access are insufficient, and the degraded mode MUST point operators back to valid normal Boundline commands.
- **FR-015**: The dashboard MUST keep one active delivery action in focus at a time and MUST NOT introduce background workers, parallel execution, hidden fan-out, or autonomous progression.
- **FR-016**: The feature MUST ship as one complete release-aligned capability with version metadata, impacted docs, wiki pages, changelog, roadmap cleanup, assistant guidance if affected, validation evidence, formatting, linting, and modified Rust coverage closure.
- **FR-017**: Repository documentation and generated feature artifacts MUST NOT mention roadmap code names for this feature.
- **FR-018**: The dashboard MUST provide an explicit current-session resolver or selector when persisted session or trace evidence contains multiple candidates, stale revisions, or ambiguous authority.
- **FR-019**: Context and evidence panels MUST show context-pack reason, source, budget cost or equivalent scope cost, authority, and provenance when those facts are available, and MUST distinguish unavailable fields from empty fields.
- **FR-020**: The dashboard MUST provide a dashboard-oriented diagnostics view that reports workspace health, runtime command availability, terminal capability limits, state-readability status, and valid fallback commands without creating dashboard-owned state.
- **FR-021**: The dashboard MUST provide an explicit refresh path for externally changed state and MAY use a non-autonomous watcher or polling mechanism only to refresh displayed authoritative state; refresh MUST NOT trigger delivery actions.
- **FR-022**: The dashboard MUST keep first render and local refresh performance testable against the plan targets without making performance validation depend on network or external provider calls.

### Scope Boundaries *(mandatory)*

- **In Scope**: An operator-facing interactive dashboard over existing Boundline delivery state; current-session resolution over existing state; context-pack and evidence inspection; state and event projections needed to make that dashboard reliable; dashboard diagnostics; read-only display of governed references when present; action handoff through existing runtime boundaries; release documentation, wiki, and validation closure.
- **Out of Scope**: A web dashboard, a second workflow engine, a new persistent state store, independent configuration or initialization flows, independent governance logic, Canon runtime changes, new provider routing, MCP server or client work, browser automation providers, AI gateway economics, autonomous background workers, distributed execution, wide ANSI banner rendering, and image-based dashboard branding.

### Key Entities *(include if feature involves data)*

- **Dashboard Session View**: The operator-visible projection of the active Boundline session, including workspace, goal, route posture, stage, current step, plan state, execution condition, next action, and blocking reason.
- **Runtime Event Projection**: The ordered facts the dashboard uses to render the trace timeline, including start, plan, confirmation, action, validation, failure, retry, replan, recovery, governance, checkpoint, and terminal events.
- **Dashboard Action Request**: A bounded operator action initiated from the dashboard, including action type, target session revision, optional operator reason, expected state transition, and refusal reason when invalid.
- **Inspection Panel State**: The focused view state for plan details, selected evidence, context degradation, stop rules, findings, checkpoints, and governed references.
- **Degraded Dashboard State**: The explicit fallback state used when the dashboard cannot render interactively, cannot read authoritative state, or cannot apply actions safely.
- **Terminal Brand Mark**: The terminal-safe `boundline` wordmark rendered with color when available and with a plain fallback when color is unavailable.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative active-session workspaces, operators can identify the current stage, current step, next action, and blocking condition from the dashboard within 30 seconds.
- **SC-002**: In representative sessions covering ready, waiting, blocked, failed, exhausted, degraded, and complete states, the dashboard state matches the authoritative normal command output in 100% of validation cases.
- **SC-003**: Operators can complete each supported action path, including confirm, reject, replan, recover, launch, and continue, with resulting session and trace state equivalent to the normal command path in 100% of paired validation cases.
- **SC-004**: For sessions with findings, checkpoints, selected evidence, and governed references, operators can locate the relevant evidence or follow-up requirement in under 2 minutes without opening raw session or trace files.
- **SC-005**: The dashboard remains usable in degraded conditions by reporting a clear reason and valid command fallback in 100% of tested degraded scenarios.
- **SC-006**: Modified Rust files for the feature meet the repository's patch coverage target, formatting passes, linting passes, and the release validation suite records no dashboard-specific regressions.
- **SC-007**: Release-facing docs, changelog, roadmap, assistant guidance if affected, and version metadata describe the dashboard consistently and contain no roadmap code names for this feature.
- **SC-008**: In representative local workspaces, the first dashboard render completes in under 1 second and local refresh after authoritative state changes completes in under 1 second, excluding the underlying command runtime.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**:
  - OpenAI model documentation: https://developers.openai.com/api/docs/models
  - OpenAI GPT-5-Codex documentation: https://developers.openai.com/api/docs/models/gpt-5-codex
  - GitHub Copilot supported models: https://docs.github.com/en/copilot/reference/ai-models/supported-models
  - Anthropic Claude model overview: https://platform.claude.com/docs/en/about-claude/models/overview
  - Google Gemini model documentation: https://ai.google.dev/gemini-api/docs/models
- **Catalog Delta**: Public docs reviewed on 2026-05-19 indicate catalog refresh work is required during planning or implementation: GitHub Copilot lists newer Codex route options and marks older model-picker entries as retired, Anthropic lists Opus 4.7, Sonnet 4.6, and Haiku 4.5 as current primary Claude choices, Google documents Gemini 3.1 and Gemini 3 families plus deprecation of earlier Gemini 3 Pro Preview naming, and OpenAI documents GPT-5.5, GPT-5.4 variants, and GPT-5-Codex. The feature plan must reconcile the bundled catalog with these findings before release closure.
- **No-Change Rationale**: Not applicable; the review found likely catalog drift that must be handled as part of release alignment.

## Assumptions

- Operators use the dashboard from a local terminal inside or near a Boundline workspace.
- The normal Boundline command surfaces remain fully supported and authoritative after the dashboard ships.
- The dashboard is an operator execution surface for delivery state and actions, not a marketing page, generic chat interface, or visual replacement for repository documentation.
- Governed artifacts are optional inputs to display, not prerequisites for dashboard operation.
- The first screen should optimize for immediate operational state, not for exhaustive historical inspection.
- The dashboard may introduce shared state projections only when they make existing Boundline truth easier to inspect; those projections must not become an alternate source of truth.
- Future roadmap items may extend the dashboard after this release, but this feature is complete only when it delivers a useful end-to-end dashboard for the current Boundline runtime baseline.
