# Feature Specification: Inspectable Routing And Assistant Decoupling

**Feature Branch**: `027-routing-assistant-decoupling`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Add inspectable model and assistant routing so operators can see and control which provider/model slot powers planning, verification, review, governance, and assistant command packs through existing session-native and compatibility surfaces, while decoupling assistant packs from hard-wired backends without creating a second orchestration runtime. Include release closeout tasks for version bump, impacted docs and changelog, coverage for modified Rust files, clippy cleanup, and cargo fmt."

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

### User Story 1 - See The Active Routing Decision (Priority: P1)

An operator can see which configured provider and model currently own each
bounded delivery slot, and can inspect why that routing decision was chosen,
before or after execution begins.

**Why this priority**: The roadmap now depends on backend clarity, and the
smallest independently valuable improvement is to stop hiding which configured
runtime or model is actually driving planning, implementation, verification,
review, or related assistant-backed work.

**Independent Test**: Configure representative slot routing, run a bounded
session-native or explicit compatibility flow, and verify that `run`, `status`,
`next`, or `inspect` expose the active routing decision and its authority
source without changing the underlying delivery path.

**Acceptance Scenarios**:

1. **Given** a workspace with explicit routing defaults for at least one slot,
  **When** the operator runs the delivery flow or inspects the resulting
  follow-up state, **Then** Boundline identifies which provider/model route owns
  the active slot and where that routing decision came from.
2. **Given** a workspace using built-in defaults instead of explicit overrides,
  **When** the operator checks the same surfaces, **Then** Boundline still reports
  the active route and makes it clear that the decision came from defaults
  rather than a hidden heuristic.
3. **Given** a non-success path such as blocked, failed, exhausted, or
  inspect-only follow-up, **When** the operator uses `status`, `next`, or
  `inspect`, **Then** the routing decision remains visible alongside the
  bounded stop condition instead of disappearing once execution is no longer
  progressing.

---

### User Story 2 - Rebind Assistant Packs Without A Second Runtime (Priority: P2)

An operator or maintainer can change which assistant backend or command-pack
family is used for a bounded delivery slot without creating a second
orchestration surface or implying that assistant packs own execution.

**Why this priority**: Visibility alone is incomplete if assistant command packs
remain effectively hard-wired to backend assumptions that the routing surface
cannot actually honor.

**Independent Test**: Change the configured route for one or more slots and
verify that the selected assistant-backed behavior follows the configured
binding while the primary session-native and explicit compatibility routes stay
unchanged.

**Acceptance Scenarios**:

1. **Given** a bounded delivery slot whose configured provider/model route is
  changed, **When** Boundline prepares the assistant-backed execution or guidance
  for that slot, **Then** it uses the bound assistant/backend family implied by
  the active route instead of a hard-coded default.
2. **Given** multiple assistant families that can express the same bounded
  workflow, **When** the operator changes routing, **Then** Boundline keeps command
  names, follow-up surfaces, and orchestration ownership stable while changing
  only the bound backend or command-pack family.
3. **Given** an explicit compatibility route or clustered session-owned route,
  **When** assistant/backend binding is projected, **Then** Boundline keeps route
  ownership explicit and does not imply that assistant selection created a new
  execution authority.

---

### User Story 3 - Ship Routing Transparency As One Release (Priority: P3)

A maintainer can ship one coherent `0.27.0` release where runtime behavior,
assistant guidance, docs, changelog, version metadata, and validation evidence
all describe the same inspectable routing and assistant-decoupling story.

**Why this priority**: This slice changes how operators interpret execution
ownership and backend selection. The release is incomplete if routing remains
clear in code but stale in docs, assistant prompts, or validation discipline.

**Independent Test**: Follow the updated docs on a representative configured
workspace, confirm that runtime output matches the documented routing story, and
complete release validation including version bump, docs/changelog updates,
coverage refresh for touched Rust files, clippy cleanup, and formatting.

**Acceptance Scenarios**:

1. **Given** the `0.27.0` release artifacts, **When** a maintainer follows the
  documented routing workflow, **Then** the observed runtime output matches the
  documented routing-decision and assistant-binding behavior.
