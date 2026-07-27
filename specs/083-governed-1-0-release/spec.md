# Feature Specification: Governed 1.0 Release

**Feature Branch**: `083-governed-1-0-release`

**Created**: 2026-07-25

**Status**: Approved for planning

**Input**: Deliver the approved Boundline, Canon, and Speckit adapter 1.0 roadmap with deterministic governance, durable execution, qualified delegation, transactional migration, and evidence-backed release gates.

## User Scenarios & Testing

### User Story 1 - Publish a Governed Change Safely (Priority: P1)

As a delivery owner, I can admit, execute, verify, approve, and publish a change through one control plane so that the authoritative workspace advances only to an outcome supported by current proof.

**Why this priority**: Safe, evidence-backed publication is the core product promise and the boundary that distinguishes proposal generation from authority.

**Independent Test**: Start from a clean repository, complete one admitted change, and verify that the target branch advances exactly once to a verified commit while the authoritative workspace remains clean.

**Acceptance Scenarios**:

1. **Given** a clean authoritative workspace and an admitted change, **When** execution and the required challenge complete successfully, **Then** publication creates a verified commit, advances the target branch using its admitted base, and leaves the authoritative workspace clean.
2. **Given** an approval or proof for an earlier revision, **When** a later mutation changes the accepted diff, **Then** the earlier approval and proof become stale and publication is blocked until they are refreshed.
3. **Given** a provider or adapter proposal, **When** it claims completion or requests publication, **Then** the system treats the result as a proposal and independently enforces authority and proof requirements.

---

### User Story 2 - Resume or Recover Without Guessing (Priority: P1)

As an operator, I can resume interrupted session work and recover interrupted publication without losing user-owned or unexplained state.

**Why this priority**: Crash consistency and fail-closed recovery are required for trustworthy mutation.

**Independent Test**: Force termination at every declared execution and publication boundary and confirm that each outcome completes, restores the prior authoritative state, or preserves unexplained state without destructive action.

**Acceptance Scenarios**:

1. **Given** an interrupted executor whose process tree has terminated, **When** the session is resumed, **Then** unchanged state resumes from the last durable revision and changed state is treated as an uncommitted candidate requiring revalidation.
2. **Given** a crash before durable publication start, **When** the repository is reopened, **Then** authoritative state is unchanged and no publication recovery is required.
3. **Given** a crash after publication start but before durable completion, **When** recovery is inspected, **Then** the repository reports recovery required and permits completion or restoration only when the journal fully explains current state.
4. **Given** a crash after durable completion but before lock release, **When** reconciliation runs, **Then** it preserves the successful publication, validates the final fingerprint, and releases the abandoned lock.

---

### User Story 3 - Govern Intent and Record the Actual Outcome (Priority: P1)

As a governance owner, I can use Canon to govern change intent and required evidence before execution and record the outcome that was actually published afterward.

**Why this priority**: Decision memory is trustworthy only when it distinguishes deterministic governance from external review and includes the terminal delivery result.

**Independent Test**: Produce an authorization bundle, publish through Boundline, temporarily withhold Canon, and verify that the successful workspace publication remains committed while the outcome is delivered exactly once when Canon returns.

**Acceptance Scenarios**:

1. **Given** governance packets and external semantic-review evidence, **When** Canon evaluates them, **Then** it applies deterministic rules and records reviewer identity, lineage, claims, findings, and evidence references without running the reviewer itself.
2. **Given** a successful publication while Canon is unavailable, **When** synchronization is retried, **Then** the publication remains successful and Canon records exactly one terminal outcome.
3. **Given** a malformed terminal outcome rejected permanently by Canon, **When** the operator inspects the session, **Then** synchronization failure and corrective action remain visible without changing the published commit or discarding evidence.

---

### User Story 4 - Delegate Through a Bounded Adapter (Priority: P2)

As an integration owner, I can delegate supported stages to an independently versioned framework adapter without granting it authority over approval, publication, secrets, or unrelated paths.

**Why this priority**: Framework integration is valuable only when it preserves the control-plane boundary.

**Independent Test**: Execute each stable adapter operation with admitted capabilities, then attempt path, environment, network, process, state-root, commit, and publication escapes and verify fail-closed rejection.

**Acceptance Scenarios**:

1. **Given** a qualified adapter, **When** it describes, preflights, executes a stage, or emits an advisory hook, **Then** Boundline receives one well-framed proposal and independently evaluates it.
2. **Given** an adapter requests an undeclared secret, path, network target, child process, or authoritative action, **When** invocation admission runs, **Then** execution is denied before mutation.
3. **Given** the Speckit adapter, **When** qualification completes, **Then** requirements, planning, and implementation proposal may be stable while clarification, checklist generation, and task decomposition remain visibly preview.

---

### User Story 5 - Upgrade and Release with Verifiable Compatibility (Priority: P2)

As a maintainer, I can migrate supported recent releases and qualify the 1.0 release across declared platforms and host integrations without relying on an assumed baseline.

**Why this priority**: A stable release requires reproducible evidence, an explicit migration boundary, and frozen compatibility claims.

