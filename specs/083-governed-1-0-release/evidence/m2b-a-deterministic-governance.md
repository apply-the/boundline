# M2b-A Deterministic Governance Evidence

**Accepted:** 2026-07-29

**Scope:** T053-T054 only

**Decision:** GO M2b-A; T055 and later tasks remain not started

## Repository identity

| Repository | Starting commit | M2b-A disposition |
|---|---|---|
| Boundline | `24eccaa0dc8451913ee0fa7d9db425ff6d0cd7be` | Joint evidence and task state only |
| Canon | `daef129dafe3c0466f26e5136c8ce7162025b18b` | T053-T054 corpus and implementation |
| Speckit adapter | `ce7cb2e57c7ad8bd38064c684a4736029397756c` | Inspected only; unchanged |

All three repositories were inspected on `083-governed-1-0-release`.
Boundline and the Speckit adapter were clean. Canon contained the already
reviewed T053 RED corpus commit and the staged T054 implementation authorized
for closure.

Canon M2b-A consists of these focused commits:

- `40a139f59683677f19bb0abc683866f819360ca1`
  (`feat: add deterministic governance corpus`);
- `d19a47f49d5114af2a88a0e9adbad93938ee453b`
  (`feat: add deterministic governance decision memory`).

No M2b-A commit was pushed during closure.

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
from the expected release identity with fingerprint
`16A8F86BAA21130A2657B41B25C2C7D21E91FD50`.

No contract module, field, enum, wire name, package version, artifact, tag,
workspace version, dependency, or lockfile changed.

## Typed governance graph

The decision-memory graph represents these node kinds explicitly:

```text
Decision
Packet
Artifact
Claim
Evidence
Approval
VerificationRequirement
Assumption
Alternative
Trigger
RiskAcceptance
```

Each node has a stable typed identity, revision, freshness state, and canonical
content digest. Typed dependency edges bind the normative relationships
without relying on labels or display strings. The standard fixture exercises
all 11 node kinds and 23 dependency edges.

The graph rejects:

- duplicate identities carrying different content;
- conflicting bundle replays;
- missing dependency endpoints;
- invalid edge kinds or source/target kinds;
- dependency cycles;
- cross-packet authority, evidence, or label borrowing;
- malformed supersession topology;
- replay state whose digest or journal topology differs from the admitted
  state.

Identical replay is idempotent. Conflict and cycle failures are typed and
deterministic.

## Deterministic validation phases

Validation runs in a fixed order:

```text
structural
cross-packet
authority
required-evidence
freshness
```

The authority and required-evidence matrix is exact for all nine stable
profiles and challenge Tiers 0-3. Evidence must bind the exact packet,
revision, accepted artifact digest, claim set, evidence references, and
reviewer lineage required by policy. Extra, missing, shared-lineage,
self-attested, stale, or cross-packet material does not satisfy the
requirement.

Tier 0 admits deterministic checks. Tier 1 additionally requires a separate
verification invocation and fresh claim-matched evidence. Tier 2 additionally
requires distinct lineage, independently constructed context, and controlled
authority for material side effects. Tier 3 additionally requires a different
provider family or qualified human challenger, named human approval, explicit
risk acceptance, and a verified rollback or recovery path.

Sharing the implementer's prompt, conclusion, or conversation is not
independent verification. Same-lineage degradation is accepted only where the
frozen policy permits a named risk owner, justification, and recorded
override. A missing Tier 3 challenge has no automatic override.

## Freshness and supersession

Every normative dependency participates in freshness:

- packet and artifact content;
- claims and acceptance bindings;
- evidence and verification requirements;
- approvals, assumptions, alternatives, triggers, and risk acceptances;
- authority and lineage bindings.

A change propagates staleness transitively through typed dependency edges.
Propagation is branch-local and idempotent: repeating it does not produce a
second semantic event or mutate an unrelated branch.

Approval, proof, verification, and risk acceptance cannot survive a later
normative mutation. A later verified revision may supersede a prior
`required_missing` terminal result only through an explicit typed supersession
event. Revision parsing removes the exact typed decision suffix before reading
the final revision token, so decision identifiers containing `-r` cannot
corrupt ordering.

## Journal, projection, and persistence

The graph owns one ordered journal with typed entries for:

