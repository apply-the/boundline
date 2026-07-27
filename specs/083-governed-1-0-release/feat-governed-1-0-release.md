# Boundline, Canon, and Speckit Adapter 1.0 Execution Roadmap

## Summary

Boundline 1.0 is the sole delivery control plane and the only component authorized to publish admitted changes into an authoritative workspace.

Canon 1.0 is a deterministic governance kernel with a standalone CLI. It publishes governance bundles-packets, decision-memory projections, evidence projections, and lineage-but never modifies the authoritative workspace.

`FrameworkAdapterV1` is stable in 1.0. The Speckit adapter becomes stable only after passing the complete adapter qualification suite.

Mutation executes in persistent, externally managed Git worktrees. Successful publication creates a verified Git commit, advances the admitted target branch using compare-and-swap, and leaves the authoritative product worktree clean and matching that commit.

Publication success is never rolled back because Canon is temporarily unavailable. Final outcome synchronization uses a durable, idempotent outbox and remains visibly pending until Canon records the terminal result.

The release train is:

```text
0.90.0 -> 0.95.0 -> 1.0.0-rc.1 -> 1.0.0
```

M0 establishes the baseline; no existing green baseline is assumed. After `1.0.0-rc.1`, no stable CLI, schema, reason code, protocol, profile, or persistence-semantic change is allowed except to satisfy an already frozen contract.

---

## 1. Stable product and interface contracts

### Product boundaries

```text
Boundline
  Delivery control plane.
  Owns sessions, mutation, verification policy, authority,
  publication, recovery, and authoritative workspace state.

Canon
  Deterministic governance kernel.
  Owns change intent, scope, risks, invariants, acceptance criteria,
  required evidence, decision memory, and governance projections.

FrameworkAdapterV1
  Bounded delegation contract.
  Adapter outputs are proposals.

Speckit adapter
  Independently versioned and qualified FrameworkAdapterV1 implementation.
```

Canon’s `change` profile governs intent, scope, risk, invariants, acceptance criteria, authority, and required evidence. Boundline’s implementation stage executes or delegates the admitted change. Canon’s former `implementation` mode remains preview or is removed from the stable surface.

Canon enforces only deterministic validation rules. Model-assisted or human semantic review enters Canon as external verification evidence containing reviewer identity, lineage, claims, findings, and evidence references. Canon verifies that the required review exists and satisfies policy; it never executes a semantic reviewer internally.

### Contract crates

Publish immutable SemVer packages from the existing repositories:

```toml
canon-contracts = "1.0"
boundline-protocol = "1.0"
```

- `canon-contracts` remains in the Canon repository.
- `boundline-protocol` remains in the Boundline repository.
- Signed source tags and matching release provenance accompany registry publication.
- No additional contract repository is introduced.

The public contracts contain typed envelopes, operation and stage identifiers, requests, results, revision fields, capabilities, route and lineage descriptors, mutation proposals, evidence references, traces, next actions, public read-only projections, and stable reason codes. Public status includes executor capability denial, stale proof, publication recovery, repository quarantine, and Canon outcome synchronization states such as `canon_outcome_sync_pending` and `canon_outcome_sync_failed`.

Internal persistence types remain private and independently versioned:

```text
RepositoryIdentityRecord
WorktreeLeaseRecord
TransactionOwnershipRecord
SessionExecutionLeaseRecord
ExecutorCapabilityGrantRecord
ExecutorInFlightRecord
RepositoryPublicationLockRecord
PublicationRecoveryRecord
CanonOutcomeOutboxRecord
```

### Mutation idempotency and revisions

Mutation requests are idempotent by `request_id`.

The key is scoped by contract line and operation, and its stored record includes the canonical request digest.

```text
Same request_id + same canonical digest:
  Return the previously recorded result.

Same request_id + different canonical digest:
  Fail with idempotency_conflict.

New request_id:
  Validate expected_state_revision before mutation.
```

