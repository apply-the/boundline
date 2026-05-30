# Feature Specification: Native Canon CLI Surface

**Feature Branch**: `042-native-canon-cli`  
**Created**: 2026-05-05  
**Status**: Draft  
**Input**: User description: "Provide a native CLI and chat-first Boundline experience where Canon is the default governed development runtime, so operators can initialize a workspace, choose assistant/model preferences and Canon autonomy behavior, ingest PRD/C4/backlog/repository evidence, answer clarification questions, and let Boundline/AI assemble Canon-ready inputs for every Canon mode without editing workspace manifests by hand. Installation diagnostics must verify the real Canon governance CLI surface rather than version only."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.
  When both a session-native workflow and a compatibility workflow exist, the spec MUST name which path is primary and keep compatibility behavior explicit rather than implicit.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Bootstrap Canon-Default Governed Work From Scratch (Priority: P1)

An operator starting in a new or lightly scaffolded repository can ask Boundline
through the primary CLI or a chat-driven command pack to run governed work,
provide the goal plus bounded authored inputs, and reach a real governed session
without hand-authoring a workspace manifest first or explicitly selecting Canon
on every run.

**Why this priority**: If Canon-governed work cannot start from the primary
operator surfaces, then governed development is still an advanced workaround
rather than a real Boundline capability.

**Independent Test**: In a clean repository with a compatible Canon binary, run
guided init, choose Canon mode-selection behavior and assistant/model routing,
then start a session-native run from a goal plus authored Markdown inputs and
verify that Boundline either enters Canon-governed execution or stops explicitly
for a real governance reason, without asking the operator to edit a local
manifest or pass `--governance canon`.

**Acceptance Scenarios**:

1. **Given** a clean workspace with a goal and bounded authored inputs,
  **When** the operator requests governed work through the primary
  session-native route, **Then** Boundline bootstraps a Canon-governed session by
  default and selects governed stages without requiring manual workspace
  manifest editing.
2. **Given** no active session, **When** the operator starts a direct native
  run and supplies the required governance fields, **Then** Boundline creates
  the session, plans the bounded work, and enters the Canon-governed route
  through the same primary operator surface.
3. **Given** an operator explicitly opts out with the documented local
  governance override, **When** they run the same bounded work, **Then**
  Boundline uses the local governance route and projects that opt-out explicitly
  through status and inspect output.

---

### User Story 2 - Move From Ingested Evidence To Canon-Ready Inputs (Priority: P2)

An operator can start from idea-level engineering artifacts such as a product
brief, PRD, architecture or C4 context, backlog notes, and repository evidence,
plus answers to targeted clarification questions, and have Boundline/AI assemble
the Canon-ready input documents needed for the selected mode or mode sequence
rather than requiring the operator to author Canon documents directly.

**Why this priority**: The product is only credible for governed development if
it can cover the normal journey from idea to code across the development
situations Canon is meant to support, not just a narrow repair slice or a
hidden advanced configuration surface.

**Independent Test**: In a greenfield or lightly scaffolded workspace, supply
bounded Markdown artifacts representing intent and structure, answer any
clarifying questions, then verify that Boundline writes or stages the Canon-ready
input bundle for the selected mode and either starts Canon or stops with a
specific missing-input reason.

**Acceptance Scenarios**:

1. **Given** authored inputs that describe product intent, PRD scope,
  architecture or C4 context, and backlog scope, **When** the operator requests
  governed development, **Then** Boundline uses those inputs as bounded evidence
  to assemble the Canon request's `input_documents` and `bounded_context` fields
  without asking the operator to construct Canon payloads.
2. **Given** conflicting or insufficient idea-to-code inputs, **When** the
  operator requests governed development, **Then** Boundline stops with
  clarification or blocked guidance instead of inventing stages, silently
  falling back, or instructing the operator to edit a manifest by hand.