```text
BundleAdmission
Validation
StalePropagation
Supersession
```

The public projection is derived from typed graph state and excludes local
paths, secrets, personal identifiers, transport metadata, and process-local
details. Canonical serialization recursively orders map keys, preserves
sequence order, and uses the named
`canon-governance-c14n-v1` domain-separated SHA-256 representation.

The persistence schema is `canon-decision-memory-v1`. State is stored at
`.canon/decision-memory/state.json` through a same-directory temporary file,
file synchronization, atomic rename, and parent-directory synchronization.
The stored snapshot contains admitted bundle roots, the exact ordered journal,
and the graph projection needed for deterministic replay.

Reload verifies schema, contract line, node and edge topology, roots, event
ordering, terminal validation, and graph digest. A torn, mismatched, truncated,
or topologically altered snapshot fails closed rather than being accepted as
decision memory.

## Semantic-review and authority boundary

Canon enforces deterministic rules only. Human or model-assisted semantic
review enters as external verification evidence carrying executor identity,
lineage, claims, findings, and evidence references. Canon evaluates whether
the evidence exists and satisfies policy; it does not execute the reviewer.

Every validation result records:

```text
process_invocations = 0
network_invocations = 0
provider_credential_reads = 0
created_evidence = 0
```

Provider-authored and adapter-authored results remain proposals. They cannot
grant authority, approve their own output, publish changes, waive required
verification, or establish completion through self-reporting.

M2b-A added no CLI, one-shot RPC, MCP, outcome ingestion, model routing, code
execution, synthetic reviewer, placeholder, hidden fallback, or
self-attested-success surface.

## TDD and defect evidence

The initial T053 focused command was:

```text
cargo test --test deterministic_governance --all-features
```

It failed with exit 101 because the `decision_memory` module did not exist.
That is the preserved RED observation preceding T054.

Independent review then added a regression for a later verified revision
superseding an earlier `required_missing` result. The test failed before the
revision parser was corrected and passed after the parser was restricted to
the exact typed decision suffix and final revision token.

The completed corpus contains 17 tests covering:

- structural, cross-packet, authority, and exact-evidence validation;
- all nine profiles and their required-evidence bindings;
- deterministic-only and explicit `no_change` cases;
- every normative dependency invalidating a prior decision;
- transitive, idempotent, branch-local stale propagation;
- typed cycles, conflicts, replays, and supersession;
- multiple bundles without cross-branch authority;
- canonical input-order invariance;
- projection privacy;
- atomic snapshot mismatch, tear, and journal-topology rejection;
- history round trip and terminal-state replay;
- zero semantic-execution capability.

## Verification results

Final M2b-A verification:

| Command | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo test --test deterministic_governance --all-features` | PASS, 17 tests |
| `cargo test --workspace --all-features` | PASS |
| `cargo nextest run --workspace --all-features` | PASS; 1,735 passed, 0 skipped; nextest summary 288.066 s |
| `cargo deny check licenses advisories bans sources` | PASS: advisories, bans, licenses, and sources all `ok` |
| Full workspace `cargo llvm-cov` in a dedicated temporary target | PASS; 45,159/47,366 lines, 95.3405% |

Patch intersection covered 2,151 of 2,277 executable changed Rust lines:
94.4664%. The frozen Canon line threshold is 95.34% and the patch threshold is
90%; both gates pass.

The final targeted scans found no panic-prone runtime or test constructs in the
M2b-A scope and no saved absolute local paths. `git diff --check` passed.

## Independent review

Independent review found no remaining Critical or Important defect. Its
revision-ordering regression was first observed RED, then fixed and rerun
green. The review confirmed exact evidence and authority binding,
deterministic replay, fail-closed persistence, zero semantic execution, and no
T055-or-later surface.

## Closure and carried work

T053 and T054 are complete. M2b-A is GO.

The following tasks remain unchecked and were not started:

- T055-T056: stable Canon CLI and one-shot RPC contract and implementation;
- T057-T059: durable Boundline outcome outbox and Canon outcome ingestion;
- every M3-M6 task.

The Speckit adapter remains at
`ce7cb2e57c7ad8bd38064c684a4736029397756c` with no M2b-A change. M2b-A
closure does not push, merge, tag, publish, or alter any remote branch.
