# Feature Specification: Authority-Zoned Delivery Councils

**Feature Branch**: `056-authority-zoned-councils`  
**Created**: 2026-05-15  
**Status**: Draft  
**Input**: User description: "crea le specs in canon e boundline e i branch. Definisci i contract tra i due repository"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Resolve Control Before Acceptance (Priority: P1)

As a Boundline operator using the primary session-native workflow, I want Boundline to resolve one explicit governance posture before work crosses an acceptance boundary, so low-risk work stays fast while structural or risky work receives proportional challenge.

**Why this priority**: This is the admission-control core of the slice. If Boundline cannot explain which council posture applies before continuation, every later review outcome becomes harder to trust.

**Independent Test**: Start representative green, yellow, red, and restricted delivery stages with compatible Canon governance input and verify that Boundline resolves one explicit control class, one council profile, and one stop posture before continuing, waiting, or stopping.

**Acceptance Scenarios**:

1. **Given** a stage whose Canon authority metadata classifies it as green and low-impact, **When** Boundline evaluates the current stage boundary, **Then** it resolves a non-blocking control posture and continues through the primary session-native workflow without inventing extra review overhead.
2. **Given** a stage whose Canon authority metadata classifies it as red or restricted, **When** Boundline reaches a structural or destructive boundary, **Then** it resolves a blocking council posture with the required human gate or hard-stop semantics before any acceptance can occur.
3. **Given** a stage where Canon governance semantics are required but the contract line is unsupported or required authority fields are missing, **When** Boundline evaluates the boundary, **Then** it stops explicitly instead of silently downgrading to an ungoverned path.

---

### User Story 2 - Make Findings Operational (Priority: P2)

As a producer or reviewer, I want council findings, responses, and adjudication outcomes to live inside the same Boundline session story, so blocking concerns become visible remediation or stop states rather than detached commentary.

**Why this priority**: Review only changes delivery quality when findings alter what the system does next. Recording review opinions without an operational response would add cost without control.

**Independent Test**: Run a stage that triggers a yellow or red council, produce representative concern and block findings, record producer responses, and verify that Boundline either creates follow-up work, requires adjudication, or stops explicitly.

**Acceptance Scenarios**:

1. **Given** a council that emits concern or block findings, **When** the producer records responses, **Then** each finding is persisted with its disposition, rationale, and next action state.
2. **Given** a mandatory reviewer role raises an unresolved blocking finding, **When** Boundline evaluates whether the stage can continue, **Then** it requires adjudication or hard-stop semantics instead of treating the finding as informational.
3. **Given** a producer accepts a blocking or concern finding, **When** Boundline records that response, **Then** the session surfaces explicit remediation work or next-step obligations before the stage can return to a proceed state.

---

### User Story 3 - Preserve The Canon And Boundline Boundary (Priority: P3)

As a maintainer, I want Boundline to consume Canon-owned authority semantics without letting Canon assign runtime roles, providers, or models, so the cross-repo contract stays stable while Boundline remains the runtime decision-maker.

**Why this priority**: The design only holds if Canon remains the semantic authority and Boundline remains the runtime orchestrator. If those boundaries blur, both repos will drift and debugging governance behavior will become harder.

**Independent Test**: Feed Boundline Canon `authority-governance-v1` metadata that includes stage role hints and optional provenance fields, and verify that Boundline keeps local control of domain-expert selection, runtime role assignment, council composition, and stop behavior while still surfacing Canon provenance.

**Acceptance Scenarios**:

1. **Given** Canon publishes `stage_role_hints` for a governed packet, **When** Boundline assembles reviewers and domain experts, **Then** it treats those hints as advisory input and records the final runtime choices locally.
2. **Given** Canon metadata conflicts with the locally routable reviewer set, **When** Boundline assembles a council, **Then** it preserves the local runtime boundary and records why hints were narrowed, ignored, or rejected.
3. **Given** a required council cannot be assembled without collapsing reviewer independence, **When** Boundline checks quorum and mandatory roles, **Then** it enters an explicit stop state rather than fabricating a credible council.

