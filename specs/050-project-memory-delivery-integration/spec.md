# Feature Specification: Project Memory Delivery Integration

**Feature Branch**: `050-project-memory-delivery-integration`  
**Created**: 2026-05-13  
**Status**: Draft  
**Input**: User description: "Define the Boundline integration-side slice for consuming Canon-owned project-memory promotion output while keeping delivery paths, stage planning, assurance, and governed stage orchestration inside Boundline."

## Governance Context *(mandatory)*

**Mode**: change  
**Risk Classification**: bounded-impact because the slice adds an external
consumer contract dependency and new delivery-context inputs while preserving
Boundline's role as the bounded delivery orchestrator and leaving Canon-owned
promotion semantics untouched  
**Scope In**:

- reading Canon-promoted project-memory and evidence surfaces
- using Canon lineage refs and promotion states in delivery-path and stage-planner decisions
- integrating Canon-promoted knowledge into assurance and governed-stage orchestration
- surfacing Canon promotion state and refs in Boundline session-native status,
  next, and inspect behavior
- handling contract-version compatibility and incompatibility explicitly

**Scope Out**:

- defining Canon publish profiles
- defining Canon promotion states or their meanings
- defining Canon lineage schema or update strategies
- turning Canon into the orchestrator
- updating existing docs or implementation in this first pass

**Invariants**:

- Boundline remains the delivery orchestrator.
- Canon remains the governed producer of project-memory promotion semantics.
- Boundline MUST NOT redefine Canon promotion semantics.
- Boundline owns delivery paths, stage planner, assurance profiles, governed
  stage orchestration, and consumption of Canon refs.
- Missing or incompatible Canon project-memory output must result in explicit
  guidance, fallback, or bounded stop behavior rather than silent guesswork.

**External Contract Dependency**: Canon contract line
`0.1.x` (currently emitted as `0.1.0`) at
`/Users/rt/workspace/apply-the/canon/specs/048-project-memory-promotion-policy/contracts/boundline-project-memory-promotion-contract.md`
is the authoritative source for producer semantics.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Use Stable Project Memory Without Treating Pending Output As Truth (Priority: P1)

As a Boundline operator driving a large initiative, I want the stage planner to
use Canon-promoted project memory and evidence when they are stable, while
keeping pending or index-only Canon outputs visible but non-authoritative, so
the next stage is chosen from credible context instead of stale or premature
assumptions.

**Why this priority**: This is the core consumer-side behavior. Boundline gains
value only if it can consume stable project memory while preserving the
distinction between accepted knowledge and pending or blocked projections.

**Independent Test**: Present Boundline with representative stable project-memory,
pending-index, evidence-only, and no-project-memory states and verify that stage
selection, confirmation, or bounded stop behavior follows Boundline-owned logic
without redefining Canon's promotion semantics.

**Acceptance Scenarios**:

1. **Given** Canon-promoted stable project-memory surfaces with compatible
   lineage metadata, **When** Boundline plans the next stage, **Then** it may
   use that material as credible context for delivery-path and stage-planner
   decisions.
2. **Given** Canon output marked `pending-index` or `index-only`, **When**
   Boundline plans or continues, **Then** it exposes the pending signal but does
   not treat the underlying content as accepted project truth.
3. **Given** no Canon project-memory surfaces or no compatible contract
   metadata, **When** Boundline still has enough other context, **Then** it may
   continue through its normal bounded delivery loop without inventing Canon
   semantics.

---

### User Story 2 - Integrate Canon Refs Into Assurance And Governed Stage Flow (Priority: P1)

As a Boundline operator at a governed or evidence-heavy boundary, I want
Boundline to consume Canon refs and promoted evidence in assurance evaluation
and governed-stage orchestration, so review, verification, and risk boundaries
see the latest credible Canon output without surrendering orchestration to
Canon.

**Why this priority**: Project memory becomes operationally useful only when it
feeds the existing delivery loop, not when it sits as passive repository text.

**Independent Test**: Run representative delivery stages that require governed
packets, evidence, or assurance checks and verify that Boundline can consume
Canon refs and promotion states while keeping orchestration, retry, stop, and
next-action logic inside Boundline.

**Acceptance Scenarios**:

1. **Given** a stage whose assurance profile requires Canon-promoted evidence,
   **When** Boundline evaluates the stage, **Then** it can consume the Canon
   evidence refs and lineage metadata without redefining how Canon produced them.
2. **Given** a governed stage whose latest Canon output is stable and reusable,
   **When** Boundline continues, **Then** session-native status, next, and
   inspect surfaces include the relevant Canon refs and promotion state.
3. **Given** a stage whose latest Canon output is evidence-only or pending,
   **When** Boundline evaluates continuation, **Then** it applies Boundline-owned
   stop, confirm, or replan rules while preserving Canon's original promotion
   meaning.

---

### User Story 3 - Fail Explicitly On Contract Incompatibility (Priority: P2)

As a Boundline maintainer, I want explicit contract-version compatibility rules
for Canon project-memory output, so a changed producer contract results in a
clear repair path instead of silent consumer drift.

**Why this priority**: Cross-repo integration is brittle if producer and
consumer semantics can drift without a compatibility check.

**Independent Test**: Exercise supported `0.1.x`, future-line, and malformed
contract scenarios and verify Boundline either consumes the output credibly or
stops with explicit compatibility guidance.

**Acceptance Scenarios**:

1. **Given** Canon output whose `contract_version` is compatible with the
  Boundline integration slice, **When** Boundline reads the output, **Then** it
  proceeds using the documented consumer behavior for the supported `0.1.x`
  line.
2. **Given** Canon output whose `contract_version` falls outside the supported
  line,
   **When** Boundline reads the output, **Then** it stops or degrades explicitly
   with repair guidance instead of silently reinterpreting producer semantics.
