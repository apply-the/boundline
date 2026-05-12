# Feature Specification: Boundline Project-Scale Delivery UX

**Feature Branch**: `049-project-scale-delivery-ux`  
**Created**: 2026-05-11  
**Status**: Draft  
**Input**: User description: "Design and specify the next product slice that makes Boundline a project-scale delivery orchestrator, not a small-task-only CLI."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Global Assistant Bootstrap (Priority: P1)

A developer opens Claude Code, Codex, Cursor, Copilot, Gemini, or another supported host in a repository that has not been initialized for Boundline. They can still discover Boundline through a user-scoped or global assistant package, run `/boundline:init` or receive exact CLI guidance, validate installation health, and continue into a normal session-native Boundline workflow after initialization.

**Why this priority**: Repo-local assistant command packs cannot solve first-run bootstrap because those commands do not exist until after `boundline init`. Without a global bootstrap path, chat-native Boundline remains discoverable only after the user already knows the CLI.

**Independent Test**: Start in a repository with no `.boundline/` directory and a supported host package installed at user scope. Invoke `/boundline:init`, `/boundline:doctor`, `/boundline:continue`, and `/boundline:status`; verify that the commands do not require repo-local state and that they either run Boundline CLI or provide exact copyable CLI commands.

**Acceptance Scenarios**:

1. **Given** a supported host chat is opened in an uninitialized repository, **When** the developer invokes `/boundline:init`, **Then** Boundline detects or asks for the workspace, explains that repo-local state is absent, and runs or provides the exact initialization command.
2. **Given** an uninitialized repository, **When** the developer invokes `/boundline:doctor`, **Then** Boundline reports installation readiness, Canon pairing status, workspace readiness, and the next concrete repair or init command.
3. **Given** no `.boundline/session.json` exists, **When** the developer invokes `/boundline:continue`, **Then** Boundline does not invent state from chat history and instead offers `/boundline:init`, `/boundline:deliver`, or the equivalent CLI command.
4. **Given** a host cannot support true global command installation, **When** the developer reads host setup docs, **Then** the docs state the limitation explicitly and provide the closest supported manual or CLI-first approach.

---

### User Story 2 - Idea-To-Code Delivery Path (Priority: P1)

A developer provides a broad idea or brief, such as "Build a customer onboarding capability with audit logging." Boundline proposes a bounded project-scale path that may include discovery, requirements, system-shaping, architecture, backlog, implementation slices, verification, and review, then proceeds one confirmed stage or work unit at a time.

**Why this priority**: Boundline's current product position covers bounded engineering work from idea intake to verified code changes. Broad initiatives need decomposition, not one unchecked autonomous run.

**Independent Test**: Provide a high-level idea with incomplete product and system context. Verify that Boundline proposes a staged path, asks for confirmation before material stage transitions, stops on insufficient context, and never claims the full project can be completed in one unchecked run.

**Acceptance Scenarios**:

1. **Given** a broad idea with unclear problem framing, **When** the developer asks Boundline to deliver it, **Then** Boundline proposes discovery before requirements or implementation.
2. **Given** product scope is unclear, **When** Boundline prepares the path, **Then** requirements appears before architecture, backlog, or implementation.
3. **Given** architecture or capability boundaries are material, **When** Boundline prepares the path, **Then** system-shaping and architecture stages can appear before backlog and implementation slices.
4. **Given** a proposed path includes implementation work, **When** execution begins, **Then** each implementation or refactor slice has its own bounded goal, checkpoint, validation expectation, trace, and next action.
5. **Given** context is insufficient for the next stage, **When** Boundline evaluates the path, **Then** it stops with a clarification or context-repair action instead of proceeding.

---

### User Story 3 - Explicit Governed Stage Work (Priority: P2)

A developer asks for a governed architecture, backlog, security-assessment, migration, supply-chain-analysis, or pr-review stage. Boundline validates the requested mode against Canon capabilities, routes the stage through Canon only at the governed boundary, and persists the resulting packet, approval, provenance, and next action through Boundline session state.