### Edge Cases

- Canon authority metadata is present but uses an unsupported contract line, so Boundline must fail closed without discarding the rest of the session context.
- Canon omits `persona_anti_behaviors`, `primary_artifact`, `artifact_order`, or `promotion_refs`, so Boundline must continue when the required `authority-governance-v1` control fields are present and expose the missing optional provenance as unavailable rather than blocking control resolution.
- Canon `stage_role_hints` name capabilities that no current runtime route can satisfy, so Boundline must show the mismatch instead of pretending the role exists.
- A council reaches quorum numerically, but the counted reviewers collapse onto the same effective route, so independence fails even though participation count looks sufficient.
- A restricted or red stage reuses earlier green-stage context, so the stronger authority posture must still govern the current decision boundary.
- A blocking finding remains unresolved when the operator resumes the session later, so Boundline must preserve the stop reason and next action rather than reverting to advisory mode.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST consume Canon `authority-governance-v1` for this slice and use `authority_zone`, `change_class`, `intended_persona`, `approval_state`, `packet_readiness`, and `risk` as the required Canon control inputs for runtime governance resolution.
- **FR-002**: Boundline MUST keep the primary session-native workflow as the operator story and apply authority-zoned councils as an admission-control overlay rather than as a separate hidden workflow.
- **FR-003**: Boundline MUST distinguish between authoring persona, runtime role, model route, provider capability, and final decision authority in inspectable runtime state.
- **FR-004**: Boundline MUST compute one explicit effective control class for each governed boundary using Canon authority semantics together with the local stage, assurance, and evidence posture.
- **FR-005**: Boundline MUST map the effective control class to one bounded council profile from this first slice: `none`, `light_single`, `yellow_pair`, `red_five`, or `restricted_manual`.
- **FR-006**: Boundline MUST select runtime roles, mandatory reviewers, and domain experts locally from repository signals, workspace configuration, target surfaces, active routes, and compatible Canon hints.
- **FR-007**: Boundline MUST treat optional Canon `stage_role_hints` as advisory only and MUST NOT let Canon assign executable runtime roles, provider routes, model routes, retry policy, or final decision authority.
- **FR-008**: Boundline MUST enforce reviewer independence for every council profile that counts quorum reviewers, and reviewers that collapse onto the same effective route MUST NOT satisfy independence or quorum by themselves.
- **FR-009**: Boundline MUST persist every council finding with reviewer identity, runtime role, severity, disposition, summary, required action, confidence, and evidence references.
- **FR-010**: Boundline MUST require a producer response of `accepted`, `rejected`, or `deferred` with rationale for every finding classified as a concern or block.
- **FR-011**: When a producer accepts a concern or block finding, Boundline MUST create explicit remediation work, follow-up work, or plan updates before the stage can continue as resolved.
- **FR-012**: Boundline MUST support this stop-semantics vocabulary for the first slice: `proceed`, `proceed_with_advisory`, `proceed_with_warning`, `degraded_proceed`, `council_required`, `adjudication_required`, `human_gate_required`, and `hard_stop`.
- **FR-013**: Boundline MUST enter `hard_stop` when governance is required and any of these conditions remain true at decision time: unsupported Canon contract line, missing Canon `authority-governance-v1` required control metadata, missing mandatory reviewer role, failed reviewer independence, required council cannot be assembled, restricted action lacks the required human gate, or a blocking finding remains unresolved.
- **FR-014**: Boundline MUST expose the resolved control class, runtime roles, selected domain experts, council profile, findings, producer responses, adjudication results, stop semantics, consumed Canon contract line, and any present provenance-only Canon fields through `plan`, `run`, `status`, `next`, and `inspect`.
- **FR-015**: Boundline MUST preserve explicit compatibility behavior for sessions where Canon input is absent and governance is not required, rather than assuming Canon is mandatory for ordinary delivery.
- **FR-016**: Boundline MUST fail closed when Canon authority semantics are required but missing, incomplete, or incompatible with the first-slice contract.
- **FR-017**: Boundline MUST keep deterministic validation, tests, security scanning, and human approval as separate controls and MUST NOT treat review councils as a replacement for those controls.
- **FR-018**: Boundline MUST keep this slice limited to bounded council structures and MUST leave adaptive activation timing, degradation timing, and advanced multi-agent reasoning profiles outside the scope of the first implementation.

