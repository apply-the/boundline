# M1E-A Versioning and Package-Readiness Evidence

**Evidence date**: 2026-07-27  
**Owning tasks**: T019 complete; T020 incomplete and `READY_TO_PUBLISH`  
**Decision**: GO M1E-A  
**Branch**: `083-governed-1-0-release` in all three repositories

This record preserves the reversible M1E-A work. It is not public-registry,
tag-signature, or remote-push evidence.

## Scope and preconditions

All three repositories started at the prompt-specified commits with no staged,
unstaged, or untracked state:

| Repository | Starting commit | T019 source commit |
|---|---|---|
| Boundline | `b425d67f93397e0aa5acfc38d7ef959ad5bdc3ab` | `860b7247d8343c63dca2940a923b58f3172632f9` |
| Canon | `ab0e54163d5b1a1173f5a3c8253c6db27a9bdcd6` | `bd361d7e2ad112e5e0d599267e8024e748293605` |
| Speckit adapter | `8821fe4eb815a825d05e98e7da6bc374e9e6b810` | `ce7cb2e57c7ad8bd38064c684a4736029397756c` |

No repository had a `0.90.0` tag. The official crates.io API reported
`boundline-protocol`, `canon-contracts`, the Boundline runtime packages, and
the Speckit adapter absent. The unrelated crates.io name `canon-cli 0.0.0`
exists, reinforcing that Canon's CLI is not in this publication scope.

The environment was:

| Item | Observed value |
|---|---|
| rustc | `1.96.0 (ac68faa20 2026-05-25)` |
| Cargo | `1.96.0 (30a34c682 2026-05-25)` |
| nextest | `0.9.121` |
| cargo-deny | `0.19.0` |
| host/target | `aarch64-apple-darwin` |
| filesystem | local APFS |
| free space before qualification | 26 GiB |
| free space after qualification | 15 GiB |
| intended registry | crates.io |
| signing | GPG secret signing key `25C2C7D21E91FD50` available |

No registry authentication, publication, tag creation, commit push, or tag
push occurred.

## Package classification and graph

| Repository | Package | Version | Classification |
|---|---|---:|---|
| Boundline | `boundline` | 0.90.0 | BinaryOnly |
| Boundline | `boundline-cli` | 0.90.0 | BinaryOnly |
| Boundline | `boundline-core` | 0.90.0 | InternalNotPublished |
| Boundline | `boundline-adapters` | 0.90.0 | InternalNotPublished |
| Boundline | `boundline-protocol` | 0.90.0 | PublicPublishable |
| Canon | `canon-workspace` | 0.90.0 | PrivateWorkspace (`publish = false`) |
| Canon | `canon-cli` | 0.90.0 | BinaryOnly |
| Canon | `canon-engine` | 0.90.0 | InternalNotPublished |
| Canon | `canon-adapters` | 0.90.0 | InternalNotPublished |
| Canon | `canon-contracts` | 0.90.0 | PublicPublishable |
| Speckit adapter | `boundline-adapter-speckit` | 0.1.0 | BinaryOnly in M1; later publication owned by T068/M4 |

No package is classified FixtureOnly. No `publish = false` setting changed.
The public publication graph is acyclic:

```text
boundline-protocol 0.90.0    canon-contracts 0.90.0
          |
          +--> boundline-adapter-speckit 0.1.0
               publication deferred to T068/M4
```

The contract roots have no dependency on each other and can be built in
parallel. T020 uses the deterministic operator order `boundline-protocol`,
then `canon-contracts`; neither is a semantic predecessor of the other.

## Test-first RED evidence

Before manifest changes, the three package-readiness scripts failed as
expected on 2026-07-27 at 20:11:48–20:11:49 Europe/Rome:

| Repository | Command | RED reason | Exit |
|---|---|---|---:|
| Boundline | `scripts/release/verify-m1e-package-manifests.sh` | workspace and package metadata still reported 0.82.0 | 1 |
| Canon | `scripts/release/verify-m1e-package-manifests.sh` | workspace and package metadata still reported 0.72.6 | 1 |
| Speckit adapter | `scripts/verify-m1e-protocol-dependency.sh` | legacy Boundline 0.66 Git bridge remained | 1 |