The idempotency lookup occurs before revision validation so that a legitimate retry still succeeds after the original mutation advanced the state.

Every approval, verification result, proof record, and risk acceptance is bound to:

```text
session_id
transaction_revision
accepted_diff_digest
worktree fingerprint
claim set
executor or reviewer lineage
```

Any later mutation-including formatting or code generation-marks the previous approval and proof stale and requires revalidation.

### Boundline command-surface ownership

M1C was blocked because the roadmap named the whole 1.0 target as stable help
before later tasks owned real command handlers. A parser-only command would be
a stable stub, so it is rejected. The command surface therefore distinguishes
two states:

- **StableTargetPending**: planning and inventory state only. The command is
  reserved for the 1.0 target but is not registered in the public parser,
  stable help, completion metadata, or any operational compatibility claim.
- **StableOperational**: parser, real operational handler, fail-closed
  behavior, contract tests, help, completion metadata, and documentation are
  complete. Only this state may appear in stable help.

An owning implementation task may promote a command from
StableTargetPending only in the same commit that adds its real handler,
authority and failure semantics, contract tests, command tree, help,
completions, and documentation. A placeholder or generic not-yet-implemented
result is not operational.

The 0.90 StableOperational/help surface is an additive subset of the final
1.0 stable target inventory. The wider 0.90 command tree also contains
explicit Preview and hidden Internal commands, so it is not itself a subset of
that stable inventory. It removes overlapping legacy lifecycle entrypoints
immediately without compatibility aliases. Later 0.90/0.95 tasks may promote
reserved commands only with their real implementation. No StableOperational
command may be removed after the 0.95 contract freeze, while preview commands
remain outside the 1.0 compatibility promise.

#### Current M1C StableOperational surface

```text
boundline init
boundline goal
boundline plan
boundline run
boundline status
boundline inspect
boundline doctor
boundline config
boundline models
boundline provider
boundline adapter
boundline index
boundline session
boundline assistant
boundline update
```

#### Documented 1.0 StableTargetPending inventory

| Command | Owning implementation |
|---|---|
| `boundline approve` | T037 |
| `boundline session abort <id>` | T037 |
| `boundline session cleanup <id>` | T037 |
| `boundline recover inspect` | T047 and T048 |
| `boundline recover complete` | T047 and T048 |
| `boundline recover restore` | T047 and T048 |
| `boundline recover abandon --publication <id> --confirm <id>` | T047 and T048 |
| `boundline rpc` | T095 |
| `boundline serve --transport mcp-stdio` | T096 |

T076 qualifies host and projection behavior only. It must not create either
the RPC or MCP stdio runtime handler.

#### 0.90 migration and command classification map

| Command | 0.90 | 1.0 | Rationale or transition |
|---|---|---|---|
| `orchestrate` | Removed | Removed | Migration diagnostic: `run --until`; no compatibility alias. |
| `step` | Removed | Removed | Migration diagnostic: `run --one-step`; no compatibility alias. |
| `continue` | Removed | Removed | Migration diagnostic: `run --resume`; no compatibility alias. |
| `next` | Removed | Removed | Migration diagnostic: `status.next_actions`; no compatibility alias. |
| `probe` | Removed | Removed | Migration diagnostic: `doctor` or `status`; no compatibility alias. |
| `help-next` | Removed | Removed | Migration diagnostic: `doctor` or `status`; no compatibility alias. |
| `govern` | Removed | Removed | Migration diagnostic: use `plan` or `run` now; add `approve` only after T037 promotes it to StableOperational. No compatibility alias. |
| `flow` | Preview | Preview | Outside the 1.0 compatibility promise. |
| `workflow` | Preview | Preview | Outside the 1.0 compatibility promise. |
| `checkpoint` | Internal | Internal | Session/runtime machinery is surfaced through status, inspect, resume, and recovery rather than a stable root command. |
| `cluster` | Preview | Preview / post-1.0 | Cluster and multi-agent orchestration are outside stable 1.0. |
| `council` | Preview | Preview / post-1.0 | Council execution is an advanced reasoning capability and not a stable lifecycle entrypoint. |
| `override` | Preview transitional | Removed, replacement `approve` | Final replacement: `boundline approve`. Remove only when T037 delivers operational approval and authority handling; never present it as stable. |
| `evals` | Preview | Preview | Evaluation tooling is not part of the stable control-plane CLI. |
| `trace` | Preview transitional | Removed as a root command | Final replacement: `boundline inspect` and read-only trace projections. Remove only when that replacement surface is operational; never present it as stable. |
| `exec` | Internal | Internal | Direct execution bypasses the admitted lifecycle and cannot be a public stable entrypoint. |

