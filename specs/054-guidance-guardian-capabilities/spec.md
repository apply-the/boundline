# Feature Specification: Guidance And Guardian Capabilities

**Feature Branch**: `054-guidance-guardian-capabilities`  
**Created**: 2026-05-14  
**Status**: Draft  
**Input**: User description: "Introduce guidance and guardian runtime capabilities for Boundline, operationalizing engineering principles as executable guidance that shapes work before action and guardians that validate work after action, with Canon-aware but not Canon-dependent calibration, deterministic-before-LLM verification, structured findings, and lifecycle integration across planning, architecture, implementation, testing, and review stages"

## Canon-Aware, Not Canon-Dependent

S2.1 is primarily a Boundline runtime specification.

Canon-governed standards are the highest-authority calibration source for guidance and guardians, but Boundline can still execute guidance and guardian capabilities from workspace overrides, shared expert packs, built-in capabilities, and deterministic tools when Canon artifacts are absent.

When Canon artifacts are available, Boundline MUST prefer them for governed interpretation. When they are absent, Boundline MUST disclose that the guidance or guardian result is based on local, pack-provided, or built-in sources rather than Canon-governed authority.

This specification consumes Canon-governed standards when available, but does not define Canon publication or promotion semantics.

### Resolution Strength

Resolution strength, highest to lowest:

1. Runtime evidence for the active task
2. Workspace overrides (`.boundline/guidance/`, `.boundline/guardians/`)
3. Canon-governed standards (artifacts from Canon)
4. Shared expert packs (installed packs)
5. Boundline built-in capabilities

Canon-governed standards are the highest external governed authority. Workspace overrides may override them only as local repository policy and must be trace-visible.

## Guidance Source Catalog

Boundline MUST discover and consume guidance sources for:

- language best practices
- framework best practices
- testing framework best practices
- clean code guidelines
- software design principles
- architecture patterns
- domain modeling rules
- error-handling conventions

These sources may come from:

1. Workspace overrides
2. Canon-governed standards
3. Shared expert packs
4. Boundline built-ins

Language and framework guidance is in scope for S2.1. Model catalog and provider readiness are out of scope.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Load And Resolve Guidance During A Bounded Delivery Session (Priority: P1)

As a Boundline operator running a bounded delivery session, I want Boundline to load guidance capabilities from the active expert pack, workspace overrides, and available Canon-governed standards, then make that guidance available to experts during planning, architecture, and implementation steps, so that engineering principles shape work before execution rather than existing only as passive documentation.

**Why this priority**: Without guidance loading and resolution, guardians have nothing to verify against, and engineering standards remain inert documents. This is the foundation for every other capability in the spec.

**Independent Test**: Can be fully tested by preparing a workspace with an expert pack that declares guidance entries, workspace-override guidance files under `.boundline/guidance/`, and optionally Canon-governed standard artifacts, then running a bounded session and verifying that the resolved guidance set follows the declared resolution-strength precedence and is surfaced in session traces.

**Acceptance Scenarios**:

1. **Given** a workspace with an expert pack declaring guidance entries and workspace-override guidance files, **When** Boundline starts a bounded delivery session, **Then** it resolves guidance from all available sources following the resolution-strength precedence and makes the resolved set available to expert roles during execution.
2. **Given** guidance entries from both a workspace override and a Canon-governed standard for the same concern, **When** Boundline resolves guidance, **Then** the workspace override takes precedence and the resolution trace records both sources and the override decision.
3. **Given** a workspace with no Canon-governed standards available, **When** Boundline resolves guidance, **Then** it operates with workspace-override, pack-provided, and built-in guidance and discloses in the trace that Canon-governed authority is absent.

---

### User Story 2 - Execute Guardian Checks And Emit Structured Findings (Priority: P1)

As a Boundline operator completing a bounded delivery step, I want Boundline to execute guardian checks against the work produced in that step and emit structured findings that explain what rule triggered, why it matters, where evidence exists, and how to resolve the concern, so that I can act on verification results without guesswork.

**Why this priority**: Structured findings are the core output of the guardian capability. Without them, guidance remains advisory-only and the feedback loop from S2.1 (observe, guide, act, verify, emit findings, govern) is incomplete.

**Independent Test**: Can be fully tested by configuring a guardian in the active expert pack, producing work that violates the guardian's rule, running the guardian check, and verifying that the emitted finding contains all required structured fields and is recorded in the session trace.

