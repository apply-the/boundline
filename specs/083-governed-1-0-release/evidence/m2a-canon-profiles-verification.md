# M2a Canon Profiles and Verification Evidence

**Accepted:** 2026-07-28

**Scope:** T049-T052 only

**Decision:** GO M2a; T053 and later tasks remain not started

## Repository identity

| Repository | Starting commit | M2a disposition |
|---|---|---|
| Boundline | `6b9641082a73df18c230bff8f6f3e715b39edbe1` | Joint evidence and task state only |
| Canon | `922a9107f5982aab33eab517d0ea1590c3fbdd25` | T049-T052 implementation |
| Speckit adapter | `ce7cb2e57c7ad8bd38064c684a4736029397756c` | Inspected only; unchanged |

All three repositories started on `083-governed-1-0-release` with clean staged,
unstaged, and untracked state.

## Published contract immutability

`canon-contracts 0.90.0` remains frozen:

- source and signed-tag target:
  `bd361d7e2ad112e5e0d599267e8024e748293605`;
- published artifact SHA-256:
  `f6874bfbeca46f10307a9345c7aba84ba9d67fbb0b7a83b1eb6e1d3e17d44680`;
- normalized payload digest:
  `681e6248d98959046a2adc40deee78cff1b8665968c85028fd382132d4b366e7`;
- provenance-manifest digest:
  `0ffffc5274ef048086a384d515d573ebbe7cf4dc6174737f0d62a22ba7f68306`.

`git diff --exit-code bd361d7e2ad112e5e0d599267e8024e748293605
-- crates/canon-contracts` returned zero. `git rev-list -n 1 0.90.0` returned
the same source commit. `git tag -v 0.90.0` reported a good EDDSA signature
from the expected release identity. No contract module, field, enum, wire name,
version, artifact, or tag changed.

## Profile inventory and boundary

The pre-change engine inventory contained 21 mode identifiers:

```text
discovery
requirements
system-shaping
architecture
system-assessment
change
backlog
pr-review
implementation
refactor
verification
review
incident
security-assessment
migration
supply-chain-analysis
domain-language
domain-model
debugging
brainstorming
policy-shaping
```

The final stable registry contains exactly nine entries in the frozen order:

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

One engine registry now supplies enumeration, exact parser admission, mode
inspection, CLI run metadata, and contract-profile mapping. The registry maps
one-to-one to the immutable `canon-contracts::Profile` variants. Case changes,
whitespace, malformed identifiers, aliases, and unknown identifiers fail
deterministically.

`implementation` is absent from stable enumeration, help, inspection, and new
CLI admission. It is not mapped to `change`, `refactor`, or another profile.
Historical parsing and historical runtime fixtures use an explicit internal
legacy boundary. `change` governs intent, scope, risk, invariants, acceptance
criteria, authority, and evidence; Canon does not execute the change.

## Reviewer, Copilot, MCP, and verify inventory

| Surface | Before M2a | After M2a |
|---|---|---|
| Reviewer abstraction | Provider-neutral trait plus a successful stub | Typed structural validator for externally supplied evidence; execution request is `Unsupported` with zero side effects |
| Copilot verification | Synthetic verification generation and critique could be represented as review | Verification and review paths use authored deterministic projections; semantic execution is denied; review-specific synthetic methods are removed |
| MCP stdio | Public no-op stub module | No transport, handler, registration, echo, or successful stub; MCP remains policy-classified and runtime-disabled for T056 |
| `canon verify` | Stable command returned unimplemented | Reads the typed persisted evidence projection, reports deterministic structural status, reports `required_missing` for external semantic judgment, and exits nonzero |

Generic Copilot-assisted authoring remains outside the semantic-review
boundary. No verification or review path calls it, and no generic authored
summary is accepted as external evidence.

## External-evidence acceptance matrix

| Input condition | Deterministic result |
|---|---|
| External producer, well-formed independent lineage, exact claims and immutable references, fresh terminal binding, sufficient tier | Structure accepted; `semantic_judgment_asserted` remains false |
| Missing, Canon-owned, current-invocation, malformed, shared, deterministic, or non-generative lineage | Rejected |
| Missing or current-invocation independent context | Rejected |
| Empty, duplicate, extra, or mismatched claims | Rejected |
| Empty, duplicate, subset, extra, or mismatched evidence references | Rejected |
| Stale binding or nonterminal producer result | Rejected |
| Insufficient challenge tier or blank named override | Rejected |
| Unknown authority-expanding serialized value | Deserialization fails closed |
| Request for Canon to execute semantic review | Typed `Unsupported`; non-success |