3. **Given** the operator supplies only a goal, **When** Canon requires richer
  input for the selected mode, **Then** Boundline asks targeted questions and
  uses the answers to produce the mode input, or stops with a specific
  unresolved-input reason.

---

### User Story 3 - Keep CLI And Assistant Surfaces Aligned (Priority: P3)

An operator can start, continue, and inspect governed work through the CLI or
through the repository-managed command packs used by Copilot, Codex, Claude,
and Gemini CLI, and the assistant surface follows the same primary workflow
instead of pointing the operator back toward manual workspace manifest editing
or a reduced, assistant-only subset of the product.

**Why this priority**: The user experience is broken if assistants can only
explain an advanced configuration path while the CLI claims a native-first
product story.

**Independent Test**: Use a supported assistant command pack and the raw CLI on
the same clean workspace, then verify that both surfaces expose the same
governed bootstrap path, the same required fields, the same idea-to-code stage
coverage, and the same follow-through guidance for blocked or approval-pending
states.

**Acceptance Scenarios**:

1. **Given** an operator asks an assistant to start governed work from scratch,
  **When** the assistant invokes Boundline through the managed command pack,
  **Then** the surfaced path matches the primary native CLI behavior and does
  not tell the operator to edit a workspace manifest manually.
2. **Given** a governed run is blocked or awaiting approval, **When** the
  operator uses `status`, `next`, or `inspect` through CLI or assistant,
  **Then** both surfaces expose the same reason, same authority, and one safe
  next action.
3. **Given** an operator starts from PRD, C4, backlog, or equivalent bounded
  authored inputs, **When** they use CLI or assistant entrypoints, **Then**
  both surfaces can drive the same supported Canon-governed development journey
  across every Canon mode exposed by the detected Canon capabilities.

---

### User Story 4 - Configure Canon Autonomy And Models During Init (Priority: P4)

An operator initializing one or more projects can choose the workspace-local
Canon mode-selection behavior and assistant/model routing during a guided
`boundline init`, and can later inspect or change those choices through
workspace-scoped config commands instead of editing config files by hand.

**Why this priority**: Canon-default execution is only safe if each workspace
can declare whether Boundline should infer modes automatically, ask for
confirmation, or require explicit mode choice, and if assistant-specific model
availability is captured where the work will run.

**Independent Test**: Run `boundline init` in a new workspace without flags,
choose `auto-confirm` for Canon mode selection and model routes for Codex and
Copilot, then verify `config show` reports the same workspace-local settings.
Change the Canon mode-selection value later through a config command and verify
subsequent runs use the updated behavior.

**Acceptance Scenarios**:

1. **Given** a repository with no Boundline config, **When** the operator runs
  guided `boundline init`, **Then** Boundline asks for Canon mode-selection
  preference, assistant surfaces, and model routes before writing workspace
  config.
2. **Given** a scripted setup environment, **When** the operator runs
  non-interactive `boundline init` with Canon mode-selection and model-route
  flags, **Then** Boundline applies the same workspace-local settings without
  interactive prompts.
3. **Given** an initialized workspace, **When** the operator changes the Canon
  mode-selection setting through config, **Then** later CLI and assistant runs
  use the new value and report its workspace-local source.

---

### User Story 5 - Verify The Real Canon Surface Before Work Starts (Priority: P5)

An operator who installed Boundline and Canon together can trust install
diagnostics to verify not just the reported Canon version but the actual
governance command surface that Boundline needs, so a wrong or shadowed Canon
binary is detected before a governed run fails deep into execution.

**Why this priority**: A version-only install check creates false confidence,
which is worse than an explicit repair stop because it pushes failure into the
middle of real governed work.

**Independent Test**: Put multiple Canon binaries on PATH, including one whose
reported version looks compatible but whose governance surface is missing or
wrong, then verify that install diagnostics reject the incompatible surface and
identify the selected binary plus the repair action.

**Acceptance Scenarios**:

1. **Given** a Canon binary whose version appears compatible but whose required
  governance commands are missing, **When** the operator runs install
  diagnostics, **Then** Boundline marks the install as not ready and reports
  the selected binary path plus repair guidance.
2. **Given** more than one Canon binary is available, **When** install
  diagnostics run, **Then** Boundline makes the authoritative Canon path
  explicit so the operator can tell whether a bundled or PATH binary is being
  used.

---

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when Canon-default governance starts on a clean workspace but the
  operator supplied only a goal and no bounded authored artifacts?
- What happens when the detected Canon binary reports the supported version but
  does not expose the required governance subcommands or capabilities surface?
- What happens when the detected Canon binary does not support one of the
  canonical modes Boundline expects to expose?
- How does the system behave when Canon is unavailable at runtime and a local
  fallback would materially change the operator's expected governance posture?
- How does the system surface approval waits, blocked governed packets, or
  missing required governance fields without losing the normal Boundline
  follow-through story?
- How does the system keep the primary session-native route explicit while still
  preserving any advanced compatibility path as subordinate rather than broken?
- What happens when the operator provides a brief or PRD but Canon's mode
  requires richer context than the input contains?
- How does the system handle a multi-stage governed journey when the operator
  stops after stage 1 and returns later?
- How does `boundline run` behave when workspace config says `manual`,
  `auto-confirm`, or `auto` and the operator did not pass an explicit mode?
- How do assistant commands resolve the workspace when the chat is attached to a
  repository but the user does not mention a workspace path?

---

### Workspace Resolution And Settings Scope

All settings introduced by this feature are workspace-local.  Boundline MUST NOT
write Canon mode-selection, assistant, or model-routing preferences to a global
config as part of this feature.

When a CLI or assistant command omits `--workspace`, Boundline resolves the
workspace in this order:

1. Use the explicit `--workspace <path>` when supplied.
2. Search upward from the current working directory for an existing `.boundline/`
   directory and use that directory's parent.
3. Search upward for the nearest git root and use that root.
4. Fall back to the current working directory when no git root exists.

Before mutating files, Boundline surfaces the resolved workspace and stops if
the target is ambiguous or outside the active repository/chat context.

---

### Canon Default And Opt-Out

Canon is the default governance runtime for this native surface.  Operators do
not pass `--governance canon` for the normal path.  A plain governed command
such as `boundline run`, `boundline run --goal "<goal>"`, or
`boundline run --brief <path>` uses Canon when the workspace is initialized for
Canon and diagnostics confirm the required Canon surface.

Operators can opt out explicitly for local governance by using the documented
override, such as `boundline run --governance local` or the user-facing alias
`boundline run --no-canon`.  Any opt-out MUST be visible in `run`, `status`,
`next`, and `inspect` output so assistants do not silently treat the session as
Canon-governed later.

---

### Supported Canon Modes

Boundline exposes every Canon mode supported by the native Canon governance
surface.  The canonical mode identifiers for this feature are:

| Canon Mode | Primary Document |
|------------|------------------|
| `requirements` | `requirements.md` |
| `discovery` | `discovery.md` |
| `system-shaping` | `system-shaping.md` |
| `architecture` | `architecture.md` |
| `backlog` | `backlog.md` |
| `change` | `change.md` |
| `implementation` | `implementation.md` |
| `refactor` | `refactor.md` |
| `review` | `review.md` |
| `verification` | `verification.md` |
| `incident` | `incident.md` |
| `security-assessment` | `security-assessment.md` |
| `system-assessment` | `system-assessment.md` |
| `migration` | `migration.md` |
| `supply-chain-analysis` | `supply-chain-analysis.md` |

Boundline MUST verify this mode surface through Canon capabilities before
claiming the workspace is Canon-ready.  If the installed Canon runtime exposes a
different mode set, diagnostics and run gating MUST report the missing,
unsupported, or renamed modes explicitly.

---

### Mode Selection Preference

