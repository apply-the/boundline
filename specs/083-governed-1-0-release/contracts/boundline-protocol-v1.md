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

## Stable Boundline command groups

Primary:

```text
init
goal
plan
run
approve
status
inspect
recover inspect
recover complete
recover restore
doctor
rpc
serve --transport mcp-stdio
```

Administrative:

```text
config
models
provider
adapter
index
session
assistant
update
session abort <id>
session cleanup <id>
recover abandon --publication <id> --confirm <id>
```

## Semantic projection equivalence

CLI, JSON-RPC, MCP, and host packs project the same authoritative typed
result. Comparison normalizes only declared timestamps, transport metadata,
and invocation identifiers. No transport may infer completion or authority
that is absent from the source result.