Adapter behavior tests rejecting `0.89.9`, `1.0.0`, and malformed versions
were added before implementation. The first run was RED because `0.89.9`
incorrectly returned `ready`. The implementation then admitted `0.90.0` and
`0.99.7`, rejected unsupported lines with
`unsupported_boundline_version`, and made no T064/T065 transport or DTO
change.

## Version and dependency changes

Boundline and Canon workspace-owned packages now inherit exactly `0.90.0`.
Registry-compatible internal requirements are exact `=0.90.0` requirements
paired with workspace paths for local builds. Both lockfiles intentionally
record their owned packages at `0.90.0`. MSRV remains `1.96.0`.

The adapter removed the full-runtime tagged Git dependency:

```text
boundline-adapters @ Boundline tag 0.66.0
```

and now depends on:

```toml
boundline-protocol = "=0.90.0"
```

Its tested compatibility metadata is separately
`>=0.90.0,<1.0.0`. The adapter version remains `0.1.0` because T019 does not
own an adapter release bump. The adapter lockfile intentionally removed the
obsolete Boundline runtime graph and retained only the protocol and adapter
dependencies.

## Candidate provenance

The source-tree digest is SHA-256 over `git ls-tree -r <commit>`. The normalized
payload digest is SHA-256 over the uncompressed `.crate` tar stream. Repeated
clean packaging produced byte-identical archives as well as identical
normalized payloads.

### boundline-protocol 0.90.0

| Field | Value |
|---|---|
| Repository / branch | Boundline / `083-governed-1-0-release` |
| Source and tag-target commit | `860b7247d8343c63dca2940a923b58f3172632f9` |
| Source-tree digest | `1afe70cf005e5b737c7a2437a63b53ab22014fa27acb0d586294bf49cb0c0124` |
| Package Cargo.toml digest | `9969e796471ee074d20dc065821ab24734538d73ceb88c7c686fb842c18aee0a` |
| Workspace Cargo.lock digest | `d097350a381d7851b158397685c5f4733cd43bb253fea6fced691210bb6ec98a` |
| Artifact | `boundline-protocol-0.90.0.crate` |
| Artifact SHA-256 | `3bde42852595c53b256afb1d459f3e9e2f0994ed4f401f6161cb266044f0ae07` |
| Normalized payload digest | `ed1c2cd636a835a07ba0b51c549aeecff19f18a5dd90ed966b7116f706dc43d8` |
| Files | 13 |
| Verification | PASS |
| Registry / predecessor | crates.io / none |
| Expected tag | signed annotated `0.90.0` at the source commit |
| Provenance digest | `5d25df8924e1009ccbe88227a9a677d735f6ed64dbc32021b93bf10bb84cdd34` |

### canon-contracts 0.90.0

| Field | Value |
|---|---|
| Repository / branch | Canon / `083-governed-1-0-release` |
| Source and tag-target commit | `bd361d7e2ad112e5e0d599267e8024e748293605` |
| Source-tree digest | `72750de23605333965185af08e6521bcebff04256e5185efa331841b3e187444` |
| Package Cargo.toml digest | `6fa9df47b600a9fea5cc60e0ba6abac1686e6cb6d82163b78cd8ae429796988a` |
| Workspace Cargo.lock digest | `8c9e4ecb3c2f5370efd7457143a3d99caf78d04174d9648edca0455913e6baf7` |
| Artifact | `canon-contracts-0.90.0.crate` |
| Artifact SHA-256 | `f6874bfbeca46f10307a9345c7aba84ba9d67fbb0b7a83b1eb6e1d3e17d44680` |
| Normalized payload digest | `681e6248d98959046a2adc40deee78cff1b8665968c85028fd382132d4b366e7` |
| Files | 12 |
| Verification | PASS |
| Registry / predecessor | crates.io / none |
| Expected tag | signed annotated `0.90.0` at the source commit |
| Provenance digest | `0ffffc5274ef048086a384d515d573ebbe7cf4dc6174737f0d62a22ba7f68306` |

The provenance digest for each package is SHA-256 over newline-delimited,
UTF-8 values in this order: package, version, full source commit, source-tree
digest, package-manifest digest, workspace-lock digest, artifact digest, and
normalized payload digest.