**Independent Test**: Establish the measured baseline, execute supported migration fixtures, run every deterministic and safety corpus, and verify the declared platform, host, coverage, and model-assisted gates.

**Acceptance Scenarios**:

1. **Given** a supported 0.95 state, **When** it is upgraded to 1.0, **Then** migration is backed up, idempotent, inspectable, atomic, crash-recoverable, and accompanied by a conversion report.
2. **Given** an unsupported active pre-0.90 session, **When** migration cannot prove semantic equivalence, **Then** it is archived read-only and restarted as a newly admitted session.
3. **Given** the release candidate, **When** the frozen release suite runs, **Then** all deterministic and fail-closed corpora pass completely and stable projections are semantically equivalent across supported interfaces and host versions.

### Edge Cases

- A retry reuses a mutation request identifier with a different canonical payload.
- Two processes attempt to execute the same session revision concurrently.
- A superseded executor remains alive after the Boundline process restarts.
- Two sessions based on the same branch finish execution before either publishes.
- An editor changes one affected file after publication starts.
- The authoritative workspace is dirty without a journal that fully explains it.
- A repository clone and a linked worktree are confused with separate local repository instances.
- A managed session worktree is opened as an authoritative workspace.
- A path crosses a symlink, collides under case folding or Unicode normalization, or is not valid UTF-8.
- The workspace uses sparse checkout, detached HEAD, Git LFS, submodules, locked files, large files, or separate state and repository volumes.
- Canon is unavailable or permanently rejects the final outcome.
- Cleanup is requested while execution, proof, publication, recovery, or outcome synchronization remains incomplete.

## Requirements

### Functional Requirements

- **FR-001**: Boundline MUST be the sole component authorized to publish admitted file changes into an authoritative workspace.
- **FR-002**: Canon MUST govern intent, scope, risks, invariants, acceptance criteria, authority, required evidence, and decision memory through deterministic rules only.
- **FR-003**: Canon MUST publish governance and evidence projections but MUST NOT apply authoritative workspace changes.
- **FR-004**: Mutation retries MUST be idempotent within contract line and operation using the request identifier and canonical request digest.
- **FR-005**: Reusing an identifier with a different canonical request digest MUST fail with `idempotency_conflict`.
- **FR-006**: Every approval, verification result, proof record, and risk acceptance MUST be bound to the exact session revision, accepted diff, fingerprint, claim set, and lineage it evaluates.
- **FR-007**: Any later mutation MUST make affected approval and proof records stale.
- **FR-008**: A session MUST admit at most one executor at a time through exclusive ownership and a monotonic fencing token.
- **FR-009**: Resumption MUST confirm that a previous executor and all descendants have terminated before analyzing or accepting its remaining changes.
- **FR-010**: A changed worktree left by an interrupted executor MUST become an uncommitted candidate and MUST NOT become session-owned without complete revalidation.
- **FR-011**: Session worktrees MUST persist outside the authoritative repository across process restart and reboot.
- **FR-012**: Repository-local Boundline metadata MUST remain untracked and excluded from product scans, mutation boundaries, and product fingerprints.
- **FR-013**: Separate clones of one remote MUST have distinct local repository identities; linked worktrees of one clone MUST share the same local identity.
- **FR-014**: Publication MUST be serialized per local repository and guarded by exclusive ownership and a monotonic fencing token.
- **FR-015**: Candidate commit, target state, complete backup, restore plan, and publication preconditions MUST be validated and durably recorded before authoritative mutation begins.
- **FR-016**: Every published path MUST be checked immediately before replacement and verified immediately afterward.
- **FR-017**: Successful publication MUST create a verified commit, conditionally advance the admitted target branch, and leave the authoritative product workspace clean.
- **FR-018**: A stale target branch MUST fail with `publication_rebase_required`; version 1.0 MUST NOT automatically merge or rebase the candidate.
- **FR-019**: Recovery MUST distinguish crashes before publication start, during publication, and after durable completion.
- **FR-020**: Recovery MUST preserve unknown, mixed, user-owned, or diverged state without destructive action.
- **FR-021**: Execution and publication intent MUST be durably recorded before effects and completion MUST be durably recorded after validation.
- **FR-022**: Fingerprints MUST be schema-versioned and cover repository revisions, index, admitted paths, content, file types, modes, symlinks, renames, deletions, and explicit exclusions.
- **FR-023**: The state root MUST reject unsafe ownership, permissions, symlinks, traversal, and worktree-role confusion, and MUST prevent secret disclosure in traces.
- **FR-024**: Provider, tool, and adapter execution MUST share one fail-closed capability model covering paths, commands, environment, secrets, network, processes, resources, output, and deadlines.
- **FR-025**: Provider and adapter results MUST remain proposals and MUST NOT self-approve, self-attest completion, publish, directly mutate authoritative state, or waive required verification.
- **FR-026**: Challenge requirements MUST be graduated from deterministic checks through independent invocation, independent lineage and context, different provider or qualified human challenge, approval, risk acceptance, and recovery proof.
- **FR-027**: Sharing an implementer’s prompt, conversation, or conclusion MUST NOT satisfy independent verification.
- **FR-028**: The stable framework adapter MUST support one-shot description, non-mutating preflight, stage execution, and advisory hook operations.
- **FR-029**: Canon MUST expose the nine approved profiles and stable human and one-shot machine interfaces.
- **FR-030**: Boundline MUST deliver terminal publication outcomes to Canon through durable, idempotent synchronization without rolling back a successful publication during Canon unavailability.
- **FR-031**: Unsupported active legacy state MUST remain inspectable and MUST NOT be silently resumed under the 1.0 transaction model.
- **FR-032**: Supported migrations MUST be backed up, idempotent, inspectable, atomic, validated, crash-recoverable, and reported.
- **FR-033**: Stable interfaces MUST expose typed status and reason codes for capability denial, stale proof, recovery, quarantine, stale publication base, idempotency conflict, and Canon synchronization.
- **FR-034**: Stable interfaces and supported host packs MUST produce semantically equivalent authoritative projections after normalization of declared volatile fields.
- **FR-035**: Stable release surfaces MUST contain no stub, synthetic executor, placeholder, hidden fallback, or self-attested completion.
- **FR-036**: Worktree cleanup MUST be prohibited until the session is terminal, its executor is terminated, and required trace, evidence, recovery, and outcome-synchronization records are durably finalized or explicitly archived with authority.
- **FR-037**: A command may appear in stable help, stable completion metadata, or operational compatibility claims only when it is StableOperational: its parser, real operational handler, fail-closed behavior, contract tests, help, completion metadata, and documentation are complete. Preview commands may remain registered outside those stable surfaces, while Internal commands remain hidden. A StableTargetPending command is a planning and inventory state only; it MUST remain absent from parser registration, stable help, completion metadata, public runtime classifications, and operational compatibility claims, and must be promoted atomically by its owning implementation task.