### Scope Boundaries *(mandatory)*

- **In Scope**: compatible Canon authority-semantic consumption; effective control-class resolution; bounded council profiles; reviewer independence checks; structured findings and producer responses; adjudication and stop-posture projection; inspectable runtime surfaces; and explicit cross-repo boundary preservation.
- **Out of Scope**: Canon-owned authority vocabulary authoring; operational interpretation of Canon `persona_anti_behaviors`, `primary_artifact`, `artifact_order`, and `promotion_refs` beyond inspectable provenance; provider-specific governance logic; distributed execution; always-on councils; advanced debate or reasoning profiles; long-term memory expansion; UI work; and deployment pipelines.

### Key Entities *(include if feature involves data)*

- **Authority Control Resolution**: The explicit decision record that combines Canon authority semantics with local stage evidence to produce one effective control class for the current boundary.
- **Council Profile Decision**: The persisted runtime outcome that names the selected council profile, mandatory reviewer expectations, independence posture, and stop semantics for the current stage.
- **Structured Finding Record**: The persisted record for one council finding, including reviewer provenance, severity, disposition, rationale, evidence references, and required action.
- **Producer Response Record**: The explicit producer-side response to a concern or blocking finding, including accepted, rejected, or deferred disposition and the rationale for that decision.
- **Adjudication Outcome**: The decision record that explains how mixed or blocking council results were resolved, escalated, or converted into a stop state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative green, yellow, red, and restricted scenarios, 100% of governed stage boundaries resolve to one explicit council profile and one explicit stop posture before continuation or stopping occurs.
- **SC-002**: In representative failure scenarios involving unsupported Canon contracts, missing authority metadata, missing mandatory reviewer roles, or failed independence checks, 100% of runs stop explicitly without silent downgrade to an ungoverned path.
- **SC-003**: Developers can identify the current control class, council profile, mandatory reviewer posture, and next action from `status`, `next`, or `inspect` in under 2 minutes.
- **SC-004**: In representative concern and block scenarios, 100% of accepted findings generate recorded remediation work or plan updates before the session returns to a proceed state.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: OpenAI Models documentation at `https://developers.openai.com/api/docs/models`, Anthropic Models Overview at `https://platform.claude.com/docs/en/docs/about-claude/models`, and Google Gemini Models documentation at `https://ai.google.dev/gemini-api/docs/models`, reviewed on 2026-05-15.
- **Catalog Delta**: No bundled catalog changes were required for this authority-zoned governance slice.
- **No-Change Rationale**: The bundled catalog at `assistant/catalog/model-catalog.toml`, version `0.55.0` and updated on `2026-05-15`, already reflects the currently documented coding-relevant model families reviewed during spec creation, including OpenAI `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.4-nano`; Anthropic `opus-4.7`, `sonnet-4.6`, and `haiku-4.5` plus existing compatibility entries; and Google `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite`, `gemini-3.1-pro-preview`, `gemini-3-flash-preview`, `gemini-3.1-flash-lite`, and `gemini-3.1-flash-lite-preview`. This slice changes governance semantics rather than provider or model availability.

## Assumptions

- Canon will publish a compatible first-slice authority-governance contract on the governed packets or metadata surfaces that Boundline already consumes.
- Boundline continues to operate one primary active session per workspace, so council state and stop posture can remain session-scoped in the first slice.
- Human approval and restricted-action gates remain real operator-controlled boundaries that Boundline can surface but not bypass.
- Councils activate only at meaningful delivery boundaries such as planning, implementation, verification, review, or destructive-action decisions rather than on every micro-step.