2. **Given** changed Rust sources for this slice, **When** maintainers run the
  release validation suite, **Then** formatting, clippy, required tests, and
  coverage refresh for modified or created Rust files complete without
  undocumented regressions.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when a slot route is partially configured, contradictory across
  scopes, or falls back to defaults because no explicit binding exists?
- How does the system behave when a configured assistant/backend family cannot
  satisfy the bounded slot it was asked to power?
- How does the system surface routing and assistant binding on explicit
  compatibility follow-up without implying that compatibility became the
  session-native authority?
- What happens when clustered delivery inherits routing defaults from the
  primary workspace but a member workspace exposes a different local default?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST make the active provider/model routing decision for
  each bounded delivery slot visible on existing operator-facing surfaces.
- **FR-002**: System MUST identify the authority source for each visible routing
  decision, distinguishing explicit overrides from defaults.
- **FR-003**: System MUST preserve routing-decision context in the same session,
  trace, or follow-up story already used for bounded delivery execution.
- **FR-004**: System MUST keep routing-decision projection visible on at least
  one representative non-success path such as blocked, failed, exhausted, or
  inspect-only follow-up.
- **FR-005**: System MUST allow assistant/backend binding for bounded delivery
  slots to follow the active routing decision instead of remaining hard-wired to
  one assistant family.
- **FR-006**: System MUST preserve the existing session-native workflow as the
  primary operator path and MUST keep any explicit compatibility behavior
  visibly separate.
- **FR-007**: System MUST preserve bounded execution ownership inside Boundline and
  MUST NOT introduce a second orchestration runtime, background daemon, or
  hidden fan-out control loop.
- **FR-008**: System MUST keep command-pack selection and backend binding
  inspectable enough that maintainers can understand why one assistant family
  was chosen over another.
- **FR-009**: System MUST preserve or improve existing trace and follow-up
  inspectability when routing or assistant binding changes.
- **FR-010**: System MUST update runtime behavior, tests, version metadata,
  impacted docs, assistant guidance, and changelog together for the `0.27.0`
  release.
- **FR-011**: System MUST refresh coverage for modified or created Rust files,
  resolve clippy issues introduced by the slice, and finish with repository
  formatting applied.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: inspectable slot-routing decisions on existing delivery
  surfaces; assistant/backend binding that follows configured routing; explicit
  route-authority preservation for session-native, compatibility, and clustered
  follow-up; `0.27.0` release closeout including version bump, impacted docs,
  changelog, coverage refresh, clippy cleanup, and formatting.
- **Out of Scope**: provider authentication flows; a generic model gateway;
  autonomous assistant orchestration outside Boundline; UI or dashboard work;
  distributed execution; long-term memory; deployment-pipeline changes; generic
  plugin ecosystems for arbitrary third-party backends.

### Key Entities *(include if feature involves data)*

- **Routing Decision Record**: The explicit summary of which bounded slot is
  active, which provider/model route owns it, where that decision came from,
  and why it is authoritative for the current execution or follow-up story.
- **Assistant Binding**: The inspectable association between an active routing
  decision and the assistant/backend family used to express or execute that
  bounded slot without changing orchestration ownership.
- **Routing Projection**: The persisted or derived follow-up view that carries
  routing and assistant-binding context through session, trace, or
  inspect-oriented surfaces.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative configured workspaces, operators can identify
  the active provider/model route and its authority source from `run`, `status`,
  `next`, or `inspect` in under 2 minutes.
- **SC-002**: 100% of representative non-success follow-up scenarios preserve
  an explicit routing story instead of dropping backend selection context.
- **SC-003**: Maintainers can change at least one bounded slot's assistant or
  backend binding without changing the command workflow or introducing a second
  runtime surface.
- **SC-004**: Maintainers can validate the `0.27.0` routing story, including
  touched-Rust coverage output, in under 20 minutes using the shipped docs and
  repository validation commands.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Session-native delivery remains the primary operator path for this slice, and
  explicit compatibility behavior remains a separate named route.
- Existing routing configuration already contains enough slot and model context
  to support inspectable projection without inventing a new control plane.
- Assistant command packs continue to wrap the local Boundline CLI instead of
  becoming a second execution authority.
- The `0.27.0` release should prefer the smallest coherent improvement to
  routing transparency and assistant binding before any broader provider-gateway
  work is considered.