Each workspace stores one Canon mode-selection preference:

- `manual`: Boundline requires an explicit mode or mode sequence before starting
  Canon.  In chat, the assistant asks which mode to run.
- `auto-confirm`: Boundline infers the best mode or mode sequence from ingested
  evidence and clarification answers, then asks the operator to confirm before
  invoking Canon.
- `auto`: Boundline infers the best mode or mode sequence and may invoke Canon
  without confirmation when confidence is high.  If confidence is low,
  consequences are broad, or inputs conflict, Boundline falls back to
  confirmation instead of guessing silently.

Guided `boundline init` MUST ask for this preference.  Non-interactive init MUST
accept a flag such as `--canon-mode-selection <manual|auto-confirm|auto>`.
After init, a workspace config command MUST let the operator inspect and change
the value without editing files manually.

---

### CLI And Chat Command Mapping

The primary CLI and assistant commands MUST map to the same Boundline-owned
workflow.  Assistant commands are ergonomic wrappers; they do not introduce a
second orchestration path.

| Intent | CLI | Chat command |
|--------|-----|--------------|
| Verify installation | `boundline doctor --install` | `/boundline-doctor` |
| Guided init | `boundline init [--workspace <path>]` | `/boundline-init` |
| Scripted init | `boundline init [--workspace <path>] --canon-mode-selection <manual|auto-confirm|auto> --assistant <name>... --route <slot>=<runtime>:<model>...` | `/boundline-init` after collecting answers |
| Show workspace config | `boundline config show [--workspace <path>]` | `/boundline-config-show` |
| Change Canon mode selection | `boundline config set-canon [--workspace <path>] --mode-selection <manual|auto-confirm|auto>` | `/boundline-config-set-canon` |
| Change model routing | `boundline config set [--workspace <path>] --slot <slot> --runtime <runtime> --model <model>` | `/boundline-config-set` |
| Capture goal and documents | `boundline goal [--workspace <path>] --goal "<goal>" --brief <path>...` | `/boundline-goal` or `/boundline-plan` |
| Plan without execution | `boundline plan [--workspace <path>]` | `/boundline-plan` |
| Confirm plan | `boundline plan [--workspace <path>] --confirm` | `/boundline-plan --confirm` |
| Run default Canon route | `boundline run [--workspace <path>]` | `/boundline-run` |
| Run one Canon mode | `boundline run [--workspace <path>] --mode <canon-mode>` | `/boundline-run <canon-mode>` |
| Run a mode alias | `boundline run [--workspace <path>] --mode requirements` | `/boundline-requirements` |
| Run local opt-out | `boundline run [--workspace <path>] --no-canon` | `/boundline-run --no-canon` |
| Continue execution | `boundline run [--workspace <path>]` | `/boundline-run` |
| Status | `boundline status [--workspace <path>]` | `/boundline-status` |
| Next action | `boundline next [--workspace <path>]` | `/boundline-next` |
| Inspect | `boundline inspect [--workspace <path>]` | `/boundline-inspect` |
| Named workflows | `boundline workflow list|run|status|resume|inspect ...` | `/boundline-workflow-*` |

Mode-specific assistant aliases MAY be provided for common modes, including
`/boundline-requirements`, `/boundline-discovery`,
`/boundline-system-shaping`, `/boundline-architecture`, `/boundline-backlog`,
`/boundline-change`, `/boundline-implementation`, `/boundline-refactor`,
`/boundline-review`, `/boundline-verification`, `/boundline-incident`,
`/boundline-security-assessment`, `/boundline-system-assessment`,
`/boundline-migration`, and `/boundline-supply-chain-analysis`.  Each alias is
equivalent to `/boundline-run <mode>`.

---

### AI-Assembled Canon Input Model

The operator does not produce Canon documents directly.  The interaction model
is:

1. **Operator provides structured input**: a goal, optional Markdown briefs
   (product brief, architecture notes, backlog sketch, etc.), and answers to
   any clarification prompts Boundline surfaces.