**Why this priority**: Boundline must support the full Canon mode set without becoming a Canon alias layer. `/boundline:govern` is the clear chat-native surface for explicit governed stage work.

**Independent Test**: Invoke `/boundline:govern` with each current Canon mode and with no mode. Verify that supported modes route through Boundline-governed stage handling, unavailable modes fail explicitly, and missing input or approval state blocks continuation with actionable guidance.

**Acceptance Scenarios**:

1. **Given** Canon capabilities include `architecture`, **When** the developer invokes `/boundline:govern architecture`, **Then** Boundline routes an architecture governed stage through Canon and records the governed packet refs in Boundline state.
2. **Given** the developer invokes `/boundline:govern` without a mode, **When** Boundline can infer a likely mode, **Then** it proposes the mode and asks for confirmation when risk or materiality requires it.
3. **Given** Canon capabilities do not include the requested mode, **When** the developer invokes `/boundline:govern <mode>`, **Then** Boundline stops with unsupported-mode guidance and does not silently fall back.
4. **Given** a governed stage is approval-gated, **When** Canon reports awaiting approval, **Then** Boundline surfaces approval state and the next action in status, next, and inspect output.

---

### User Story 4 - Voting At Risky Quality Boundaries (Priority: P2)

A high-risk architecture decision, validation-exhausted implementation slice, security finding, migration cutover, incident follow-up, supply-chain critical finding, or PR-ready diff triggers multi-reviewer voting. Low-risk local changes proceed without unnecessary voting unless the operator explicitly requests it.

**Why this priority**: Voting improves quality only when tied to material risk and evidence. Project-scale delivery needs review escalation at risky boundaries without turning every step into a committee.

**Independent Test**: Configure representative high-risk and low-risk stages. Verify that voting triggers for high-risk architecture, validation-exhausted implementation, and PR-ready diff scenarios, and that low-risk local implementation proceeds without voting by default.

**Acceptance Scenarios**:

1. **Given** an architecture stage is classified as high structural impact, **When** Boundline reaches the stage boundary, **Then** multi-reviewer voting is required before proceeding.
2. **Given** an implementation slice exhausts validation retries, **When** Boundline evaluates recovery, **Then** voting can be triggered with reviewer findings, vote result, and adjudication state persisted.
3. **Given** a PR-ready diff is available, **When** Boundline reaches pr-review, **Then** reviewer findings and vote resolution are visible in status, next, and inspect.
4. **Given** a low-risk local refactor has preserved-behavior evidence, **When** Boundline evaluates the next action, **Then** voting is skipped unless explicitly requested by policy or operator.

---

### User Story 5 - Delivery Pilot Model Documentation (Priority: P3)

A new user reads Boundline documentation and understands how Boundline supports large initiatives without unbounded autonomy: broad work is decomposed into bounded stages and bounded work units, and each loop observes evidence, decides the next bounded action, acts, verifies, and updates context.

**Why this priority**: The project-scale UX must be explained in product language so users do not expect a one-shot autonomous project executor.

**Independent Test**: Review the user-facing docs and verify that they include the "Delivery Pilot Model", the observe-decide-act-verify-update-context loop, stopping rules, and one project-scale example.

**Acceptance Scenarios**:

1. **Given** a new user reads the architecture or delivery model docs, **When** they look for project-scale behavior, **Then** they find the principle "Large work is supported by decomposition, not by unbounded autonomy."
2. **Given** a user reads the Delivery Pilot Model, **When** they inspect the loop explanation, **Then** observe, decide, act, verify, and update context are each explained with concrete Boundline evidence and state examples.
3. **Given** a user reads the project-scale example, **When** they compare it to a broad initiative, **Then** they see each implementation slice as bounded and independently checkpointed, validated, traced, reviewed, and optionally governed.

### Edge Cases

