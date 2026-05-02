# Feature Specification: Native Direct Run

**Feature Branch**: `030-native-direct-run`  
**Created**: 2026-05-02  
**Status**: Draft  
**Input**: User description: "Make the direct synod run --workspace <workspace> --goal <goal> command bootstrap and execute the session-native goal-plan route by default so the primary delivery story no longer depends on the explicit fixture-backed compatibility path. Keep explicit compatibility execution as a subordinate opt-in path. Include version bump, impacted docs and changelog, coverage above 95% for modified Rust files, clippy cleanup, and cargo fmt."

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

### User Story 1 - Run A Goal Natively In One Command (Priority: P1)

An operator can start from a workspace plus a goal and use one `run` command to
enter the primary session-native delivery path, reach real workspace changes,
run validation, and inspect persisted follow-up state without manually calling
`start`, `capture`, and `plan` first.

**Why this priority**: The roadmap says the meaningful delivery story must stop
depending on the explicit compatibility path. If direct `run --goal` still goes
through compatibility by default, Synod's primary product surface remains split.

**Independent Test**: In a representative Rust workspace with a failing test,
run `synod run --workspace <workspace> --goal "Fix the failing add test"` and
verify that the command completes on the native goal-plan route, mutates the
workspace, records decisions and traces, and leaves `status`, `next`, and
`inspect` usable from the persisted session.

**Acceptance Scenarios**:

1. **Given** a writable workspace with no active session and a goal that maps to
  a bounded delivery flow, **When** the operator runs `synod run --workspace
  <workspace> --goal "Fix the failing add test"`, **Then** Synod bootstraps a
  native session, produces an executable goal plan, executes on the native
  route, and reports native routing and terminal output instead of explicit
  compatibility routing.
2. **Given** a writable workspace with no active session and a goal that does
  not produce a credible clarified plan yet, **When** the operator runs the
  same direct `run` command, **Then** Synod stops explicitly before unsafe
  execution, persists inspectable session or trace state, and tells the
  operator what clarification or follow-up is needed.
3. **Given** a direct native run has completed, **When** the operator runs
  `synod status`, `synod next`, or `synod inspect`, **Then** those surfaces
  continue from the persisted native session and trace story instead of acting
  like a compatibility-only run occurred.

---

### User Story 2 - Keep Compatibility Explicit And Session-Safe (Priority: P2)

An operator can still choose the compatibility execution path deliberately when
needed, while Synod avoids silently overwriting an already meaningful native
session when a new direct-run goal arrives.

**Why this priority**: Making direct `run --goal` native by default is only
credible if compatibility stays explicit and if convenience does not come from
secretly discarding active session state.

**Independent Test**: Exercise direct run on a workspace with an existing active
session and on a workspace where compatibility is explicitly requested, then
verify that Synod either preserves the active session safely or surfaces the
deliberate compatibility route explicitly.

**Acceptance Scenarios**:

1. **Given** a workspace with an active session that already contains captured,
   planned, or in-progress delivery state, **When** the operator provides a new
   direct-run goal, **Then** Synod does not silently overwrite that state and
   instead requires an explicit reset or continuation decision.
2. **Given** an operator deliberately chooses the compatibility route,
   **When** Synod executes that run, **Then** routing, execution path, trace
   inspection, and follow-up output stay explicitly compatibility-owned instead
   of being mistaken for session-native delivery.
3. **Given** a direct run goal does not infer a credible built-in flow,
   **When** the operator executes the command, **Then** Synod still produces an
   executable bounded native plan without stopping on a manual flow-confirmation
   detour.

---

### User Story 3 - Ship Native Direct Run As 0.30.0 (Priority: P3)

A maintainer can ship `0.30.0` with runtime behavior, docs, assistant guidance,
version metadata, and validation evidence all describing the same native direct
run story.

**Why this priority**: The product change is user-facing and route-defining. If
the CLI changes but docs, changelog, assistant prompts, and validation evidence
still describe direct `run --goal` as compatibility-first, the release will be
internally contradictory.

**Independent Test**: Follow the updated `0.30.0` operator guidance on a
representative workspace, then run the release validation suite and confirm the
modified or created Rust files stay above 95% coverage with clean clippy and
formatting results.

**Acceptance Scenarios**:

1. **Given** the `0.30.0` release artifacts, **When** a maintainer follows the
  documented direct-run workflow, **Then** the observed output, routing, and
  follow-up behavior match the native-first release story.
