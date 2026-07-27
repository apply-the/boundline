# M1D transactional bridge-migration evidence

Evidence date: 2026-07-27

Owning tasks: T017 and T018

Decision: **GO M1D**

## Repository and environment boundary

The clean starting commits on `083-governed-1-0-release` were:

- Boundline: `2583101afd1b7a037c68b692b13f7c60b980c5c9`
- Canon: `6f883253273218cce38190b547db60c77a8b8869`
- Speckit adapter: `8821fe4eb815a825d05e98e7da6bc374e9e6b810`

The run used Rust and Cargo 1.96.0, cargo-nextest 0.9.121, target
`aarch64-apple-darwin`, and local APFS. Free space before coverage was 30 GiB.
The adapter remained clean and unchanged. No package version, dependency,
lockfile, public protocol crate, stable CLI command, tag, or published package
changed; T019 and later tasks did not begin.

## Historical fixture provenance

The Boundline fixture is derived by hand from the exact 0.82.0 tag
`d97b5c1d775e892f80ce2b7b3f0672a48e2d6b1c`. Its normalized tree digest is:

```text
ee0b51e2a34cccd2ba288bdaf3fd421e607019de7ae5fe88f6f5c2e9e4f0004c
```

The Canon fixture is derived by hand from the exact 0.72.6 tag
`d13f494ba17f8a56b146a27054d5a6eb35750e4a`. Its normalized tree digest is:

```text
b514fd7bddf9d86c94a07d9d9d8dff25c1a12cc306c6d9450f7abff04c87ce42
```

Each repository stores a `PROVENANCE.toml` beside its compact fixture. The
digest includes normalized relative path, path length, file length, Unix mode,
and bytes in deterministic path order. The fixtures were not produced by the
new migrators and contain no credentials, user content, or absolute paths.

## Test-first record

The first authoritative focused Boundline execution was RED:

```text
cargo test --test bridge_090 --all-features
```

It exited 101 because `boundline_core::migration` did not exist.

The first authoritative focused Canon execution was RED after correcting the
test harness constructor:

```text
cargo test --test bridge_090 --all-features
```

It exited 101 because the bridge API and migration record types did not exist.
No implementation preceded either authoritative RED result.

## Lifecycle and persistence semantics

Both repository-local implementations use:

```text
Inspect -> Plan -> Backup -> Stage -> Verify -> Commit -> Complete
                                                     \-> RecoveryRequired
```

Inspection and planning are non-mutating. Apply revalidates the plan and exact
source digest under an exclusive OS-backed file lock. Intent and completion
records use typed JSON, atomic temporary-file replacement, file synchronization,
and parent-directory synchronization. The verified backup and its read-back
manifest exist before staging. The staged tree is validated before a durable
pre-replacement phase is recorded. Commit retains the source, renames the
verified staging tree into place on the same filesystem, reopens and validates
the target, durably records completion, and never deletes the backup.

A matching retry returns or reconstructs the terminal outcome. The migration
identity covers source product, identity, schema, digest, target 0.90 line, and
implementation version. A reused migration identity with a different digest
fails with `idempotency_conflict`. Competing migrators use the same durable
record and OS lock; only one may commit. A stale record is recoverable only
after OS ownership is released and the journal fully explains the state.

## Supported, archived, and rejected state

Boundline converts exact 0.82.0 inactive and terminal session records and
preserves goals, plan/checkpoint/traces, evidence references, provider routes,
workspace references, and legacy metadata when present. Active or partially
mutated sessions lacking 0.90 ownership, revision, fingerprint, authority, or
idempotency data are archived read-only and require a new admitted session.

Canon converts exact 0.72.6 terminal governance runs whose legacy meaning is
representable. Authority, approval, claims, evidence, policy, and verification
references remain historical. Label-only verification is explicitly reported
as not fresh. Active or ambiguous runs and legacy Implementation mode are
archived read-only and require a new admitted run; Implementation is not
promoted into the nine-profile registry.

Unknown older releases, future schemas, corrupt data, mixed generations,
unsafe symlinks/traversal, changed sources, tampered backup/staging, and
identity conflicts fail closed without falsely successful replacement.

## Forced-termination and safety matrix

Both products exercised actual child-process termination plus in-process
stopping at every durable boundary:

```text
before backup
after backup creation
after backup verification
during target staging
after staged target verification
before replacement
after replacement but before completion record
after completion record
```