**Acceptance Scenarios**:

1. **Given** a configured guardian and work that violates its rule, **When** the guardian executes, **Then** it emits a structured finding with guardian identity, rule identity, disposition, summary, evidence references, confidence level, and recommended action.
2. **Given** a guardian that cannot complete its check due to missing context or a tool failure, **When** execution fails, **Then** Boundline records the failure as an explicit guardian-error finding with the reason, does not silently skip the check, and surfaces the failure in the session trace.
3. **Given** multiple guardians configured for the same lifecycle phase, **When** Boundline executes guardian checks for that phase, **Then** it runs each guardian sequentially, collects all findings, and reports the aggregate result without exceeding configured execution limits.

---

### User Story 3 - Deterministic-Before-LLM Guardian Execution (Priority: P2)

As a Boundline operator, I want deterministic guardian checks (static analysis, linting, AST rules, architecture tests, scripts) to run before LLM-based semantic guardians, so that deterministic evidence is available to augment LLM reasoning and avoidable LLM invocations are skipped when deterministic checks already produce blocking findings.

**Why this priority**: Running deterministic checks first reduces cost, improves speed, and provides concrete evidence that LLM guardians can reference. This ordering is a core design principle of S2.1 but depends on guardian execution (US2) being functional first.

**Independent Test**: Can be fully tested by configuring both a deterministic guardian and an LLM guardian for the same phase, running guardian checks, and verifying from the execution trace that the deterministic guardian completed before the LLM guardian started, and that blocking deterministic findings prevented unnecessary LLM invocations.

**Acceptance Scenarios**:

1. **Given** a deterministic guardian and an LLM guardian both configured for the same lifecycle phase, **When** Boundline executes guardian checks, **Then** the deterministic guardian runs first and the LLM guardian runs after, as recorded in the execution trace.
2. **Given** a deterministic guardian that produces a blocking finding, **When** Boundline evaluates whether to proceed to LLM guardians, **Then** it skips LLM guardians for that phase when the blocking finding makes further checks redundant, and records the skip reason in the trace.
3. **Given** a hybrid guardian that combines deterministic evidence with LLM reasoning, **When** Boundline executes the hybrid guardian, **Then** it runs the deterministic component first, passes the evidence to the LLM component, and records both stages in the trace.

---

### User Story 4 - Workspace Injection Of Custom Guidance And Guardians (Priority: P2)

As a repository maintainer, I want to inject custom guidance and guardian definitions into my workspace without modifying shared expert packs, so that project-specific engineering rules are enforced alongside shared pack capabilities.

**Why this priority**: Custom workspace injection enables teams to adopt Boundline incrementally by adding project-specific rules while retaining shared pack capabilities. This is essential for real-world adoption but depends on the core guidance and guardian loading (US1, US2) being functional first.

**Independent Test**: Can be fully tested by placing custom guidance files under `.boundline/guidance/` and custom guardian definitions under `.boundline/guardians/`, running a bounded session, and verifying that workspace-injected capabilities appear in the resolved set and execute correctly alongside pack-provided capabilities.

**Acceptance Scenarios**:

1. **Given** custom guidance Markdown files under `.boundline/guidance/` and custom guardian TOML definitions under `.boundline/guardians/`, **When** Boundline resolves capabilities for a session, **Then** the workspace-injected entries appear in the resolved set and take precedence over shared-pack and built-in entries for the same concern.
2. **Given** a workspace guardian definition with an invalid or unsupported schema, **When** Boundline attempts to load it, **Then** it reports a structured load error, skips the invalid entry, and continues loading remaining valid entries without crashing.
3. **Given** a workspace with no `.boundline/guidance/` or `.boundline/guardians/` directories, **When** Boundline resolves capabilities, **Then** it proceeds with pack-provided, Canon-governed, and built-in sources without error.

---

### User Story 5 - Guidance And Guardian Lifecycle Integration (Priority: P3)

As a Boundline operator running a multi-step bounded delivery session that spans planning, architecture, implementation, testing, and review, I want guidance to influence the appropriate phases and guardians to execute at the appropriate verification points, so that engineering principles are applied at the right time rather than all at once.