2. **Boundline and the active assistant assemble Canon-ready inputs**: the
   assistant may draft, normalize, and structure the input documents required by
   the selected Canon mode from ingested evidence and operator answers.
3. **Boundline packages input transparently**: Boundline stores the assembled
   inputs and maps them into the `input_documents` and `bounded_context` fields
   of the Canon governance request for the current mode.
4. **Canon produces the governed document**: Canon's mode-specific runtime
   consumes the packaged input and produces the governed artifact (e.g.,
   `requirements.md`, `architecture.md`) under `.canon/runs/<run-id>/`.
5. **Boundline records and forwards**: Boundline stores the governed packet
   reference in the session, evaluates readiness, and uses the produced
   document as bounded context for subsequent stages.

The operator's effort is bounded to:
- Providing raw Markdown inputs, repository evidence, references, or chat
  answers.  The operator is not responsible for constructing Canon request
  payloads or final Canon input documents by hand.
- Answering clarification questions when Boundline detects missing required
  governance fields or insufficient bounded context.
- Reviewing governed artifacts when Canon signals approval-required.

Boundline owns workspace state, orchestration, mode selection, input packaging,
stage progression, and inspectability.  The active assistant owns chat-native
input drafting when the operator is working in chat.  Canon owns governed
document production and approval semantics.

---

### Governance Lifecycle And Continuation

A governed session progresses through a bounded lifecycle that the spec must
model explicitly:

1. **Bootstrap**: Operator provides goal + inputs → Boundline resolves the
   workspace and Canon mode-selection preference → Boundline selects or asks for
   the governed route → Canon governance `start` for the first mode.
2. **Stage Completion or Block**: Canon returns a governed packet with
   readiness.  If `governed_ready`, the produced document joins bounded
   context and Boundline advances to the next stage.  If `awaiting_approval`,
   `blocked`, or `incomplete`, Boundline halts with explicit guidance.
3. **Refresh**: Operator returns later → Boundline runs Canon governance
   `refresh` with the existing `run_ref` to check approval state or updated
   readiness without re-running the full stage.
4. **Stage Progression**: After a stage completes, Boundline selects the next
   governed stage, packages the accumulated bounded context (including prior
   governed documents), and invokes Canon `start` for the next mode.
5. **Terminal States**: The governed journey ends when all governed stages
   complete, when a required stage is `blocked` or `rejected`, or when the
   operator explicitly abandons the session.

Boundline surfaces the current lifecycle position, next safe action, and any
blocked or approval-pending reason through `status`, `next`, and `inspect` at
every point.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST let operators request governed development from
  the primary session-native CLI and managed assistant command surfaces on a
  workspace that has no pre-authored custom execution manifest, with Canon as
  the default governance runtime when the workspace is Canon-ready.
- **FR-002**: System MUST accept bounded authored inputs such as goal text,
  product briefs, architecture or C4 context, backlog slices, and repository
  evidence as first-class native planning inputs for governed development.
- **FR-003**: System MUST synthesize or persist the native governance policy and
  stage state required for supported governed flows from operator intent,
  current session state, and authored inputs, rather than requiring operators to
  hand-author a workspace manifest before governed work can begin.
- **FR-004**: System MUST support governed native-first operation for every
  canonical Canon mode: `requirements`, `discovery`, `system-shaping`,
  `architecture`, `backlog`, `change`, `implementation`, `refactor`, `review`,
  `verification`, `incident`, `security-assessment`, `system-assessment`,
  `migration`, and `supply-chain-analysis`.
- **FR-005**: System MUST prompt for, validate, and persist the governance
  fields required to run Canon credibly, including risk, zone, owner, Canon
  mode-selection preference, assistant/model routes, and any mode selections
  that cannot be derived from the bounded context.