- An assistant host supports repo-local prompts but no true global command package; docs must not claim global support and must provide a manual install or CLI fallback path.
- `/boundline:continue` runs in a repository with no `.boundline/session.json`; Boundline must say no active session exists and offer init, deliver, status, or exact CLI guidance.
- `.boundline/session.json` exists but is invalid, stale, or points to missing traces; Boundline must stop with repair or inspect guidance rather than chat-history inference.
- Canon is installed but capability output is missing, malformed, incompatible, or lacks a requested mode; governed paths must stop explicitly.
- A Canon mode is recommendation-only; Boundline must say so and keep delivery decisions in Boundline-owned orchestration state.
- A high-risk stage has no available reviewers or adjudicator; voting must block or escalate according to policy rather than silently continuing.
- A blocking vote finding exists; execution must stop unless adjudicated or explicitly overridden by allowed policy.
- Validation retry budget is exhausted; Boundline must stop, trigger recovery or review policy, and preserve trace/checkpoint refs.
- The next action would exceed the current stage or risk boundary; Boundline must require confirmation or stage transition approval.
- A compatibility manifest exists; normal operation still uses session-native state and does not require hand-editing JSON or manifests.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST provide a global or user-scoped assistant package model for supported hosts that can be installed before any repository-local Boundline initialization.
- **FR-002**: Global assistant commands MUST include `/boundline:init`, `/boundline:doctor`, `/boundline:help`, `/boundline:continue`, and `/boundline:status`.
- **FR-003**: Global assistant commands MAY include `/boundline:deliver` and `/boundline:govern` only when they can avoid assuming repo-local state.
- **FR-004**: Global assistant commands MUST detect or ask for the current workspace and determine whether Boundline workspace state exists.
- **FR-005**: Global assistant commands MUST invoke Boundline CLI when shell execution is available and provide exact CLI commands when shell execution is unavailable.
- **FR-006**: `/boundline:continue` MUST NOT infer active delivery state from chat history when no session exists.
- **FR-007**: Boundline MUST distinguish global assistant packages, repo-local assistant packages, and the CLI runtime in docs and user-facing behavior.
- **FR-008**: Repo-local assistant packages MUST remain generated by workspace initialization and MUST read authoritative state through Boundline CLI and `.boundline/session.json`.
- **FR-009**: The CLI runtime MUST remain authoritative for automation, debugging, hosts without shell execution, and session state reconciliation.
- **FR-010**: Boundline MUST support broad engineering initiatives as bounded stages and bounded work units, not as one unchecked autonomous run.
- **FR-011**: Boundline MUST be able to propose an idea-to-code path that can include discovery, requirements, system-shaping, architecture, backlog, implementation slices, verification, and review or pr-review.
- **FR-012**: Boundline MUST be able to propose an existing-system change path that can include system-assessment, change, implementation or refactor, verification, and pr-review.
- **FR-013**: Boundline MUST be able to propose operational and risk paths that can begin with incident, security-assessment, system-assessment, migration, or supply-chain-analysis and route into delivery work when appropriate.
- **FR-014**: Boundline MUST maintain a governed stage catalog that covers every current Canon mode: discovery, requirements, system-shaping, architecture, backlog, change, implementation, refactor, review, verification, pr-review, incident, security-assessment, system-assessment, migration, and supply-chain-analysis.
- **FR-015**: The governed stage catalog MUST record each mode's use case, required system context, stage category, voting applicability, delivery-follow-up behavior, and recommendation-only status.
- **FR-016**: Boundline MUST use Canon capabilities to verify available governed modes before proposing or running a governed stage.
- **FR-017**: Boundline MUST keep Canon at governed stage boundaries and MUST NOT call Canon for every internal observe-decide-act-verify-update-context step.
- **FR-018**: Boundline MUST preserve the product boundary: Canon governs packets; Boundline drives delivery.
- **FR-019**: `/boundline:govern` MUST be the primary assistant command for explicit Canon-governed stage work.
- **FR-020**: `/boundline:govern` MUST validate a specified mode against Canon capabilities and stop explicitly for unsupported or unavailable modes.
- **FR-021**: `/boundline:govern` MUST infer likely modes and present choices when the user does not specify a mode.
- **FR-022**: `/boundline:govern` MUST require confirmation when confidence is high but the proposed governed stage is material or high risk.
- **FR-023**: `/boundline:govern` MUST stop when required system context, input shape, Canon compatibility, approval state, or governed packet readiness is missing.
- **FR-024**: Boundline MUST provide or document a CLI equivalent for explicit governed stage work without requiring hand-edited JSON or manifests.
- **FR-025**: Boundline MUST NOT promote primary UX commands such as `/boundline-architecture`, `/boundline-backlog`, or `/boundline-incident` as Canon mode aliases.
- **FR-026**: Voting MUST be triggered by risk and evidence, not by every stage or every step.
- **FR-027**: Voting policy MUST support majority, weighted, reject-on-blocking, adjudication, escalation when adjudication is unavailable, persisted findings, and trace projection.
- **FR-028**: Voting triggers MUST include high-impact architecture decisions, high-risk change boundaries, public contract or API changes, validation exhaustion, pr-ready diffs, material security findings, critical supply-chain findings, migration cutover decisions, and material incident follow-up decisions.
- **FR-029**: Voting MUST NOT run by default for every discovery packet, every requirements packet, every low-risk local code change, or every low-risk refactor with strong preserved-behavior evidence.
- **FR-030**: Boundline session state MUST expose latest vote state, reviewer findings, vote result, adjudication result, reviewed evidence packet, execution-blocking status, and next action.
- **FR-031**: `status`, `next`, and `inspect` MUST show governance state, voting state, trace refs, checkpoint refs, and CLI-equivalent next action consistently across CLI and chat surfaces.
- **FR-032**: Boundline MUST document the Delivery Pilot Model with the principle "Large work is supported by decomposition, not by unbounded autonomy."
- **FR-033**: Boundline MUST document the observe-decide-act-verify-update-context loop with concrete evidence and state examples for each step.
- **FR-034**: Boundline MUST stop explicitly when context is insufficient, governance is blocked, validation is exhausted, risk exceeds policy, voting blocks continuation, approval is pending, or the next action exceeds the current boundary.
- **FR-035**: Boundline MUST require confirmation for material stage transitions, risk-boundary changes, and high-impact governed stages.
- **FR-036**: Boundline MUST document a project-scale example showing a broad initiative decomposed into bounded stages and implementation slices.
- **FR-037**: CLI and assistant surfaces MUST report the same current session, next action, governance state, voting state, trace refs, and checkpoint refs.

