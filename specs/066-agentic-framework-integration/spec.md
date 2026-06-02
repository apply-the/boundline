# Feature Specification: Agentic Framework Integration

**Feature Branch**: `066-agentic-framework-integration`

**Created**: 2026-05-30

**Status**: Draft

**Input**: User description from `roadmap/features/02-agentic-framework-integration.md`

## Clarifications

### Session 2026-05-30

- Q: What should happen if an adapter fails during an overridden stage after the stage has already started? → A: Mark the stage as failed, stop the run, and require operator intervention.
- Q: What trust and permission model should v1 use for configured adapters? → A: V1 treats the adapter as an explicitly configured, operator-trusted local subprocess and does not add a separate permission mediation layer.
- Q: How many adapters should v1 support per lifecycle run? → A: V1 supports exactly one active adapter per lifecycle run.
- Q: What should happen in non-interactive runs when adapter-required configuration is missing? → A: Fail fast before adapter execution begins, with actionable feedback listing the missing fields, the adapter that requested them, and the command or configuration path needed to resolve them; do not silently fall back, skip adapter-controlled stages, or prompt implicitly.
- Q: Where should the reusable adapter template live and where should local template work happen for this slice? → A: The reusable adapter template lives in the dedicated sibling repository `boundline-framework-template`, and local template work for this slice happens there rather than inside this repository.
- Q: How should built-in Canon behavior, known framework integrations, and adapter registration work in v1? → A: Canon-aware behavior remains the built-in default and does not require an external adapter; Speckit is a known external adapter profile; custom company harnesses are custom external adapters; adapter registration is explicitly configuration-based, created through initialization or adapter-management surfaces, and local executable discovery may assist setup but must not auto-enable an adapter without operator selection.

### Session 2026-05-31

- Q: Where does the Speckit adapter itself live and where should local Speckit adapter work happen? → A: The Speckit adapter is maintained in the dedicated sibling repository `boundline-adapter-speckit`, and local work for that adapter happens in the parent-folder sibling workspace rather than inside this repository.
- Q: What should happen if an operator exits guided adapter setup before all required fields are collected? → A: Guided setup is atomic in the initial release: partial values are not persisted, any existing valid adapter selection remains unchanged, and the operator receives feedback that setup is incomplete plus the command to resume it.
- Q: What does a declared stage override mean after adapter preflight succeeds? → A: A declared override is authoritative stage ownership. Boundline may assemble host-owned context before invoking the adapter, but it must not complete the built-in implementation for that stage first. If the adapter succeeds, the adapter response becomes the stage outcome; if it blocks, the host records the stage as blocked and incomplete; if it fails after claim, the stage fails and the lifecycle stops pending operator intervention.
- Q: What level of behavior must the known Speckit adapter provide in the initial release? → A: The Speckit adapter must act as a real bridge to Speckit workflows rather than a placeholder claimed-stage marker. It must consume host-provided context for declared stages, invoke the appropriate Speckit workflow, return real produced artifacts or actionable blocked and failure outcomes, and remain distinct from the generic template scaffold.

### Session 2026-06-01

- Q: What is the authoritative Boundline-to-Speckit stage map for the corrected feature slice? → A: `goal` remains native Boundline only; a Speckit-claimed `plan` stage owns the full Speckit planning lifecycle (`speckit.specify`, `speckit.clarify` when required, `speckit.plan`, `speckit.tasks`, mandatory `speckit.analyze`, bounded remediation work, and analyze re-checks); a Speckit-claimed `run` stage owns implementation only through `speckit.implement` plus implementation validation or status capture; and `status` and `inspect` remain Boundline-owned visibility surfaces over adapter outputs.
- Q: What workflow identifiers and response fields must the corrected Speckit bridge use? → A: `execute-stage(plan)` must identify workflow ID `speckit-planning`, `execute-stage(run)` must identify workflow ID `speckit-implementation`, and both responses must include explicit command lists, produced artifact refs, and stage-specific findings or validation fields rather than generic summaries alone.
- Q: What execution limits govern analyze and remediation inside a claimed `plan` stage? → A: Adapter-owned stages inherit Boundline host retry and stop controls, and within one claimed `plan` attempt the Speckit bridge may perform at most one initial analyze pass plus two remediation or analyze re-check cycles before it must return a blocked outcome with remaining findings and recovery guidance.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run With Safe Default and Optional Framework Adapter (Priority: P1)

