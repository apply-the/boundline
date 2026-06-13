# Feature Specification: Boundline Completion Verification Runtime

**Feature Branch**: `079-completion-verification-runtime`

**Created**: 2026-06-12

**Status**: Draft

**Input**: User description: "Add a runtime-owned completion-verification gate to Boundline so the system cannot mark a task, step, or run as complete until a claim-matched proof has been freshly executed in the current working state. Canon remains the governed owner of completion packet semantics, readiness, and approval metadata. Boundline owns proof selection, command execution, blocked-state projection, and evidence capture." See `./feat-completion-verification-runtime.md`.

## Clarifications

### Session 2026-06-12

- Q: What should invalidate a previously passing proof? → A: Any meaningful workspace content change invalidates the proof, using a normalized workspace content fingerprint that includes tracked files plus non-ignored untracked files, while excluding Boundline-owned runtime artifacts and other configured volatile paths.
- Q: Where should the completion claim come from in the first slice? → A: Prefer explicit task or stage metadata, otherwise infer from the current completion action and task context, and make the inferred claim explicit in the completion verification record before proof selection and closeout.
- Q: How should `stale` be represented in the first-slice projection? → A: Keep `stale` as a blocking finding, not a top-level state value; use `blocked` or `proof_required` as the state and surface `stale_proof` details in completion-verification findings.
- Q: When should Boundline require operator confirmation for an inferred claim? → A: Require operator confirmation only when inference is low-confidence or ambiguous, when the inferred claim is broader than the available proof, when risky surfaces are involved, or when metadata conflicts with runtime context.
- Q: How should completion verification behave when closing a stage or an entire run? → A: Stage and run closeout aggregate child verification state; child task proof remains authoritative, and any explicit stage-level or run-level claim adds to child verification rather than replacing it.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Block Unsafe Completion (Priority: P1)

An operator reaches the end of a task in a normal Boundline run and asks the runtime to mark it complete. Boundline checks whether the claimed outcome has a fresh, matching proof from the current working state and refuses to close the task when that proof is missing, stale, or failed.

**Why this priority**: This is the minimum safe slice. If Boundline can still close work without fresh proof, the feature fails its primary purpose even if evidence capture and output rendering exist.

**Independent Test**: Start a task with a claim such as `tests_pass`, attempt to complete it without running a matching proof in the current working state, and verify the task remains blocked with the missing claim and required proving command shown.

**Acceptance Scenarios**:

1. **Given** a task claims `tests_pass` and no current proof exists, **When** completion is requested, **Then** Boundline keeps the task open and projects `completion_verification_state` as `proof_required`.
2. **Given** a task claims `bug_fixed` and only stale evidence from an earlier working state exists, **When** completion is requested, **Then** Boundline keeps the task open and records a stale-proof finding.
3. **Given** a task claims an outcome for which no proving command can be selected, **When** completion is requested, **Then** Boundline keeps the task open and reports the claim as blocked instead of marking success.

---

### User Story 2 - Prove the Claimed Outcome (Priority: P1)

An operator asks Boundline to finish work on a task. Boundline derives the concrete claim being made, selects the narrowest available falsifying command for that claim, runs that command against the current working state, and captures evidence that can justify completion.

**Why this priority**: Completion gating depends on a trustworthy proof path, not only on blocked-state messaging. This story delivers the runtime behavior that transforms a claim into auditable evidence.

**Independent Test**: Complete a task that claims `build_clean`, let Boundline run the selected proving command, and verify the recorded result includes the command, exit code, summary lines, and fresh evidence references from the current working state.

**Acceptance Scenarios**:

1. **Given** a task claims `build_clean`, **When** completion is requested, **Then** Boundline runs the proving command selected for that claim before deciding whether the task can close.
2. **Given** the proving command exits with failure, **When** the proof run finishes, **Then** Boundline records the failure, leaves the task blocked, and does not report completion.
3. **Given** the proving command exits successfully in the current working state, **When** the proof run finishes, **Then** Boundline records fresh evidence references and allows the task to move to complete.
4. **Given** a task has no explicit completion claim, **When** Boundline infers one from the completion action and task context, **Then** the inferred claim is recorded visibly before proof selection and closeout continues.
5. **Given** claim inference is low-confidence, ambiguous, broader than the selected proof, or touches risky surfaces, **When** completion is requested, **Then** Boundline asks the operator to confirm or override the inferred claim before proving it.

---

### User Story 3 - Surface Verification State in Runtime Output (Priority: P2)

An operator, reviewer, or downstream governance consumer inspects Boundline status or assistant-rendered output near the end of work. They can see whether completion is ready, blocked on proof, or failed, along with the blocked claims, findings, proving command, and the most recent evidence references.

**Why this priority**: A runtime gate is only operationally useful if the blocked state is visible where users already look for progress and next actions.

**Independent Test**: Inspect session status and rendered run output for a task blocked on `migration_valid` proof and verify the response includes the additive completion-verification fields plus the exact next proving command without success language.

**Acceptance Scenarios**:

1. **Given** completion is blocked by missing proof, **When** status is rendered, **Then** the output includes `completion_verification_state`, `completion_verification_findings`, and `completion_blocked_claims`.
2. **Given** completion is blocked or failed, **When** assistant assets render the current state, **Then** the output does not use success language and instead shows the proving command as the next action.
3. **Given** fresh proof has passed, **When** status is rendered, **Then** the output includes `completion_evidence_refs` and reports readiness without changing existing status consumers.
4. **Given** a previously passing proof is stale, **When** status is rendered, **Then** `completion_verification_state` remains `blocked` or `proof_required` and `completion_verification_findings` includes a blocking `stale_proof` record with rerun guidance.

---

### User Story 4 - Preserve Canon Boundary While Emitting Runtime Evidence (Priority: P3)

A downstream governance system later consumes Boundline completion evidence. Boundline emits the runtime-owned `claim -> proof -> evidence_ref` projection without taking over readiness, approval, or packet-close semantics that belong to Canon.

**Why this priority**: The feature must improve runtime safety without reassigning ownership of governed semantics or coupling Boundline to Canon packet generation.

**Independent Test**: Complete a task with a passing proof and verify the runtime state exposes claim-linked evidence references while the generated output continues to describe Canon as the owner of packet readiness and approval language.

**Acceptance Scenarios**:

1. **Given** a fresh proof passes, **When** Boundline projects completion state, **Then** the claim, proving command result, and evidence references are available for downstream consumption.
2. **Given** Canon packet generation has not yet happened, **When** Boundline evaluates task completion safety, **Then** Boundline still blocks or allows task completion based on runtime proof state alone.

---

### User Story 5 - Aggregate Stage And Run Verification (Priority: P3)

An operator closes a stage or a full run after multiple tasks have executed. Boundline aggregates the verification readiness of required child tasks and stages, surfaces unresolved child proof issues, and only allows top-level closeout when all required children are verification-ready unless an additional explicit stage-level or run-level claim also needs proof.

**Why this priority**: The runtime must not hide child verification failures behind a stage-level or run-level success state. Aggregation is the simplest safe first-slice behavior because it reuses task proof ownership instead of inventing a second closeout model.

**Independent Test**: Attempt to close a stage with seven ready child tasks, one stale child proof, and one missing child proof. Verify the stage remains blocked and the projection reports child counts plus child-specific findings and required actions.

**Acceptance Scenarios**:

1. **Given** all required child tasks are verification-ready, **When** stage closeout is requested, **Then** the stage may move to complete without requiring a separate proof unless an explicit stage claim exists.
2. **Given** one or more required child tasks are blocked, stale, failed, or missing proof, **When** run or stage closeout is requested, **Then** closeout remains blocked and the unresolved child findings are surfaced.
3. **Given** an explicit stage-level or run-level claim exists, **When** all required children are verification-ready, **Then** Boundline may require an additional proof for that explicit parent claim without replacing child verification requirements.

