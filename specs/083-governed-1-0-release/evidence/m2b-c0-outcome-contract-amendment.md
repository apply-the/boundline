# M2b-C0-A Outcome Contract Amendment Evidence

**Evidence date**: 2026-07-29  
**Decision**: GO  
**Scope**: T097 and T098 complete; T099 `READY_TO_PUBLISH`; T057-T059 not started

## Repository boundary

| Repository | Starting commit | Closure state |
|---|---|---|
| Boundline | `8a5a9226b70714a69220a83cf4c31b9699f6377a` | normative documentation amendment only |
| Canon | `0cded7eb7e555964212be315e6f61075a20976d8` | implementation and package candidate at `cd9209a861acf8d2960b185c476fedfdac9e41d2` |
| Speckit adapter | `ce7cb2e57c7ad8bd38064c684a4736029397756c` | unchanged |

No package was uploaded, no tag was created or moved, and no branch was
pushed.

## Contract decision

The additive one-shot operation is `record_outcome`. The exact inventories
are:

```text
0.90:
capabilities start refresh approve inspect publish

0.91:
capabilities start refresh approve inspect publish record_outcome
```

The human root inventory remains:

```text
init run resume status approve inspect publish assistant rpc
```

The existing six operation names and semantics are unchanged. `publish`
remains a read-only governance, evidence, and decision-memory projection with
an empty payload. `CanonContractVersion::V1` remains serialized as `"1.0"`
because the operation is additive and capability-discovered.

Rejected alternatives:

- overloading `publish` would violate read-only projection semantics;
- overloading `start` would confuse terminal outcome recording with admission;
- overloading `approve` would confuse an outcome with authority approval;
- a private internal wire format would violate the public integration
  boundary;
- changing 0.90.0 would violate the published package and signed-tag
  immutability boundary.

## Public exchange

`RecordOutcomeRequest` freezes event identity and digest, source product and
clone-local repository identity, governance bundle identity and digest,
session and final transaction revision, terminal status, optional commit and
fingerprint, proof references, deviations, terminal claims, authority,
optional approval, challenge, lineage, and optional authoritative occurrence
time.

`RecordOutcomeResponse` freezes event identity and digest, disposition,
decision-memory revision and digest for recorded/replayed outcomes, rejection
reason for rejected outcomes, and typed next actions.

Accepted terminal status invariants:

| Status | Commit | Fingerprint |
|---|---:|---:|
| `published` | required | required |
| `no_change` | forbidden | required |
| `failed` | forbidden | optional |
| `cancelled` | forbidden | optional |
| `rejected` | forbidden | optional |

`blocked` and `stale` are closed candidates but are rejected with
`nonterminal_outcome`. Unknown values fail closed.

Stable rejection reasons are `unsupported_contract_line`, `invalid_outcome`,
`nonterminal_outcome`, `identity_digest_conflict`,
`authority_binding_invalid`, `approval_binding_invalid`,
`evidence_binding_invalid`, `lineage_invalid`, `stale_outcome`,
`decision_memory_conflict`, `persistence_failed`, and
`unsupported_operation`.

## Canonical digest

The domain is `canon-boundline-outcome-c14n-v1`. SHA-256 binds the domain,
one zero-byte separator, and canonical JSON containing every authoritative
field except the digest itself. Object keys are recursively sorted, semantic
sequence order is retained, set-like collections are sorted and
duplicate-free, integers are exact, and floats and duplicate JSON keys fail
closed. The decoder recomputes the digest.

The independent golden digest fixture is:

```text
sha256:eb492104900410462528226f5fca56a758ef13b940d8a0c5d962110d5de2bafa
```

## Pre-T059 service boundary

Capabilities expose `record_outcome` with `available: false` and
`unsupported_operation`. A valid direct request returns a typed rejected
`RecordOutcomeResponse`, exit code 8, no decision-memory revision or digest,
and no state snapshot. A request/event identity mismatch returns
`identity_digest_conflict`. No pre-T059 path returns `recorded` or `replayed`.

T059 alone may atomically install transactional ingestion and promote
availability. No T057, T058, or T059 source file was created or modified by
this amendment.

## RED and GREEN evidence

The first focused T097 run failed at compile time with exit 101 because the
new operation and twenty public outcome types did not exist. After the DTO
implementation, privacy, Tier 0 independence, and decision-memory digest tests
were each observed failing before their corresponding validation was added.

The first T098 stable-surface run reported 12 passed and 3 failed: the
capability inventory still exposed six operations, historical-six assertions
did not distinguish 0.90 from 0.91, and no typed pre-T059 outcome response
existed. The final focused results are:

| Command | Result | Duration |
|---|---|---:|
| `cargo test -p canon-contracts --all-features` | 26 passed: 8 historical contract + 18 outcome; 0 failed | 0.55 s |
| `cargo test --test stable_surface --all-features` | 15 passed; 0 failed | 7.32 s |
| direct pending-outcome dispatcher test | 1 passed; 0 failed | included in focused run |

## Repository verification