- **FR-005a**: System MUST provide sensible defaults for governance fields to
  minimize bootstrap friction: `risk` defaults to `standard`, `zone` defaults
  to `development`, `owner` defaults to the current OS user or git
  `user.name`, and `system_context` is inferred from the mode's context type
  (`new` or `existing`).  Operators MAY override any default through CLI
  flags or interactive prompts, but the system MUST NOT block bootstrap solely
  because the operator omitted a defaultable field.
- **FR-005b**: Guided `boundline init` MUST ask the operator to choose Canon
  mode-selection behavior (`manual`, `auto-confirm`, or `auto`) and the
  assistant/model routes available for the workspace.  Non-interactive init MUST
  accept equivalent flags so the same settings can be scripted.
- **FR-005c**: All Canon mode-selection and assistant/model routing settings in
  this feature MUST be workspace-local.  When `--workspace` is omitted,
  Boundline MUST resolve the workspace from an existing `.boundline/` parent,
  nearest git root, or current working directory, in that order, and MUST surface the
  resolved workspace before mutating files.
- **FR-006**: System MUST stop explicitly when governed work cannot proceed due
  to missing governance metadata, insufficient bounded context, unsupported
  stage-mode selection, incompatible Canon surface, approval wait, or blocked or
  rejected governed output.
- **FR-007**: System MUST keep Boundline as the orchestrator of routing,
  planning, execution, state, and inspection even when Canon is selected as the
  governed runtime for a stage.
- **FR-008**: System MUST expose the same governed bootstrap and continuation
  path through repository-managed assistant command packs for Copilot, Codex,
  Claude, and Gemini CLI that it exposes through the raw CLI, without reducing
  supported development situations or requiring assistant-specific operator
  workarounds.
- **FR-009**: System MUST treat a Canon installation as compatible only when the
  selected Canon binary exposes the governance command surface Boundline needs,
  not only when a version string appears to match the supported window.
- **FR-010**: System MUST report the authoritative Canon binary path, detected
  compatibility state, and repair guidance through install diagnostics whenever
  Canon is missing, shadowed, wrong-version, missing the required governance
  surface, or missing one or more canonical modes.
- **FR-011**: System MUST project requested governance intent, selected runtime,
  selected mode or mode sequence, mode-selection preference, approval state,
  blocked reason, governed artifact references, local opt-out state, and
  authoritative next actions through `run`, `status`, `next`, and `inspect` for
  native governed flows.
- **FR-012**: System MUST keep explicit compatibility manifests available only
  as an advanced override path and MUST NOT treat manual manifest editing as the
  normal operator entry to Canon-governed development.
- **FR-013**: System MUST provide primary documentation and assistant guidance
  for greenfield governed development that begins from idea-level artifacts
  such as PRD, C4, backlog, or bounded product briefs and ends in either
  bounded code change, governed approval wait, or an explicit blocked or
  clarification state.
- **FR-014**: System MUST let both the step-by-step session-native route and
  the direct native run route access the same Canon-governed bootstrap behavior
  when the operator requests governed work.
- **FR-015**: System MUST include unit, integration, and contract validation for
  native Canon bootstrap, assistant guidance parity, install diagnostics surface
  verification, and governed non-success states.
- **FR-016**: System MUST NOT require operators to hand-edit `.boundline/execution.json`
  or any equivalent workspace manifest to access the primary Canon-governed
  development experience.
- **FR-017**: System MUST expose governed bootstrap through the normal CLI
  surface without requiring `--governance canon`: `boundline run`, optional
  `--goal`, repeated `--brief <path>`, optional `--mode <canon-mode>`, optional
  `--risk`, `--zone`, and `--owner`, plus explicit opt-out through
  `--governance local` or `--no-canon`.  The same fields MUST be expressible
  through assistant command-pack instructions without requiring the operator to
  construct CLI commands manually.
