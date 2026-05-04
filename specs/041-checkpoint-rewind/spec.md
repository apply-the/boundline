# Feature Specification: Checkpoint Rewind

**Feature Branch**: `041-checkpoint-rewind`  
**Created**: 2026-05-04  
**Status**: Draft  
**Input**: User description: "Per 041 in roadmap, procedi con speckit specify, plan, tasks e implements per l'ultima feature. Non fare slicing, voglio feature complete. Al solito, un task per fare bump della versione e uno per aggiornare tutte le docs impattate e il changelog. Infine coverage dei file rust modificati o creati e soluzione di problemi su clippy e esecuzione di cargo fmt. Assicurati che la coverage dei file rust modificati sia sopra il 95%. aggiorna la roadmap togliendo quanto fatto. Infine stampa un commit message descrittivo. Ti chiedo anche di migliorare le docs considerando questo feedback ... README con due livelli: Quick path brutale e Advanced architecture ... Synod = runtime operativo che decide, esegue, valida, tiene stato; Canon = runtime governato che registra, struttura, approva, pubblica."

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

### User Story 1 - Capture A Reversible Workspace Snapshot (Priority: P1)

An operator who is about to execute a bounded mutating `run` or `step` can rely
on Boundline to create an implicit workspace checkpoint first, so a failed or
undesirable mutation has an explicit local rollback point even when the
repository is dirty or not under version control.

**Why this priority**: Without a pre-mutation checkpoint, deeper autonomous
mutation remains unsafe. This is the smallest safety capability that makes the
existing session-native delivery path more trustworthy on real repositories.

**Independent Test**: Start a normal session-native flow, run a mutating `run`
or `step`, and verify that a checkpoint manifest is persisted before the change
lands, with enough file-state detail to support a later restore.

**Acceptance Scenarios**:

1. **Given** an active workspace session with a confirmed goal plan whose next
  step mutates files, **When** the operator runs `boundline run`, **Then**
  Boundline creates one checkpoint before the mutation, links it to the active
  session and task, and records the captured file states explicitly.
2. **Given** a mutating clustered delivery session owned by a primary
  workspace, **When** the operator runs `boundline step --cluster
  <primary-workspace>`, **Then** Boundline creates one linked checkpoint group
  that preserves per-member captured state without hiding which workspace owns
  each snapshot.

---

### User Story 2 - Restore A Checkpoint Explicitly And Safely (Priority: P2)

An operator can inspect saved checkpoints and restore one intentionally through
an explicit command, with safe refusal when unrelated newer edits would be
overwritten unless the operator deliberately overrides that protection.

**Why this priority**: Checkpoint creation is incomplete unless the operator can
actually recover from a bad mutation. Safe refusal is part of the feature, not
an optional enhancement.

**Independent Test**: Create a checkpoint through a mutating run, change the
same workspace again, and verify that `checkpoint restore` either restores the
captured state or stops explicitly when unrelated newer edits would be lost.

**Acceptance Scenarios**:

1. **Given** a checkpoint captured before a failed bounded run, **When** the
  operator runs `boundline checkpoint restore <id>`, **Then** Boundline
  restores the recorded file states, records the restore event, and keeps
  trace history append-only.
2. **Given** a checkpoint whose captured files now contain unrelated newer
  edits, **When** the operator runs `boundline checkpoint restore <id>`
  without an override, **Then** Boundline refuses the restore explicitly and
  surfaces the conflicting paths plus the command needed to force it.

---

### User Story 3 - Keep Checkpoint Authority Visible Across CLI Surfaces (Priority: P3)

An operator can see the latest checkpoint identity, ownership scope, and
restore hint from normal Boundline outputs such as `run`, `status`, `next`, and
`inspect`, without confusing checkpoint authority with Canon governance or the
existing session authority story.

**Why this priority**: Safety only helps if the operator can discover and use
it from the normal follow-through surfaces rather than reading raw files.

**Independent Test**: Execute a mutating run that leaves the workspace failed or
blocked, then verify that `status`, `next`, and `inspect` all project the same
checkpoint headline and restore cue.

**Acceptance Scenarios**:

1. **Given** a mutating run that fails after creating a checkpoint, **When** the
  operator runs `boundline status` or `boundline next`, **Then** Boundline
  surfaces the latest checkpoint identity and the suggested restore command.
2. **Given** an authoritative compatibility or clustered follow-up surface,
  **When** the operator inspects the latest state, **Then** Boundline keeps the
  route authority explicit while still showing checkpoint provenance when a
  checkpoint exists for that authoritative path.

---

### User Story 4 - Ship 0.41.0 As A Rust Workspace Without Changing The Product Boundary (Priority: P4)

A maintainer can ship the checkpoint-rewind feature as `0.41.0` while refounding
the repository into a Rust workspace with clear core, adapters, and CLI crate
boundaries, preserving the repo-root command surface and clarifying the docs
into a brutal quick path plus a deeper architecture layer.

**Why this priority**: The roadmap explicitly ties checkpoint safety to the Rust
workspace migration. The slice is not complete unless both ship together as one
coherent release surface.

**Independent Test**: Build and test the workspace from the repository root,
verify the existing commands still run there, and confirm the updated docs keep
Synod as the operational control plane while Canon remains the governed
companion.

**Acceptance Scenarios**:

1. **Given** the `0.41.0` workspace layout, **When** a maintainer runs repo-root
  commands such as `cargo run --bin boundline -- status --workspace .` or
  `cargo test --workspace`, **Then** the product surface behaves the same while
  the crate boundaries are explicit.
