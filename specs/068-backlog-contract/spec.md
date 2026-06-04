# Feature Specification: Backlog Contract

**Feature Branch**: `068-backlog-contract`

**Created**: 2026-06-03

**Status**: Implemented

**Input**: User description from `feat-backlog-contract.md`, promoted as the
next Boundline planning-readiness slice with Canon `0.67.0` compatibility,
release-surface alignment, documentation updates, and changed-file coverage
closure.

## User Scenarios & Testing

### User Story 1 - Block Unsafe Backlog Handoff (Priority: P1)

As a repository operator, I can rely on Boundline to stop before execution when
Canon backlog output is structurally unsafe or still lacks governed handoff
evidence, so planning does not silently become executable work on the basis of
a weak backlog packet.

**Independent Test**: In an isolated temporary workspace, evaluate one
closure-limited Canon backlog packet, one full packet without
`execution-handoff.md`, and one full packet with stable `slice_id`,
implementation refs, verification anchors, and execution handoff. Confirm that
Boundline blocks or requests clarification before any run handoff and advances
only for the credible packet.

**Acceptance Scenarios**:

1. **Given** goal quality and plan quality are already ready, **When** Canon
   emits only the closure-limited backlog packet, **Then** Boundline marks
   backlog quality as `blocked`, records the finding, emits no execution
   handoff, and keeps planning non-terminal.
2. **Given** goal quality and plan quality are already ready, **When** Canon
   emits a full backlog packet but omits `execution-handoff.md` or equivalent
   downstream-ready evidence, **Then** Boundline marks backlog quality as
   `clarification_required`, emits exactly one actionable `phase_request`, and
   does not advance to execution.
3. **Given** goal quality and plan quality are already ready, **When** Canon
   emits a full backlog packet with stable `slice_id`, implementation refs,
   independent verification anchors, and `execution-handoff.md`, **Then**
   Boundline marks backlog quality as `ready` and may continue toward
   execution admission.

### User Story 2 - Inspect Backlog Readiness (Priority: P2)

As a repository operator, I can inspect backlog quality state, findings, task
count, MVP scope, and unmapped items through the normal runtime surfaces, so I
can understand why execution is blocked or allowed without reconstructing the
decision from chat history.

**Independent Test**: Evaluate ready, clarification-required, and blocked
backlog packets and confirm that session status, orchestration snapshots,
rendered output, and traces expose the same additive backlog-quality contract
without breaking older consumers.

### User Story 3 - Preserve Backlog Gates Through Assistant Surfaces
(Priority: P3)

As an assistant user, I receive a consistent planning response across supported
hosts when backlog quality prevents progress, so I can answer one focused
question or resume planning without any assistant inventing a run step that the
runtime has not allowed.

**Independent Test**: Validate the supported plan and run assistant assets
against blocked and clarification-required backlog states and confirm that each
host preserves the backlog-quality projection and planning-stage resume
routing.

### Edge Cases

- Goal quality or plan quality is unresolved when backlog evaluation would
  otherwise run; Boundline preserves the earlier gate and does not evaluate
  backlog quality prematurely.
- No Canon backlog packet is present yet; Boundline keeps planning active and
  does not fabricate backlog readiness from plan text alone.
- A full packet exists but `execution-handoff.md` is missing or incomplete;
  Boundline uses `clarification_required` instead of guessing executable work.
- A packet exposes delivery slices but no stable `slice_id`; Boundline blocks
  instead of inventing identifiers.
- Older session snapshots or consumers ignore the additive backlog-quality
  fields; existing status and orchestration behavior remains compatible.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST evaluate backlog quality only after goal quality
  and plan quality are satisfied and before planning analysis or execution
  handoff is offered.
- **FR-002**: The first release slice MUST validate only Canon backlog fields
  already available through the Canon `0.67.0` backlog packet and MUST NOT
  require new Boundline-owned schema invention.
- **FR-003**: The system MUST represent backlog quality with the states
  `ready`, `clarification_required`, and `blocked`.
- **FR-004**: The system MUST treat closure-limited or risk-only Canon backlog
  packets as `blocked`.
- **FR-005**: The system MUST require a full Canon backlog packet plus stable
  `slice_id` evidence before backlog quality can be `ready`.
- **FR-006**: The system MUST require governed downstream-ready evidence such
  as `execution-handoff.md`, implementation refs, and independent verification
  anchors before backlog quality can be `ready`.
- **FR-007**: When a full packet is structurally credible but still lacks
  governed handoff evidence, the system MUST keep planning non-terminal, mark
  backlog quality as `clarification_required`, emit exactly one actionable
  `phase_request`, and preserve the existing planning-stage resume routing.
