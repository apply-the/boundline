# Feature Specification: Delivery Control Consumer

**Feature Branch**: `051-delivery-control-consumer`  
**Created**: 2026-05-13  
**Status**: Draft  
**Input**: User description: "Consume Canon-owned project-memory and delivery-control contracts through existing workflow, config, and runtime surfaces, with tiered stop conditions and explicit consumer-version pinning, without redefining Canon promotion policy."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Plan Credibly From Repo-Visible Canon Knowledge (Priority: P1)

As a Boundline operator driving a bounded delivery session, I want Boundline to
consume stable Canon project memory and evidence as planning inputs while
stopping or warning explicitly when required inputs are missing, so the next
stage comes from credible context instead of silent guesswork.

**Why this priority**: This is the core value of the consumer slice. If
Boundline cannot distinguish stable, missing, stale, and blocked inputs, the
control layer is ornamental instead of operational.

**Independent Test**: Present Boundline with representative stable,
missing-artifact, stale-memory, and blocked-governance scenarios and verify
that it either proceeds credibly, warns, or hard-stops according to the V1
consumer rules.

**Acceptance Scenarios**:

1. **Given** stable Canon project memory and evidence with a compatible
   contract line, **When** Boundline proposes the next bounded stage, **Then**
   it may use that material as credible context for planning and continuation.
2. **Given** Canon governance is blocked or a required source artifact is
   missing, **When** Boundline evaluates continuation, **Then** it stops in an
   explicit hard-stop state instead of inferring missing producer facts.
3. **Given** project memory is stale or an evidence source is missing but other
   credible context still exists, **When** Boundline evaluates continuation,
   **Then** it warns or replans without escalating that condition to a V1 hard
   stop.

---

### User Story 2 - Extend Existing Runtime Surfaces Without Registry Collisions (Priority: P1)

As a Boundline maintainer, I want delivery-control consumption to extend the
existing workflow and topology surfaces instead of introducing competing files,
so V1 integrates into the current product model rather than creating duplicate
registries.

**Why this priority**: Introducing `delivery-paths.toml` or overloading
`cluster.toml` would create ambiguity before the control layer is even usable.

**Independent Test**: Inspect the consumer spec and follow-up design artifacts
and verify that delivery paths remain part of `.boundline/workflows.toml`, while
project semantics live in `project.boundline.toml` and workspace topology stays
in `.boundline/cluster.toml`.

**Acceptance Scenarios**:

1. **Given** a repo that defines delivery-control stages, **When** Boundline
   models those stages for V1, **Then** it represents them inside the existing
   `.boundline/workflows.toml` registry rather than introducing a second runtime
   registry file.
2. **Given** a repo-visible `project.boundline.toml` and a workspace-local
   `.boundline/cluster.toml`, **When** Boundline resolves project context,
   **Then** it treats the first as project semantics and the second as workspace
   topology.
3. **Given** a project index that references workspace IDs from the cluster,
   **When** Boundline inspects ownership and path mapping, **Then** it can join
   the two views without collapsing product semantics into cluster topology.

---

### User Story 3 - Expose Contract Compatibility And Mixed Evidence Authorship (Priority: P2)

As a reviewer or maintainer, I want session-native inspection surfaces to expose
Canon refs, compatibility status, and mixed Canon/Boundline evidence authorship,
so I can tell whether a session is blocked by producer policy, consumer policy,
or missing evidence.

**Why this priority**: Cross-repo integration becomes fragile if compatibility
and evidence authorship stay implicit.

**Independent Test**: Inspect `status`, `next`, or equivalent session-native
surfaces for supported, warning, and unsupported contract scenarios and verify
that shared evidence blocks remain attributable to their producer.

**Acceptance Scenarios**:

1. **Given** a compatible Canon contract line and relevant promotion refs,
   **When** a user inspects session state, **Then** Boundline surfaces the
   Canon refs and consumer compatibility state that influenced continuation.