### Edge Cases

- What happens when the claimed outcome maps to more than one possible proving command? Boundline chooses the narrowest falsifying command defined for that claim and records which rule selected it.
- What happens when a proof passed earlier but the working state changed afterward? The earlier proof is treated as stale and cannot satisfy completion.
- What happens when Boundline writes its own trace or proof evidence after a successful proof run? Those runtime-owned writes are excluded from the normalized workspace content fingerprint so the proof does not invalidate itself.
- What happens when a proving command cannot start or is interrupted? Boundline records the failure, leaves completion blocked, and surfaces the proving command as the next action.
- What happens when a task, stage, or run has no completion claim? Boundline must not invent success; it blocks closure until a supported claim can be derived or the work is explicitly reframed.
- What happens when Boundline cannot infer a specific enough claim from task context? Closeout is blocked and the operator is asked to confirm or override the claim before proof selection continues.
- What happens when task metadata and runtime inference disagree about the claim? Boundline blocks closeout and asks the operator to resolve the conflict before proof selection continues.
- What happens when optional or deferred child tasks are not verification-ready during stage or run closeout? They are represented explicitly as skipped or deferred with a reason and do not count as required blocked children.
- What happens when status consumers do not know about the new fields? Existing status behavior remains intact because the completion-verification fields are additive and optional.
- What happens when too many files changed to explain a stale proof concisely? Boundline reports a capped changed-path list with a truncation marker and still blocks completion until the proof is rerun.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST derive a concrete completion claim whenever a task, stage, or run is about to move to complete.
- **FR-002**: Boundline MUST support at least the following first-slice completion claims: `tests_pass`, `bug_fixed`, `build_clean`, and `migration_valid`.
- **FR-002a**: If task or stage metadata contains an explicit completion claim, Boundline MUST use that claim as the source of truth for completion verification.
- **FR-002b**: If no explicit completion claim exists, Boundline MUST infer a claim from available runtime context, including the current completion action, task title, task description, changed files, declared outputs, selected proof command, and recent execution trace.
- **FR-002c**: Boundline MUST make any inferred claim explicit in the completion verification record and surface it in status, inspect, or closeout output before proof selection and closeout proceed.
- **FR-002d**: Boundline MUST select the proof command against the claim; it MUST NOT derive the claim solely from the proof command.
- **FR-002e**: If Boundline cannot infer a sufficiently specific claim, it MUST block closeout and request clarification instead of guessing.
- **FR-002f**: When inference confidence is low, the operator MUST be able to confirm or override the inferred claim before completion continues.
- **FR-002g**: Boundline MUST proceed without operator confirmation when it infers one high-confidence claim and no conflicting or risky conditions apply.
- **FR-002h**: Boundline MUST require operator confirmation when inference confidence is low, when multiple plausible claims remain, when the inferred claim is broader than the available proof, when the selected proof command validates only part of the claim, when the claim affects risky surfaces such as migrations, release readiness, deployment, secrets, or data transformations, or when task metadata conflicts with runtime context.
- **FR-002i**: For a medium-confidence inferred claim, Boundline MUST proceed only when policy allows silent continuation; otherwise it MUST request operator confirmation.
- **FR-002j**: When confirmation is required, the prompt MUST show the inferred claim, confidence, evidence used for inference, selected proof command, alternative claims when present, and the consequence of proceeding.
- **FR-003**: Boundline MUST select one active proving command for the derived claim and MUST prefer the narrowest available falsifying command over a broader aggregate proof.
- **FR-004**: Boundline MUST execute the selected proving command against the current working state before allowing a task, stage, or run to move to complete.
- **FR-005**: Boundline MUST treat previously recorded proof as insufficient when the current normalized workspace content fingerprint differs from the fingerprint captured for the most recent passing proof of that claim.
- **FR-006**: Boundline MUST record the selected proving command, exit code, concise summary lines, proof outcome, and evidence references produced by the most recent proof run.
- **FR-007**: Boundline MUST block completion when no proving command exists for the derived claim.
- **FR-008**: Boundline MUST block completion when the proving command fails, cannot start, is interrupted, or produces evidence that does not match the derived claim.
- **FR-009**: Boundline MUST project the following additive completion-verification fields whenever relevant to closeout or blocked state: `completion_verification_state`, `completion_verification_findings`, `completion_blocked_claims`, and `completion_evidence_refs`.
- **FR-010**: Boundline MUST support `completion_verification_state` values of `ready`, `proof_required`, `blocked`, and `failed`.
- **FR-011**: Boundline MUST expose concise findings for missing, stale, failed, or mismatched proof without replacing or redefining the existing task, stage, or run status model, and stale findings MUST identify changed paths when possible.
- **FR-012**: Boundline MUST preserve blocked claims as user-visible next-action context so operators can see exactly what still requires proof.
- **FR-013**: Boundline status, orchestrate snapshots, and assistant-rendered run/status output MUST suppress success language whenever `completion_verification_state` is not `ready`.
- **FR-014**: Boundline assistant-rendered output MUST show the exact proving command to run next whenever completion is blocked on proof.
- **FR-015**: Boundline MUST emit the runtime-owned `claim -> proof -> evidence_ref` projection in a form Canon can consume later without requiring Canon packet generation before runtime completion gating occurs.
- **FR-016**: The first slice MUST support one blocked completion state at a time and sequential proof execution only.
- **FR-017**: The first slice MUST NOT require a new CLI command to trigger completion verification.
- **FR-017a**: Task-level claims and proof remain the authoritative unit of completion verification in the first slice.
- **FR-017b**: A stage is verification-ready only when all required child tasks are verification-ready.
- **FR-017c**: A run is verification-ready only when all required child stages or directly included required tasks are verification-ready.
- **FR-017d**: Stage and run closeout MUST aggregate unresolved child claims, stale proofs, failed proofs, and missing proofs into their completion-verification projection.
- **FR-017e**: Stage and run closeout MUST NOT report top-level success while any required child verification remains blocked, stale, failed, or missing.
- **FR-017f**: If an explicit stage-level or run-level claim exists, Boundline MAY require an additional proof for that parent claim, but parent proof MUST be additive and MUST NOT replace required child verification.
- **FR-017g**: Optional or deferred child tasks MUST be represented explicitly as skipped or deferred with a reason and MUST be excluded from required blocked-child counts.
- **FR-018**: Boundline MUST record the normalized workspace content fingerprint immediately before and immediately after each proof run, and a proof is fresh only when the current fingerprint still matches the fingerprint captured for the passing proof.
- **FR-019**: The normalized workspace content fingerprint MUST include tracked source, config, test, build, and claim-relevant documentation files, plus non-ignored untracked files inside the workspace.
- **FR-019a**: In the first slice, documentation is claim-relevant only when the active claim or selected proof rule explicitly treats documentation content as part of the verified outcome, such as release-readiness notes, migration instructions, operator runbooks, or generated user-facing docs that the proof command validates.
- **FR-019b**: When documentation is not explicitly tied to the active claim or proof rule, documentation-only changes MUST be excluded from the stale-proof invalidation set for that proof.
- **FR-020**: The normalized workspace content fingerprint MUST exclude `.git/`, `.boundline/traces/`, `.boundline/artifacts/`, `.boundline/cache/`, proof evidence files written by Boundline itself, `.gitignore`-ignored paths unless policy explicitly includes them, and configured volatile build/cache directories such as `target/`, `node_modules/`, `dist/`, `build/`, `.next/`, and `.venv/`.
- **FR-021**: If the current fingerprint differs from the fingerprint captured for a passing proof, Boundline MUST project the proof state as `stale` in findings, block completion, and require the proving command to be rerun.
- **FR-021a**: A stale proof MUST NOT introduce a new `completion_verification_state` value. Boundline MUST represent stale proof using `completion_verification_state = blocked` when closeout is actively blocked or `completion_verification_state = proof_required` when rerunning proof is the next action.
- **FR-021b**: A stale proof finding MUST include, at minimum, a finding kind of `stale_proof`, blocking severity, a user-visible message, the reference to the stale proof when available, changed paths when available, and a required action of `rerun_proof`.
- **FR-022**: When the stale-path set is too large to render completely, Boundline MUST report a capped changed-path list with a truncation marker.

