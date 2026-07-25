# M0 Baseline: Governed 1.0 Release

**Recorded**: 2026-07-25

**Status**: M0 closed. The baseline established here is preserved throughout
the release train; no earlier green baseline was assumed.

## Historical repository identity

`repository_id` in the future runtime is clone-local. The hashes below are
one-way observations of each clone's Git common-directory identity; no local
absolute path is persisted in this specification. These original observations
are retained and are not overwritten by the M0 closure commits.

| Repository | Branch | Original baseline commit | Common-directory identity hash | Lockfile SHA-256 | Initial state |
|---|---|---|---|---|---|
| Boundline | `083-governed-1-0-release` | `d97b5c1d775e892f80ce2b7b3f0672a48e2d6b1c` | `0b52f8a3e6949fad6299ae177c6ac9079f96c86ebb01ca529cedbd9e6e547df2` | `2081f05ab8e40e639012e25c5f08a5b7bcbaf459261320a98763e7e4215661c3` | Clean before feature setup |
| Canon | `083-governed-1-0-release` | `d13f494ba17f8a56b146a27054d5a6eb35750e4a` | `3e54869b287bbe11bc862f9435e51b18f3c30a4c638c9ae2128cce5f1978cd06` | `bfec2fac3a20aaf0b09d2582146bc92ff83f43e702dfd2df1f7c33e9b8c9eea5` | Clean |
| Speckit adapter | `083-governed-1-0-release` | `fc1f22e6079344d032bea568c9824da4dc9f4549` | `9ac293242e8b516d97380e675e51f81a7d9bc6dda49ceca2be13ece82d567ec8` | `2a7e57c0a4170a05619826e61af7ce70e80d84d90723956b5dd4008620128f4b` | Clean |

## M0 closure commit identities

| Repository | Commit | M0 change |
|---|---|---|
| Boundline | `50481587168d29e7e25e64abcf241b43d894e321` | Namespace-separated model lifecycle catalog and typed validation |
| Boundline | `7d4c1221142be1fd58ba2714addc5feb3a06799b` | Exact MSRV and separate current-stable CI lanes |
| Boundline | `05864abce9ca202cf092dff799383fa7b0427e0d` | Negative catalog-policy and patch-coverage tests |
| Boundline | `926c35198c0338911f9260d9ac231e313101a7d7` | Obsolete cargo-deny exception removal; final executable M0 evidence HEAD |
| Canon | `103c53b5388b242d533faadf5ac2edbff1900bc1` | Exact MSRV and separate current-stable CI lanes; final M0 evidence HEAD |
| Speckit adapter | `8821fe4eb815a825d05e98e7da6bc374e9e6b810` | Rust 1.96 package/toolchain policy, cargo-deny policy, and stable lane; final M0 evidence HEAD |

The later specification-only closure commit does not change executable
evidence and is reported separately in the M0 handoff.

## Execution environment

| Property | Observed value |
|---|---|
| Host | `aarch64-apple-darwin` |
| Operating system | Darwin 25.5.0, arm64 |
| Filesystem | APFS, local, journaled; case behavior not yet qualified |
| Git | 2.50.1 (Apple Git-155) |
| MSRV lane | Rust/Cargo 1.96.0, exact patch |
| Current-stable lane | Rust 1.97.1 / Cargo 1.97.1 after refreshing a stale local 1.95.0 channel |
| Boundline LLVM | 22.1.2 |

Package `rust-version` remains 1.96 where compatible. Current stable is a
separate forward-compatibility lane; M0 does not silently raise the MSRV.

## Immutable command evidence

Every row below has a sibling compressed log. The linked metadata records the
exact command, repository HEAD, branch, toolchain, host target, relevant
environment, UTC start/end time, wall duration, exit status, log location, and
log SHA-256. All recorded closure commands exited zero.

### Boundline

| Command | Duration | Result | Evidence |
|---|---:|---|---|
| `cargo fmt --all -- --check` | 2 s | PASS | [metadata](evidence/commands/boundline/fmt.meta.md) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | <1 s | PASS | [metadata](evidence/commands/boundline/clippy.meta.md) |
| `cargo test --workspace --all-features` | 128 s | PASS | [metadata](evidence/commands/boundline/test.meta.md) |
| `cargo nextest run --workspace --all-features` | 97 s | PASS: 2,662 passed, 2 skipped | [metadata](evidence/commands/boundline/nextest.meta.md) |
| `cargo deny check licenses advisories bans sources` | 1 s | PASS WITH DECLARED WARNINGS | [metadata](evidence/commands/boundline/deny.meta.md) |
| `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` | 169 s | PASS: 104,916 / 113,088 lines, 92.77% | [metadata](evidence/commands/boundline/coverage.meta.md) |
| `cargo test --workspace --all-features catalog` | 3 s | PASS | [metadata](evidence/commands/boundline/catalog-tests.meta.md) |
| `rustup run stable cargo check --workspace --all-targets --all-features` | <1 s | PASS on 1.97.1 | [metadata](evidence/commands/boundline/current-stable.meta.md) |
| Patch coverage for `src/cli/init.rs` from the original baseline | 1 s | PASS: 316 / 342 executable changed lines, 92.40% | [metadata](evidence/commands/boundline/patch-coverage.meta.md) |

The two skipped nextest cases retain their declared adapter-binary fixture
preconditions. Nextest marked
`evaluate_canon_install_reports_missing_capabilities_output` leaky after a
pass. The Boundline Runtime Owner owns process-lifecycle investigation under
T039/M3; no success assertion depends on that fixture proving process
containment.

