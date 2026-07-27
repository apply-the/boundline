# Boundline Protocol V1

## Stable request processing

Mutation operations use a typed request envelope containing contract line,
operation, request identifier, canonical payload digest, and expected state
revision.

Processing order:

1. Look up the idempotency record by contract line, operation, and request ID.
2. Return its recorded result when the canonical digest matches.
3. Return `idempotency_conflict` when the digest differs.
4. For a new identifier, validate the expected revision.
5. Persist intent before effects and completion after validation.

Read-only requests are not promised mutation-style replay semantics.
Completed accepted results and completed business rejections are replayed
exactly. An execution failure before terminal commit remains explicitly
nonterminal: it is never reported as success and is not silently retried.

## M1A contract correction and crate boundary

M1B found that M1A's ordinary `serde_json` conversion was not sufficient to
support its canonicalization claim. This narrow correction keeps the existing
public `canonical_json` boundary, rejects floating-point and duplicate-key
ambiguity in one serialization pass, and moves the named mutation preimage,
version, and cryptographic digest into Boundline core. No unrelated stable
protocol DTO is added.

`boundline-protocol` serializes typed public DTOs only. Public structs accept
unknown additive fields so a V1 consumer can read a compatible extended
payload. Frozen schema-version, authority, status, and reason-code enums reject
unknown values rather than granting authority or inferring success.

Canonical JSON V1 recursively orders object keys, preserves sequence order,
uses Serde's declared JSON representation for nulls, booleans, strings,
integers, enums, and optional fields, and emits compact JSON. Floating-point
values are outside the mutation contract and fail before digesting or
mutation.

The canonical mutation preimage includes protocol version, contract line,
operation, request ID, expected state revision, and the complete typed
payload. `canonical_request_digest` is the sole excluded envelope field
because a digest cannot be part of its own preimage. The digest record carries
the named canonicalization version separately and applies a versioned domain
separator before cryptographic hashing.

The M1B reference coordinator uses the focused `sha2` dependency and records
its lowercase SHA-256
`sha256:<hex>` value together with `canonical_json_v1`. It provides atomic
process-local admission and completion semantics; crash-consistent SQLite
persistence remains owned by the later durability milestone and is not
claimed here.

The M1A adapter surface is descriptor-only: executable identity, transport,
operation and stage identifiers, requested capabilities, route lineage, and
proposal-only results. FrameworkAdapterV1 framing, invocation, and runtime
operation types remain assigned to T060/T061.

The crate does not expose repository identity, worktree lease, transaction
ownership, execution lease, capability grant, executor-in-flight, publication
lock, publication recovery, or Canon outbox persistence records.

## Stable reason codes

The 1.0 line includes typed reason codes for:

```text
idempotency_conflict
state_revision_mismatch
executor_already_in_flight
executor_fencing_token_stale
executor_termination_unconfirmed
executor_capability_denied
executor_boundary_violated
uncommitted_candidate_requires_validation
approval_stale
proof_stale
repository_identity_mismatch
authoritative_worktree_dirty
publication_lock_held
publication_rebase_required
publication_precondition_failed
publication_recovery_required
repository_quarantined
canon_outcome_sync_pending
canon_outcome_sync_failed
unsupported_git_or_filesystem_state
```

## Boundline command inventory and promotion

`StableOperational` is the only public stable command state. It requires a
parser, real operational handler, fail-closed behavior, contract tests, help,
completion metadata, and documentation. Stable help contains exactly the
StableOperational surface for the current release milestone.

`StableTargetPending` is planning and inventory state only. It is not a
runtime command classification and must not appear in parser registration,
stable help, completion metadata, or operational compatibility claims. Its
owning implementation task promotes it atomically only after the real handler,
authority and failure semantics, contract tests, command tree, help,
completions, and documentation are complete. A placeholder or generic
not-yet-implemented result cannot satisfy this rule.

Current M1C StableOperational commands:

```text
init
goal
plan
run
status
inspect
doctor
config
models
provider
adapter
index
session
assistant
update
```

Documented 1.0 StableTargetPending commands and owners:

| Command | Owner |
|---|---|
| `approve` | T037 |
| `session abort <id>` | T037 |
| `session cleanup <id>` | T037 |
| `recover inspect` | T047 and T048 |
| `recover complete` | T047 and T048 |
| `recover restore` | T047 and T048 |
| `recover abandon --publication <id> --confirm <id>` | T047 and T048 |
| `rpc` | T095 |
| `serve --transport mcp-stdio` | T096 |

The 0.90 StableOperational/help surface is an additive subset of this final
stable target inventory. The wider 0.90 command tree also contains explicit
Preview and hidden Internal commands, so it is not itself a subset of the
stable target inventory. It removes overlapping legacy lifecycle entrypoints
immediately without compatibility aliases. Later 0.90/0.95 work may add a
reserved command only when its real implementation is complete. No
StableOperational command may be removed after the 0.95 contract freeze;
preview commands remain outside the 1.0 compatibility promise.

## Semantic projection equivalence

CLI, JSON-RPC, MCP, and host packs project the same authoritative typed
result. Comparison normalizes only declared timestamps, transport metadata,
and invocation identifiers. No transport may infer completion or authority
that is absent from the source result.
