# Feature Specification: Governed Reasoning Profile Contracts

**Feature Branch**: `061-reasoning-profile-contracts`  
**Created**: 2026-05-18  
**Status**: Implemented  
**Input**: User description: "Implement S6 advanced reasoning profiles by defining a Boundline runtime contract and Canon posture contract for governed challenge activation, profile execution, confidence handoff, traceability, version alignment, and bilateral compatibility updates."

**Implementation Closeout Note**: `061-reasoning-profile-contracts` closes the first release reasoning-profile contract slice: runtime activation inside the existing session flow, fail-closed Canon posture compatibility, operator-visible reasoning projections, and additive reasoning trace vocabulary. Roadmap-level follow-through needed to close every conceptual S6 profile uniformly is intentionally tracked in [`roadmap/S6.1 - reasoning-profile-closure.md`](../../roadmap/S6.1%20-%20reasoning-profile-closure.md) and is intended to seed the next implementation spec (`062-*`) rather than reopen `061`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Activate Bounded Challenge In Delivery Flow (Priority: P1)

A developer using Boundline's primary session-native workflow can activate a bounded reasoning profile inside the current planning, implementation, verification, or review journey when Canon posture or Boundline governance requires stronger challenge.

**Why this priority**: This is the core delivery value of S6. Without an executable activation path inside the existing session lifecycle, reasoning profiles remain theory instead of a usable delivery control.

**Independent Test**: Can be fully tested by preparing a representative planning or verification session with compatible reasoning-profile policy and Canon challenge posture, running the normal workflow, and confirming that Boundline starts one explicit bounded reasoning profile, records its limits and participants, and either completes, degrades, or escalates without creating a second hidden workflow.

**Acceptance Scenarios**:

1. **Given** a session stage that requires stronger challenge and a compatible reasoning-profile policy, **When** the developer runs the normal session-native command, **Then** Boundline activates one explicit reasoning profile within the same session, records the selected profile, participant topology, execution limits, and activation reason, and continues in one bounded lifecycle.
2. **Given** a stage that requests a reasoning profile but cannot satisfy the required independence or participant topology, **When** the developer runs or resumes the session, **Then** Boundline blocks, degrades, or escalates through the existing governance path with an explicit explanation instead of silently reducing the challenge posture.
3. **Given** an active reasoning profile that exhausts its branch, round, revision, or reviewer budget without credible convergence, **When** the bounded profile execution ends, **Then** Boundline records an explicit adjudication, escalation, or terminal condition before delivery continues.

---

### User Story 2 - Inspect Reasoning Evidence And Confidence Handoff (Priority: P2)

An operator can inspect why a reasoning profile activated, which participants were used, where disagreement occurred, how adjudication resolved it, what cost was incurred, and how the resulting confidence contributes back into the existing governance and admission path.

**Why this priority**: Reasoning profiles are not credible if they only run. Operators need one inspectable explanation surface that shows why stronger challenge happened and whether it improved or degraded confidence.

**Independent Test**: Can be fully tested by running representative self-consistency activation plus blocked blind-review, interruption, and contract-drift scenarios through the normal session flow, then checking `plan`, `run`, `status`, `next`, and `inspect` to confirm that the activation reason, Canon posture provenance, disagreement summary, confidence contribution, and next action are visible. Additive heterogeneous, debate, reflexion, and adjudication vocabulary for the first release is then verified through the focused contract and unit coverage recorded in the validation report.

**Acceptance Scenarios**:

1. **Given** a completed or paused reasoning-profile execution, **When** the operator inspects the session or trace, **Then** the surfaced story explains the profile trigger, Canon posture input, participant topology, independence result, disagreement or convergence summary, adjudication result, confidence contribution, and next command without contradicting the primary Boundline workflow.
2. **Given** a reasoning-profile execution that stagnates, oscillates, or is interrupted, **When** the operator checks the same session-native surfaces, **Then** Boundline surfaces the specific bounded failure reason and required next action instead of collapsing the result into a generic failure message.

---

### User Story 3 - Keep Boundline And Canon Contract-Aligned (Priority: P3)