### Speckit adapter qualification candidate

| Field | Value |
|---|---|
| Source commit | `ce7cb2e57c7ad8bd38064c684a4736029397756c` |
| Source-tree digest | `baef73673c381410587fbe39c219eb6154e48e4705289a354f61de965a263c48` |
| Cargo.toml digest | `83c9c1c7a7f5bd6265219f461c1609dd6377d5d3a0c9b75ab9a8feeeecf3f8a6` |
| Cargo.lock digest | `b4a1f0a9b4b5f34abd62d7dcf5aefa330dd964b5c95bdedd252a6af9c4b3b22a` |
| Artifact | `boundline-adapter-speckit-0.1.0.crate` |
| Artifact SHA-256 | `1608cdf0507866f719199e5c23b1be5d1782cffb71383962f4fde32dafc2ef48` |
| Normalized payload digest | `16db67db8c2eeb2ba5e288b0ae415969f46f524fe3d3575b026475447deb48e4` |
| Files | 36 |
| Verification | PASS; publication deferred |

All candidates were created outside the repositories. Generated manifests
preserve license, repository, description, Rust 1.96 MSRV, edition 2024, and
the intended publish policy. They contain no local path or Git dependency.
Archive-name and content audits found no developer-local absolute path,
credential material, target output, generated coverage output, cache,
temporary state, or unrelated evidence. The adapter intentionally packages
its repository-owned coverage runner scripts; no coverage output is packaged.
Cargo refused the adapter package-list operation while the source tree was
dirty, proving that an uncommitted candidate could not silently become the
reviewed clean artifact.

## Registry-shaped local consumers

An isolated local registry contained the exact reviewed archives and their
checksums. Three clean consumers used new lockfiles and exact name/version
requirements:

| Consumer | Result | Registry proof |
|---|---|---|
| Boundline protocol | PASS, then PASS offline | metadata source is `registry+https://github.com/rust-lang/crates.io-index`; manifest came from isolated Cargo registry storage |
| Canon contracts | PASS, then PASS offline | same; no path, Git, workspace, or patch source |
| Speckit adapter | 1 test PASS, then PASS offline | adapter and transitive protocol both resolved as registry packages; protocol requirement is exactly `=0.90.0` |

Extracted-package tests also passed: 13 Boundline protocol tests and 8 Canon
contract tests. These results qualify package readiness only; T020 must repeat
the consumers against the real registry.

## Repository verification

Commands were run from each repository root unless the row says otherwise.

| Repository | Command | Result | Duration |
|---|---|---|---:|
| Boundline | `cargo fmt --all -- --check` | PASS | <1 s |
| Boundline | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS | <1 s incremental |
| Boundline | `cargo test --workspace --all-features` | PASS | 140 s |
| Boundline | `cargo nextest run --workspace --all-features` | 2,719 passed, 2 skipped | 108 s |
| Boundline | `cargo deny check licenses advisories bans sources` | PASS; pre-existing duplicate-version warnings only | <1 s |
| Boundline | `cargo package --locked -p boundline-protocol` | PASS | 4 s |
| Boundline | `cargo publish --dry-run --locked -p boundline-protocol` | PASS; no upload | 4 s |
| Boundline | focused LLVM coverage plus accepted patch helper for the changed release-surface test | no uncovered executable changed line | PASS |
| Canon | `cargo fmt --all -- --check` | PASS | <1 s |
| Canon | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS | <1 s incremental |
| Canon | `cargo test --workspace --all-features` | PASS | 217 s |
| Canon | `cargo nextest run --workspace --all-features` | 1,715 passed, 0 skipped | 255 s |
| Canon | `cargo deny check licenses advisories bans sources` | PASS | <1 s |
| Canon | `cargo package --locked -p canon-contracts` | PASS | 2 s |
| Canon | `cargo publish --dry-run --locked -p canon-contracts` | PASS; no upload | 2 s |
| Canon | focused LLVM coverage plus accepted patch helper for the changed release-surface test | no uncovered executable changed line | PASS |
| Speckit adapter | `cargo fmt --all -- --check` | PASS | <1 s |
| Speckit adapter | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS | <1 s incremental |
| Speckit adapter | `cargo test --workspace --all-features` | 23 tests passed | <1 s incremental |
| Speckit adapter | `cargo nextest run --workspace --all-features` | 23 passed, 0 skipped | <1 s incremental |
| Speckit adapter | `cargo deny --offline check licenses advisories bans sources` | PASS; four unused-license-allowance warnings | <1 s |
| Speckit adapter | `cargo package --locked` against the isolated registry | PASS, 36 files; repeated candidate SHA matched | 9.5 s combined |
| Speckit adapter | focused LLVM coverage plus accepted patch helper | 96% patch coverage (24/25) | PASS |