- **FR-018**: System and assistant command packs MUST assemble
  operator-provided Markdown briefs, repository evidence, goal text, and
  clarification answers into Canon's `input_documents` and `bounded_context`
  request fields transparently, so the operator never constructs Canon request
  payloads or final Canon input documents by hand.
- **FR-019**: System MUST, when Canon signals `incomplete` or `pending_selection`
  for a governed stage, surface the specific missing input or unresolved mode
  choice to the operator as a clarification prompt rather than failing silently
  or falling back to local governance.
- **FR-020**: System MUST forward governed documents produced by Canon in prior
  stages as `bounded_context.reused_packets` or `input_documents` for
  subsequent governed stages, so multi-stage idea-to-code journeys accumulate
  governed evidence automatically.
- **FR-021**: System MUST support governed session refresh via Canon's
  `refresh` operation when the operator returns to an `awaiting_approval` or
  `blocked` session, updating approval state and governed packet readiness
  without re-invoking the full `start` operation.
- **FR-022**: System MAY surface Canon mode template hints (expected document
  structure, required sections) to the operator when available through the
  Canon `capabilities` response, to guide authored input toward the structure
  each mode expects.
- **FR-023**: System MUST provide a workspace config command to inspect and
  update Canon mode-selection preference after init, such as
  `boundline config set-canon --mode-selection <manual|auto-confirm|auto>`.
- **FR-024**: System MUST let chat users run mode-specific commands with
  ergonomic syntax such as `/boundline-run requirements` and MAY expose
  aliases such as `/boundline-requirements`; these assistant commands MUST map
  to the same CLI mode selection and session state as `boundline run --mode`.
- **FR-025**: System MUST treat `manual`, `auto-confirm`, and `auto` as
  inspectable mode-selection states.  `manual` requires explicit mode input,
  `auto-confirm` requires confirmation of the inferred mode or sequence, and
  `auto` may proceed without confirmation only when confidence is high and no
  broad-risk ambiguity is present.
- **FR-026**: System MUST keep Canon diagnostics and runtime gating aligned with
  Canon capabilities, including supported operations, supported modes, template
  hints when available, and selected binary path.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: native Canon-default bootstrap from scratch through CLI and
  assistant command packs, guided and non-interactive workspace init,
  workspace-local Canon mode-selection settings, assistant/model route capture,
  ergonomic chat command mapping, bounded journeys driven by ingested evidence
  including PRD, C4, backlog, repository artifacts, and clarification answers,
  AI assembly of Canon-ready inputs, surfaced governance metadata and
  follow-through, install diagnostics that verify the real Canon governance
  surface and all canonical modes, documentation that makes the native
  Canon-default route primary, transparent input packaging from operator inputs
  to Canon request payloads, sensible governance field defaults, multi-stage
  governed document forwarding, governed session refresh for approval and
  blocked states, and every canonical Canon mode listed in this spec.
- **Out of Scope**: turning Canon into the Boundline orchestrator, supporting
  Canon features that cannot be represented as bounded mode runs with
  inspectable inputs and outcomes, generic dashboards or UI work, deployment
  automation, unbounded project management, removing advanced compatibility
  customization for operators who still need bespoke policy, global config for
  the new Canon mode-selection settings, and Canon template authoring or
  template management within Boundline.

### Key Entities *(include if feature involves data)*

- **Governed Development Intent**: the operator-declared request to use Canon on
  a bounded path, including risk, zone, owner, selected or inferred Canon mode,
  mode-selection preference, local opt-out state, and the authored inputs that
  justify that request.
- **Idea-To-Code Input Bundle**: the bounded authored set of PRD, C4,
  architecture, backlog, and repository inputs that Boundline can use to start
  a governed development path without requiring manual manifest authoring.
- **Native Governance Policy**: the Boundline-owned policy snapshot that tells a
  native session which Canon modes are governed, which mode-selection behavior
  applies, and which stage-specific choices still need operator clarification.