### Scope Boundaries *(mandatory)*

- **In Scope**: Global assistant bootstrap specification, project-scale bounded delivery path modeling, full Canon mode catalog through `/boundline:govern`, risk-triggered voting policy, Delivery Pilot Model documentation, and acceptance tests for the specified UX.
- **Out of Scope**: Open-ended autonomous project execution, turning Canon into the orchestrator, replacing Canon mode contracts, requiring hand-edited JSON or manifests for normal operation, top-level Boundline aliases for every Canon mode, hidden approval or governance bypasses, provider-specific marketplace packaging beyond what is required for global command installation, and implementing code in this specification phase.
- **Compatibility Path**: Compatibility manifests may remain as an advanced path, but normal project-scale delivery must not depend on hand-editing compatibility manifests.
- **Sequential Execution Boundary**: Initial project-scale delivery remains sequential-first with one active bounded stage or work unit at a time. Parallel execution is out of scope.

### Key Entities *(include if feature involves data)*

- **Global Assistant Package**: A user-scoped host package installed once per assistant host. It is available before workspace initialization and exposes init, doctor, help, continue, status, and optionally safe deliver/govern guidance.
- **Repo-Local Assistant Package**: A workspace-generated package produced by initialization. It contains workspace-specific command bindings, prompts, metadata, defaults, and CLI-backed state guidance.
- **Delivery Initiative**: A broad user goal or brief decomposed into bounded stages and work units. It owns the project-scale path but never grants unbounded execution.
- **Bounded Stage**: A named phase in a delivery path, such as discovery, requirements, architecture, backlog, implementation, verification, or pr-review. A stage has entry conditions, completion evidence, stop conditions, and next-action guidance.
- **Bounded Work Unit**: A specific implementation, refactor, verification, review, or recovery slice inside a stage. It has a checkpoint, validation expectation, trace, and terminal outcome.
- **Governed Stage Catalog**: The registry that maps Canon modes to Boundline stage use cases, required context, category, voting applicability, follow-up routing, and recommendation-only behavior.
- **Governed Packet Ref**: The Boundline-visible reference to Canon-produced packet artifacts, approvals, provenance, readiness, and missing sections.
- **Voting Decision**: A stage-boundary quality-control record containing reviewer findings, vote strategy, vote result, adjudication state, blocking status, reviewed evidence, and next action.
- **Delivery Pilot Loop**: The recurring observe-decide-act-verify-update-context loop Boundline uses to pilot broad work through bounded steps.