The four adapter cargo-deny warnings are unused allowances for BSD-2-Clause,
BSD-3-Clause, CDLA-Permissive-2.0, and ISC after removal of the old runtime
graph. They do not convert a failure to success. The Adapter Release Owner
owns policy cleanup and review under T068/M4 and the final T092 deny gate
before any adapter publication. Boundline's pre-existing permitted
duplicate-version advisories remain owned by the Boundline Release Owner under
T092; any finding that becomes release-blocking must be resolved before RC.
No waiver was created.

The initial post-change adapter check without the isolated registry attempted
the still-unpublished crates.io package and failed before compilation. The
qualified check used the exact local candidate. This is the expected boundary
between T019 package readiness and T020 public-registry availability.

## Prepared T020 operation

T020 remains incomplete. The later approved operator sequence is:

1. Verify clean branch state and rebuild the candidates from the two recorded
   source commits. Stop if either candidate digest differs.
2. Confirm `0.90.0` is absent from crates.io and that repository-local tag
   `0.90.0` is unused.
3. Re-run:

   ```bash
   cargo publish --dry-run --locked -p boundline-protocol
   cargo publish --dry-run --locked -p canon-contracts
   ```

4. Publish, without altering the reviewed artifacts:

   ```bash
   cargo publish --locked -p boundline-protocol
   cargo publish --locked -p canon-contracts
   ```

5. Retrieve each exact crates.io version, compare its checksum, and repeat the
   clean-consumer build, tests, metadata-source inspection, and offline run.
   Stop without tagging on timeout, mismatch, unresolved dependency, build
   failure, or test failure. A crates.io upload cannot be rolled back; yanking
   requires a separate incident decision and is not an automatic fallback.
6. Create signed annotated tags using GPG key `25C2C7D21E91FD50`:

   ```bash
   # In Boundline:
   git tag -s -u 25C2C7D21E91FD50 \
     -m "boundline-protocol 0.90.0; provenance 5d25df8924e1009ccbe88227a9a677d735f6ed64dbc32021b93bf10bb84cdd34" \
     0.90.0 860b7247d8343c63dca2940a923b58f3172632f9

   # In Canon:
   git tag -s -u 25C2C7D21E91FD50 \
     -m "canon-contracts 0.90.0; provenance 0ffffc5274ef048086a384d515d573ebbe7cf4dc6174737f0d62a22ba7f68306" \
     0.90.0 bd361d7e2ad112e5e0d599267e8024e748293605
   ```

7. Run `git tag -v 0.90.0` and
   `git rev-list -n 1 0.90.0` in each repository. After exact signature,
   target, and message verification, run `git push origin refs/tags/0.90.0`
   separately in each repository. No unsigned fallback, reuse, or force update
   is allowed.

Evidence to capture includes registry API identity and response time, upload
receipt, registry checksum, resolved metadata and lockfiles, clean-consumer
logs, tag objects, signer fingerprint, signature verification, remote
pre-existence checks, and push receipts.

## Independent review passes

Three distinct review passes were completed before closure:

1. **Version and dependencies** — only the intended workspaces advanced to
   0.90.0; internal package requirements are registry-compatible; the adapter
   uses only the exact protocol candidate; no T064/T065 work was pulled in.
2. **Package contents** — repeat artifacts match; generated manifests and file
   inventories are minimal for their current Cargo include policy; no local or
   sensitive material is present; provenance hashes match the reviewed files.
3. **Release safety** — no package or tag exists as a result of this run;
   signing is available; T020 remains unchecked; every irreversible action is
   reserved for a separately approved run.

No Critical or Important finding remains. T019 is complete. T020 is
`READY_TO_PUBLISH`, not complete.