**Why this priority**: Lifecycle integration is what turns guidance and guardians from a batch check into a contextual delivery capability. It depends on all prior stories being functional and is the integration story that connects S2.1 to the existing session-native orchestrator.

**Independent Test**: Can be fully tested by running a bounded session that includes at least a planning and implementation phase, configuring guidance that applies only to planning and a guardian that applies only to implementation, and verifying from the session trace that guidance was consumed during planning and the guardian executed during implementation.

**Acceptance Scenarios**:

1. **Given** guidance declared with `applies_to = ["planning"]` and a guardian declared with `applies_to = ["implementation", "review"]`, **When** a session executes a planning step followed by an implementation step, **Then** the guidance influences the planning step and the guardian executes after the implementation step, as recorded in the trace.
2. **Given** a guardian declared for the `review` phase but the session does not include a review step, **When** the session completes, **Then** the guardian is not invoked and no finding is emitted for it.
3. **Given** a session where a guardian produces findings during the implementation phase, **When** the session proceeds to a subsequent review phase, **Then** the prior guardian findings are available as context for reviewers.

---

### Edge Cases

- A guidance file referenced by a pack manifest does not exist on disk: Boundline records a structured load warning, excludes the missing guidance from the resolved set, and continues execution.
- A deterministic guardian command exits with a non-zero code but produces no structured output: Boundline treats the raw exit code and stderr as a guardian-error finding with disposition `error`.
- All configured guardians produce `concern` findings but no `error` findings: Boundline does not block execution and surfaces findings as advisory in the session trace.
- A workspace override and a Canon-governed standard define guidance for the same concern with conflicting content: Boundline applies the workspace override per resolution-strength precedence and records the conflict in the trace.
- A guardian is configured for a lifecycle phase that the current session does not include: the guardian is never invoked and no finding is emitted.
- An LLM guardian invocation exceeds the configured timeout or token limit: Boundline records a guardian-timeout finding and does not retry the LLM invocation by default.
- No suitable runtime route is available for an LLM or hybrid guardian: Boundline degrades explicitly, records why the guardian could not run, and does not silently substitute a different authority source.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST load guidance capabilities from expert pack manifests, workspace overrides (`.boundline/guidance/`), Canon-governed standards, and built-in sources, resolving conflicts according to the declared resolution-strength precedence.
- **FR-002**: System MUST load guardian capabilities from expert pack manifests, workspace overrides (`.boundline/guardians/`), Canon-governed standards, and built-in sources, resolving conflicts according to the declared resolution-strength precedence.
- **FR-003**: System MUST support three guardian kinds: `deterministic` (static analysis, scripts, linters), `llm` (semantic LLM reasoning), and `hybrid` (deterministic evidence combined with LLM reasoning).
- **FR-004**: System MUST execute deterministic guardian checks before LLM-based guardian checks within the same lifecycle phase and MUST skip LLM guardians when deterministic checks produce blocking findings that make further checks redundant.
- **FR-005**: System MUST emit structured findings from guardian checks containing at minimum: guardian identity, rule identity, disposition (advise, warn, concern, error, block), summary, evidence references, confidence level, and recommended action.
- **FR-006**: System MUST record all guidance resolution decisions, guardian executions, and emitted findings in the session trace for inspectability.
- **FR-007**: System MUST handle guardian execution failures (tool crash, timeout, missing context) as explicit guardian-error findings rather than silently skipping the check.
- **FR-008**: System MUST respect lifecycle phase declarations (`applies_to`) on guidance and guardian manifests, invoking capabilities only during their declared phases.
- **FR-009**: System MUST disclose the authority source (workspace override, Canon-governed, shared pack, built-in) for each resolved guidance and guardian capability in the session trace.
- **FR-010**: System MUST avoid agent explosion by supporting grouped guardian capabilities (e.g., one SOLID guardian covering SRP, OCP, LSP, ISP, DIP) rather than requiring separate agents per principle.
- **FR-011**: System MUST support workspace injection of custom guidance (`.boundline/guidance/*.md`) and custom guardians (`.boundline/guardians/*.toml`) without requiring modifications to shared expert packs.
- **FR-012**: System MUST surface guardian findings to downstream consumers (S3 councils and S4 governance) through the existing session and trace surfaces.
- **FR-013**: System MUST apply configured execution limits (maximum guardian count per phase, timeout per guardian invocation) and stop guardian execution explicitly when limits are reached.
- **FR-014**: System MUST support language-specific guidance files such as Rust, TypeScript, Python, Java, Go, and C# best practices when declared by packs, workspace overrides, or Canon-governed standards.
- **FR-015**: System MUST support framework-specific guidance files such as React, Spring, Django, Rails, Next.js, and testing-framework guidance when declared by packs, workspace overrides, or Canon-governed standards.
- **FR-016**: System MUST disclose which language-specific, framework-specific, testing-framework, and clean-code guidance sources were loaded and which were skipped.
- **FR-017**: System MUST use existing Boundline runtime routing for LLM and hybrid guardian invocations and MUST degrade explicitly when no suitable route is available.

