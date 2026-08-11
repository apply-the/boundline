# T099 canon-contracts 0.91 Publication and Consumer Evidence

**Evidence date**: 2026-08-11

**Decision**: GO T099

**Scope**: T099 only; T057-T059 not started

## Immutable release identity

| Field | Value |
|---|---|
| Package | `canon-contracts 0.91.0` |
| Source and signed-tag target | `cd9209a861acf8d2960b185c476fedfdac9e41d2` |
| Source-tree SHA-256 | `eb5f20db71872d0d73fa065a47abc17fe2598cfc5ab931056ac040ab41bb98f5` |
| Package-manifest SHA-256 | `10cb130804af53215e5b889d44211bcc12cea2c56a24d7b965829a8da990af86` |
| Workspace-lock SHA-256 | `00e88e6bf7a7cbc4d104d8a4dd742d1c4a33b6538ee0a27377a40d7c6517ead7` |
| Artifact and registry checksum | `ec789921edf1ac39028cfb68107d5e09d425d4efab95b13292102703aca155ed` |
| Normalized payload SHA-256 | `f45c4689f6a255e53723c3b2eeb72958e93b512dbf943e125e1b16f69eb746e9` |
| Corrected provenance SHA-256 | `78e2d6f8b6f580bcd123c8e7bdc7ac80bef00059dd7058c8e0acd15253dcc414` |
| Published 0.90 control | `0ffffc5274ef048086a384d515d573ebbe7cf4dc6174737f0d62a22ba7f68306` |

T100 corrected only the newline-delimited provenance calculation. The earlier
value
`f2cb4ba8607f7463d780fb6d6e66a1a309d9a540b465749720cee912261a8aca`
remains historical failure evidence and is not an active release identity.
Every package input and the signed source commit remained unchanged.

## Registry publication and retrieval

`cargo publish --locked -p canon-contracts` exited 0. crates.io records:

```text
published_at: 2026-08-10T17:07:26.947098Z
owner: robertotru
yanked: false
checksum: ec789921edf1ac39028cfb68107d5e09d425d4efab95b13292102703aca155ed
crate_size: 20006
license: MIT
rust_version: 1.96
edition: 2024
```

The first archive retrieval did not follow the registry redirect and produced
an empty local response file. That file was rejected as evidence. A fresh
retrieval with redirect handling enabled reproduced the frozen archive,
normalized payload, and corrected provenance hashes exactly. No retry publish,
yank, republish, or artifact modification occurred.

## Registry-only consumer

An isolated consumer declared only `canon-contracts = "=0.91.0"`. Cargo
metadata proved a crates.io registry source with no path, Git, patch,
workspace, or local-registry override. Its lockfile SHA-256 is:

```text
f20339c5fdba79b78e48a1b1d846f0fbc985db8cc7c1b3d9766fd4d2b02c363b
```

Three tests passed on the current toolchain online, the current toolchain
offline, and Rust 1.96 locked/offline. They prove V1 `"1.0"`, the exact
historical six operations, additive seventh `record_outcome`, publish/outcome
separation, request and response round trips, the golden digest, and terminal
status invariants.

## Signed source tag

| Field | Value |
|---|---|
| Tag | `0.91.0` |
| Tag object | `ee8a84c435dc4a5fcbd18beb6c7579199f1b418d` |
| Target | `cd9209a861acf8d2960b185c476fedfdac9e41d2` |
| Signer fingerprint | `16A8F86BAA21130A2657B41B25C2C7D21E91FD50` |
| Remote verification | isolated fetch matched object and target; signature PASS |

The tag was pushed individually without force. It was not amended, replaced,
moved, or recreated during closure.

## Boundline consumption boundary

`boundline-core` is the smallest existing owner of the future T058 outbox and
therefore owns the exact workspace dependency. `Cargo.lock` resolves
`canon-contracts 0.91.0` from crates.io with the exact published checksum.
The focused consumer adds contract qualification only: no outbox, delivery,
retry, archival, Canon invocation, outcome ingestion, or T057-T059 production
source exists.

The final official Boundline coverage gate used `scripts/coverage.sh`, which
aggregates the workspace and three library reports. It measured 104,353 of
110,770 lines, or 94.206915%, above the frozen 92.77% threshold. A preceding
workspace-only report measured 106,405 of 114,913 lines, or 92.596138%; that
intermediate report is not the repository's aggregate CI gate. Its first
parallel attempt exposed two existing routing-fixture interference failures;
both passed independently and in the fresh serial full-workspace rerun without
source changes.

No executable production source changed in Boundline, so patch coverage is not
applicable. The consumer and manifest changes do not begin T057-T059.

## Canon coverage closure

The first completely fresh Canon report reproduced the post-publication block:

```text
46,353 / 48,666 = 95.247195%
```

Line-level inspection selected existing public requirements generation and
critique behavior. Two external integration tests verify that missing authored
context remains visible, blocks downstream design, and produces corrective
guidance, while complete packets distinguish open-question routing. No
production Canon source and no published-package input changed. Focused
coverage of `copilot_cli.rs` rose from 178/244 to 238/244 lines. The fresh full
result is:

```text
46,413 / 48,666 = 95.370485%
```

This is 60 additional covered production lines with an unchanged denominator,
above the frozen 95.34% threshold. No synthetic test or coverage exclusion was
introduced.

## Quality and safety disposition

Canon verification:

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0; 34.39 s including format/build startup |
| `cargo test -p canon-contracts --all-features` | 26 passed |
| `cargo test --test stable_surface --all-features` | 15 passed; 5.52 s |
| `cargo test --workspace --all-features` | all unit, integration, and doc tests passed |
| `cargo nextest run --workspace --all-features` | 1,777 passed; 0 skipped; 380.407 s |
| `cargo deny check licenses advisories bans sources` | all four checks `ok`; 0.82 s |
| `bash scripts/check-rust-no-panic.sh` | exit 0 |
| `bash scripts/coverage.sh` | 46,413 / 48,666; 95.370485%; PASS |

Boundline verification:

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --workspace --all-features` | all unit, integration, and doc tests passed; 2 fixture tests ignored by declared policy |
| `cargo nextest run --workspace --all-features` | 2,722 passed; 2 skipped; 193.135 s |
| `cargo deny check licenses advisories bans sources` | all four checks `ok`; existing policy-accepted duplicate-version warnings retained |
| `bash scripts/check-rust-no-panic.sh` | exit 0; 19.95 s |
| `git diff --check` | exit 0 |
| `npm run docs:build` | exit 0; 13.19 s; non-fatal existing chunk-size advisory |
| `bash scripts/coverage.sh -- --test-threads=1` | 104,353 / 110,770; 94.206915%; PASS |

The adapter remained untouched at
`ce7cb2e57c7ad8bd38064c684a4736029397756c`. `record_outcome` remains
discoverable but unavailable before T059. T057, T058, and T059 remain
incomplete and no M2b-C runtime work began.

Local Cargo credentials were removed with `cargo logout`. On 2026-08-11 the
operator explicitly confirmed server-side revocation of the fresh T099 token.
No token value is recorded. All T099 gates are complete; T057-T059 remain
separately gated and incomplete.