Canon stable CLI:

```text
canon init
canon run --profile <profile>
canon resume
canon status
canon approve
canon inspect
canon publish
canon assistant install
```

The stable Canon one-shot machine interface is `canon rpc --stdio`: exactly one JSON request on stdin and one JSON response on stdout. It supports:

```text
capabilities
start
refresh
approve
inspect
publish
```

Canon stable profiles are exactly:

```text
discovery
requirements
architecture
backlog
change
refactor
verification
pr-review
incident
```

### Provider and adapter authority

Provider and adapter results are proposals. They cannot:

- Approve their own output.
- Publish authoritative changes.
- Modify Boundline authoritative state directly.
- Satisfy completion through self-reporting.
- Waive verification, proof, challenge, or authority requirements.

---

## 2. Transaction, execution, and publication model

### Local repository identity and state roots

`repository_id` is a clone-local opaque UUID stored in clone-local Git configuration and validated against a non-reversible fingerprint of the Git common-directory identity.

- Linked worktrees belonging to one clone share `repository_id`.
- Separate clones of the same remote receive different identifiers.
- Separate clones never share leases, publication locks, or recovery records.
- A distinct authoritative-worktree identity identifies the admitted checkout.

Managed state paths are:

```text
<state-root>/worktrees/<repository_id>/<session_id>
<state-root>/repositories/<repository_id>/
<state-root>/publication-locks/<repository_id>/
```

`.boundline/worktrees/<session_id>/` contains only lease and reconciliation metadata plus an opaque reference to the external worktree. It never contains a nested Git checkout.

`.boundline/` must be Git-ignored and excluded from mutation boundaries, context scans, product fingerprints, and publication commits.

The state root must:

- Be canonicalized and user-only.
- Reject symlinks, traversal, and ownership or permission mismatches.
- Keep secrets out of traces and redact environment and command output.
- Deny adapters direct access except to explicit invocation staging.
- Mark authoritative and managed-session worktrees distinctly.
- Reject opening a managed session worktree as an authoritative workspace.

### Fingerprint contract

Every fingerprint and digest carries `fingerprint_schema_version`.

The versioned fingerprint covers:

- Repository, base, HEAD, and target branch revision.
- Index state.
- Tracked files and explicitly admitted untracked files.
- Rename and deletion state.
- File type, executable bit, symlink target, normalized path, and content.
- Declared exclusions.

Ignored build outputs and caches are excluded only by explicit policy and cannot be published. Unexpected non-ignored untracked paths fail closed. `.boundline/` is always excluded.

UTF-8 paths are preserved; case-folding and Unicode-normalization collisions fail closed. Non-UTF-8 paths are unsupported in 1.0 and fail before mutation.

### Executor capability and sandbox contract

The same execution boundary applies to every provider, tool, framework adapter, and deterministic command runner. No executor class receives an implicit bypass.

Before invocation, Boundline persists an admitted capability grant bound to the session, transaction revision, invocation ID, executor identity, and fencing token. The grant declares:

```text
read path allowlist
write path allowlist
command and executable allowlist
network capability
named secret capability
environment allowlist
process-spawn policy
resource and output limits
deadline and cancellation policy
```

Stable defaults are fail closed:

- Secret inheritance is denied unless an individual named secret is admitted.
- Network access is denied unless a bounded network capability is admitted.
- Writes are confined to the managed session worktree and explicit invocation staging.
- Access to the Boundline state root, authoritative worktree, Git credentials, SSH agents, cloud credentials, and unrelated user directories is denied unless a specific stable capability permits it.
- Background processes, detached children, direct Git ref updates, commits, publication, and nested agent spawning are denied.
- Environment variables are constructed from an allowlist rather than inherited wholesale.
- Stdout, stderr, files, process count, memory, and execution time have declared limits.

The executor result records capabilities requested, admitted, denied, and observed. A result produced after capability escape, sandbox loss, fencing-token mismatch, or mutation outside the admitted boundary is rejected and cannot become session-owned evidence.

Platform enforcement strength is part of the support matrix. Where Boundline cannot enforce a declared stable capability boundary on a platform, the affected mutating workflow fails before execution rather than silently degrading.

### Persistent session worktree and executor serialization

The session worktree survives process restart and reboot. It is retained while the session is:

```text
active
paused
blocked
approval-pending
proof-pending
executor-in-flight
uncommitted-candidate
publication-pending
recovery-required
```

Cleanup is allowed only after:

- Successful publication and durable terminal trace/evidence finalization.
- Durable archival of an unsuccessful terminal session.
- Explicit abort, after executor termination and archival.

One session may have at most one executor in flight. Admission requires:

```text
exclusive session execution lock
monotonic fencing token
expected transaction revision
durable ExecutorInFlightRecord
```

Every subsequent journal or state write must present the current fencing token. A superseded executor cannot update session state.

Before each executor, Boundline durably records and flushes:

```text
initial worktree fingerprint
mutation boundary
invocation ID
executor identity and executable digest
transaction revision
base and HEAD
index state
tracked/untracked path ledger
fencing token
executor_in_flight state
```

The accepted diff digest starts as the clean-delta digest. It advances only after a completed executor boundary has been validated and durably journaled; Boundline never predicts the final diff before mutation.

Executor lifecycle control uses:

- Process groups on Unix.
- Job Objects on Windows.
- Cooperative cancellation followed by timeout escalation and forced termination.
- Confirmation that the old executor and descendants are dead before reconciliation.
- A worktree lock retained until termination is confirmed.

If a crash leaves a new delta, it becomes `uncommitted_candidate`. It is not automatically owned and cannot be inspected while the previous executor may still write. After termination is confirmed, it must re-enter proposal validation, authority, proof, and challenge evaluation.

### Challenge matrix

Sharing the implementer’s prompt, conclusion, or conversation does not constitute independent verification.

```text
Tier 0
  Deterministic checks.

Tier 1
  Tier 0.
  Separate verification invocation.
  Fresh, claim-matched evidence.

Tier 2
  Tier 1.
  Distinct model or executor lineage.
  Independently constructed verification context.
  Approval for material side effects.
  Same-lineage degradation requires a named risk owner,
  justification, and recorded override.

Tier 3
  Tier 2.
  Different provider family or qualified human challenger.
  Named human approval.
  Explicit risk acceptance.
  Verified rollback or recovery path.
  No automatic override when the challenge is missing.
```

### Durability contract

Intent records are committed and durably flushed before side effects. Completion records are committed and durably flushed after validation.

The persistence layer must provide crash-consistent transactions and declared `fsync` semantics:

- SQLite uses atomic transactions, WAL journaling, and `synchronous=FULL`.
- The database/WAL is flushed before executor or publication effects.
- Backup content, manifests, temporary files, and their parent directories are flushed before publication starts.
- Platform-specific file and directory flush primitives are used.
- Recovery after power loss is promised only on supported local filesystems that honor the declared durability primitives; unsupported filesystems fail closed before mutation.
- Forced-termination tests cover every boundary between durable intent, side effect, validation, and durable completion.

### Repository-scoped publication