Before backup, restart performs a safe retry from the unchanged source and no
recovery artifact remains. At every later boundary, restart acquires the
abandoned OS lock only after the child is dead, verifies journal/source/backup/
staging identity, and reaches a verified complete or already-migrated outcome.
No case produced silent partial success, competing targets, destructive
cleanup, or cross-product mutation.

Backup-manifest corruption and backup-content tampering return
`backup_verification_failed`. Staging tampering returns
`staged_verification_failed`. Permission and write failures preserve the
authoritative source. Archive tests inspect persisted bytes, immutable
manifests, digests, read-only metadata, and active-registry absence rather than
accepting returned status labels alone.

## Conversion report

The typed portable report contains migration and product identity; exact source
release/schema/digest; target line/schema; inspection/start/completion time;
logical backup and staged-output identity/digest; converted, archived, and
skipped counts; warnings; semantic losses; unsupported-state records;
verification results; terminal status/reason; and recovery instructions.
Absolute host paths and secrets are excluded. Equivalent reports compare
identically after normalizing timestamps and the explicitly volatile
archive-derived staged digest.

## Verification

Final focused coverage executions:

```text
CARGO_TARGET_DIR=<temporary> cargo llvm-cov --workspace --all-features \
  --test bridge_090 --lcov --output-path <temporary-report>
```

- Boundline: exit 0; 15 passed; 27.97 seconds.
- Canon: exit 0; 16 passed; 13.34 seconds.
- Boundline patch coverage: 90.63% (871/961).
- Canon patch coverage: 90.67% (875/965).

The complete final command results are:

| Repository | Command | Exit / result | Duration |
|---|---|---|---:|
| Boundline | `cargo fmt --all -- --check` | 0 | 1.31 seconds |
| Boundline | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 | 17.76 seconds |
| Boundline | `cargo test --test bridge_090 --all-features` | 0; 15 passed | 13.22 seconds |
| Boundline | `cargo test --workspace --all-features` | 0; final post-review rerun | 151.04 seconds |
| Boundline | `cargo nextest run --workspace --all-features` | 0; 2,719 passed, 2 skipped | 1,037.03 seconds wall; nextest test time 110.683 seconds |
| Boundline | `cargo deny check licenses advisories bans sources` | 0; advisories, bans, licenses, sources OK; declared duplicate warnings only | 0.89 seconds |
| Canon | `cargo fmt --all -- --check` | 0 | 0.55 seconds |
| Canon | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 | 13.76 seconds |
| Canon | `cargo test --test bridge_090 --all-features` | 0; 16 passed | 8.96 seconds |
| Canon | `cargo test --workspace --all-features` | 0; final post-review rerun | 2,965.70 seconds |
| Canon | `cargo nextest run --workspace --all-features` | 0; 1,715 passed, 0 skipped | 1,593.90 seconds wall; nextest test time 250.882 seconds |
| Canon | `cargo deny check licenses advisories bans sources` | 0; advisories, bans, licenses, sources OK | 0.57 seconds |

Canon's full test command required `commit.gpgsign=false` in the managed
environment because existing fixture tests create local Git commits and the
sandbox cannot reach the configured signing service. Canon nextest additionally
ran outside the managed filesystem sandbox because an existing Unix fixture
must create a non-UTF directory; the sandbox returned `EPERM`. The two exact
signing-dependent tests passed with the process-local override before the full
rerun. An initial concurrent Boundline run also observed one unrelated Canon
subprocess reading a partially written fixture; every final suite was therefore
run in repository isolation. These were environmental reconciliations, not
converted test failures.

Unbounded workspace coverage was not repeated: 30 GiB free space was below the
safe estimate for two full instrumented workspaces. Focused migration coverage
plus the repository's accepted diff/LCOV intersection was used. The frozen M0
whole-repository thresholds remain unchanged and no waiver was created.

## Independent reviews

Specification compliance found and resolved missing backup-manifest read-back
verification, incomplete terminal-outcome reconciliation, and report
normalization for archive timestamps/digests. Exact supported releases remain
frozen and T019 did not begin.

Safety and quality found and resolved file-presence-only locking,
same-migration/different-digest replay, monolithic storage responsibilities,
incomplete reconciliation after durable completion, and archive directories
that remained writable after their files were protected. Ownership now uses an
OS-backed lock; manifests are typed and verified; storage/durability helpers
are isolated; persisted source, backup, staged, and terminal state are checked;
archive files, nested directories, and roots are all made read-only post-order.

No Critical or Important finding remains. Linux, Windows, reboot, power-loss,
and non-APFS qualification remains assigned to T071, T072, and T093; M1D makes
no broader platform claim.