### Governed Stage Catalog

| Canon Mode | Boundline Should Consider When | Required System Context | Category | Voting May Be Required | Can Lead To Implementation/Refactor | Canon Posture |
|------------|--------------------------------|--------------------------|----------|------------------------|-------------------------------------|---------------|
| discovery | Problem, user, or evidence is ambiguous | Goal, available briefs, known unknowns | Planning | Rarely | Yes, through requirements or backlog | Recommendation-only unless policy says otherwise |
| requirements | Product scope or acceptance boundaries must be bounded | Goal, stakeholders or authored brief, constraints | Planning | Sometimes for material scope | Yes, through system-shaping, architecture, or backlog | Recommendation-only unless policy says otherwise |
| system-shaping | Capability structure or domain boundaries are not fixed | Requirements, current system evidence, domain constraints | Planning | Sometimes | Yes, through architecture or backlog | Recommendation-only unless policy says otherwise |
| architecture | Boundaries, invariants, C4, ADR, or structural decisions matter | Requirements, system-shaping evidence, current architecture | Planning / review | Often for high impact | Yes, through backlog, change, implementation, or refactor | Recommendation-only unless approval policy requires |
| backlog | Governed decomposition into delivery slices is needed | Requirements or architecture packet, constraints, priorities | Planning | Sometimes | Yes, directly | Recommendation-only |
| change | Existing-system modification boundary must be established | Current system evidence, target slice, validation strategy | Execution guidance | Often for high risk | Yes, through implementation or refactor | Recommendation-only unless policy requires |
| implementation | A bounded behavior slice is ready to execute | Confirmed plan, target files, validation command | Execution guidance | Sometimes for risky changes | Directly executes bounded work | Recommendation-only from Canon; Boundline executes |
| refactor | Structural cleanup is needed without new behavior | Current behavior evidence, preservation tests, target slice | Execution guidance | Rarely unless high impact | Directly executes bounded work | Recommendation-only from Canon; Boundline executes |
| review | Work product or packet needs governed review | Evidence packet, changed files or artifacts, criteria | Review | Often | May route to change or verification | Recommendation-only unless policy requires |
| verification | Claims need governed validation evidence | Validation outputs, changed files, acceptance criteria | Verification | Often after failures | May route to recovery, change, or pr-review | Recommendation-only unless policy requires |
| pr-review | A diff or worktree is ready for merge review | Base/head refs, diff summary, validation evidence | Review | Often | May route to change, refactor, or publish readiness | Recommendation-only unless policy requires |
| incident | Operational issue requires containment or follow-up reasoning | Incident brief, timeline, impact, current system state | Operational | Often for material blast radius | Yes, through change, verification, or review | Recommendation-only unless policy requires |
| security-assessment | Security risk or control coverage must be assessed | Threat context, assets, findings, current controls | Assessment | Often for material findings | Yes, through change, refactor, verification, or pr-review | Recommendation-only unless policy requires |
| system-assessment | Current-state understanding is weak or systemic risk exists | System inventory, traces, architecture docs, known gaps | Assessment | Sometimes | Yes, through change, backlog, architecture, or refactor | Recommendation-only |
| migration | Cutover, fallback, compatibility, or data movement is material | Source/target state, rollback plan, validation strategy | Operational / planning | Often for cutover | Yes, through backlog, implementation, verification, pr-review | Recommendation-only unless policy requires |
| supply-chain-analysis | Dependency, provenance, license, or package risk is material | Dependency evidence, manifests, findings, policy | Assessment | Often for high/critical findings | Yes, through change, verification, pr-review | Recommendation-only unless policy requires |

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In supported hosts, a user can discover an initialization path from chat in an uninitialized repository without relying on repo-local command packs.
- **SC-002**: 100% of global bootstrap commands tested in uninitialized workspaces either run Boundline CLI or show exact copyable CLI commands.
- **SC-003**: Given representative broad initiative briefs, Boundline proposes staged paths that decompose work into bounded stages and work units without claiming one unchecked run completes the initiative.
- **SC-004**: `/boundline:govern` can accept, infer, or explicitly reject every current Canon mode with capability-backed reasoning.
- **SC-005**: 100% of unsupported or unavailable Canon modes stop with explicit repair or unsupported-mode guidance.
- **SC-006**: High-risk architecture, validation-exhausted implementation, and PR-ready scenarios expose voting state and next action in status, next, and inspect.
- **SC-007**: Low-risk local changes do not trigger voting by default in representative tests.
- **SC-008**: Documentation includes the Delivery Pilot Model, the observe-decide-act-verify-update-context loop, stopping rules, and at least one project-scale example.
- **SC-009**: CLI and assistant output for the same active session agree on session state, next action, governance state, voting state, trace refs, and checkpoint refs in representative scenarios.
- **SC-010**: A developer can identify why a project-scale path is blocked and where to continue in under five minutes from status or inspect output.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**:
  - OpenAI Models documentation, reviewed 2026-05-11: https://platform.openai.com/docs/models
  - Anthropic Claude Models overview, reviewed 2026-05-11: https://docs.anthropic.com/en/docs/about-claude/models/overview
  - Google Gemini API Models documentation, reviewed 2026-05-11: https://ai.google.dev/gemini-api/docs/models
  - GitHub Copilot supported models documentation, reviewed 2026-05-11: https://docs.github.com/en/copilot/reference/ai-models/supported-models