2. **Given** an unknown major contract line, **When** Boundline reads the
   repo-visible Canon output, **Then** it rejects that input explicitly instead
   of silently accepting a new producer contract.
3. **Given** a shared `docs/evidence/` document with Canon and Boundline
   managed blocks, **When** a reviewer inspects the file, **Then** each block is
   attributable to its producer and source ref through the shared managed-block
   contract.

### Edge Cases

- A repo has stable project memory but no `project.boundline.toml`, so Boundline
  must decide whether to warn, replan, or stop based on remaining credible
  context.
- A project index is present, but it references a workspace ID that is not part
  of the current `.boundline/cluster.toml` topology.
- Canon exposes a compatible contract line but omits optional lineage fields
  that a reviewer would like to see.
- `docs/evidence/` contains producer-neutral managed blocks from Canon and
  Boundline, but one referenced source artifact is no longer available.
- Validation is already exhausted before Boundline reaches a stage that would
  otherwise use Canon evidence.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST consume Canon-owned repo-visible project memory
  and evidence as planning inputs rather than as authoritative runtime state.
- **FR-002**: Boundline MUST preserve `.boundline/` session, trace,
  checkpoint, and runtime surfaces as the authoritative state for execution.
- **FR-003**: Boundline MUST represent V1 delivery paths inside the existing
  `.boundline/workflows.toml` registry rather than introducing a separate
  `.boundline/delivery-paths.toml` file.
- **FR-004**: Boundline MUST treat `project.boundline.toml` as the repo-visible
  project semantics index and MUST keep `.boundline/cluster.toml` focused on
  workspace topology.
- **FR-005**: Boundline MUST read producer-neutral managed blocks and preserve
  the distinction between Canon-owned and Boundline-owned evidence
  contributions.
- **FR-006**: Boundline MUST enforce V1 hard stops for insufficient context,
  blocked Canon governance, missing required approval, exhausted validation,
  unsupported stage or mode, and missing required source artifacts.
- **FR-007**: Boundline MUST treat stale project memory, missing evidence
  sources, incomplete project indexes, and unknown assurance profiles as V1
  warnings or bounded replan conditions rather than automatic hard stops.
- **FR-008**: Boundline MUST defer provider runtime unavailability, project
  index conflicts, cross-workspace ownership mismatches, and evidence-freshness
  expiration to post-V1 hard-stop policy unless an existing runtime rule already
  stops earlier.
- **FR-009**: Boundline MUST expose Canon promotion refs, governed stage refs,
  evidence refs, and consumer compatibility state in session-native inspection
  surfaces when those facts affect continuation or blocking decisions.
- **FR-010**: Boundline MUST pin and validate the Canon control-layer contract
  major line, reject unknown major versions, and tolerate additive same-line
  fields without redefining Canon semantics.
- **FR-011**: Boundline MUST NOT redefine Canon promotion policy, lineage
  generation, or Canon-owned write rules for Canon-produced content.
- **FR-012**: Boundline MUST allow Boundline-owned runtime evidence to populate
  `docs/evidence/` through the shared managed-block contract without claiming
  ownership of the canonical cross-repo contract.

### V1 Delivery Control Decision Matrix *(mandatory)*

- **Proceed as credible context**: compatible Canon contract line, stable
  project-memory or evidence inputs, and a supported delivery stage or Canon
  mode for the active path.
- **Warn and continue**: stale project memory, missing evidence source,
  incomplete project index, or unknown assurance profile when the current stage
  still has enough other credible context to continue.
- **Warn and replan once**: stale or partial Canon knowledge when the current
  stage is no longer credible but another supported stage remains available from
  the active path.
- **Hard stop**: insufficient context, blocked Canon governance, missing
  required approval, exhausted validation, unknown major contract line,
  unsupported stage or mode, or missing required source artifact.
- **Project-index workspace mismatch**: warning by default, but hard stop when
  the selected system or active stage requires the missing workspace as a
  required source artifact.