2. **Given** the updated README and architecture docs, **When** a new operator
  reads the quick path first, **Then** they see the short session-native path
  without being forced into routing, cluster, delegation, or Canon details
  until they choose the advanced architecture layer.


### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when a checkpoint is requested for a workspace where the next
  bounded step does not materially change any files?
- How does the system handle files that were newly created, deleted, already
  dirty, or missing by the time restore is attempted?
- How does the system keep cluster restore behavior explicit when only one
  member workspace has conflicting newer edits?
- How does the system preserve append-only trace history while still making a
  restore action inspectable?
- How does the system surface checkpoint guidance on native, clustered, and
  explicit compatibility follow-up surfaces without implying that Canon owns the
  checkpoint lifecycle?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST create one implicit checkpoint before any bounded
  `run` or `step` action that can mutate files inside the active workspace or
  an active cluster member workspace.
- **FR-002**: System MUST persist each checkpoint under the workspace-local
  checkpoint state directory with an explicit manifest that records the
  triggering command, session or cluster authority, related task or step
  identity, timestamp, and captured file list.
- **FR-003**: System MUST record whether each captured file path was
  pre-existing, newly created, deleted, or already modified relative to the
  checkpoint baseline.
- **FR-004**: System MUST capture Boundline-owned state files when the same
  mutating action changes them, while preserving trace history through explicit
  restore events instead of destructive trace deletion.
- **FR-005**: System MUST surface `boundline checkpoint list` and
  `boundline checkpoint restore <id>` as first-class CLI commands using the same
  workspace and cluster scoping model as the session-native commands.
- **FR-006**: System MUST refuse checkpoint restore when it would overwrite
  unrelated newer edits unless the operator passes an explicit override.
- **FR-007**: System MUST surface the latest checkpoint identity and suggested
  restore command through `run`, `status`, `next`, and `inspect` whenever a
  mutating action fails or leaves the workspace in a blocked state.
- **FR-008**: System MUST preserve per-member checkpoint state explicitly for
  clustered execution and link those member snapshots under one
  primary-workspace checkpoint group.
- **FR-009**: System MUST refound the repository into a Rust workspace with
  explicit core, adapters, and CLI members while preserving repo-root command
  entry and existing command semantics.
- **FR-010**: System MUST keep Boundline independently usable without requiring
  Canon for checkpoint creation, listing, restore, or restore-related follow-up.
- **FR-011**: System MUST include unit, integration, and contract validation
  covering checkpoint manifests, restore conflict refusal, clustered restore
  behavior, and CLI-visible checkpoint guidance.
- **FR-012**: System MUST include an explicit task for the `0.41.0` version bump
  across the crate and release surfaces.
- **FR-013**: System MUST include an explicit task for updating impacted docs,
  assistant guidance, roadmap, and changelog, with the README split clearly into
  a quick path and an advanced architecture layer.
- **FR-014**: System MUST complete the release with touched Rust coverage above
  95%, successful formatting, and clippy free of slice-introduced issues.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: implicit workspace checkpoints for mutating `run` and `step`,
  checkpoint list and restore commands, safe restore refusal plus override,
  clustered checkpoint linkage, checkpoint projection across CLI follow-through
  surfaces, Rust workspace refoundation into core/adapters/cli crates, `0.41.0`
  version and release-surface updates, and documentation layering improvements.
- **Out of Scope**: Git-native rollback, remote checkpoint storage, arbitrary
  user-named snapshots outside the bounded execution path, automatic restore on
  failure, snapshotting `.git/` or build outputs, restoring files outside the
  declared workspace scope, new governance modes, UI work, or distributed
  orchestration.

### Key Entities *(include if feature involves data)*

- **Checkpoint Manifest**: records one bounded checkpoint, including its
  identity, authority scope, triggering command, related task or step, captured
  files, creation time, and any later restore records.
- **Checkpoint File Record**: records one captured path, its workspace owner,
  lifecycle state at capture time, and the snapshot payload needed for safe
  restore.
- **Checkpoint Restore Record**: records one restore attempt, including whether
  it succeeded, was refused, or was forced, together with the conflicting paths
  if refusal occurred.
- **Checkpoint Group**: links the primary-workspace manifest with any member
  workspace manifests for clustered execution so restore authority remains
  explicit.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative mutating runs and steps, 100% of executions that
  can modify workspace files create an inspectable checkpoint before the first
  mutation occurs.
- **SC-002**: In representative restore scenarios, 100% of restore attempts end
  in an explicit success, refusal, or forced-override outcome without silent
  data loss.
- **SC-003**: Operators can identify the latest checkpoint and the restore
  command from normal Boundline output in under 2 minutes after a failed or
  blocked mutating run.
- **SC-004**: Modified or newly created Rust files for this slice finish the
  release validation suite above 95% line coverage with clean formatting and
  lint results.
- **SC-005**: Maintainers can run the full workspace from the repository root,
  including build, tests, and the shipped CLI binary, without changing the
  documented product entry commands.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Existing session-native commands remain the primary product path, while any
  explicit compatibility route remains available but subordinate.
- The current single-crate repository can be split into a Rust workspace in this
  slice without changing the documented repo-root operator commands.
- Checkpoint restore defaults to safety-first refusal when unrelated newer edits
  are detected; the explicit operator override is the accepted default for this
  release.
- Boundline-owned workspace state under `.boundline/` is part of the bounded
  checkpoint story when the active mutating action changes that state.
