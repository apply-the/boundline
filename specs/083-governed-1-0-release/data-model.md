# Data Model: Governed 1.0 Release

Public protocol models and internal persistence records are intentionally
separate. All stable serialized records are typed and schema-versioned.

## Public protocol entities

### MutationRequestEnvelope

- `contract_line`
- `operation`
- `request_id`
- `canonical_request_digest`
- `expected_state_revision`
- typed operation payload

Lookup order is idempotency record first, then revision validation for a new
request.

### MutationResultEnvelope

- `request_id`
- `canonical_request_digest`
- `previous_revision`
- `resulting_revision`
- typed result or stable reason code
- evidence and trace references

### EvidenceBinding

- `session_id`
- `transaction_revision`
- `accepted_diff_digest`
- `worktree_fingerprint`
- `claim_set`
- `reviewer_lineage`
- evidence references
- freshness status

Any accepted-diff or fingerprint change transitions the binding to `stale`.

### PublicSessionProjection

- session and repository identifiers
- transaction revision
- lifecycle and next actions
- executor capability status
- proof freshness
- publication and recovery status
- repository quarantine status
- Canon outcome synchronization status

## Internal Boundline persistence

### IdempotencyRecord

- key: contract line, operation, and request ID
- canonicalization version and trusted cryptographic request digest
- execution state: `in_flight`, `terminal`, or `nonterminal`
- exact terminal public result, evidence and trace references, and next actions
- resulting state revision when terminal

Lookup is atomic and precedes revision validation. A matching in-flight
request waits for the single admitted execution and receives its exact
terminal result. A completed business rejection is terminal and replayable.
A process-local or incomplete execution failure is retained as explicit
`nonterminal` state and cannot be replayed as success or silently retried.
Stale new requests are rejected before intent admission and create no success
record.

### RepositoryIdentityRecord

- clone-local opaque identifier
- Git common-directory fingerprint
- authoritative-worktree identifier and marker
- schema version
- creation and validation metadata

### WorktreeLeaseRecord

- repository and session identifiers
- external managed-worktree reference
- registered branch/base/HEAD
- lease lifecycle
- retained-state reason
- last validated fingerprint

### SessionExecutionLeaseRecord

- session identifier
- current fencing token
- owner process identity
- expected transaction revision
- acquired, renewed, and termination-confirmed state

Only one lease may be current.

### ExecutorCapabilityGrantRecord

- session, revision, invocation, executor, and fencing identities
- read/write paths
- commands and executable identities
- environment and named-secret allowlists
- network policy
- process-spawn policy
- resource, output, and deadline limits
- requested, admitted, denied, and observed capabilities

### ExecutorInFlightRecord

- invocation and fencing identities
- initial fingerprint
- mutation boundary
- base, HEAD, and index
- tracked/untracked ledger
- executable digest
- lifecycle: `prepared`, `running`, `termination_pending`,
  `no_delta`, `uncommitted_candidate`, `validated`, or `rejected`

The accepted diff advances only from a validated terminal record.

### Fingerprint

- `fingerprint_schema_version`
- repository/base/HEAD/target revisions
- index identity
- normalized path entries
- file type, mode, symlink target, content digest, and size
- rename/delete state
- admitted untracked entries
- explicit exclusion-policy identity

### RepositoryPublicationLockRecord

- repository identifier
- publication identifier
- monotonic fencing token
- owner identity
- expected target branch and base
- state: `tentative`, `started`, `recovery_required`, `completed`,
  `released`, or `quarantined`

### PublicationRecoveryRecord

- candidate commit/tree and target fingerprint
- pre-publication fingerprint and index tree
- affected-path ledger
- backup manifest/content/digest
- restore plan for ref, index, files, modes, symlinks, and deletions
- per-path preconditions and progress
- publication start/completion durability markers
- current fencing token
- recovery decision and audit trail

### CanonOutcomeOutboxRecord

- outcome event identifier and canonical payload digest
- governance bundle identity and digest
- session and final transaction revision
- published commit and fingerprint, if present
- proof references, deviations, outcome, and terminal claims
- delivery attempts and last reason
- Canon decision-memory revision
- state: `pending`, `delivered`, `failed`, or `archived_with_authority`

## Canon entities

### GovernanceBundle

- bundle identity and digest
- profile
- change intent, scope, risks, invariants, and acceptance criteria
- authority and required evidence
- cross-packet references
- decision-memory revision

### ExternalVerificationEvidence

- executor or human identity
- provider/executor lineage
- independently constructed context identity
- claims, findings, and evidence references
- challenge tier and any named override

Canon checks policy satisfaction but does not produce this evidence itself.

### DecisionMemoryNode

- stable node identity and kind
- source packet/evidence/outcome
- revision introduced
- relationships and supersession links
- freshness and stale reason

### PublicationOutcomeProjection

- idempotent event identity and digest
- source governance bundle
- published commit and fingerprint
- proof references
- terminal claims and deviations
- publication status

## Adapter entities

### AdapterDescriptor

- adapter identity, version, executable digest
- supported protocol line and transport
- supported operations and stages
- requested capabilities

### AdapterInvocation

- request identifier and canonical digest
- admitted capability grant
- operation, stage, deadline, and limits
- invocation staging references

### AdapterProposal

- invocation identity
- status and summary
- proposed mutations or artifacts
- evidence and diagnostic references
- requested next actions
- explicit non-authoritative marker

## Lifecycle invariants

1. One session has at most one current execution fencing token.
2. One local repository has at most one current publication fencing token.
3. A stale token cannot mutate journal, index, ref, worktree, or projection.
4. `uncommitted_candidate` cannot become owned without full revalidation.
5. Publication cannot enter `started` until the candidate, backup, restore
   plan, metadata, and fencing state are valid and durably flushed.
6. Durable completion dominates an abandoned OS lock; recovery reconciles it
   as success.
7. Unknown authoritative state is preserved and quarantined.
8. Cleanup cannot remove evidence required by an incomplete Canon outbox.