### Key Entities

- **Governed Session**: The admitted change, current transaction revision, worktree identity, accepted diff, claims, authority, lifecycle state, and terminal outcome.
- **Local Repository Identity**: The stable identity of one local repository instance shared by its linked worktrees but not by separate clones.
- **Execution Lease**: Exclusive executor ownership, fencing token, admitted capabilities, invocation identity, and lifecycle.
- **Worktree Fingerprint**: A schema-versioned representation of the admitted product state and its deliberate exclusions.
- **Evidence Binding**: A verification, approval, proof, or risk record tied to exact state and reviewer lineage.
- **Publication Transaction**: Candidate commit, expected base, backups, affected paths, restore plan, fencing token, progress, and terminal status.
- **Governance Bundle**: Canon’s deterministic projection of intent, requirements, authority, and required evidence.
- **Outcome Event**: The idempotent terminal publication result delivered back to Canon.
- **Adapter Proposal**: A bounded, non-authoritative result from an independently versioned framework adapter.

## Success Criteria

### Measurable Outcomes

- **SC-001**: 100% of deterministic contract and golden-corpus cases pass.
- **SC-002**: 100% of safety and rejection cases produce the declared fail-closed outcome.
- **SC-003**: 100% of terminal success claims are backed by fresh proof bound to the final session state and published outcome.
- **SC-004**: Every declared crash point results in completion, restoration of the previous authoritative state, or preservation of unexplained state without destructive action.
- **SC-005**: Concurrent execution tests admit no more than one executor per session and reject every stale fencing token.
- **SC-006**: Concurrent publication tests admit no more than one publisher per local repository and never cause lock collisions between separate clones.
- **SC-007**: Canon receives exactly one terminal decision-memory event for each publication outcome despite retry, restart, or temporary unavailability.
- **SC-008**: Model-assisted utility reaches at least 95% for every declared benchmark case and provider/model configuration over at least 20 fixed-fixture runs.
- **SC-009**: Stable projections are semantically equivalent across CLI, machine protocols, and every declared compatible host version after normalization of volatile fields.
- **SC-010**: All supported migration fixtures complete without unexplained semantic loss and all unsupported active states are preserved as inspectable archives.
- **SC-011**: All required formatting, linting, testing, dependency-policy, coverage, patch-coverage, platform, and packaging gates pass before 1.0.
- **SC-012**: No nonterminal or unsynchronized session worktree can be removed through the stable cleanup surface.
- **SC-013**: Stable help and completion metadata contain exactly the StableOperational surface for the current release milestone; the documented 1.0 target inventory separately records every StableTargetPending command without claiming operational availability.

## Assumptions

- Boundline, Canon, and the Speckit adapter remain separate repositories.
- Git is required for stable mutation and publication.
- A new mutating session starts from a clean authoritative product workspace.
- Normal resume applies only to the persistent managed session worktree.
- Stable publication always creates a commit; leaving authoritative changes uncommitted remains preview-only.
- Persistent adapter daemons, remote transports, nested delegation, multi-adapter orchestration, and automatic candidate rebasing are outside 1.0.
- M0 measures and pins the toolchain, platform matrix, filesystem matrix, host versions, coverage baseline, benchmark matrix, and waiver policy before later release gates are frozen.