2. **Given** modified or created Rust sources for this slice, **When** the
  maintainer runs the release validation suite, **Then** formatting, clippy,
  focused tests, and refreshed coverage complete successfully and touched-file
  coverage remains above 95%.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when direct `run --goal` is invoked against a workspace that
  already has a captured or planned native session?
- How does the system handle a direct native run that fails validation or has no
  deterministic workspace change available for the current target?
- How does the system surface primary native routing versus an explicitly chosen
  compatibility route after the route default changes?
- What happens when goal text does not produce a confident built-in flow, but
  the operator still requested one-command execution?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST let `synod run --workspace <workspace> --goal <goal>`
  enter the primary session-native delivery path without requiring a prior
  manual `start`, `capture`, or `plan` command sequence.
- **FR-002**: System MUST persist the same core native session state a manual
  session-native flow would create, including captured goal state, negotiation
  projection, goal-plan state, decisions, and trace continuity needed by later
  `status`, `next`, and `inspect` commands.
- **FR-003**: System MUST turn direct-run input into an immediately executable
  bounded native route by confirming an inferred built-in flow when credible or
  by using no-flow native planning when no credible flow is inferred.
- **FR-004**: System MUST NOT silently overwrite an active session that already
  contains meaningful captured, planned, or in-progress delivery state when a
  new direct-run goal is provided.
- **FR-005**: System MUST keep explicit compatibility execution available only
  when the operator chooses it deliberately, and MUST continue surfacing
  compatibility routing ownership explicitly on run, next, status, and inspect.
- **FR-006**: System MUST no longer require a workspace execution profile as a
  prerequisite for the native direct-run entry path.
- **FR-007**: System MUST handle at least one non-success path such as
  clarification-required input, validation failure, or unavailable native code
  change without losing inspectable session or trace state.
- **FR-008**: System MUST keep direct-run-native output aligned across `run`,
  `status`, `next`, `inspect`, and any workflow continuation surfaces that read
  the same persisted session state.
- **FR-009**: System MUST update tests, version metadata, impacted docs,
  assistant guidance, and changelog together for the `0.30.0` release.
- **FR-010**: System MUST refresh coverage for modified or created Rust files,
  keep touched-file coverage above 95%, resolve clippy issues introduced by the
  slice, and finish with repository formatting applied.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: native direct-run bootstrapping from explicit goal and optional
  briefs on a workspace target; automatic creation of executable native session
  state; explicit subordinate compatibility selection; session safety for
  pre-existing active work; `0.30.0` release closeout including version bump,
  docs, changelog, coverage, clippy cleanup, and formatting.
- **Out of Scope**: Canon-governed development stages inside the live code
  delivery loop; provider-gateway or model-capability abstraction; distributed
  orchestration; background daemons; UI work; replacing the bounded native code
  mutation heuristics with a broader autonomous coding system.

### Key Entities *(include if feature involves data)*

- **Direct Run Bootstrap Request**: The operator's one-command request to start
  bounded delivery from a workspace plus goal, optionally with briefs and
  governance intent. It decides whether Synod can create a native session or
  must stop explicitly.
- **Bootstrap Session Projection**: The persisted native session state created
  by direct run, including goal, negotiation summary, goal plan, decisions,
  trace reference, and any next-step follow-through.
- **Route Choice**: The explicit ownership choice between native direct-run and
  deliberately requested compatibility execution. It must remain inspectable on
  all current read-side surfaces.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative Rust workspaces with no active session,
  100% of direct `run --workspace <workspace> --goal <goal>` executions use the
  native goal-plan route by default instead of the explicit compatibility path.
- **SC-002**: In representative direct-run scenarios, operators can reach a
  persisted session, changed files, validation outcome, and usable `status` or
  `inspect` follow-up in one command and under 2 minutes of operator time.
- **SC-003**: In representative non-success direct-run scenarios, 100% of runs
  stop in an explicit terminal or blocked state with enough persisted session or
  trace output for a maintainer to understand the next action in under 5 minutes.
- **SC-004**: Maintainers can validate the `0.30.0` native direct-run story,
  including touched-Rust coverage above 95%, using the shipped repository
  commands in under 20 minutes.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Operators invoking direct `run --goal` expect Synod to prefer the primary
  session-native route unless they explicitly ask for compatibility behavior.
- The current bounded native decision loop and deterministic workspace-change
  adapters are sufficient for this release to claim a native direct-run entry
  story without introducing a broader autonomous coding engine.
- Existing session and trace persistence surfaces under `.synod/` remain the
  authoritative state model for the feature.
- Canon-governed code-development stages remain deferred to the next roadmap
  feature rather than being bundled into this release.