As a repository operator, I can run the orchestration lifecycle with built-in default behavior when no external framework adapter is configured, while also being able to enable a configured adapter without breaking baseline operation.

**Why this priority**: This preserves current out-of-the-box value and prevents adoption risk while enabling extensibility.

**Independent Test**: Can be fully tested by running one lifecycle with no adapter configured and one lifecycle with a valid adapter configured; both complete successfully and produce expected stage outcomes.

**Acceptance Scenarios**:

1. **Given** no adapter is configured, **When** an operator starts a lifecycle run, **Then** all stages execute through built-in Canon-aware default behavior and the run completes without requiring an external adapter installation.
2. **Given** exactly one adapter is configured and available, **When** an operator starts a lifecycle run, **Then** adapter-provided behavior is applied only to that adapter's declared stages and the run completes.

---

### User Story 2 - Selective Stage Overrides (Priority: P2)

As a framework author, I can declare only the lifecycle stages and hooks my adapter wants to control, so I can customize targeted behavior without re-implementing the full lifecycle.

**Why this priority**: Partial override adoption lowers implementation effort and reduces migration cost for existing users.

**Independent Test**: Can be fully tested by registering an adapter that declares a subset of stages and hooks and verifying only those declared points are intercepted.

**Acceptance Scenarios**:

1. **Given** an adapter declares overrides for a subset of stages, **When** a lifecycle run reaches those stages, **Then** the adapter handles those stages and built-in behavior handles all other stages.
2. **Given** an adapter declares overrides for a subset of stages and preflight succeeds, **When** a lifecycle run reaches one of those stages, **Then** the adapter owns that stage as the authoritative execution path, Boundline may prepare context before invocation, and built-in behavior must not complete that same stage first.
3. **Given** the known Speckit profile declares `plan` and `run`, **When** Boundline reaches `goal`, `plan`, `run`, `status`, or `inspect`, **Then** `goal` remains native, `plan` executes the Speckit planning lifecycle, `run` executes Speckit implementation only, and `status` plus `inspect` remain Boundline-owned visibility surfaces.
4. **Given** Speckit owns the `plan` stage, **When** `speckit.analyze` reports blocking findings, **Then** Boundline must not record the `plan` stage as complete until remediation work has been executed and analyze passes again, or the adapter returns a blocked outcome after the bounded remediation limit is reached.
5. **Given** an adapter declares specific lifecycle hooks, **When** matching lifecycle events occur, **Then** the adapter receives those events and unregistered hook events are ignored.
6. **Given** an adapter has taken control of a declared stage, **When** the adapter fails after that stage has started, **Then** the stage is marked failed, the lifecycle run stops, and the operator is required to intervene before execution continues.

---

### User Story 3 - Guided Adapter Configuration (Priority: P3)

As a repository operator, I can complete adapter setup with guided prompts for required settings, so I can safely enable external framework behavior without manual trial-and-error.

**Why this priority**: Guided setup reduces misconfiguration and support burden, especially for first-time integration.

**Independent Test**: Can be fully tested by configuring a new adapter that declares required settings and verifying missing settings are collected before execution.

**Acceptance Scenarios**:

1. **Given** an adapter declares required configuration fields and the run is interactive, **When** required values are missing, **Then** the operator is prompted for missing values before adapter execution.
2. **Given** an operator selects a known external adapter profile or a custom external adapter, **When** setup completes, **Then** the active adapter selection and required launch details are persisted in workspace configuration for subsequent runs.
3. **Given** an adapter declares required configuration fields and the run is non-interactive, **When** required values are missing, **Then** the run fails before adapter execution begins and returns actionable feedback naming the missing fields, the adapter that requested them, and how to resolve them.
4. **Given** guided adapter setup has started and the operator exits before all required values are collected, **When** setup terminates, **Then** no partial adapter configuration is persisted, any previously valid adapter selection remains unchanged, and the operator is told how to resume setup.

### Edge Cases

