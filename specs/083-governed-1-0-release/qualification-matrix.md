# M0 Qualification Matrix

This matrix distinguishes an observed baseline from a supported 1.0 promise.
An untested row is not implicitly supported.

## Coverage policy

| Gate | M0 observation | Frozen threshold |
|---|---|---|
| Boundline line coverage | 92.77% (104,916 / 113,088) at the final executable M0 HEAD | 92.77% |
| Canon line coverage | 95.34% (42,270 / 44,334) | 95.34% |
| Speckit adapter line coverage | 88.85% (1,004 / 1,130) | 88.85% |
| Boundline M0 catalog patch coverage | 92.40% (316 / 342 executable changed lines) | 90% for every Rust patch |
| Modified `src/cli/init.rs` file coverage | 94.22% (5,594 / 5,937) | 95% before 1.0; Release Owner, T088 |

Coverage is computed from the committed baseline and the exact candidate diff.
Generated code, examples, binaries, and unreachable platform adapters require
an explicit policy; they are not silently removed from denominators.

## Command-surface qualification

| Surface state | Required evidence | Owner / disposition |
|---|---|---|
| StableOperational | Parser, real operational handler, fail-closed behavior, contract tests, stable help, completion metadata, and documentation are complete | Owning implementation task may promote atomically |
| StableTargetPending | Documented target and explicit task ownership only; absent from parser registration, stable help, completion metadata, and operational compatibility claims | Remains hidden until its owner promotes it atomically |
| Preview | Explicit preview placement and no 1.0 compatibility promise | M1C preserves preview status or a later task changes it explicitly |
| Internal | Not shown in public stable help or completion metadata | M1C hides the command behind supported lifecycle and inspection surfaces |
| Removed | Parser absence plus migration diagnostic when applicable | M1C removes the overlapping legacy lifecycle entrypoint without an alias |

The 0.90 help surface is an additive subset of the final 1.0 target inventory,
not proof that every future target command is operational. Parser-only stable
commands, generic not-yet-implemented handlers, and placeholder handlers fail
this qualification row.

## Platform and filesystem matrix

Unqualified M0 rows are carried risks, not M1 prerequisites, when owner,
milestone, disposition, and fail-closed behavior are explicit. T071 supplies
fixtures, T072 implements support-or-reject checks, and T093 is the final
M5/RC execution gate.

| Surface | M0 state | Owner / target | Disposition and fail-closed policy |
|---|---|---|---|
| macOS arm64 on local APFS | Baseline commands qualified; durability not yet qualified | Boundline Runtime Owner / T071-T072, M5 | Support only after fault-injection qualification; otherwise reject mutation |
| Linux x86_64 on local ext4 | Not run | Boundline Runtime Owner / T071-T072 and T093, M5/RC | Qualify support or reject mutation during admission |
| Windows x86_64 on NTFS | Not run | Boundline Runtime Owner / T071-T072 and T093, M5/RC | Qualify support or reject mutation during admission |
| Case-insensitive paths | Not qualified | Security and Reliability Reviewer / T071-T072, M5 | Detect path collisions before admission or reject |
| Unicode-normalizing paths | Not qualified | Security and Reliability Reviewer / T071-T072, M5 | Detect ambiguity before admission or reject |
| Repository and state root on different volumes | Not qualified | Boundline Runtime Owner / T071-T072, M5 | Use the qualified copy/flush protocol or reject |
| Read-only or non-local filesystem | Not qualified | Security and Reliability Reviewer / T071-T072, M5 | Reject unless the durability contract is proven |
| Ordered process restart | Existing tests provide partial evidence | Boundline Runtime Owner / T044-T047, M3 | No recovery mutation without a fully explanatory journal |
| Orderly reboot | Not exercised | Security and Reliability Reviewer / T044-T047 and T093, M3/RC | Promise only after durability qualification; otherwise scope recovery to process restart |
| Sudden power loss | Not exercised | Security and Reliability Reviewer / T044-T047 and T093, M3/RC | Not promised until filesystem and SQLite flush semantics are proven |

## Git and product-state matrix

| Case | Required stable behavior | M0 state and owned disposition |
|---|---|---|
| Symlink | Versioned fingerprint and no path traversal | Unqualified; Security and Reliability Reviewer; T071-T072/M5; reject until qualified |
| Executable bit | Preserved and fingerprinted | Unqualified; Boundline Runtime Owner; T071-T072/M5; reject mutation if preservation cannot be proven |
| Submodule / gitlink | Declared support or fail closed | Unqualified; Boundline Runtime Owner; T071-T072/M5; changed gitlinks reject |
| Git LFS pointer | Declared support or fail closed | Unqualified; Boundline Runtime Owner; T071-T072/M5; affected LFS paths reject |
| Sparse checkout | Declared support or fail closed | Unqualified; Boundline Runtime Owner; T071-T072/M5; reject |
| Detached HEAD | Explicit target required or fail closed | Unqualified; Boundline Runtime Owner; T071-T072/M5; reject without admitted target |
| Binary and large file | Bounded hashing/publication or fail closed | Unqualified; Boundline Runtime Owner; T071-T072/M5; reject outside qualified limits |
| Windows locked file | Controlled recovery without destructive action | Unqualified; Security and Reliability Reviewer; T071-T072/M5; preserve state and require recovery |
| User/IDE concurrent edit | Per-path precondition failure and recovery | Unqualified; Security and Reliability Reviewer; T035-T047/M3; stop publication and preserve state |
| Second session on stale base | `publication_rebase_required`; no automatic rebase | Unqualified; Boundline Runtime Owner; T035-T047/M3; reject publication |