- **FR-008**: When backlog structure is contradictory or unsafe to interpret,
  the system MUST mark backlog quality as `blocked`, withhold execution
  handoff, and avoid hidden fallback ordering or task synthesis.
- **FR-009**: The system MUST record concise machine-readable backlog-quality
  findings for missing, invalid, or weak backlog structure.
- **FR-010**: The system MUST expose additive backlog projection fields in
  session status, orchestration snapshots, and rendered runtime output whenever
  backlog data is expected or available: `backlog_quality_state`,
  `backlog_quality_findings`, `backlog_task_count`, `backlog_mvp_scope`, and
  `backlog_unmapped_items`.
- **FR-011**: The system MUST preserve compatibility with persisted session
  snapshots that do not contain backlog-quality fields and with consumers that
  ignore additive backlog projections.
- **FR-012**: Supported plan and run assistant assets MUST preserve
  backlog-quality blocked or clarification-required states, additive
  backlog-quality fields, any emitted `phase_request`, and planning-stage
  resume routing without deriving execution handoff from chat-only
  assumptions.
- **FR-013**: Supported plan and run assistant assets MUST explain that Canon
  backlog is governed source material while Boundline owns execution-readiness
  validation.
- **FR-014**: The release MUST update the workspace version and aligned
  metadata, adopt Canon `0.67.0` in compatibility-facing Boundline
  documentation or manifests where applicable, and update `README.md`,
  user-facing docs under `docs/`, engineering docs under `tech-docs/`, and
  `CHANGELOG.md`.
- **FR-015**: The release MUST pass formatting and clippy validation with
  warnings rejected.
- **FR-016**: The release MUST demonstrate at least 95 percent changed-file
  coverage for every touched Rust implementation file and MUST use the
  repository patch-coverage helper when reporting that result.

### Scope Boundaries

- This feature adds the first backlog-readiness gate only; deeper planning
  analysis remains a separate roadmap slice.
- This feature does not add a new backlog or tasks command, a second planning
  runtime, or a file-first Speckit workflow.
- This feature does not change Canon ownership boundaries; Canon may supply
  governed backlog packets, but Boundline owns validation and execution
  admission.

## Key Entities

- **Canon Backlog Packet**: The governed backlog payload emitted by Canon and
  consumed by Boundline as source material for backlog-readiness validation.
- **Backlog Quality Assessment**: The typed runtime decision that records
  backlog state, concise findings, task count, MVP scope, and unmapped items
  for an active session.
- **Backlog Quality Finding**: A machine-readable reason that backlog
  structure is missing, invalid, weak, or incomplete, including whether the
  issue blocks execution or can be resolved through clarification.
- **Backlog Clarification Request**: The single highest-priority operator
  question emitted through the existing `phase_request` contract while planning
  remains non-terminal.

## Success Criteria

- **SC-001**: In all validation scenarios where Canon emits only the
  closure-limited backlog packet, Boundline withholds execution handoff and
  reports backlog quality as `blocked`.
- **SC-002**: In all validation scenarios where the full packet is otherwise
  credible but omits execution-handoff evidence, Boundline returns exactly one
  actionable clarification path and does not advance to execution.
- **SC-003**: In all validation scenarios where the full packet includes stable
  `slice_id`, implementation refs, verification anchors, and
  `execution-handoff.md`, Boundline reports backlog quality as `ready` and
  exposes task count and MVP scope.
- **SC-004**: In all compatibility scenarios using older persisted session
  snapshots without backlog-quality fields, status and orchestration inspection
  complete successfully.
- **SC-005**: All supported plan and run assistant assets preserve
  backlog-quality gates and do not route to execution while backlog quality is
  blocked or requires clarification.
- **SC-006**: Release validation completes with formatting checks passing,
  clippy producing zero warnings, and changed or created Rust implementation
  files meeting at least 95 percent changed-file coverage.
- **SC-007**: Boundline compatibility-facing release surfaces reference Canon
  `0.67.0` consistently wherever the backlog-contract feature depends on Canon
  backlog output.

## Assumptions

- Canon `0.67.0` is the compatibility target for the source backlog packet used
  by this slice.
- The existing goal-quality gate, plan-quality gate, `phase_request`,
  planning-stage resume command, session status, orchestration snapshot,
  inspect, and trace surfaces remain the reusable runtime contracts.
- Canon now publishes the full packet evidence this slice needs, including
  stable `slice_id` and additive execution-handoff artifacts.
- Release closure uses the next minor pre-1.0 workspace version because the
  feature adds a new planning-readiness gate and additive runtime projections.