- Adapter executable is configured but unavailable at runtime.
- Adapter returns an invalid or incomplete capability declaration.
- Adapter returns a command-specific result that is not wrapped in the standard host-visible response envelope.
- Adapter omits supported transport declarations or advertises only transports the initial release does not accept.
- Adapter claims override for unsupported or unknown lifecycle stages.
- Adapter fails during an overridden stage after prior stages completed; the current stage is marked failed and the run stops without mid-stage fallback.
- Adapter emits plain-text or malformed structured stderr; trace ingestion remains best-effort and must not change lifecycle ownership or result classification on its own.
- Adapter advertises a long-running transport that would require explicit shutdown semantics; the initial release must leave that transport unsupported rather than partially activating it.
- Boundline assembles context for an adapter-owned `plan` or `run` stage, but must not finish the built-in stage result first and then treat the adapter as a post-processing side effect.
- The known Speckit adapter returns only placeholder markers or generic scaffold success payloads instead of invoking Speckit and returning real produced artifacts.
- Speckit reaches `speckit.analyze` with blocking findings, and those findings still remain after two remediation or analyze re-check cycles within the same claimed `plan` attempt.
- Speckit tries to execute planning commands from a claimed `run` stage or tries to treat `goal`, `status`, or `inspect` as adapter-owned surfaces.
- Operator supplies partial configuration and exits setup before completion; the system must leave persisted adapter state unchanged and report setup as incomplete.
- A non-interactive run starts with adapter-required configuration missing; the run must fail deterministically without implicit prompts or built-in fallback.
- A locally discoverable adapter executable exists but was never explicitly selected; the system must not auto-enable it.

## Requirements *(mandatory)*

### Normative Boundline-To-Speckit Stage Mapping

| Boundline surface | Owner | Workflow ID | Required command sequence | Minimum artifact classes | Completion rule |
|-------------------|-------|-------------|---------------------------|--------------------------|-----------------|
| `goal` | Boundline built-in only | `boundline-native-goal` | Native Boundline goal capture only | Goal or session context artifacts | The adapter must not claim `goal`. |
| `plan` | Speckit when `plan` is declared and preflight succeeds; otherwise Boundline built-in | `speckit-planning` | `speckit.specify`; `speckit.clarify` when required; `speckit.plan`; `speckit.tasks`; mandatory `speckit.analyze`; remediation work when blocking findings exist; analyze re-check after each remediation cycle | Specification artifact, plan artifact, tasks artifact, planning-readiness artifact | The stage is complete only when analyze has no blocking findings. It stays blocked or fails otherwise. |
| `run` | Speckit when `run` is declared and preflight succeeds; otherwise Boundline built-in | `speckit-implementation` | `speckit.implement` plus implementation validation or status capture only | Implementation artifact and validation or status artifact | The stage must not rerun planning commands or planning-readiness analysis. |
| `status` / `inspect` | Boundline built-in only | `boundline-native-visibility` | Native Boundline status and inspect surfaces over adapter evidence | Audit, trace, ownership, findings, and validation visibility artifacts | The adapter may contribute artifacts, but it does not own the visibility surface. |

### Functional Requirements