Publication acquires an exclusive lock keyed by clone-local `repository_id`, combining an OS lock, a durable lock record, and a monotonic publication fencing token. Every publication journal write, file side effect, index update, ref update, completion record, and recovery action must present the current fencing token.

Before `publication-start`, Boundline must already have created, validated, and durably flushed:

```text
candidate commit and exact target diff
candidate tree identity and object-availability verification
expected branch ref and base revision
pre-publication authoritative fingerprint and index tree
target fingerprint and digest
affected-path ledger
backup manifest
backup content
backup digest
restore plan covering branch ref, index, worktree files, modes, symlinks, and deletions
required file metadata
publication fencing token
```

Publication then:

1. Acquires exclusive repository publication ownership.
2. Revalidates repository and authoritative-worktree identities.
3. Revalidates branch, HEAD/base, clean product state, index, and every path precondition.
4. Fails with `publication_rebase_required` if the admitted base is no longer the target branch head.
5. Persists and flushes `publication-start`.
6. Before every replacement, verifies current digest, file type, metadata, and absence of symlink traversal.
7. Writes the temporary file on the target’s filesystem, flushes it, atomically replaces the target, and verifies the resulting digest.
8. Updates the index to the candidate tree.
9. Advances the target branch from expected base to candidate commit using compare-and-swap.
10. Validates the final product fingerprint and clean worktree.
11. Durably records publication completion.
12. Releases the OS and durable locks only after the completion flush.

If compare-and-swap fails, or an external ref/index/worktree change is detected after `publication-start`, publication enters `recovery_required`. Recovery restores branch ref, index, and worktree only when the journal and current preconditions prove that doing so will not overwrite unexplained external state. Otherwise it preserves the repository unchanged for operator resolution.

Boundline 1.0 never automatically merges or rebases a stale candidate. A stale session must be re-admitted, rebased, and fully reverified.

Crash semantics are:

```text
Before durable publication-start:
  No authoritative mutation.
  No recovery required.
  A tentative lock may be cleared only after confirming branch,
  HEAD, index, and fingerprint remain unchanged.

After publication-start and before durable completion:
  recovery_required.

After durable completion and before OS-lock release:
  Reconcile as completed.
  Validate the published commit and final fingerprint.
  Release the abandoned lock.
  Never roll back the successful publication.
```

`recover complete` or `recover restore` first reacquires exclusive recovery ownership and validates that the publication journal completely explains the authoritative state. Unknown, mixed, user-owned, or diverged changes fail closed and are preserved without destructive action.

A dirty authoritative worktree may indicate interrupted publication. Recovery is permitted only when the publication journal fully explains it; otherwise all execution and publication fail closed.

### Git and filesystem qualification

Before RC, the implementation must support or deliberately reject:

| Case | Stable 1.0 behavior |
|---|---|
| Symlinks | Supported only when type and target can be preserved and verified; otherwise fail closed |
| Executable bit | Preserved from the Git tree and verified; fail if the platform cannot honor the admitted change |
| Submodules | Unchanged gitlinks allowed; mutation inside or replacement of a gitlink fails closed |
| Git LFS | Affected LFS paths fail closed unless explicitly qualified before RC |
| Sparse checkout | Publication fails closed |
| Detached HEAD | Mutation and publication fail closed |
| Case-insensitive filesystem | Supported with collision detection; ambiguous paths fail closed |
| Unicode paths | UTF-8 supported with normalization/case collision checks |
| Windows locked files | Publication stops in controlled recovery without destructive retries |
| Different repo/state volumes | Supported; backups are preflushed and replacement temporaries remain on the target filesystem |
| Large files | Binary-safe up to 512 MiB per file; larger admitted files fail before execution |
| Binary files | Supported byte-for-byte with content digests and no text transforms |

---

## 3. Adapter and Canon integration

### FrameworkAdapterV1

Stable transport:

```text
One-shot local subprocess
JSON request on stdin
Exactly one framed JSON response on stdout
Diagnostics only on stderr
```

Stable operations:

