# Canon Contracts V1 and the 0.91 Outcome Amendment

## Published package boundary

`canon-contracts 0.91.0` is published on crates.io and source-tagged at
`cd9209a861acf8d2960b185c476fedfdac9e41d2`. Boundline consumes the exact
registry requirement `canon-contracts = "=0.91.0"` through `boundline-core`;
no path, Git, patch, or local-registry source is admitted. Publication freezes
the package bytes and contract surface described below. T099 consumption does
not make `record_outcome` operational and does not begin T057-T059.

## Version boundary

`canon-contracts 0.91.0` adds one public operation and its DTOs while retaining
`CanonContractVersion::V1`, serialized as `"1.0"`. Capability discovery makes
the extension additive. The 0.90 inventory remains exactly
`capabilities`, `start`, `refresh`, `approve`, `inspect`, and `publish`; 0.91
appends `record_outcome`. Existing semantics do not change and `publish`
remains a read-only projection with an empty payload.

The human root inventory remains `init`, `run`, `resume`, `status`, `approve`,
`inspect`, `publish`, `assistant`, and `rpc`.

## Request

`RecordOutcomeRequest` contains:

- `event_id` and `event_digest`;
- `source_product` (`boundline`) and `source_repository_identity`;
- `governance_bundle_id` and `governance_bundle_digest`;
- `session_id` and `final_transaction_revision`;
- `terminal_status`, optional `published_commit`, and optional
  `final_fingerprint`;
- duplicate-free `proof_references`, `deviations`, and `terminal_claims`;
- `authority_binding`, optional `approval_binding`, `challenge_binding`, and
  producer/verifier `lineage`;
- optional `occurred_at` only when authoritative.

The public exchange rejects secrets, tokens, raw prompts, private
conversations, and absolute or process-local paths. Authority, approval,
challenge, evidence, and lineage records bind the exact terminal claims and
final transaction revision. Lineage cannot self-attest independence.

## Terminal matrix

| Status | Commit | Fingerprint | Contract result |
|---|---:|---:|---|
| `published` | required | required | accepted terminal |
| `no_change` | forbidden | required | accepted terminal |
| `failed` | forbidden | optional | accepted terminal |
| `cancelled` | forbidden | optional | accepted terminal |
| `rejected` | forbidden | optional | accepted terminal |
| `blocked` | forbidden | optional | rejected as nonterminal |
| `stale` | forbidden | optional | rejected as nonterminal |

Unknown values fail closed.

## Digest

The event identity is `event_id`; every authoritative field is bound by
`event_digest`. The digest is SHA-256 over the bytes of
`canon-boundline-outcome-c14n-v1`, one zero-byte separator, and canonical JSON.
Object keys are recursively sorted. Semantic sequence order is retained.
Set-like collections are sorted and must be duplicate-free. Only integer JSON
numbers are admitted; floats and duplicate JSON keys fail closed. The decoder
recomputes the digest. The same identity with changed content is an
`identity_digest_conflict`.

## Response

`RecordOutcomeResponse` contains `event_id`, `event_digest`, `disposition`,
optional `decision_memory_revision`, optional `decision_memory_digest`,
optional `reason_code`, and duplicate-free `next_actions`.

- `recorded` and `replayed` require the same durable revision and
  decision-memory digest and forbid a rejection reason.
- `rejected` forbids a revision and decision-memory digest and requires a
  reason.

Stable rejection reasons are `unsupported_contract_line`, `invalid_outcome`,
`nonterminal_outcome`, `identity_digest_conflict`,
`authority_binding_invalid`, `approval_binding_invalid`,
`evidence_binding_invalid`, `lineage_invalid`, `stale_outcome`,
`decision_memory_conflict`, `persistence_failed`, and
`unsupported_operation`.

## Pre-T059 runtime

Capabilities include `record_outcome` with `available: false` and reason
`unsupported_operation`. Direct invocation returns a typed rejected response,
creates no graph revision or snapshot, and cannot return `recorded` or
`replayed`. T059 is the sole task that may atomically promote availability and
install transactional ingestion.

## Rejected alternatives

- Overloading `publish` violates its read-only projection semantics.
- Overloading `start` confuses terminal outcome recording with run admission.
- Overloading `approve` confuses outcome recording with authority approval.
- A private internal wire format violates the stable public integration
  boundary.
- Modifying 0.90.0 violates the immutability of the published artifact and
  signed tag.