### Scope Boundaries *(mandatory)*

- **In Scope**: guidance loading and resolution from multiple sources; guardian execution with deterministic-before-LLM ordering; structured finding emission; workspace injection of custom rules; lifecycle phase integration for planning, architecture, implementation, testing, and review; Canon-aware resolution with source authority disclosure; session trace recording of all guidance and guardian activity; execution limits for guardian invocations.
- **Out of Scope**: council or voting systems for guardian findings (deferred to S3); governance escalation or trust degradation based on findings (deferred to S4); Canon publication or promotion semantics (Canon-owned); distributed guardian execution across workspaces; model catalog currency and provider readiness management; UI or dashboard surfaces for findings; custom guardian SDK or plugin API beyond TOML manifest and script invocation.

### Key Entities

- **Guidance Capability**: A structured engineering knowledge entry declared in a pack manifest or workspace override, with metadata including title, applicable lifecycle phases, consuming roles, content path, and priority. Guidance influences expert behavior during execution but does not enforce or block.
- **Guardian Capability**: A verification capability declared in a pack manifest or workspace override, with metadata including title, kind (deterministic, llm, hybrid), applicable lifecycle phases, rules, severity floor, and either a command (deterministic) or instruction (llm). Guardians emit structured findings after execution.
- **Structured Finding**: The output of a guardian check, containing guardian identity, rule identity, disposition, summary, evidence references, confidence level, and recommended action. Findings are recorded in the session trace and may be consumed by S3 and S4.
- **Resolution Trace**: A record of how guidance and guardian capabilities were resolved from available sources, including which sources were considered, which entries were selected, which were overridden, and which authority source applied.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Expert packs can declare guidance and guardian capabilities for at least the software design, testing, and architecture engineering pillars, and Boundline resolves them correctly during bounded sessions.
- **SC-002**: 100% of guardian executions produce either a structured finding or an explicit guardian-error finding; no guardian check completes silently without output.
- **SC-003**: Developers can identify which guidance influenced a delivery step and which guardian findings were emitted by inspecting the session trace in under 5 minutes.
- **SC-004**: Deterministic guardians always complete before LLM guardians within the same lifecycle phase, as verifiable from execution trace timestamps.
- **SC-005**: Workspace overrides for guidance and guardians take effect without modifying shared expert packs and are visible in the resolution trace.
- **SC-006**: When Canon-governed standards are absent, Boundline still executes guidance and guardian capabilities from remaining sources and discloses the absence in the trace.

## Assumptions

- The existing expert pack infrastructure from S2 (Domain Expert Packs And Runtime Role Selection) provides the pack manifest format (`pack.toml`) and pack loading mechanism that this spec extends with guidance and guardian declarations.
- Canon-governed standard artifacts follow a stable, discoverable path convention (e.g., `clean-code-guidelines.md`, `language-best-practices/`, `framework-best-practices/`, `testing-framework-best-practices/`) that Boundline can resolve without Canon-specific runtime APIs.
- Deterministic guardians are invoked as external commands (shell scripts, binaries, linters) and communicate results through structured stdout or exit codes; Boundline does not embed third-party static analysis tools directly.
- LLM guardians use the existing Boundline runtime routing for model selection, and route availability failures surface as explicit degraded outcomes instead of silent fallback behavior.
- The engineering pillars referenced in the S2.1 roadmap draft (software design, domain modeling, error handling, language idioms, testing, UX, architecture) represent the initial content catalog; the first implementation slice does not require all pillars to be fully populated with guardian implementations.
- Guardian execution is sequential within a phase; parallel guardian execution is out of scope for this slice.