Supported V1 delivery stages are `discovery`, `requirements`,
`domain-language`, `domain-model`, `system-shaping`, `architecture`, `backlog`,
`implementation`, `verification`, `pr-review`, `system-assessment`, `change`,
`migration`, `security-assessment`, `incident`, `supply-chain-analysis`, and
`refactor`. Any delivery-path or Canon mode reference outside the active
registry mapping is `unsupported` for V1.

### Scope Boundaries *(mandatory)*

- **In Scope**: consumer contract pinning, project-index semantics,
  workflow-registry extension, Canon-evidence ingestion, tiered stop behavior,
  and inspection-surface exposure.
- **Out of Scope**: a new workflow-registry file, Backstage integration,
  provider-readiness rework, unbounded multi-repo autonomy, voting, or changes
  to Canon publish and promotion rules.

### Key Entities *(include if feature involves data)*

- **Project Index**: The repo-visible `project.boundline.toml` document that
  declares project systems, domains, owners, paths, docs locations, and other
  semantics relevant to delivery control.
- **Delivery Path Entry**: A higher-level stage map represented inside the
  existing `.boundline/workflows.toml` registry for V1 execution planning.
- **Project Memory Context**: The Boundline-side view of Canon project memory,
  evidence, refs, and compatibility facts used during planning and inspection.
- **Evidence Contribution**: A producer-attributed managed block inside
  `docs/evidence/` that links readable summaries back to Canon runs or
  Boundline traces.
- **Consumer Compatibility State**: The Boundline-owned result of deciding
  whether Canon repo-visible output can be consumed, warned on, or rejected.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Boundline can use compatible Canon project memory and evidence as
  planning context while always ending V1 hard-stop scenarios in an explicit
  terminal state.
- **SC-002**: Maintainers can distinguish project semantics from workspace
  topology in under 5 minutes by inspecting `project.boundline.toml` and
  `.boundline/cluster.toml`.
- **SC-003**: V1 delivery-control planning does not require a new runtime
  registry file beyond the existing `.boundline/workflows.toml` surface.
- **SC-004**: Reviewers can identify whether a block, warning, or stop was
  caused by Canon producer facts, Boundline consumer policy, or missing source
  evidence from the inspection surfaces alone.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: OpenAI Models documentation at
  `https://developers.openai.com/api/docs/models`, Anthropic Models Overview at
  `https://platform.claude.com/docs/en/docs/about-claude/models/overview`, and
  Google Gemini Models documentation at
  `https://ai.google.dev/gemini-api/docs/models` on 2026-05-13.
- **Catalog Delta**: No bundled catalog changes are required for this feature
  slice based on the preliminary spec-time audit; T002 revalidates this result
  before implementation sign-off.
- **No-Change Rationale**: The current bundled catalog already matches the
  public text-and-coding model families used by Boundline runtime selection:
  OpenAI still documents `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and
  `gpt-5.4-nano`; Anthropic still documents `claude-opus-4-7`,
  `claude-sonnet-4-6`, and `claude-haiku-4-5`; Google still documents
  `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite`,
  `gemini-3.1-pro-preview`, and `gemini-3.1-flash-lite`. New audio, media, and
  specialized research models do not require catalog changes for this delivery
  control slice.

## Assumptions

- Canon publishes the stable owner-side contract under `docs/integration/`, and
  Boundline pins that owner-side contract instead of maintaining a second
  canonical contract text.
- V1 uses `docs/project/` and `docs/evidence/` as default repo-visible paths,
  while `project.boundline.toml` may override those defaults explicitly through
  its `[docs]` section without changing the consumer-side ownership boundary.
- Projects that need delivery-control consumption can add `project.boundline.toml`
  incrementally; absence of that file does not erase Boundline's existing
  bounded delivery path when other credible context remains available.
- The current bundled assistant catalog remains accurate enough that this spec
  can focus on delivery-control semantics rather than catalog maintenance.