```text
describe
preflight          # non-mutating
execute_stage
emit_hook          # advisory
```

Persistent daemons, HTTP transport, nested delegation, background processes, and multi-adapter orchestration remain post-1.0.

`preflight` is non-mutating. `emit_hook` is advisory and non-authoritative; each hook declares whether failure blocks the current stage, with non-blocking as the default. Adapter execution also satisfies the shared executor capability and sandbox contract above.

Every adapter invocation enforces:

- Explicit environment allowlist.
- Secret inheritance denied by default.
- Declared and admitted network capability.
- Read/write path allowlists.
- Timeout, cancellation, and output limits.
- Strict stdout framing and separate stderr.
- Executable identity and digest.
- No background process.
- No direct commit or publication.
- No state-root access outside authorized invocation staging.

Speckit stable-stage candidates:

```text
requirements
planning
implementation proposal
```

Preview-only:

```text
clarification
checklist generation
task decomposition
```

### Canon authorization and outcome synchronization

Canon first publishes the governance bundle Boundline uses for admission and authority evaluation.

After successful or terminally failed publication, Boundline creates a durable `CanonOutcomeOutboxRecord` containing:

```text
outcome event ID and canonical payload digest
governance bundle ID and digest
session ID and final transaction revision
published commit, if any
final product fingerprint
proof references
publication outcome
recorded deviations
terminal claims
```

The outbox record is persisted before the session is considered fully synchronized. Delivery to Canon is idempotent by outcome event ID and canonical digest. Canon records the event transactionally and returns the resulting decision-memory revision.

A temporary Canon failure never rolls back a successful workspace publication. Boundline exposes `canon_outcome_sync_pending`, retries safely, and prevents cleanup of the final evidence needed for synchronization. Permanent rejection becomes `canon_outcome_sync_failed` with an inspectable reason and operator action. Explicit archival may close a failed synchronization only with named authority and preserved payload; it cannot rewrite the publication result.

Canon validates and records the outcome as decision-memory and evidence projections. Canon never applies the commit or workspace diff itself.

---

## 4. Milestones and release train

Roles are fixed accountabilities:

```text
Release Owner
Boundline Runtime Owner
Canon Kernel Owner
Adapter Owner
Security/Reliability Reviewer
Independent Release Reviewer
```