- **Catalog Delta**: No catalog file change is required during this specification-only slice. The current bundled catalog already includes the active OpenAI, Claude, Gemini, and Copilot-facing model families needed for Boundline routing decisions in this slice, including GPT-5.5/GPT-5.4 variants, Claude Sonnet/Opus/Haiku 4.x entries, and Gemini 2.5/3.x entries.
- **No-Change Rationale**: This feature defines delivery UX, governance routing, global command packaging, and documentation behavior. It does not require adding a new assistant runtime or changing default model routing. The reviewed public docs confirm that the current catalog remains sufficient for planning this slice; implementation tasks must repeat the catalog check before code changes.

## Assumptions

- Supported hosts for the global assistant package are Claude Code, Codex, Cursor, Copilot-style prompt environments, and Gemini-style CLI/chat environments, but each host's true global install capability must be verified before claiming support.
- When a host cannot install user-scoped commands, the accepted behavior is explicit documentation plus manual import or CLI fallback, not a fake global command claim.
- Canon `0.45.0` is the active compatibility target for this Boundline planning slice.
- Canon capability output is the source of truth for available governed modes; the spec does not assume every deployed Canon binary supports every mode.
- Project-scale delivery remains sequential-first; a later feature may introduce controlled parallelism if the roadmap reprioritizes it.
- Voting is already a valid roadmap direction for bounded review slices and is included here only as risk-boundary quality control, not as a generic multi-agent council.
- `/boundline:govern` may accept a CLI equivalent such as `boundline govern --mode ...`, but the exact command grammar is deferred to implementation planning.
- This turn produces specification, plan, tasks, and consistency checks only; code implementation is deferred until explicitly requested.