- **FR-001**: The system MUST execute the full lifecycle with built-in default behavior when no external adapter is configured.
- **FR-002**: The system MUST allow operators to declare an external adapter command in configuration.
- **FR-003**: The system MUST discover adapter capabilities, including supported transports, before lifecycle execution and determine declared stage overrides and hook subscriptions.
- **FR-004**: The system MUST apply adapter behavior only to the stages explicitly declared by the adapter, MUST preserve built-in behavior for undeclared stages, and MUST treat a declared override with successful preflight as authoritative ownership of that stage rather than post-processing after built-in completion.
- **FR-005**: The system MUST collect any missing adapter-required configuration values through a guided operator workflow before first execution when the run is interactive.
- **FR-006**: The system MUST pass the resolved adapter configuration and current run context to adapter executions.
- **FR-007**: The system MUST detect malformed capability declarations and block adapter activation with actionable operator feedback.
- **FR-008**: The system MUST record adapter involvement per lifecycle stage for auditability.
- **FR-009**: The system MUST support adapter lifecycle hook subscriptions for declared events and ignore undeclared events.
- **FR-010**: The system MUST fail safely by returning control to built-in lifecycle behavior when adapter discovery or activation fails before any stage ownership is claimed.
- **FR-011**: The system MUST mark the current stage as failed, stop the lifecycle run, and require operator intervention when an adapter fails after an overridden stage has already started.
- **FR-012**: The system MUST treat a configured adapter in the initial release as an explicitly configured, operator-trusted local subprocess and MUST NOT require a separate permission mediation layer before executing declared overrides or hooks.
- **FR-013**: The system MUST support exactly one active adapter per lifecycle run in the initial release and MUST reject concurrent multi-adapter execution within the same run.
- **FR-014**: The system MUST fail before adapter execution begins in non-interactive runs when adapter-required configuration is missing and MUST return actionable feedback listing the missing fields, the adapter that requested them, and the command or configuration path needed to resolve them.
- **FR-015**: The system MUST NOT silently fall back to built-in behavior, skip adapter-controlled stages, or prompt implicitly when a configured adapter is missing required configuration during a non-interactive run.
- **FR-016**: The system MUST keep Canon-aware built-in behavior available in the initial release without requiring an external adapter.
- **FR-017**: The system MUST provide an explicit operator-controlled adapter registration and activation path through initialization or adapter-management surfaces.
- **FR-018**: The system MUST support known setup profiles for Speckit and for custom external adapters in the initial release.
- **FR-019**: The system MUST NOT auto-enable an adapter solely because its executable is locally discoverable; explicit operator selection remains required.
- **FR-020**: The system MUST persist adapter registration in workspace configuration and use that configuration as the authoritative source for active adapter selection.
- **FR-021**: The system MUST treat guided adapter setup as atomic in the initial release by leaving persisted adapter selection and configuration unchanged when the operator exits before all required values are collected, and it MUST report how to resume setup.
- **FR-022**: The system MUST use a consistent host-visible success/error response envelope for all V1 stdio interactions with an adapter and MUST preserve command-specific domain outcomes within that envelope rather than using ad hoc top-level payload shapes.
- **FR-023**: The system MUST require the adapter capability declaration returned by `describe` to list supported transport(s) and MUST accept JSON over stdin/stdout as the bounded V1 transport while leaving room for future transport declarations.
- **FR-024**: The system MAY capture optional structured stderr diagnostics emitted by an adapter into Boundline trace records, but V1 MUST NOT require adapters to implement structured stderr or a long-running logging subsystem.
- **FR-025**: The system MUST keep graceful shutdown and other long-running transport lifecycle concerns out of scope for the initial release, because the V1 one-shot subprocess model avoids orphan-process concerns without additional lifecycle management.
- **FR-026**: Once a configured adapter has claimed a declared stage, Boundline MAY assemble host-owned context before invocation but MUST NOT complete the built-in implementation for that stage before the adapter returns.
- **FR-027**: When a claimed-stage adapter invocation succeeds, the adapter response, including `produced_artifacts`, MUST become the authoritative stage outcome recorded by the host; when a claimed-stage adapter invocation returns a blocked outcome, the host MUST record the stage as blocked and incomplete rather than marking it completed through built-in behavior.
- **FR-028**: The known Speckit adapter profile in this feature MUST act as a real bridge to Speckit workflows for its declared stages and MUST NOT satisfy acceptance only by returning placeholder marker files or generic scaffold success payloads.
- **FR-029**: The known Speckit adapter profile MUST follow the normative stage mapping in this feature: `goal` remains native Boundline only, a claimed `plan` stage owns the full Speckit planning lifecycle, a claimed `run` stage owns implementation only, and `status` plus `inspect` remain Boundline-owned visibility surfaces.
- **FR-030**: When Speckit claims `plan`, `execute-stage(plan)` MUST identify workflow ID `speckit-planning` and MUST execute these command surfaces in order: `speckit.specify`, `speckit.clarify` when missing-context or readiness checks require clarification, `speckit.plan`, `speckit.tasks`, mandatory `speckit.analyze`, remediation task execution when analyze reports blocking findings, and an analyze re-check after each remediation cycle until the stage passes, blocks, or fails.
- **FR-031**: `speckit.analyze` MUST be a mandatory planning-readiness gate for a Speckit-owned `plan` stage. Boundline MUST NOT consider `plan` complete while analyze has unresolved blocking findings, and the adapter MUST either execute bounded remediation work and re-run analyze or return a blocked outcome with remaining findings and recovery guidance.
- **FR-032**: `execute-stage(plan)` MUST return a success-envelope payload containing at least `status`, `summary`, `workflow_id`, `executed_commands`, `produced_artifacts`, `planning_findings`, `remediation_status`, `analyze_pass_count`, `remediation_cycles_used`, and `next_action`. When `status = succeeded`, `produced_artifacts` MUST include at least one specification artifact reference, one plan artifact reference, one task breakdown artifact reference, and one planning-readiness artifact reference.
- **FR-033**: When Speckit claims `run`, `execute-stage(run)` MUST identify workflow ID `speckit-implementation`, MUST invoke `speckit.implement` plus implementation validation or status capture only, and MUST NOT invoke `speckit.specify`, `speckit.clarify`, `speckit.plan`, `speckit.tasks`, or `speckit.analyze` from the claimed `run` stage.
- **FR-034**: `execute-stage(run)` MUST return a success-envelope payload containing at least `status`, `summary`, `workflow_id`, `executed_commands`, `produced_artifacts`, `implementation_status`, `validation_refs`, and `next_action`. When `status = succeeded`, `produced_artifacts` MUST include at least one implementation artifact reference and one validation or status artifact reference.
- **FR-035**: Adapter-owned stages MUST inherit Boundline's existing host retry and stop controls. Within one claimed `plan` stage attempt, the Speckit bridge MUST perform at most one initial analyze pass and no more than two remediation or analyze re-check cycles; if blocking findings still remain after the second re-check, the adapter MUST return `status = blocked` with the remaining findings and recovery guidance rather than completing the stage.
- **FR-036**: Boundline-owned `status` and `inspect` surfaces MUST expose adapter stage ownership, workflow IDs, produced artifact refs, planning findings summaries, remediation loop counts, validation refs, and the reason a claimed stage is succeeded, blocked, or failed without delegating those visibility responsibilities to the adapter.