| Command | Result | Duration |
|---|---|---:|
| `cargo fmt --all -- --check` | exit 0 | 0.63 s |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 | 15.03 s |
| `cargo test --workspace --all-features` | exit 0; all unit, integration, and doc tests passed | 240.92 s |
| `cargo nextest run --workspace --all-features` | 1,775 passed; 0 skipped | 422.56 s |
| `cargo deny check licenses advisories bans sources` | advisories, bans, licenses, sources all `ok` | 0.87 s |
| `bash scripts/check-rust-no-panic.sh` | exit 0 | included in final quick gate |
| `bash scripts/coverage.sh` | exit 0; LCOV generated | 306.28 s |

The pre-amendment nextest floor was 1,755. The final increase is twenty:
seventeen original outcome contract cases, one stable-surface case, one
response fail-closed case, and one direct pre-T059 dispatcher case.

Coverage from the final full workspace LCOV:

```text
repository lines: 45,246 / 47,214 = 95.8317%
patch lines:         522 /    568 = 91.9014%
```

Both gates exceed the required 95.34% repository and 90% patch thresholds.

The initial sandboxed workspace test exposed two environmental fixture
boundaries: Git commits inherited global GPG signing, and the sandbox rejected
a deliberately non-UTF directory. The Git fixture now disables signing in the
same way as sibling fixtures. The non-UTF test passed unchanged outside the
sandbox, proving no product change was required. Full final test and coverage
runs used that qualified filesystem boundary.

The coherent 0.91 workspace bump also exposed and corrected stale versions in
the five assistant/host package manifests and in the governed-reasoning
runtime compatibility metadata.

## Package readiness

The immutable local candidate is:

```text
artifact: canon-contracts-0.91.0.crate
source/tag target: cd9209a861acf8d2960b185c476fedfdac9e41d2
source tree SHA-256: eb5f20db71872d0d73fa065a47abc17fe2598cfc5ab931056ac040ab41bb98f5
Cargo.toml SHA-256: 10cb130804af53215e5b889d44211bcc12cea2c56a24d7b965829a8da990af86
Cargo.lock SHA-256: 00e88e6bf7a7cbc4d104d8a4dd742d1c4a33b6538ee0a27377a40d7c6517ead7
artifact SHA-256: ec789921edf1ac39028cfb68107d5e09d425d4efab95b13292102703aca155ed
normalized payload SHA-256: f45c4689f6a255e53723c3b2eeb72958e93b512dbf943e125e1b16f69eb746e9
provenance manifest SHA-256: f2cb4ba8607f7463d780fb6d6e66a1a309d9a540b465749720cee912261a8aca
```

Two successive package runs were byte-identical. The archive contains 16
expected package files, has no path or Git dependency, and contains no target,
cache, coverage, temporary, credential, or local-path artifact. The generated
manifest retains edition 2024, Rust 1.96.0, MIT license, repository metadata,
and registry dependencies on `serde`, `serde_json`, and `sha2`.

`cargo publish --dry-run --locked -p canon-contracts` reached the upload step
and aborted it as required. `cargo info --registry crates-io
canon-contracts@0.91.0` returned not found, confirming that 0.91 was not
published during this task.

The extracted final archive passed 26 tests. A registry-shaped consumer used
exact `canon-contracts = "=0.91.0"` with no path, Git, or patch source,
round-tripped the outcome DTO, and independently asserted the seven-operation
inventory. It passed with the current toolchain and with Rust 1.96.0 in locked
offline mode. Its final lockfile digest is
`80abe0fc62df82bc9e80bba05c0396fe111fc78c603aad37e151d9b9141f257d`.

## 0.90 immutability

The signed `0.90.0` tag still resolves to:

```text
bd361d7e2ad112e5e0d599267e8024e748293605
```

`git tag -v 0.90.0` reports a good EDDSA signature with key
`16A8F86BAA21130A2657B41B25C2C7D21E91FD50`. The artifact retrieved from the
official crates.io static endpoint still has SHA-256:

```text
f6874bfbeca46f10307a9345c7aba84ba9d67fbb0b7a83b1eb6e1d3e17d44680
```

No local `0.91.0` tag exists.

## Independent reviews

Contract review found the original 947-line outcome module too large. It was
split into focused model, digest, and validation modules without changing the
public API. No Critical or Important finding remains: operation identity,
request/response completeness, terminal invariants, digest determinism,
replay/conflict representation, and internal-record isolation are covered.

Compatibility review confirmed the exact historical six, one additive
operation, unchanged human roots, read-only `publish`, justified V1 envelope,
and immutable 0.90 artifact/tag.

Safety review confirmed no successful pre-T059 stub, authority inference,
semantic evidence creation, hidden wire format, outcome persistence,
publication, tag creation, or T057-T059 implementation.

## T099 publication plan

T099 remains incomplete and `READY_TO_PUBLISH`. Its irreversible sequence is:

1. verify clean source exactly at
   `cd9209a861acf8d2960b185c476fedfdac9e41d2`;
2. reproduce every digest above and recheck registry absence;
3. publish `canon-contracts 0.91.0`;
4. retrieve the real registry artifact and require exact checksum/provenance;
5. run exact online and locked-offline registry consumers;
6. create a signed annotated `0.91.0` tag at the same source commit, verify it,
   and push only that verified tag;
7. update Boundline to exact registry dependency
   `canon-contracts = "=0.91.0"` and rerun its consumer gates.

Any source, artifact, checksum, registry, consumer, or signature mismatch
stops T099. Only after T099 closes may T057-T059 begin.
