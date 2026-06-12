# Feature Specification: Safe Command Execution and Evidence Capture

**Feature Branch**: `077-safe-command-execution`

**Created**: 2026-06-11

**Status**: Clarified

**Input**: User description: "Safe local command execution with evidence capture, artifact manifest, secret redaction, and explicit mutation boundaries. First slice: command intent classification, dry-run/no-mutation modes, stdout/stderr/exit-code capture, artifact manifest, and evidence packet."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Classify and Dry-Run a Command (Priority: P1)

An operator runs a potentially destructive command. Boundline intercepts the command, classifies its intent (read, mutate, install, test, deploy), evaluates it against local execution policy, and — when the policy requires it — executes in dry-run mode showing what would change without applying mutations.

**Why this priority**: This is the minimum safety net. Without classification and dry-run enforcement, there is no audit trail and no pre-execution safety barrier. Every other feature (evidence capture, redaction, governance hooks) depends on the classification and policy framework being in place first.

**Independent Test**: Run `boundline exec --dry-run "rm -rf ./data"` against a workspace. Verify that the command is classified as `mutate`, the dry-run mode prevents file deletion, and the output reports what would have been deleted.

**Acceptance Scenarios**:

1. **Given** a Boundline workspace with test files, **When** the operator runs `boundline exec --dry-run "rm test.txt"`, **Then** the file is NOT deleted and the output shows "Would delete: test.txt" with an `intent: mutate` classification.
2. **Given** a read-only command like `ls`, **When** Boundline classifies it, **Then** the intent is `read` and no dry-run enforcement is triggered.
3. **Given** a command classified as `mutate` without `--dry-run`, **When** the local policy is set to `deny-mutation`, **Then** execution is blocked with an error explaining the policy violation.

---

### User Story 2 - Capture Structured Evidence (Priority: P1)

Every command executed through Boundline produces a structured evidence packet containing the command, timing, exit code, stdout, stderr, and a manifest of files produced or modified. Evidence is persisted to `.boundline/traces/` for audit and verification.

**Why this priority**: Evidence capture is the foundation for verification (B18) and orchestration (B19). Without structured evidence, Boundline cannot prove what happened, and downstream gates cannot validate claims.

**Independent Test**: Run `boundline exec "echo hello"` and verify that `.boundline/traces/` contains a JSON evidence file with `command`, `exit_code: 0`, `stdout: "hello\n"`, `stderr: ""`, `timing`, and an `artifact_manifest`.

**Acceptance Scenarios**:

1. **Given** a successful command execution, **When** the command completes, **Then** a `.boundline/traces/<trace-id>.json` file exists with `exit_code: 0`, full stdout/stderr capture, timing metadata, and an artifact manifest.
2. **Given** a failing command (exit code ≠ 0), **When** the command fails, **Then** the evidence packet records the non-zero exit code and captures stderr, and the trace is still persisted.
3. **Given** a command that produces output files, **When** evidence is captured, **Then** the artifact manifest lists each file produced or modified with path, size, and modification time.

---

### User Story 3 - Redact Secrets from Captured Output (Priority: P2)

When command output (stdout or stderr) contains patterns matching known secret formats (API keys, tokens, passwords), Boundline redacts those values before persisting the evidence packet.

**Why this priority**: Evidence capture is only trustworthy if it doesn't leak secrets into the audit trail. This must come before governance hooks, which may transmit evidence to external systems.

**Independent Test**: Run `boundline exec "echo SECRET_API_KEY=sk-abc123"` with a configured secret pattern matcher for `sk-*`. Verify the evidence packet contains `SECRET_API_KEY=<REDACTED>` instead of the actual key.

**Acceptance Scenarios**:

1. **Given** a configured secret pattern `sk-[a-zA-Z0-9]+`, **When** a command outputs `API_KEY=sk-abc123`, **Then** the evidence packet contains `API_KEY=<REDACTED>`.
2. **Given** a command with no secret-like output, **When** evidence is captured, **Then** no redaction is applied and the output is preserved verbatim.
3. **Given** multiple secrets in the same output line, **When** redaction runs, **Then** each secret is independently replaced with `<REDACTED>`.

---

### User Story 4 - Track Mutation Boundaries (Priority: P2)

When a command modifies the workspace, Boundline records an explicit mutation boundary: what files changed, how they changed (created, modified, deleted), and a snapshot of the workspace state before and after execution.

**Why this priority**: Orchestration (B19) and checkpoint/rewind need to know exactly what a command changed. Without mutation boundaries, rollback is guesswork.

**Independent Test**: Run `boundline exec "echo new-content > new-file.txt"` and verify `.boundline/traces/` records a mutation boundary with `created: ["new-file.txt"]` and a pre/post file listing diff.

**Acceptance Scenarios**:

