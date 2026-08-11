# Canon Governance and Outcome Synchronization

## Canon stable surfaces

Human CLI:

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

One-shot machine interface:

```text
canon rpc --stdio
```

Historical 0.90 operations:

```text
capabilities
start
refresh
approve
inspect
publish
```

The additive 0.91 inventory appends:

```text
record_outcome
```

The public `canon-contracts 0.91.0` package is qualified from crates.io, and
`boundline-core` owns Boundline's exact `=0.91.0` registry dependency.
Boundline now persists the complete request in a durable fenced outbox and
delivers it through a bounded one-shot subprocess. Canon records it in the
existing atomic decision-memory graph and journal.

The six historical operation names and semantics remain exact. In particular,
`publish` is still a read-only projection operation with an empty payload.
Capability metadata advertises `record_outcome` as available only because its
transactional handler is installed. Direct invocation returns a typed
recorded, replayed, or rejected response and never reports success from a stub.

Stable profiles:

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

## M1A contract-crate boundary

`canon-contracts` freezes exactly the nine profiles above. `implementation` is
not a stable profile and unknown profiles, approval decisions, challenge
tiers, verification kinds, projection kinds, operations, and contract
versions fail closed during decoding. Public structs remain compatible with
unknown additive fields.

The contract distinguishes deterministic verification from external semantic
review evidence. It contains no semantic-review executor, model route, code
execution, workspace mutation, or persistence implementation.

Deterministic verification evidence carries a stable check identity, its bound
claims, a closed pass/fail status, and immutable evidence references.

Stable governance publication is limited to governance-bundle,
decision-memory, and evidence projections. The one-shot contract contains
typed request/response envelopes and the six frozen operation identifiers, but
no RPC dispatcher or CLI implementation.

A governance publication carries the complete typed governance bundle
alongside decision-memory, evidence, and approval projections so packet,
authority, and evidence requirements survive the exchange.

## Authorization phase

Canon deterministically validates packets, references, evidence declarations,
authority, and policy. `canon publish` emits governance, decision-memory, and
evidence projections only.

External semantic review records include reviewer identity, lineage, claims,
findings, context identity, and evidence references. Canon validates whether
the required record exists and satisfies policy; it does not execute the
reviewer.

## Terminal outcome phase

After terminal publication handling, Boundline durably records an outcome event
containing:

- Event identifier and canonical payload digest.
- Governance bundle identifier and digest.
- Session and final transaction revision.
- Published commit and final fingerprint, when present.
- Proof references, deviations, outcome, and terminal claims.

Delivery is idempotent by event identifier and canonical digest. Canon records
the event transactionally and returns the resulting decision-memory revision.

Boundline persists `in_flight` intent before execution, uses a monotonic
fencing token under an exclusive outbox lock, and records every attempt. The
transport permits one request and one response, separates stderr, enforces
timeout and output bounds, and creates no retry daemon or background process.
Finite retries use an injected clock. Lost responses and post-commit crashes
replay the exact event and converge without a second decision event.

Canon validates the envelope identity before idempotency lookup, then performs
exact event/digest replay before allocating a revision. A changed digest is a
conflict. Validation and `OutcomeRecorded` append share the existing atomic
snapshot; fault injection before persistence leaves no event, while a fault
after durable commit replays the one committed event.

Temporary Canon failure leaves the outcome pending and never reverses a
successful Git publication. Permanent rejection remains visible as
`canon_outcome_sync_failed`. Authorized archival preserves the payload and
audit trail and cannot rewrite the publication outcome.

The public 0.91 exchange is frozen in
[`canon-contracts-v1.md`](canon-contracts-v1.md). Its digest domain is
`canon-boundline-outcome-c14n-v1`; the existing envelope remains
`CanonContractVersion::V1`, serialized as `"1.0"`, because the amendment is
additive and capability-discovered.