| Milestone | Owner / exit reviewer | Duration | Dependencies and repositories | Deliverable, parallel work, and blocking risks |
|---|---|---:|---|---|
| **M0 - Baseline and qualification policy** | Release Owner / Independent Release Reviewer | 1 week | All three repositories | Record exact commits, Git identities, the Rust toolchain selected and pinned by M0, lockfiles, supported OS/filesystems, and complete command outputs. Define `.boundline/` hygiene, fingerprint v1, fault points, benchmark matrix, host versions, coverage thresholds, and waiver policy. No green baseline assumed. Blocking risk: unreproducible dependencies or unsupported platform behavior. |
| **M1 - 0.90 contracts, CLI break, migrations** | Boundline Runtime Owner / Canon Kernel Owner | 2 weeks | Boundline + Canon; adapter can update fixtures in parallel | Publish prerelease contract crates, implement idempotency/revision envelopes and reason codes, remove duplicate CLI commands, restore Canon CLI/machine surface, and provide transactional bridge migrators. Blocking risk: active legacy sessions without equivalent transaction semantics. |
| **M2a - Honest Canon exchange** | Canon Kernel Owner / Security/Reliability Reviewer | 2 weeks | Canon; can overlap M3 foundations after M1 | Typed authoring exchange, profile registry, structural and cross-reference validation, external verification lineage, governance-bundle publication, and removal of synthetic Copilot execution, MCP stub, self-attested verification, and unimplemented stable commands. |
| **M2b - Decision memory** | Canon Kernel Owner / Independent Release Reviewer | 3 weeks | Canon; requires M2a, overlaps M3 and M4 | Typed decision-memory graph, lifecycle and stale propagation, deterministic semantic-policy checks, final-outcome synchronization, and 100% nine-profile deterministic corpus. Blocking risk: unbounded semantic rules; only deterministic rules may enter Canon. |
| **M3 - Persistent execution and publication** | Boundline Runtime Owner / Security/Reliability Reviewer | 4 weeks | Boundline; requires M1, overlaps M2b/M4 | External persistent worktrees, session executor fencing, child-process containment, uncommitted-candidate reconciliation, Tier 0–3 challenge matrix, versioned fingerprints, durable persistence, publication backup/start/completion protocol, CAS commit publication, recovery commands, stale-base handling, and state-root hardening. Blocking risks: cross-platform flush semantics, Windows file locking, Git metadata preservation. |
| **M4 - Adapter qualification** | Adapter Owner / Boundline Runtime Owner | 2 weeks | Boundline + Speckit adapter; requires M1 contract | Remove duplicated DTOs, implement the stable subprocess protocol and sandbox contract, qualify stable Speckit stages, keep preview stages visibly non-stable, and publish signed compatible packages. |
| **M5 - Vertical slice and RC candidate** | Release Owner / Independent Release Reviewer | 2 weeks | All repositories; requires M2b–M4 | Complete end-to-end workflow through governance, mutation, challenge, publication, Git ref/index/worktree recovery, durable Canon outcome synchronization and retry, across CLI, RPC, MCP, and five host packs. Resolve every compatibility and fault-injection case before `1.0.0-rc.1`. |
| **M6 - Freeze and 1.0 release** | Release Owner / Independent Release Reviewer | 2 weeks minimum soak | All repositories; requires RC | Run the frozen release suite on Linux, macOS, and Windows; permit contract corrections only; publish registry crates, signed tags, provenance, compatibility matrices, migration reports, and release artifacts. |

Expected calendar duration is approximately 12–14 weeks because M2b, M3, and M4 overlap after their prerequisites.

### Migration policy

Guaranteed:

```text
0.95.x -> 1.0
```

Supported bridge:

```text
Boundline 0.82.0 -> 0.90/0.95
Canon 0.72.6 -> 0.90/0.95
```

Older states use the historical upgrade path or remain inspectable archives.

Pre-0.90 active or partially mutated sessions are not automatically resumable unless an explicit migration fixture proves semantic equivalence. Unsupported active state is archived read-only and restarted as a new admitted session.

Every migration is:

- Preceded by a durable backup.
- Idempotent and inspectable before execution.
- Atomic and verified before replacement.
- Recoverable after forced termination.
- Accompanied by a report of conversions, archives, warnings, and semantic loss.

---

## 5. Verification and release gates

### Required repository checks

Run formatting, linting, tests, dependency policy, and packaging in every applicable repository on every supported platform. Run coverage on the designated reproducible coverage runner:

```text
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo nextest run
cargo deny check licenses advisories bans sources
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
patch coverage check
```

Before 0.95, freeze:

- Line coverage threshold as the greater of 80% or the M0 baseline.
- Patch coverage threshold at 90%.
- Waivers as named, justified, expiring records approved by the Release Owner and Security/Reliability Reviewer.

### Deterministic, safety, and utility gates

- Contract and deterministic golden corpus: 100% pass.
- Safety and rejection corpus: 100% correct fail-closed behavior.
- No stub, synthetic executor, placeholder, hidden fallback, or self-attested completion in stable surfaces.
- No adapter or provider result can bypass authority or independent verification.
- Every provider, tool, adapter, and deterministic executor is covered by the same admitted capability and sandbox contract; unsupported enforcement fails before mutation.
- A successful publication remains successful during Canon unavailability, while its durable outcome event remains retryable and cannot be silently dropped or duplicated.
- 100% of terminal success claims have fresh, claim-matched proof bound to the final transaction revision, session-worktree fingerprint, and published-commit fingerprint.
- Fault injection at every declared boundary must either complete the operation, restore the previous authoritative state, or preserve unexplained state without destructive action.
- Model-assisted utility: at least 95% for every benchmark case and every declared provider/model configuration, using fixed fixtures, a versioned rubric, and at least 20 runs per case/configuration. Aggregate averages cannot hide a failing case.
- CLI, JSON-RPC, MCP, and host-pack outputs must produce semantically equivalent authoritative projections after normalizing declared volatile timestamps, transport metadata, and invocation identifiers.
- Host compatibility matrices identify the tested Claude, Codex, Copilot, Cursor, and Antigravity versions. Stability covers generated schemas, calls into Boundline services, and projection equivalence-not untested future host versions.