1. **Given** a command that creates a file, **When** the command completes, **Then** the evidence packet includes a mutation boundary listing the created file with its pre-execution absence and post-execution presence.
2. **Given** a command that modifies an existing file, **When** the command completes, **Then** the mutation boundary records the file as `modified` with pre and post hashes.
3. **Given** a read-only command, **When** the command completes, **Then** the mutation boundary is empty (no files changed).

---

### User Story 5 - Governance Hooks for Risky Commands (Priority: P3)

Commands classified with high-risk intent (deploy, destructive mutate) or executed in red-zone workspaces trigger governance hooks that can require explicit approval, log to an audit channel, or escalate to Canon for a governed decision.

**Why this priority**: Governance hooks extend safety to enterprise risk scenarios but depend on classification (P1), evidence (P1), and redaction (P2) being in place first.

**Independent Test**: Configure a governance hook for `intent: deploy` commands that requires approval. Run `boundline exec "deploy.sh"`. Verify execution is blocked pending approval and the hook event is logged to `.boundline/traces/governance/`.

**Acceptance Scenarios**:

1. **Given** a governance hook for `intent: deploy`, **When** a deploy command is attempted, **Then** execution is paused and an approval request is logged with the command details and evidence preview.
2. **Given** a governance hook for red-zone workspaces, **When** any mutating command runs in a red zone, **Then** execution is blocked regardless of intent.
3. **Given** an approved governance hook, **When** execution proceeds, **Then** the approval decision and approver identity are recorded in the evidence packet.

---

### Edge Cases

- What happens when stdout or stderr output exceeds a configured size limit? → Truncation with a marker and metadata noting the original byte count.
- What happens when a command times out or is killed (SIGKILL)? → Evidence packet records the signal, partial output captured up to termination, and exit code from the signal.
- What happens when secret redaction pattern matching is too aggressive (redacts non-secret content)? → Redaction is configurable per pattern; a dry-run redaction preview mode shows what would be redacted before persisting.
- What happens when mutation detection fails (e.g., filesystem race condition)? → Mutation boundary is marked as `incomplete` with the error in the evidence packet, and the trace is still persisted.
- How is concurrent command execution handled? → Each command execution produces an independent trace; mutation boundaries are per-command, not global.

## Clarifications

### Session 2026-06-11