### Canon

| Command | Duration | Result | Evidence |
|---|---:|---|---|
| `cargo fmt --all -- --check` | 1 s | PASS | [metadata](evidence/commands/canon/fmt.meta.md) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | <1 s | PASS | [metadata](evidence/commands/canon/clippy.meta.md) |
| `cargo test --workspace --all-features` | 343 s | PASS | [metadata](evidence/commands/canon/test.meta.md) |
| `cargo nextest run --workspace --all-features` | 360 s | PASS: 1,691 passed, 0 skipped | [metadata](evidence/commands/canon/nextest.meta.md) |
| `cargo deny check licenses advisories bans sources` | 1 s | PASS, no warning | [metadata](evidence/commands/canon/deny.meta.md) |
| `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` | 194 s | PASS: 42,270 / 44,334 lines, 95.34% | [metadata](evidence/commands/canon/coverage.meta.md) |
| `rustup run stable cargo check --workspace --all-targets --all-features` | 26 s | PASS on 1.97.1 | [metadata](evidence/commands/canon/current-stable.meta.md) |

Canon fixture commands set only `commit.gpgsign=false` through Git's
per-process configuration so temporary fixture repositories do not inherit the
developer signing policy. The final commands ran outside the filesystem
sandbox because the suite intentionally creates non-UTF fixture paths.

### Speckit adapter

| Command | Duration | Result | Evidence |
|---|---:|---|---|
| `cargo fmt --all -- --check` | <1 s | PASS | [metadata](evidence/commands/adapter/fmt.meta.md) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 1 s | PASS | [metadata](evidence/commands/adapter/clippy.meta.md) |
| `cargo test --workspace --all-features` | 4 s | PASS | [metadata](evidence/commands/adapter/test.meta.md) |
| `cargo nextest run --workspace --all-features` | 11 s | PASS: 21 passed, 0 skipped | [metadata](evidence/commands/adapter/nextest.meta.md) |
| `cargo deny check licenses advisories bans sources` | 4 s | PASS WITH DECLARED WARNINGS | [metadata](evidence/commands/adapter/deny.meta.md) |
| `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` | 56 s | PASS: 1,004 / 1,130 lines, 88.85% | [metadata](evidence/commands/adapter/coverage.meta.md) |
| `rustup run stable cargo check --workspace --all-targets --all-features` | 26 s | PASS on 1.97.1 | [metadata](evidence/commands/adapter/current-stable.meta.md) |

Catalog-specific tests are not applicable to Canon or the adapter. Patch
coverage is not applicable there because M0 changed no executable Rust in
those repositories. Coverage was nevertheless rerun for the complete T002
baseline.

## Warnings, failed attempts, and dispositions

| Observation | Owner / task | Disposition |
|---|---|---|
| Boundline cargo-deny reports five permitted transitive duplicate families | Release Owner / T092 | Track dependency convergence; no advisory, license, ban, or source failure is waived |
| Adapter cargo-deny reports four permitted transitive duplicate families, substantially inherited through the owned 0.66 bridge | Adapter Owner / T019 and Release Owner / T092 | Replace the bridge only after the protocol prerelease exists, then re-evaluate duplicates |
| One passing Boundline nextest is marked leaky | Boundline Runtime Owner / T039 | Qualify process containment in M3; the M0 build result remains reproducible |
| Initial sandboxed Canon runs could not use GPG fixture signing or create a non-UTF path | Canon Kernel Owner / T087 evidence harness | Final evidence uses unsigned per-process fixture commits and the required filesystem access |
| Initial sandboxed cargo-deny runs could not acquire the read-only advisory lock | Release Owner / T092 evidence harness | Final evidence was rerun with advisory-database locking enabled |
| An out-of-sandbox Boundline retry waited on a provider endpoint and was interrupted with exit 130 | Boundline Runtime Owner / T087 evidence harness | Final immutable test evidence uses the network-restricted deterministic environment and passes |

No warning is reclassified as success. Each successful result is tied to its
final command record, while unsuccessful environmental attempts remain
explicit here.

## M0 exit review

The exit review found no foundational blocker:

- all three builds and required command sets are reproducible;
- every repository change is explained and the final status is verified
  separately after the specification commit;
- the intended durability boundary is implementable but not yet claimed as
  qualified on untested platforms;
- unimplemented security and filesystem surfaces are specified to fail closed;
- no M0 result depends on self-attestation or an unsupported host/platform
  claim.

The following are carried work, not M0 prerequisites:

| Carried risk | Owner / task / milestone | Required disposition |
|---|---|---|
| Adapter remains on the exact Boundline 0.66 bridge | Adapter Owner / T019 / M1 | Keep unchanged until `boundline-protocol` prerelease exists, then replace and verify |
| Linux, Windows, Git-feature, filesystem, and durability qualification | Boundline Runtime Owner and Security and Reliability Reviewer / T071-T072, T093 / M5-RC | Qualify support or reject mutation fail closed |
| Five-host semantic projection qualification | Release Owner / T073, T076 / M5 | Test frozen packs and normalized projections; inventory alone is not qualification |
| Power-loss recovery | Security and Reliability Reviewer / T044-T047, T093 / M3-RC | Do not promise it before declared SQLite/filesystem flush semantics pass fault injection |

## M0 exit decision

**Decision: GO M1.**

T002, T005, T006, and T008 meet their M0 acceptance conditions. This decision
authorizes only M1 planning and implementation under the normative task graph;
it does not pre-qualify any later host, platform, durability, adapter, or
semantic-projection gate.