- **Canon Capability Snapshot**: the inspected view of the selected Canon
  binary's supported version, operations, canonical modes, template hints, and
  compatibility state that install diagnostics and runtime gating use before
  governed work starts.
- **Governed Session Continuity**: the persisted session projection that records
  requested governance intent, selected Canon mode or sequence, approval or
  blocked state, governed artifacts, and the next safe operator action.
- **Input Assembly Bundle**: the Boundline-owned intermediate representation
  that maps operator-provided goal text, Markdown briefs, repository evidence,
  and clarification answers to Canon's `input_documents` and `bounded_context`
  request fields, including any governed documents forwarded from prior stages.
- **Governed Stage Chain**: the ordered sequence of governed stages with their
  Canon mode bindings, produced documents, and accumulated bounded context,
  representing one complete idea-to-code governed journey.
- **Workspace Canon Preferences**: the workspace-local settings captured during
  init or config updates, including Canon mode-selection behavior, assistant
  surfaces, and model routes.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In every supported greenfield governed acceptance scenario,
  operators can reach governed execution, governed approval wait, or an explicit
  blocked or clarification state without authoring a workspace manifest by hand
  and without passing `--governance canon`.
- **SC-002**: Install diagnostics correctly reject all sampled Canon binaries
  that report a compatible version string but lack the required governance
  command surface or one or more canonical modes, and always identify the
  authoritative Canon path they tested.
- **SC-003**: CLI and managed assistant command-pack scenarios expose the same
  governed bootstrap requirements, same canonical mode coverage, same
  mode-selection behavior, and same follow-through guidance for supported native
  governed flows.
- **SC-004**: Representative scenarios across the canonical Canon mode set can
  start from bounded product, PRD, C4, backlog, repository evidence, incident,
  review, security, migration, or supply-chain inputs and produce one Canon-ready
  input bundle plus one explicit continuation or terminal outcome without
  requiring operators to leave the primary Boundline experience or edit a
  manifest manually.
- **SC-005**: Multi-stage governed journeys that progress through at least two
  Canon modes demonstrate automatic forwarding of governed documents from
  prior stages as bounded context for subsequent stages.
- **SC-006**: Operators who provide only a goal and no authored briefs can
  bootstrap governed work using defaulted governance fields and receive clear
  clarification prompts for any missing input Canon requires, rather than an
  opaque error or silent fallback.
- **SC-007**: Guided init and non-interactive init both persist equivalent
  workspace-local Canon mode-selection and assistant/model routing settings, and
  config commands can later inspect and update those settings without manual
  file editing.
- **SC-008**: Chat commands such as `/boundline-run requirements` and
  `/boundline-requirements` map to the same workspace, Canon mode, session
  state, and status/inspect projections as the equivalent CLI commands.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Operators can provide bounded Markdown inputs, repository evidence, or chat
  answers for product, architecture, backlog, review, incident, security,
  migration, supply-chain, and system-assessment contexts.
- Boundline will support every canonical Canon mode listed in this spec when
  the installed Canon runtime exposes the required surface, while still limiting
  execution to bounded, inspectable mode runs.
- A compatible Canon runtime continues to expose machine-readable governance
  start, refresh, and capabilities operations, including supported mode metadata,
  through the supported Boundline compatibility window.
- Advanced operators may still want a bespoke compatibility override surface,
  but that path remains secondary to the primary CLI and assistant experience.
- Canon's `capabilities` response MAY expose template hints or expected
  document structure for each mode; if it does, Boundline and the active
  assistant will use those hints to assemble Canon-ready inputs but will not
  require the operator to author those inputs manually.
- When Canon requires richer input than the operator provided, Canon will
  return `incomplete` or `pending_selection` status with enough diagnostic
  information for Boundline to surface a targeted clarification prompt.
- Chat environments may or may not provide shell access.  Assistant command
  packs therefore either run the mapped CLI command directly or provide the
  exact command and continue from pasted output without changing the workflow
  semantics.
