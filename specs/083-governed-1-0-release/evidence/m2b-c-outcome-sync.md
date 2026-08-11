# M2b-C Outcome Synchronization Evidence

**Decision date**: 2026-08-11  
**Scope**: T057, T058, and T059 only

## Repository and contract identity

The work began from clean local and remote feature-branch tips:

| Repository | Starting commit |
| --- | --- |
| Boundline | `3b99e3353e4b1ccaa01686fd13ca1a26ae9d9b3b` |
| Canon | `1a1aff9c6e93ea395a77ed14fd3f466694a28f1e` |
| Speckit adapter | `ce7cb2e57c7ad8bd38064c684a4736029397756c` |

The implementation consumes immutable `canon-contracts 0.91.0`. Its source
tag still targets `cd9209a861acf8d2960b185c476fedfdac9e41d2`; the retrieved
registry artifact SHA-256 is
`ec789921edf1ac39028cfb68107d5e09d425d4efab95b13292102703aca155ed`.
The signed tag verifies with key
`16A8F86BAA21130A2657B41B25C2C7D21E91FD50`. Diffing the tag against
`crates/canon-contracts` reports no change. Boundline's published
`boundline-protocol 0.90.0` source is unchanged.

## T057 test-first evidence

The first focused run used:

```text
cargo test --test canon_outcome_sync --all-features
```

It failed at compile time because `boundline_core::publication` did not exist.
This was the intended RED state and was preserved in commit `b7f4460`. The
contract then grew only to cover implementation findings; the response-
identity durability test was also observed RED with `ResponseMismatch` before
the minimal T058 correction.

The final suite covers:

- all five accepted terminal statuses: `published`, `no_change`, `failed`,
  `cancelled`, and `rejected`;
- rejection of `blocked` and `stale` as nonterminal;
- enqueue-before-delivery, restart, acknowledgement, and evidence retention;
- exact duplicate enqueue and Canon replay;
- changed local payload and Canon identity/digest conflict;
- unavailable process, early exit, request-write, response-read, timeout, and
  process-exit failures;
- Canon persistence failure, response loss after Canon commit, and Boundline
  acknowledgement loss;
- permanent contract, authority, approval, evidence, lineage, freshness, and
  decision-memory rejections;
- concurrent delivery ownership and monotonic fencing;
- authorized archival after synchronization and permanent rejection;
- a real one-shot Canon subprocess, restart, exact replay, and exactly one
  `outcome_recorded` journal event.

## T058 durable Boundline outbox

The internal state machine is:

```text
pending -> in_flight -> synchronized
                     -> retry_scheduled -> in_flight
                     -> permanent_rejected
                     -> conflict
synchronized | permanent_rejected -> archived
```

`enqueue_terminal_outcome` validates and durably writes the complete frozen
request before transport. A repository-state-root lock grants one active
delivery owner; each admitted attempt increments and durably writes a
monotonic fencing token and `in_flight` state before the subprocess starts.
Every call makes at most one transport attempt. There is no retry daemon or
background process.

Transient transport and `persistence_failed` responses use finite exponential
delays from an injected clock: 1 second initially, capped at 60 seconds, with
five attempts maximum. Contract and policy rejections are permanent;
`identity_digest_conflict` is a distinct conflict state. A response failing
contract or identity validation is durably classified as `response_read` and
scheduled for retry rather than being stranded in flight.

The subprocess transport accepts explicit executable and state roots, writes
one JSON request, reads one framed response, keeps stderr separate, enforces
the frozen request limit plus configured output and time limits, and kills and
waits for the direct child on interruption. It persists no process-local path
or diagnostic string in the outbox. This Canon handler starts no descendants;
general executor process-tree ownership remains outside T057-T059.

After a valid Canon result, Boundline durably stores the exact response,
revision, decision digest, and attempt history. Response or acknowledgement
loss leaves `in_flight`; explicit exact retry obtains Canon's `replayed`
response and converges. Archival requires a named actor, reason, and time,
retains the complete payload and evidence, and never rewrites the publication
outcome.

## T059 transactional Canon ingestion

T059 is the only change that promotes capability `record_outcome` from
`available: false` to `available: true`. The public request, response,
operation inventory, reason codes, and canonical digest remain exactly the
published 0.91 contract. The historical six operations are unchanged.

Canon applies the following 18 deterministic phases:

1. contract version;
2. operation;
3. request identity;
4. canonical digest;
5. terminal status;
6. repository binding;
7. governance-bundle binding;
8. transaction revision;
9. authority;
10. approval;
11. challenge;
12. evidence;
13. lineage;
14. freshness;
15. idempotency;
16. decision-memory event application;
17. atomic persistence;
18. response projection.

Envelope identity is checked before idempotency lookup. Exact event/digest
replay returns the original revision and decision digest without rewriting the
snapshot. Changed content under the identity returns
`identity_digest_conflict`. A new accepted event appends one boxed
`OutcomeRecorded` event containing the complete request, assigned revision,
decision digest, full phase trace, and zero process, network, provider-
credential, and evidence-creation counters.

The event, graph revision, journal, terminal projection, and digest use the
existing `.canon/decision-memory/state.json` atomic persistence boundary. No
second outcome store exists. Failures before persistence create no accepted
event. A deliberately lost response after durable commit leaves the one event
intact, and exact retry returns `replayed` for that same revision.

## Fault matrix

