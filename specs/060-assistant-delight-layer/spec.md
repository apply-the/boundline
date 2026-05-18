# Feature Specification: S7 Assistant Delight Layer

**Feature Branch**: `060-assistant-delight-layer`  
**Created**: 2026-05-17  
**Status**: Implemented  
**Input**: User description: "Implement roadmap S7 assistant delight and cognitive affordance layer in Boundline, aligned to Canon 057 delight-provider contract"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fast Explanations On Real Runtime State (Priority: P1)

As an operator working inside a bounded Boundline session, I want to ask why a
plan, risk posture, evidence set, or next action exists and get an immediately
useful answer from the current runtime state, so I can trust the system before I
understand all of its architecture.

**Why this priority**: The roadmap promise for S7 starts with fast first value.
If `/boundline:why`, `/boundline:risk`, `/boundline:evidence`, and
`/boundline:next-best` are not useful on the active session, the delight layer
fails its first operator promise.

**Independent Test**: Start a session on a workspace with changed files, run the
assistant-facing command assets and CLI-backed inspect surfaces for `why`,
`risk`, `evidence`, and `next-best`, then verify that the output cites runtime
sources, missing evidence, confidence, and the next safe action without relying
on chat history.

**Acceptance Scenarios**:

1. **Given** an active session with changed files and no Canon-governed inputs,
   **When** an operator asks `/boundline:risk current change`, **Then** Boundline
   returns a useful risk summary from workspace and trace signals and explicitly
   says that Canon-governed input is missing.
2. **Given** an active session with Canon packet, readiness, or security input
   available under the 057 delight-provider contract, **When** an operator asks
   `/boundline:why this plan?`, **Then** Boundline cites both runtime and Canon
   sources, separates them clearly, and surfaces any missing evidence.
3. **Given** a blocked, failed, or clarification-required session,
   **When** an operator asks `/boundline:next-best`, **Then** Boundline reports
   the stop reason, the evidence behind it, and the next safe action instead of
   masking the stop state.

---

### User Story 2 - Deep Cognitive Affordances Without Hidden Magic (Priority: P2)

As an operator reviewing a bounded change, I want to inspect assumptions,
indirect impact, objections, and the current plan in human terms, so I can
challenge the work and understand what still needs proof.

**Why this priority**: S7 is more than quick summaries. The roadmap explicitly
calls for assumptions, hidden impact, challenge, explain-plan, and human-facing
inspect lenses, but those surfaces must still stay bounded and explainable.

**Independent Test**: On an active session with traces and optional advanced
context, run `/boundline:assumptions`, `/boundline:hidden-impact`,
`/boundline:challenge`, `/boundline:explain-plan`, and `inspect` views for risk,
assumptions, evidence, blockers, and next, then verify that the outputs remain
source-attributed and disclose when advanced capabilities are unavailable.

**Acceptance Scenarios**:

1. **Given** an active session with traces and advanced-context signals
   available, **When** an operator asks `/boundline:hidden-impact src/auth/tokens.rs`,
   **Then** Boundline lists likely affected capabilities, invariants, tests,
   reviewers, and the evidence used to infer them.
2. **Given** an active session without advanced-context or Canon project memory,
   **When** an operator asks `/boundline:hidden-impact this PR`, **Then**
   Boundline falls back to structured repository and trace signals, discloses the
   missing capability, and suggests how to improve the answer.
3. **Given** governed or high-risk work,
   **When** an operator asks `/boundline:challenge`, **Then** Boundline returns
   the strongest objection, weakest assumption, missing evidence, likely failure
   mode, and any required reviewers or council profile without replacing formal
   governance.

---

### User Story 3 - Compact Assistant Surfaces And Context Diagnosis (Priority: P3)

As a maintainer, I want S7 commands and inspect views to ship through the
existing assistant packages and bootstrap flow without making the palette noisy,
so the feature feels intentional instead of sprawling.

**Why this priority**: The roadmap requires a compact, assistant-native surface.
If S7 adds commands without contextual visibility, package alignment, and
`doctor-context`, the result becomes clutter rather than delight.