The validator checks structure and binding only. It never asserts that a
reviewer's semantic judgment is true.

## No-execution proof

The semantic-execution boundary records:

```text
process_invocations = 0
network_invocations = 0
provider_credential_reads = 0
created_evidence = 0
```

Contract tests assert all four counters. Invocation tests also assert that
verification records no `CopilotCli` adapter use, records the unsupported
semantic request as `Denied`, keeps validation independence false, keeps
evidence incomplete, and leaves the run blocked even when authored text says
`supported`.

## TDD evidence

The first profile-registry test run failed with unresolved imports for the
not-yet-created registry API. The first external-verification test run failed
with unresolved imports for the not-yet-created evidence validator. The first
CLI verification test run failed because the previous `execute` signature did
not accept a service and run identity.

Independent review added two further regressions before their fixes:

- exact evidence binding: 4 tests passed and 1 failed because a strict subset
  of the required references was accepted;
- fail-closed CLI status: the focused test failed with actual exit `0`,
  expected exit `5`, and `external_semantic_judgment: not_evaluated`.

After implementation, both focused cases pass with exact reference equality,
`external_semantic_judgment: required_missing`, and exit code 5.

## Verification results

The pre-change selection was:

```text
cargo nextest run --workspace --all-features
1715 passed, 0 skipped
```

M2a acceptance commands:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS; 0.52 s |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS; 7.61 s |
| `cargo test --test profile_registry --all-features` | PASS, 6 tests; 6.88 s |
| `cargo test --test external_verification --all-features` | PASS, 5 tests; 0.65 s |
| `cargo test --workspace --all-features` | PASS; 167.66 s |
| `cargo nextest run --workspace --all-features` | PASS; 1,718 passed, 0 skipped; nextest summary 217.922 s |
| `cargo deny check licenses advisories bans sources` | PASS: advisories, bans, licenses, and sources all `ok`; 0.72 s |
| Full workspace `cargo llvm-cov` in a dedicated temporary target | PASS; 42,926/44,984 lines, 95.43%; 197.57 s |

Patch intersection covered 361 of 372 executable added Rust lines: 97.04%.
The frozen Canon line threshold is 95.34% and the patch threshold is 90%;
both gates pass.

Boundline `git diff --check` passed. `npm run docs:build` completed in 8.44 s.
VitePress repeated its existing advisory that some minified chunks exceed
500 kB; this is a documentation-bundle performance advisory, not a broken
page or M2a semantic gate, and remains owned by normal documentation tooling
maintenance. The adapter acceptance checks only branch, commit, and clean
status.

## Dependency and lockfile disposition

No version changed. Canon added only workspace-local dependencies on the
already published and frozen `canon-contracts` crate:

- `canon-adapters` runtime dependency;
- `canon-engine` runtime dependency;
- workspace-root development dependency for cross-crate golden tests.

`Cargo.lock` records those three dependency edges and no new third-party
package.

## Independent review resolutions

1. Stable CLI tests for historical modes were moved behind explicit internal
   historical test helpers instead of reopening stable parser admission.
2. The safety review found that the legacy `review` path still called synthetic
   Copilot review helpers. Those helpers and calls were removed; authored
   review labels now remain blocked and explicitly unverified.
3. A denied semantic-review invocation previously carried exit code zero. It
   now carries no successful exit code.
4. Evidence-reference validation previously accepted a subset of the admitted
   binding. It now requires exact, duplicate-free set equality.
5. `canon verify` previously returned success after only deterministic
   structure validation. It now reports the missing external judgment and
   exits nonzero.
6. Making the MCP stub internal exposed dead code. The module was removed
   entirely; only the existing policy classification and runtime-disabled
   posture remain.
7. Full coverage initially exceeded available disk space. Generated Canon
   build output was cleaned, then coverage ran in a dedicated temporary target;
   no repository artifact was removed or rewritten.

## Carried work

T053-T059 remain unchecked and were not started. In particular:

- T053/T054 still own the deterministic governance corpus, typed governance
  bundles, decision-memory graph, and stale propagation;
- T055/T056 still own the frozen complete CLI and one-shot RPC/MCP transport;
- T057-T059 still own outcome synchronization and ingestion.

M2b may start only from the committed M2a boundary and must preserve the
truthfulness and immutable-contract gates recorded here.