A maintainer can update Boundline and Canon together with explicit reasoning-profile contracts, version windows, and compatibility tests so the two repositories stay aligned on posture ownership, activation semantics, and profile vocabulary.

**Why this priority**: S6 fails if Boundline and Canon drift on who owns challenge posture, confidence inputs, or contract versions. Bilateral alignment is part of the feature, not release cleanup.

**Independent Test**: Can be fully tested by validating that supported Boundline and Canon versions accept the same reasoning-profile and challenge-posture vocabulary, while incompatible contract lines or version windows fail explicitly before runtime execution begins.

**Acceptance Scenarios**:

1. **Given** an unsupported Canon reasoning-posture contract line or missing required posture fields, **When** Boundline validates the active configuration or governed input, **Then** it rejects the incompatible contract explicitly before any reasoning profile runs.
2. **Given** a supported pair of Boundline and Canon repositories, **When** bilateral contract validation runs, **Then** the shared profile vocabulary, challenge-posture fields, confidence handoff fields, and compatibility window align without manual interpretation.
3. **Given** a release candidate for this feature, **When** a maintainer reviews the resulting plan and implementation closeout, **Then** the first task covers Boundline and Canon version bumps plus compatibility-test updates, and the final work closes docs, roadmap, changelog, clippy, and coverage validation coherently.

### Edge Cases

- What happens when Canon challenge posture is present but the required independence dimensions cannot be satisfied because the candidate participants collapse onto the same route, provider, or context?
- What happens when a reasoning profile is selected for an unsupported stage, or when a stage has no requested profile but Canon posture still requires stronger challenge?
- What happens when debate rounds or reflexion revisions stop producing materially new evidence before the configured budget is exhausted?
- What happens when profile execution is interrupted by human override, approval gating, or a higher-authority governance stop before adjudication finishes?
- What happens when Boundline and Canon agree on the profile identifier but disagree on contract line, compatibility window, or confidence-handoff fields?
- What happens when no reasoning profile is required or requested and the existing workflow should continue unchanged?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST preserve the existing session-native Boundline workflow as the primary operator story while activating reasoning profiles only inside already governed planning, implementation, verification, review, or adjudication boundaries.
- **FR-002**: System MUST represent reasoning-profile activation explicitly with the selected profile identifier, target stage, activation trigger, Canon posture provenance, participant topology, execution limits, and current condition.
- **FR-003**: System MUST support explicit runtime vocabulary for bounded self-consistency, blind independent pair review, heterogeneous review, bounded reflexion, and controlled debate.
- **FR-004**: System MUST evaluate effective independence across participant route, provider family, context basis, and prompting pattern when a profile requires independent challenge, and MUST reject or degrade correlated topologies explicitly.
- **FR-005**: System MUST keep Canon as the owner of challenge posture, authority posture, approval semantics, and evidence semantics while Boundline owns profile execution, participant coordination, disagreement handling, cost exposure, and trace emission.
- **FR-006**: System MUST convert Canon-required challenge posture and existing Boundline governance state into one explicit activation decision without introducing a second governance system or replacing existing stop semantics.
- **FR-007**: System MUST enforce explicit bounds for branch count, debate rounds, revision depth, reviewer count, token or call budget, and adjudication scope for every reasoning-profile execution.
- **FR-008**: System MUST return profile outcomes as explicit completion, degradation, blocked, interrupted, escalated, or terminal conditions that hand control back to the existing governance and admission path.
- **FR-009**: System MUST preserve human interruption and override during active reasoning-profile execution.
- **FR-010**: System MUST record reasoning-profile traces for activation, participant start and completion, convergence or disagreement, debate and reflexion progress, adjudication, confidence transitions, cost exposure, and final condition.
- **FR-011**: System MUST surface activation reason, Canon posture provenance, independence assessment, disagreement summary, confidence contribution, cost summary, and next action across the operator-visible planning and inspection surfaces.
- **FR-012**: System MUST feed profile confidence into the existing governance confidence path and MUST NOT define a second standalone trust model.
- **FR-013**: System MUST preserve existing behavior for sessions and stages that do not request or require a reasoning profile.
- **FR-014**: System MUST reject unsupported Canon reasoning-posture contract lines, missing required posture fields, or incompatible Boundline and Canon version windows before profile execution begins.
- **FR-015**: System MUST keep Boundline and Canon documentation, bilateral contracts, compatibility tests, roadmap references, and release notes aligned to the shipped feature and supported version window.
- **FR-016**: System MUST provide focused automated validation coverage for profile activation, degraded and exhausted paths, reasoning trace surfaces, bilateral contract alignment, and version drift detection before the feature is complete.