**Independent Test**: Verify that assistant manifests, host-specific prompt
assets, global bootstrap assets, and runtime-backed doctor diagnostics expose the
new S7 commands with the expected default/core/contextual split and that missing
setup is disclosed with actionable fix commands.

**Acceptance Scenarios**:

1. **Given** a supported assistant host with repo-local Boundline assets,
   **When** the package metadata is generated or validated, **Then** the default
   visible palette includes the core session-native commands plus the planned S7
   cognitive commands without surfacing every advanced command by default.
2. **Given** an incomplete workspace setup,
   **When** an operator asks `/boundline:doctor-context`, **Then** Boundline
   reports missing Canon project memory, Boundline config, provider readiness,
   indexes, or evidence together with concrete fix commands.
3. **Given** a host that cannot expose global bootstrap commands,
   **When** the operator uses the documented fallback path, **Then** the package
   docs and generated prompts still point to `boundline assistant install` and
   the equivalent repo-local entry points.

---

### Edge Cases

- The active session has no Canon-governed inputs and no advanced-context index,
  but S7 must still produce advisory output without inflating confidence.
- The runtime and Canon signals disagree on risk or readiness and the output
  must surface the conflict without overriding stop semantics.
- A host package can expose assistant commands but cannot show every contextual
  command in the default palette.
- The current trace contains enough state for `next-best` but not enough proof
  for `hidden-impact` or `challenge`.
- `inspect` must explain the latest authoritative runtime state even when the
  session ended in blocked, failed, exhausted, or clarification-required.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST expose S7 assistant-facing cognitive affordance
  commands for `why`, `risk`, `assumptions`, `hidden-impact`, `challenge`,
  `evidence`, `next-best`, `explain-plan`, and `doctor-context` through the
  Boundline assistant package surface, with host naming allowed to follow local
  package conventions.
- **FR-002**: The system MUST back S7 answers with Boundline runtime authority:
  `.boundline/session.json`, persisted traces, existing CLI projections, and the
  current workspace evidence remain authoritative over chat history.
- **FR-003**: The system MUST provide equivalent CLI-backed or inspect-backed
  operator surfaces for the S7 explanations so the delight layer is not limited
  to chat hosts.
- **FR-004**: Every S7 explanation MUST separate Boundline runtime evidence,
  Canon-governed evidence, model inference, and missing evidence, and MUST
  disclose confidence and fallback mode explicitly.
- **FR-005**: Boundline MUST consume Canon inputs only from the active
  `057-s7-delight-provider` contract line: governed packets, approval states,
  readiness signals, security findings, audit findings, and promotion
  references.
- **FR-006**: S7 outputs MUST remain useful when Canon-governed inputs are
  missing by falling back to workspace, trace, and runtime signals while
  explicitly stating the missing Canon context.
- **FR-007**: The system MUST surface stale, incompatible, contradictory, or
  absent Canon inputs as visible degradation signals and MUST NOT silently merge
  or suppress them.
- **FR-008**: `/boundline:risk`, `/boundline:why`, `/boundline:evidence`, and
  `/boundline:next-best` MUST work on the active session-native route, including
  blocked, failed, exhausted, clarification-required, and terminal states.
- **FR-009**: `/boundline:assumptions` MUST group assumptions by product,
  domain, architecture, implementation, validation, and governance categories,
  and each assumption MUST report source, status, and risk.
- **FR-010**: `/boundline:hidden-impact` MUST use structured repository and
  trace signals by default, MAY incorporate advanced-context intelligence when
  available, and MUST disclose when it is falling back because higher-order
  capability is unavailable.
- **FR-011**: `/boundline:challenge` MUST report objections, missing evidence,
  likely failure modes, and required reviewers or councils without replacing the
  formal governance path for governed work.
- **FR-012**: `inspect` MUST support S7-oriented human-facing lenses for at
  least risk, assumptions, evidence, blockers, and next on the active trace.
