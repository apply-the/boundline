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

Supported operations:

```text
capabilities
start
refresh
approve
inspect
publish
```

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

Temporary Canon failure leaves the outcome pending and never reverses a
successful Git publication. Permanent rejection remains visible as
`canon_outcome_sync_failed`. Authorized archival preserves the payload and
audit trail and cannot rewrite the publication outcome.