### Mandatory vertical-slice scenarios

M5 must cover:

- Same `request_id` and same payload returns the stored result.
- Same `request_id` and different payload returns `idempotency_conflict`.
- A concurrent second executor for one session is rejected.
- A superseded fencing token cannot write journal or state.
- An executor survives the Boundline process; reconciliation waits until termination is confirmed.
- Provider, tool, and adapter attempts to escape path, environment, secret, network, process, or state-root capabilities fail closed.
- Executor crash produces a validated `uncommitted_candidate`, never automatic ownership.
- Tier 2 and Tier 3 challenge flows, including missing-challenger rejection.
- Shared implementer conversation is rejected as independent verification.
- Approval and proof become stale after mutation, formatting, or code generation.
- Observe and Adapt both record a valid `no_change` event.
- Restart and reboot preserve resumable session state.
- Two linked worktrees share local repository identity.
- Two clones of one remote have distinct identities and no lock collision.
- Concurrent publication in one clone admits one publisher only.
- Publication by one session makes the second session fail with `publication_rebase_required`.
- Crash before durable publication-start requires no recovery.
- Crash after publication-start but before completion produces `recovery_required`.
- Crash after durable completion but before lock release reconciles as completed.
- Recovery completion and restoration succeed only for fully journaled state.
- Unknown or mixed authoritative changes are preserved and rejected.
- Per-file external modification during publication triggers controlled recovery.
- Abort and cleanup respect executor termination and retention rules.
- Retained nonterminal worktrees cannot be deleted.
- Cleanup occurs only after durable trace and evidence finalization.
- Final commit, fingerprint, outcome, proof, deviations, and claims appear in Canon decision memory.
- Canon is unavailable after a successful publication; the workspace result remains committed, the durable outcome outbox retries idempotently, and eventual Canon recording produces exactly one decision-memory event.
- Canon permanently rejects a malformed outcome payload; Boundline exposes a terminal synchronization failure without altering the published commit or discarding evidence.
- Complete supported/fail-closed coverage of the Git and filesystem matrix.
- Semantically equivalent projections across Boundline CLI, JSON-RPC, MCP, Canon, and all five host packs.

## Assumptions

- Boundline, Canon, and the Speckit adapter remain separate repositories.
- M0 selects and pins the supported Rust toolchain; edition 2024 remains the language-edition baseline.
- Git is mandatory for stable mutation and publication.
- A new mutating session requires a clean authoritative product worktree.
- Normal resume applies to the persistent session worktree, never to arbitrary dirty authoritative state.
- Providers, tools, and adapters share one executor capability and sandbox model; no integration-specific bypass is stable.
- Canon outcome synchronization is a durable idempotent follow-up to publication, not part of the atomic workspace commit.
- Stable publication always creates a verified commit and leaves the authoritative product worktree clean.
- Applying a diff while leaving the authoritative repository dirty is preview-only and outside the 1.0 stable contract.
- Persistent adapter daemons, remote adapter transports, nested delegation, multi-adapter orchestration, and automatic candidate rebasing remain post-1.0.


## Roadmap readiness decision

This roadmap is approved for execution once M0 records the measured baseline and pins the supported toolchain and filesystem matrix. The architecture, stable product boundaries, transaction model, adapter contract, migration policy, and release gates are otherwise frozen by this document.