- **FR-013**: `/boundline:doctor-context` MUST disclose missing Canon project
  memory, Boundline config, expert-pack inputs, provider readiness, indexes, and
  evidence together with actionable fix commands.
- **FR-014**: The assistant package metadata, global bootstrap assets, and
  host-specific prompts MUST keep the always-visible command palette compact,
  exposing advanced or setup-specific commands contextually where the host
  supports it.
- **FR-015**: S7 surfaces MUST preserve governance boundaries: they MUST report
  authority zone, stop conditions, and required review where relevant, and MUST
  NOT downgrade or bypass governance semantics.
- **FR-016**: The feature MUST remain aligned to Canon 057 through contract and
  integration validation that confirms the Boundline implementation consumes only
  the authorized Canon provider semantics.

### Scope Boundaries *(mandatory)*

- **In Scope**: Boundline-side runtime and assistant-surface implementation for
  S7 cognitive commands; inspect and output projections for human-facing S7
  lenses; assistant package metadata and prompts; zero-config advisory behavior;
  Canon-aware source attribution and degradation disclosure using the Canon 057
  provider contract.
- **Out of Scope**: New Canon provider semantics; new review council or
  reasoning-profile algorithms; a new dashboard product; replacing the existing
  session-native runtime loop; hidden ambient Canon concepts; deployment-only UX
  work unrelated to the S7 surfaces.

### Key Entities *(include if feature involves data)*

- **S7 Command Surface**: The assistant-facing or CLI-facing entry point that
  requests one cognitive affordance such as `why`, `risk`, or `doctor-context`.
- **S7 Explanation View**: The rendered answer that combines runtime signals,
  Canon signals, missing evidence, confidence, and the next safe action.
- **S7 Evidence Source**: One attributed input to an answer, classified as
  runtime, workspace, Canon-governed, model-inferred, or missing.
- **S7 Inspect Lens**: A human-facing `inspect` view such as risk,
  assumptions, evidence, blockers, or next.
- **S7 Capability Disclosure**: The explicit explanation of what S7 could not
  use, why it matters, what fallback was applied, and how the operator can
  improve the result.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new user can obtain a useful `why` or `risk` answer on a real
  changed workspace within five minutes of install or bootstrap, even when Canon
  project memory is missing.
- **SC-002**: In 100% of representative S7 explanation scenarios, the answer
  identifies which statements came from Boundline runtime, Canon-governed input,
  or missing evidence.
- **SC-003**: In 100% of representative degraded scenarios, S7 reports the
  missing, stale, incompatible, or contradictory capability together with the
  fallback mode and the next fix command.
- **SC-004**: The default always-visible assistant palette remains compact: the
  core session-native commands plus the recommended cognitive affordances are
  visible by default, while `explain-plan` and `doctor-context` remain
  contextual or explicitly documented rather than always shown.
- **SC-005**: `inspect` exposes human-facing risk, assumptions, evidence,
  blockers, and next views for the active trace without requiring the operator to
  read raw trace payloads.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: GitHub Copilot supported models reference,
  Anthropic Claude models overview, OpenAI models reference, and Google Gemini
  models reference, reviewed on 2026-05-17.
- **Catalog Delta**: No bundled catalog changes were applied as part of this
  feature specification.
- **No-Change Rationale**: S7 uses the existing assistant host surfaces and
  runtime/model routing already present in Boundline. Current provider docs do
  not require a catalog delta to deliver the S7 assistant-facing layer.

## Assumptions

- Canon 057 remains the authoritative provider contract for governed packets,
  approval states, readiness signals, security findings, audit findings, and
  promotion references.
- Boundline continues to treat session state, traces, and CLI projections as the
  authoritative runtime source for assistant surfaces.
- The existing assistant asset pipeline under `assistant/` remains the correct
  integration surface for Codex, Claude, Copilot, Cursor, and other supported
  hosts.
- Advanced-context intelligence improves `hidden-impact` and related answers
  when available, but S7 must still provide bounded fallback output without it.
- The S7 implementation may adapt names across host packages, but the semantic
  command set and operator affordances remain stable across hosts.