### Scope Boundaries *(mandatory)*

- **In Scope**: a Boundline-owned reasoning-profile contract and execution layer that reuses existing governance semantics; Canon-owned challenge-posture contract inputs; explicit activation, independence, cost, confidence-handoff, traceability, and bilateral version-alignment behavior; release-facing updates needed to keep both repos coherent.
- **Out of Scope**: unbounded swarm orchestration; recursive agent spawning; Canon-owned reasoning orchestration; replacing existing governance stop semantics; hidden background debate loops; full provider-mode parity for every future profile variant; UI work; deployment pipelines; and persistent autonomous multi-agent societies.

### Key Entities *(include if feature involves data)*

- **Reasoning Profile Activation**: The explicit session-scoped decision that binds one governed stage to one reasoning profile, including its trigger, Canon posture provenance, target stage, limits, and current lifecycle condition.
- **Participant Topology**: The bounded set of reasoning participants, roles, effective routes, and independence requirements used by one profile execution.
- **Independence Assessment**: The recorded result that determines whether the selected participants provide enough diversity or blind review separation to satisfy the requested challenge posture.
- **Reasoning Outcome**: The explicit result of a profile execution, including convergence or disagreement summary, adjudication state, degradation or escalation result, and profile-level next action.
- **Confidence Contribution**: The bounded profile-produced confidence or uncertainty signal that feeds into the existing governance confidence path.
- **Canon Challenge Posture**: The Canon-owned input that states when stronger challenge is required, what minimum posture is expected, and which contract line and compatibility window Boundline may consume.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative planning, verification, and review scenarios, Boundline can activate the expected reasoning profile and keep the work inside one bounded session lifecycle without creating a second hidden workflow.
- **SC-002**: 100% of representative insufficient-independence, budget-exhaustion, interruption, and contract-drift scenarios stop, degrade, or escalate in an explicit condition rather than silently falling back.
- **SC-003**: Operators can identify the activation reason, Canon posture provenance, disagreement summary, confidence contribution, and next action for a reasoning-profile execution from session-native surfaces in under 2 minutes.
- **SC-004**: Bilateral compatibility validation rejects unsupported Boundline and Canon version windows or incompatible posture contract lines in 100% of representative mismatch scenarios.
- **SC-005**: The shipped feature keeps the profile vocabulary, documentation, and release-facing compatibility story aligned across the Boundline and Canon repositories.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: `https://developers.openai.com/api/docs/models`, `https://platform.claude.com/docs/en/docs/about-claude/models`, and `https://ai.google.dev/gemini-api/docs/models`
- **Catalog Delta**: No bundled catalog changes were required for this feature.
- **No-Change Rationale**: The public provider pages still expose the primary coding and reasoning models already captured in `assistant/catalog/model-catalog.toml`, including OpenAI `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.4-nano`; Anthropic `Opus 4.7`, `Sonnet 4.6`, and `Haiku 4.5`; and Google `Gemini 2.5 Pro`, `Gemini 2.5 Flash`, `Gemini 2.5 Flash-Lite`, `Gemini 3.1 Pro Preview`, `Gemini 3 Flash Preview`, and `Gemini 3.1 Flash-Lite`. Newly listed specialized media, audio, and research models fall outside the bundled routing catalog that Boundline uses for the current runtime slots.

## Assumptions

- Boundline will remain the runtime owner of orchestration, traces, and profile execution while Canon remains the owner of challenge posture and governance semantics.
- The sibling Canon repository is available during contract validation so the bilateral compatibility tests can compare the two repositories directly.
- The outer delivery workflow remains sequential-first even when a reasoning profile internally coordinates multiple bounded participants.
- Existing governance, review, confidence, and trace surfaces in Boundline are mature enough to host a first-class reasoning-profile contract layer without redefining the rest of the session model.