Final platform and filesystem support claims are frozen only after T093/RC.
No M0 observation expands the supported surface.

## M1D bridge-migration qualification

| Product bridge | Local result | Qualified scope | Carried disposition |
|---|---|---|---|
| Boundline 0.82.0 to 0.90 state | 15 focused tests pass; every declared durable boundary exercised in-process and by forced child termination | macOS arm64, local APFS, exact fixture digest `ee0b51e2a34cccd2ba288bdaf3fd421e607019de7ae5fe88f6f5c2e9e4f0004c` | Linux, Windows, reboot, power loss, and other filesystems remain unqualified under T071, T072, and T093 and must fail closed |
| Canon 0.72.6 to 0.90 state | 16 focused tests pass; every declared durable boundary exercised in-process and by forced child termination | macOS arm64, local APFS, exact fixture digest `b514fd7bddf9d86c94a07d9d9d8dff25c1a12cc306c6d9450f7abff04c87ce42` | Linux, Windows, reboot, power loss, and other filesystems remain unqualified under T071, T072, and T093 and must fail closed |

The supported source versions are exact. Older, unknown, mixed, corrupt, and
future schema state is not converted. Pre-0.90 active or ambiguous state is
preserved in a read-only product archive with an immutable manifest and is
excluded from resume.

Focused M1D coverage uses repository-specific temporary target directories
because 30 GiB free space was insufficient to safely repeat two unbounded
workspace coverage runs. The accepted diff/LCOV intersection measured:

| Repository | M1D patch coverage | Gate |
|---|---:|---:|
| Boundline | 90.63% (871 / 961 executable changed lines) | 90% |
| Canon | 90.67% (875 / 965 executable changed lines) | 90% |

The frozen whole-repository M0 thresholds are unchanged. M1D did not claim a
new whole-repository measurement or a waiver.

## M1E package-readiness qualification

| Package surface | Classification | Local qualification | Publication disposition |
|---|---|---|---|
| `boundline-protocol 0.90.0` | PublicPublishable | Reproducible package candidate, generated-manifest audit, extracted-package tests, public-registry dry run, and exact registry-shaped clean consumer pass | T020 `READY_TO_PUBLISH`; not yet published |
| `canon-contracts 0.90.0` | PublicPublishable | Reproducible package candidate, generated-manifest audit, extracted-package tests, public-registry dry run, and exact registry-shaped clean consumer pass | T020 `READY_TO_PUBLISH`; not yet published |
| `boundline-adapter-speckit 0.1.0` | BinaryOnly in M1 | Reproducible package candidate and registry-shaped consumer resolve exact `boundline-protocol = "=0.90.0"`; host range rejection tests pass | Publication remains T068/M4; no M1 publication claim |

The remaining Boundline and Canon runtime crates are BinaryOnly,
InternalNotPublished, or PrivateWorkspace as recorded in
`evidence/m1e-package-readiness.md`. They are not implicit publication
predecessors. The two public contract crates are independent roots; the
adapter depends on `boundline-protocol`, but its publication is outside T020.

The adapter Rust patch measured 96% patch coverage (24 of 25 executable changed
lines). The single uncovered branch is the fail-closed guard for a statically
owned, compile-time-validated version-range constant. No waiver or threshold
change was created.

## Deterministic and probabilistic gates

| Corpus | Gate |
|---|---|
| Contract and deterministic golden corpus | 100% pass |
| Safety and rejection corpus | 100% correct fail-closed behavior |
| Publication fault points | Complete, restore previous authoritative state, or preserve unexplained state without destruction |
| Model-assisted utility | At least 95% for every case and every declared provider/model configuration over at least 20 fixed-fixture runs |
| Terminal completion proof | 100% fresh and bound to the final transaction revision, session fingerprint, and published commit fingerprint |

## Waiver schema

Every waiver is a reviewed record with all fields below:

```text
waiver_id
gate_id
scope
measured_value
required_value
risk_owner
approver
justification
compensating_controls
created_at
expires_at
maximum_release
evidence_references
```

A waiver must be named, narrowly scoped, time-bounded, and approved by someone
other than the implementer. It cannot waive authority separation,
non-destructive recovery, secret handling, a deterministic safety failure, or
self-attested completion. Expired waivers fail the release gate.