- Q: How should Boundline determine a command's intent? → A: Command-name whitelist + argument heuristics. Known commands mapped by name (rm→mutate, cargo test→test). Argument flags refine or escalate: --dry-run/--check downgrade toward read, --force/--delete/--push increase risk. Unknown commands default to mutate (not read). Path-based inspection is secondary context only. LLM classification excluded from v1 (must be deterministic, fast, testable, auditable).
- Q: How should Boundline implement dry-run at the OS level? → A: Deterministic dry-run tiers. Four modes: `native_dry_run_executed` (map known commands to their native safe mode), `read_only_executed` (run read-only commands directly), `plan_only` (emit plan for mutating commands without native dry-run, do NOT execute), `unsupported_for_safe_dry_run` (unknown commands, do NOT execute). V1 must never execute an unknown mutating command. Mappings must be curated and tested.
- Q: What shape should the execution policy take? → A: Intent × Zone matrix as baseline, with optional command-specific overrides. Resolution order: classify intent → apply command overrides → resolve matrix → apply safety escalation flags → produce final mode. Missing entries default to deny. Unknown intent defaults to require-approval.
- Q: What secret pattern detection mechanism should v1 use? → A: Curated regex patterns with built-in defaults + project-level `.boundline/redaction.toml`. Built-in patterns for common token formats (GitHub, AWS, JWT). Per-pattern severity, replacement, and allowlist rules. Redaction audit metadata records what was redacted without leaking original values.
- Q: What are the v1 hard limits for evidence capture? → A: stdout/stderr capped at 1MB each with truncation marker; trace ID = `{timestamp}-{command_hash}` (SHA-256 first 12 chars); concurrent commands independent; mutation boundary capped at 10,000 entries. All limits configurable in `.boundline/evidence-limits.toml`.
## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST classify every executed command into one of five intent categories (`read`, `mutate`, `install`, `test`, `deploy`) using a deterministic rule engine: first by known command name whitelist, then refined by argument heuristics. Unknown commands MUST default to `mutate`. Argument flags that indicate safety (`--dry-run`, `--check`, `--list`, `--help`, `--version`) MAY downgrade intent. Argument flags that indicate risk (`--force`, `--delete`, `--push`, `--install`, `--write`, `--fix`, `--apply`) MUST escalate intent. Path-based inspection is secondary context only. LLM-based classification is excluded from v1.
- **FR-002**: System MUST support three execution modes: `normal` (execute and apply mutations), `dry-run` (deterministic tiered simulation; see FR-002a for sub-statuses), `no-mutation` (execute but block filesystem writes). The four dry-run sub-statuses (`native_dry_run_executed`, `read_only_executed`, `plan_only`, `unsupported_for_safe_dry_run`) are diagnostic detail under the `dry-run` mode, not additional top-level execution modes.
- **FR-002a**: Dry-run MUST use a deterministic tiered mechanism with four result statuses: `native_dry_run_executed` (map known commands to native safe mode, e.g., `cargo check`, `terraform plan`, `npm install --dry-run`), `read_only_executed` (run read-only commands directly), `plan_only` (emit a dry-run plan with intent, risk level, expected mutation surfaces, and required approval — do NOT execute), `unsupported_for_safe_dry_run` (unknown mutating commands, do NOT execute). V1 MUST never execute an unknown mutating command just to discover mutations. Native dry-run mappings MUST be curated and tested.
- **FR-003**: System MUST capture stdout, stderr, exit code, wall-clock timing, and command string for every execution. Default caps: 1MB per stream with truncation marker (`[TRUNCATED: original N bytes]`). Caps MUST be configurable in `.boundline/evidence-limits.toml`.
- **FR-004**: System MUST persist evidence packets to `.boundline/traces/` as JSON with a stable trace identifier. Default trace ID format: `{ISO8601_timestamp}-{sha256_first_12_hex}`.
- **FR-005**: System MUST redact values matching configured regex secret patterns from stdout and stderr before persisting evidence. Built-in defaults for common token formats (GitHub, AWS, JWT). Per-pattern severity, replacement strategy, and allowlist rules configured in `.boundline/redaction.toml`. Redaction audit metadata MUST record what was redacted (pattern ID, count) without storing the original value.
- **FR-006**: System MUST compute a mutation boundary after each command: list of files created, modified, or deleted, with pre and post hashes for modified files. Default cap: 10,000 entries. When exceeded, mark as truncated and include total observed count when available. Cap MUST be configurable.
- **FR-007**: System MUST support a local execution policy file (`.boundline/execution-policy.toml`) using an Intent × Zone matrix as baseline: six intents (read, test, mutate, install, deploy, unknown) × three zones (green, yellow, red) mapping to five execution modes (allow, dry-run, no-mutation, require-approval, deny). Optional command-specific overrides processed before matrix resolution. Safety escalation flags (--force, --delete, --write, --push) applied last. Missing entries default to deny; unknown intent defaults to require-approval. Final decision MUST record command, intent, zone, matching policy entry, matching override, mode, and rationale.
- **FR-008**: System MUST produce an artifact manifest listing every file produced or modified by the command with path, size in bytes, and last-modified timestamp.
- **FR-009**: System MUST support governance hooks that can block, require-approval, or log commands matching intent and/or zone criteria.
- **FR-010**: System MUST redact secrets in a deterministic, repeatable way — the same input with the same patterns produces the same redacted output.
- **FR-011**: System MUST NOT require Docker or any container runtime to execute commands.
- **FR-012**: System MUST handle commands that produce no output (empty stdout/stderr) gracefully, recording `exit_code` and empty output fields.

### Key Entities

- **CommandIntent**: Classification of a command's purpose (`read`, `mutate`, `install`, `test`, `deploy`).
- **EvidencePacket**: Structured record of a single command execution: command string, intent, timing, exit code, stdout (redacted), stderr (redacted), artifact manifest, mutation boundary, trace ID, timestamp.
- **ArtifactManifest**: List of files produced or modified: path, size, modification time, operation (created/modified/deleted).
- **MutationBoundary**: Pre and post execution file state diff: created files, modified files (with hashes), deleted files, and a completeness flag.
- **ExecutionPolicy**: Rules mapping intent + zone → execution mode (allow, deny, dry-run, no-mutation).
- **SecretPattern**: A regex or glob pattern used to identify secrets in command output for redaction.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every command executed through Boundline produces a valid, complete evidence packet within 100ms of command termination.
- **SC-002**: Secret redaction catches 100% of values matching configured patterns (no false negatives on registered patterns).
- **SC-003**: Dry-run mode correctly predicts filesystem mutations without applying any changes (0 false-positive mutations).
- **SC-004**: Command classification accuracy ≥ 95% for the five intent categories on a golden test set of 50 common developer commands.
- **SC-005**: An operator can configure a new governance hook and have it enforced on the next matching command execution without restarting Boundline.

## Assumptions

- Commands are executed in the local shell environment (bash/zsh) available on the operator's machine.
- The `.boundline/` workspace directory is writable and not on a network filesystem that would cause race conditions in trace persistence.
- Secret patterns are configured via `.boundline/redaction.toml`; Boundline ships with built-in defaults for common token formats but they can be disabled per-project.
- Intent classification is rule-based (command name matching, argument heuristics) with a curated whitelist; updates to the whitelist are versioned with Boundline releases.
- Hard limits (1MB stdout/stderr, 10K mutation entries) are configurable defaults, not hardcoded constants.
- Governance hooks are local-only in v1; Canon integration for governed approval is deferred to a later feature.