3. **Given** Canon output on the supported `0.1.x` line that includes extra
  non-required metadata fields, **When** Boundline reads the output, **Then** it
  continues without redefining Canon-owned semantics.

### Edge Cases

- Stable project memory exists for one surface, but the most recent related
  Canon output is only pending or evidence-only.
- Canon-promoted evidence is available, but stable project-memory targets are not.
- A surface is marked `auto-if-approved`, but emitted metadata does not show
  `approval_state = Completed` and `readiness = complete`.
- A contract-compatible output omits a non-required additive field.
- The Canon contract line changes to `0.2.x` before Boundline updates its
  integration slice.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST treat the Canon project-memory promotion contract
  as the authoritative source for producer-side promotion semantics.
- **FR-002**: Boundline MUST own delivery paths, stage planner behavior,
  assurance profiles, governed stage orchestration, and consumption of Canon refs.
- **FR-003**: Boundline MUST NOT redefine Canon promotion states, update
  strategies, lineage-field semantics, or publish-profile meanings.
- **FR-004**: Boundline MUST be able to consume Canon-promoted stable
  project-memory surfaces as credible context for delivery-path and stage-planner
  decisions.
- **FR-005**: Boundline MUST distinguish stable project-memory output from
  `pending-index`, `index-only`, `evidence-only`, and `manual` producer outcomes,
  and MUST treat `auto-if-approved` as stable only when emitted approval
  metadata satisfies Canon policy.
- **FR-006**: Boundline MUST integrate Canon refs, promotion state, and lineage
  metadata into session-native `status`, `next`, and `inspect` surfaces where
  those refs affect continuation or blocking decisions.
- **FR-007**: Boundline MUST use Canon-promoted evidence and refs in assurance
  and governed-stage orchestration without transferring orchestration ownership
  to Canon.
- **FR-008**: Boundline MUST validate `contract_version` compatibility before
  relying on Canon project-memory output.
- **FR-009**: Boundline MUST surface explicit repair or incompatibility guidance
  when the Canon contract line is unsupported.
- **FR-010**: Boundline MUST continue to support bounded delivery when Canon
  project-memory output is absent, as long as other credible context is sufficient.
- **FR-011**: Boundline MUST NOT mutate Canon-managed project-memory content as
  part of its orchestration logic in this slice.
- **FR-012**: Boundline MUST preserve the product boundary that Canon governs
  and publishes knowledge while Boundline pilots delivery.

### Key Entities *(include if feature involves data)*

- **Project-Memory Context Snapshot**: The consumer-side view of Canon-promoted
  project memory, evidence, and lineage that Boundline can evaluate during
  delivery planning.
- **Promotion State Projection**: The Boundline-visible representation of the
  latest Canon producer outcome that influences continue, stop, confirm, or
  replan behavior.
- **Canon Ref Consumption Record**: The session-native projection of Canon refs,
  lineage metadata, and compatibility status used in status, next, and inspect.
- **Compatibility Outcome**: The Boundline-owned result of checking a Canon
  `contract_version` against the supported consumer contract window.

## Shared Contract Alignment *(mandatory)*

Boundline must stay aligned with the Canon-owned contract on:

- stage taxonomy and mode mapping
- stable project-memory, evidence, and index surface categories
- promotion-state vocabulary and meanings
- lineage metadata field names and meanings
- update-strategy vocabulary and meanings
- compatibility rules and pre-1.0 change policy

Boundline must not redefine any of those Canon-owned meanings inside this spec.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Boundline can use Canon-promoted stable project memory as
  planning context while keeping pending or evidence-only outputs explicitly
  non-authoritative.
- **SC-002**: Session-native status, next, and inspect surfaces can expose the
  latest relevant Canon promotion state and refs without replacing Boundline's
  orchestration role.
- **SC-003**: Unsupported major `contract_version` values lead to explicit
  contract-line changes lead to explicit repair guidance instead of silent
  semantic drift.
- **SC-004**: The integration-side spec stays strictly consumer-side and does
  not redefine Canon-owned publish or promotion behavior.

## Validation Plan *(mandatory)*

- **Structural validation**: review the integration-side spec against the Canon
  contract brief and confirm Canon-owned semantics are referenced, not copied or
  redefined.
- **Logical validation**: verify representative stable, pending, evidence-only,
  and incompatible-contract scenarios once implementation begins.
- **Independent validation**: confirm a Canon maintainer could read this spec
  and still find Boundline's ownership limited to orchestration and consumption.
- **Evidence artifacts**: this `spec.md`, the referenced Canon contract brief,
  and later Boundline planning and validation artifacts created under the same
  feature folder.

## Decision Log *(mandatory)*

- **D-001**: Boundline consumes Canon promotion output but does not own its
  meaning, **Rationale**: producer-owned semantics must remain centralized in
  Canon to avoid cross-repo drift.
- **D-002**: Contract-version compatibility is part of the integration slice,
  **Rationale**: a cross-repo integration surface without compatibility rules is
  not a credible bounded delivery dependency.

## Non-Goals

- Defining or changing Canon publish profiles.
- Defining or changing Canon promotion states or lineage-field meanings.
- Turning Canon into the delivery orchestrator.
- Updating existing docs or code in this first artifact-writing pass.

## Assumptions

- Canon remains the initial and authoritative producer for promoted
  project-memory semantics.
- Boundline may need to extend its current stage taxonomy over time, but any
  consumer-side mapping must still honor the Canon-owned contract.
- Stable project-memory consumption should improve planning quality, but lack of
  Canon-promoted memory does not eliminate Boundline's existing bounded-delivery
  path when other context remains credible.