### Key Entities *(include if feature involves data)*

- **CompletionClaim**: The concrete outcome Boundline is asserting before closure, such as `tests_pass` or `build_clean`.
- **CompletionClaimSource**: The origin of the active completion claim, limited in the first slice to `explicit_metadata`, `runtime_inference`, `operator_confirmed`, or `operator_override`.
- **ClaimInferenceConfidence**: The runtime confidence assigned to an inferred claim, expressed in the first slice as `high`, `medium`, or `low` for confirmation and policy decisions.
- **ProofCommandSelection**: The runtime decision that maps a completion claim to one proving command and records why that command was chosen.
- **ProofRun**: The fresh execution attempt for the selected proving command, including timing, exit result, summary lines, and whether the result satisfies the claim.
- **WorkspaceContentFingerprint**: The normalized representation of meaningful workspace content used to decide whether a passing proof is still fresh.
- **CompletionVerificationProjection**: The additive runtime state projected into status and rendered output: verification state, findings, blocked claims, and evidence references.
- **CompletionVerificationScope**: The scope to which a projection applies in the first slice, such as `task`, `stage`, or `run`.
- **CompletionVerificationFinding**: A structured explanation for why completion is not ready, including kinds such as `stale_proof` and actionable data such as changed paths and required rerun behavior.
- **ChildVerificationSummary**: The aggregated child readiness counters and child-specific findings used when a stage or run is being closed.
- **CompletionEvidenceRef**: A reference to the evidence produced by the most recent successful proof run for a claim in the current working state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of attempts to mark a task complete without a supported fresh proof are blocked and projected as `proof_required`, `blocked`, or `failed`.
- **SC-002**: 100% of passing completion proofs produce at least one fresh evidence reference linked to the derived claim before completion is reported as ready.
- **SC-003**: 100% of stale proof attempts are rejected after a working-state change and never allow completion without a new proof run.
- **SC-004**: Status and orchestrate output show completion-verification projection fields for every blocked or near-close task in the first-slice flow.
- **SC-005**: Assistant run/status assets emit zero success statements while `completion_verification_state` is `proof_required`, `blocked`, or `failed`.

## Assumptions

- The default sequential run path already has a point where Boundline decides whether a task, stage, or run should move to complete.
- Existing command execution and evidence capture surfaces can run the selected proving command and persist the resulting evidence references.
- Canon will consume completion evidence later, but Canon packet generation is outside the scope of this runtime slice.
- The first slice can derive one dominant completion claim per closure attempt and does not need parallel or speculative proof scheduling.
- Additive completion-verification fields can be introduced to session status and rendered output without breaking existing consumers that ignore unknown fields.
- Claim-relevant documentation can be determined deterministically from the active claim or selected proof rule without requiring a separate user workflow in the first slice.
- Existing task and stage models can store an explicit claim source and inferred-claim record without redesigning the broader task lifecycle.
- Policy evaluation can distinguish whether medium-confidence inferred claims may proceed silently or must be confirmed without introducing a new top-level completion state.
- Stage and run membership already identifies which child tasks are required versus optional or deferred, or can be extended additively to do so in planning.