### Key Entities *(include if feature involves data)*

- **Built-in Canon-aware Behavior**: The default lifecycle behavior shipped with Boundline that remains available without any external adapter registration.
- **Adapter Registration**: Operator-provided declaration of the external adapter identity and launch command.
- **Adapter Capability Profile**: Adapter-declared metadata describing supported transport(s), stage overrides, hook subscriptions, and required configuration fields.
- **Adapter Configuration Set**: Resolved adapter-specific settings, including operator-supplied values for required fields.
- **Protocol Response Envelope**: The standard host-visible wrapper that separates protocol success or error from command-specific outcomes such as blocked preflight results or failed claimed stages.
- **Optional Adapter Diagnostic Event**: A structured stderr line that an adapter may emit for trace ingestion without becoming a required part of the V1 contract.
- **Known Adapter Profile**: A named external adapter setup path, such as Speckit, that simplifies registration compared with fully custom adapter configuration.
- **Speckit Adapter Repository**: The dedicated external repository `boundline-adapter-speckit` that hosts the Speckit adapter implementation separately from Boundline core.
- **Adapter Template Repository**: The dedicated reusable template repository, `boundline-framework-template`, where starter adapter scaffolding is maintained separately from this repository.
- **Speckit Planning Workflow**: The corrected workflow surface with workflow ID `speckit-planning` that is bound to the claimed Boundline `plan` stage and runs the full Speckit planning lifecycle plus the mandatory planning-readiness gate.
- **Planning Readiness Finding**: A normalized Speckit analyze result item classified as blocking or non-blocking for the claimed `plan` stage outcome.
- **Remediation Cycle Record**: A bounded record of one remediation attempt plus the corresponding analyze re-check executed during the claimed `plan` stage.
- **Speckit Implementation Workflow**: The corrected workflow surface with workflow ID `speckit-implementation` that is bound to the claimed Boundline `run` stage and runs implementation-only behavior through `speckit.implement` plus validation or status capture.
- **Lifecycle Stage Execution Record**: Per-stage run record indicating whether built-in or adapter behavior was used and the resulting status.
- **Hook Event Record**: Structured record of declared hook events delivered to the adapter and their outcomes.
- **Claimed Stage Outcome Record**: The authoritative persisted outcome for one adapter-owned stage, including adapter status, produced artifacts, blocked or failure state, and any host-owned context references used before invocation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of lifecycle runs without adapter configuration complete using built-in behavior with no additional setup steps required.
- **SC-002**: In guided validation testing, 100% of first-time `speckit` and custom-adapter registration flows collect the required fields and persist a runnable configuration without manual edits to workspace config files.
- **SC-003**: In validation testing, 100% of declared stage overrides are applied only to declared stages, with zero unintended overrides of undeclared stages.
- **SC-004**: In controlled failure testing, 100% of adapter discovery failures are surfaced with actionable operator feedback before stage execution begins.
- **SC-005**: In acceptance testing, audit records identify lifecycle stage execution source (built-in vs adapter) for 100% of completed stages.
- **SC-006**: In non-interactive validation testing, 100% of runs with missing adapter-required configuration fail before adapter execution begins and identify the missing fields and recovery path in operator-visible feedback.
- **SC-007**: In validation testing, 100% of runs without an explicitly selected external adapter continue to use built-in Canon-aware behavior even when adapter executables are locally discoverable.
- **SC-008**: In contract validation, 100% of supported V1 adapter commands expose outcomes through the same host-visible success/error response structure, so operators receive consistent success and failure reporting across capability discovery, preflight, stage execution, and hook delivery.
- **SC-009**: In authoritative-routing validation, 100% of successful claimed `plan` and `run` stages are recorded from adapter outcomes, and 0 claimed stages are first completed by built-in behavior before adapter invocation.
- **SC-010**: In cross-repo Speckit validation, 100% of successful Speckit-claimed stages return at least one real Speckit-produced artifact or Speckit-authored artifact reference rather than only placeholder marker files.
- **SC-011**: In stage-mapping validation, 100% of successful Speckit-owned `plan` stages report workflow ID `speckit-planning`, execute the planning command sequence required by this feature, and never complete while blocking analyze findings remain unresolved.
- **SC-012**: In stage-mapping validation, 100% of successful Speckit-owned `run` stages report workflow ID `speckit-implementation`, invoke `speckit.implement` plus implementation validation or status capture only, and invoke none of `speckit.specify`, `speckit.clarify`, `speckit.plan`, `speckit.tasks`, or `speckit.analyze`.
- **SC-013**: In bounded-remediation validation, 100% of claimed `plan` stage runs with persistent blocking findings stop with a blocked outcome after no more than two remediation or analyze re-check cycles and surface the remaining findings plus `next_action` feedback.
- **SC-014**: In status and inspect validation, 100% of adapter-owned `plan` and `run` stages surface workflow ID, ownership, produced artifact refs, findings or validation summaries, and blocked or failed reasons through Boundline-owned visibility surfaces.