| Boundary | Durable result | Retry result |
| --- | --- | --- |
| before Boundline enqueue | no transport, no record | caller may enqueue |
| after enqueue, before attempt | `pending` | one fenced attempt |
| after durable `in_flight`, before process | `retry_scheduled` after observed failure | exact request |
| request write / early exit / timeout | attempt and stable reason retained | finite explicit retry |
| malformed or mismatched response | `response_read`, retry time retained | exact request |
| Canon before mutation | no outcome event, `persistence_failed` | exact request may record |
| Canon after journal, before persist | no durable outcome event | exact request may record |
| Canon at persistence boundary | no accepted event | exact request may record |
| Canon after durable commit, before response | exactly one event | `replayed`, same revision/digest |
| Boundline before acknowledgement | Canon event exists; outbox `in_flight` | `replayed`, then synchronized |
| after Boundline acknowledgement | full response and attempt durable | no transport |
| archive write | complete record and evidence retained | terminal archive projection |

## Qualification and review

All commands below ran from the named repository on 2026-08-11 after the final
test changes.

### Canon

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass |
| `cargo test -p canon-contracts --all-features` | pass: 8 base contract tests and 18 outcome-contract tests |
| `cargo test --test stable_surface --all-features` | pass: 16 |
| `cargo test --workspace --all-features` | pass |
| `cargo nextest run --workspace --all-features` | pass: 1,791 passed, 0 skipped, 410.720 seconds |
| `cargo deny check licenses advisories bans sources` | pass: all four checks `ok` |
| `bash scripts/check-rust-no-panic.sh` | pass |
| fresh `bash scripts/coverage.sh` | 46,920 / 49,210 = 95.346474% |
| changed executable lines from the M2b-C start, intersected with fresh LCOV | 580 / 605 = 95.867769% |

The reproduced pre-correction coverage was 46,353 / 48,666 = 95.247195%.
Meaningful RPC and ingestion tests increased the covered-line count by 567;
they exercise typed decoding, identity, rejection, persistence, output-write,
governance-binding, and fault-injection paths in the already signed source.
No production source, exclusion, threshold, package artifact, release commit,
or tag was changed to obtain the result.

### Boundline

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass |
| real-Canon `cargo test --test canon_outcome_sync --all-features` | pass: 17 |
| `cargo test --workspace --all-features` | pass |
| `cargo nextest run --workspace --all-features` | pass: 2,739 passed, 2 declared skips, 115.258 seconds on the final confirmation run |
| `cargo deny check licenses advisories bans sources` | pass; existing allowed duplicate-version warnings remain visible |
| `bash scripts/check-rust-no-panic.sh` | pass |
| `git diff --check` | pass |
| `npm run docs:build` | pass; existing Vite chunk-size advisory remains visible |
| official aggregate `bash scripts/coverage.sh` with the real Canon test binary | 104,746 / 111,222 = 94.177411% |
| changed executable lines from the M2b-C start, intersected with aggregate LCOV | 411 / 452 = 90.929204% |

The first aggregate coverage run omitted `BOUNDLINE_CANON_TEST_BINARY`; the
real cross-repository test therefore performed its declared no-binary skip and
the report produced only 61.725664% patch coverage. Re-running the official
script with the qualified Canon binary exercised the production subprocess
transport. Additional meaningful boundary tests for finite retry exhaustion,
early retry, archive authority, persistence tampering, framing limits, process
exit, and timeout raised the final patch result above the frozen 90% gate.
No coverage file, production exclusion, or threshold was changed.

### Review findings and resolutions

Specification review found no scope expansion: Boundline owns only the outbox
and transport, Canon owns only outcome validation/ingestion and capability
promotion, and the adapter plus T021-T048 and T060+ remain untouched.

Safety review found one Important identity-ordering issue: Canon could replay
an existing digest before rejecting a mismatched envelope `request_id`.
Commit `3939484` moves identity validation before the lookup and the direct RPC
test proves the rejection does not rewrite state.

Persistence review found one Important response-validation issue: a typed but
mismatched Canon response returned a process-local error while leaving the
durable outbox `in_flight`. Commit `df500a9` adds a failing contract test and
classifies the boundary durably as retryable `response_read`. No Critical or
Important finding remains open.

Privacy review confirms that portable records contain contract identities,
digests, claims, and evidence references only. Transport diagnostic text,
secrets, environment, raw prompts, private conversations, and local paths are
not persisted. Canon event counters prove zero semantic reviewer, model,
provider, network, credential, or evidence-creation execution.

The final fault and coverage review added no production correction. The
coverage-only additions exercise existing behavior and preserve the T058/T059
architecture. The prior two Important findings are resolved, and no Critical
or Important finding remains open.

### Verified implementation checkpoints

Boundline checkpoints through the final test gate are `b7f4460` (T057 RED),
`4c014ab` and `4b95e0e` (outbox and modularization), `461f82b` (initial
qualification), `df500a9` (durable mismatched-response handling), and
`4f92522` (final boundary and coverage qualification). Canon checkpoints are
`0942856` (transactional ingestion), `3939484` (identity-before-replay), and
`b4ca8d8` (final ingestion-boundary qualification). Documentation commits are
kept separate from these implementation and test checkpoints.

## Task decision

- T057: complete.
- T058: complete.
- T059: complete.

No later roadmap task was started by this checkpoint.