## Assumptions

- Operators enabling adapters have permission to configure local workspace settings.
- Canon remains built-in default behavior in the initial release and is not packaged as an external adapter for this slice.
- The initial release treats configured adapters as trusted local binaries; provider-style permission mediation is out of scope for this slice.
- The initial release scope covers one active adapter per workspace lifecycle run.
- The initial release does not support composing or chaining multiple adapters within the same lifecycle run.
- Speckit is treated as a known external adapter profile in the initial release, while company-specific harnesses remain custom external adapters.
- The Speckit adapter implementation is maintained in the sibling repository `boundline-adapter-speckit`, and local Speckit adapter work for this slice happens there rather than in this repository.
- The generic template scaffold and the known Speckit adapter are not interchangeable acceptance targets; the template may stay placeholder, while the Speckit adapter must bridge real Speckit workflow behavior for claimed stages.
- The reusable starter template for adapters is maintained in the sibling repository `boundline-framework-template`, and local template work for this slice happens there rather than in this repository.
- Workspace configuration is the authoritative source of active adapter selection; local discovery is only a setup aid.
- External adapters are distributed and versioned outside this repository.
- Existing built-in lifecycle behavior remains the baseline contract for stages not overridden.
- Boundline-owned workflow-definition assets for the corrected Speckit bridge may live under `.specify/workflows/speckit/`, but those assets remain Boundline-controlled integration surfaces rather than adapter-owned source of truth.
- Interactive runs may collect missing adapter-required values through guided prompts, but non-interactive runs must fail deterministically instead of prompting.
- The V1 adapter contract uses a standard host-visible success/error envelope while preserving command-specific domain outcomes inside successful response data.
- Adapter capability declarations list supported transports explicitly, and the initial release accepts only JSON over stdin/stdout.
- Structured stderr remains optional best-effort observability in the initial release rather than a required adapter feature.
- Graceful shutdown and other lifecycle controls for long-running or persistent transports are deferred beyond the initial release because the one-shot subprocess model keeps V1 bounded.
