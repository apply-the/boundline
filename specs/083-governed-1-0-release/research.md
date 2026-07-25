# Research: Governed 1.0 Release

## Decision 1: Keep the three product repositories separate

**Decision**: Publish `boundline-protocol` from Boundline and
`canon-contracts` from Canon. The Speckit adapter consumes the immutable
registry package and matching signed release provenance.

**Rationale**: Product ownership remains clear without creating a fourth
repository or copying protocol DTOs.

**Alternatives considered**:

- One monorepo: rejected because the products remain independently releasable.
- Git-only contract dependencies: rejected because an external adapter needs
  immutable SemVer packages.
- Copied DTOs: rejected because they already drifted in the adapter repository.

## Decision 2: Treat local clone identity as the lock domain

**Decision**: Generate an opaque repository identifier per Git common
directory. Linked worktrees share it; separate clones of the same remote do
not. Maintain a distinct marker for the authoritative worktree.

**Rationale**: Remote identity is too broad for local locks and recovery, while
path identity alone is unstable under moves and linked worktrees.

**Alternatives considered**:

- Remote URL hash: rejected because separate clones would collide.
- Absolute path hash: rejected because repository moves would silently change
  identity.

## Decision 3: Use persistent external session worktrees

**Decision**: Keep the real session Git worktree beneath the protected external
state root. Keep only ignored lease and reconciliation metadata in
`.boundline/`.

**Rationale**: Resume must survive restart and reboot, while nested worktrees
would interfere with scans, tooling, and authoritative-workspace invariants.

**Alternatives considered**:

- Temporary directories: rejected because resume is not durable.
- Nested `.boundline/worktrees/` checkouts: rejected because they pollute the
  repository namespace.
- Mutating the authoritative checkout during execution: rejected because it
  mixes resume and publication recovery.

## Decision 4: Serialize execution and publication with fencing

**Decision**: Admit at most one executor per session and one publisher per
local repository. Both leases use monotonic fencing tokens that every later
write must present.

**Rationale**: OS locks alone disappear after process death and do not prevent
an older surviving process from writing after ownership changes.

**Alternatives considered**:

- Transaction revision only: rejected because two processes can read the same
  revision before either writes.
- OS lock only: rejected because recovery state must survive process death.

## Decision 5: Publish a verified commit, never an unexplained dirty result

**Decision**: Build and verify a candidate commit before publication. Under
the repository lock, validate every path, update the worktree and index, and
advance the target ref from the admitted base using compare-and-swap.

**Rationale**: A clean commit is an inspectable terminal result. Leaving
authoritative files dirty would require another stable lifecycle state and
weaken recovery guarantees.

**Alternatives considered**:

- Apply-only publication: retained as preview, not stable.
- Automatic rebase or merge: deferred because it invalidates admitted proof.

## Decision 6: Use durable intent/effect/completion records

**Decision**: Use crash-consistent local transactions with declared flush
semantics. Before publication start, flush the candidate identity, exact
target, complete backup, restore plan, metadata, and fencing token.

**Rationale**: Recovery after restart, orderly reboot, or qualified power loss
requires proof of what was intended before any authoritative effect.

**Alternatives considered**:

- Best-effort JSON journaling: rejected because ordering and partial writes are
  not sufficiently constrained.
- Roll back every post-start crash: rejected because durable completion may
  already represent a successful publication.

## Decision 7: Keep semantic reviewers external to Canon

**Decision**: Canon enforces deterministic rules and validates externally
submitted semantic-review evidence. It does not route models or execute code.

**Rationale**: This preserves Canon as a deterministic kernel and keeps model
execution under Boundline’s capability and authority boundary.

**Alternatives considered**:

- Model execution inside Canon: rejected as a product-boundary violation.
- Declarative self-attestation: rejected because it cannot prove independent
  verification.

## Decision 8: Synchronize terminal outcomes through an outbox

**Decision**: Boundline persists an idempotent terminal outcome event and
retries delivery to Canon. A successful publication is never undone because
Canon is temporarily unavailable.

**Rationale**: Workspace publication and remote governance recording cannot be
one atomic filesystem transaction.

**Alternatives considered**:

- Make Canon availability part of publication: rejected because it couples
  authoritative delivery to an external runtime.
- Best-effort notification: rejected because decision memory could silently
  omit the actual outcome.

## Decision 9: One-shot adapter transport

**Decision**: Stabilize one local subprocess request/response exchange with
strict JSON stdout framing and separate stderr. Stable operations are
`describe`, non-mutating `preflight`, `execute_stage`, and advisory
`emit_hook`.

**Rationale**: It is inspectable, bounded, cross-platform, and sufficient for
the qualified Speckit stages.

**Alternatives considered**:

- Daemon or HTTP transport: deferred because lifecycle, authentication, and
  recovery would enlarge the 1.0 contract.
- Nested delegation and multiple adapters: deferred because they violate the
  sequential first release boundary.

## Decision 10: Current model-catalog delta

**Decision**: M0 refreshes the bundled catalog from first-party sources while
keeping four independent namespaces: OpenAI API, Anthropic API, Gemini API,
and GitHub Copilot. Copilot exposure is not evidence that the same identifier
is available through a direct provider runtime.

**Evidence date**: 2026-07-25.

| Source identity | Namespace | Evidence used |
|---|---|---|
| OpenAI API model catalog | `openai-api` | <https://developers.openai.com/api/docs/models> |
| Anthropic model overview and deprecations | `anthropic-api` | <https://platform.claude.com/docs/en/about-claude/models/overview> and <https://platform.claude.com/docs/en/about-claude/model-deprecations> |
| Google Gemini models and deprecations | `gemini-api` | <https://ai.google.dev/gemini-api/docs/models> and <https://ai.google.dev/gemini-api/docs/deprecations> |
| GitHub Copilot supported-model matrix | `copilot` | <https://docs.github.com/en/copilot/reference/ai-models/supported-models> |

The catalog records `stable`, `preview`, `deprecated`, and `retired`
lifecycles independently from `canonical`, `alias`, and `pinned` identifier
kinds. Retirement dates and replacement identifiers are mandatory for
deprecated and retired entries. An alias is recorded only where the direct
provider publishes alias semantics; dated or otherwise snapshot-stable
Anthropic identifiers are represented as pinned identifiers rather than
invented aliases.

The direct-runtime fallback changes are explicit and independently tested:

| Runtime | Previous first bundled fallback | M0 fallback |
|---|---|---|
| OpenAI API through Codex | `openai/gpt-5.5` | `openai/gpt-5.6-sol` |
| Anthropic API through Claude | `opus-4.7` | `claude-opus-5` |
| Gemini API | `gemini-3.1-pro-preview` | `gemini-3.6-flash` |

The four explicit GitHub Copilot slot defaults remain unchanged:
planning/review use `gpt-5.4`, implementation uses `opus-4.6`, and
verification uses `sonnet-4.6`. Retired entries are retained as lifecycle
evidence but excluded from guided suggestions. Explicit user-pinned routes,
including a retired identifier, remain parseable and are never silently
rewritten.

**Rationale**: Catalog currency and retirement evidence must be namespace
specific. Default changes require reviewable tests, while an existing
user-owned route is configuration evidence rather than permission to migrate
it.

**Alternatives considered**:

- Record no change: rejected because official current families and retirement
  data differ materially from repository defaults.
- Change defaults during transaction work: rejected because catalog refresh
  needs an independent review and compatibility fixture.

## Decision 12: Separate MSRV and current-stable lanes

**Decision**: Keep package `rust-version` at 1.96 where compatible. The MSRV
lane uses the exact `1.96.0` patch release, while a separate CI lane resolves
the current `stable` channel and runs workspace/all-target/all-feature
checking. The M0 machine resolved current stable to Rust 1.97.1 after
refreshing a stale local channel.

**Rationale**: An exact MSRV lane proves the declared floor; a moving stable
lane detects forward incompatibility. Using one label for both would either
silently raise the floor or fail to test the current compiler.

**Alternatives considered**:

- Raise MSRV to current stable: rejected because M0 found no compatibility
  requirement that justifies a breaking toolchain change.
- Leave the local `stable` alias at 1.95.0: rejected as stale evidence; it
  predates the declared 1.96 floor and is not the current channel.

## Decision 11: Explicit Git and filesystem qualification

**Decision**: Support byte-preserving binary files, verified executable modes,
qualified symlinks, UTF-8 paths with collision checks, separate state/repo
volumes, and files up to 512 MiB. Fail closed for affected LFS paths,
sparse checkout, detached HEAD, mutated gitlinks, unsupported symlinks,
ambiguous case/Unicode paths, and larger files.

**Rationale**: Publication safety cannot depend on accidental platform or Git
behavior.

**Alternatives considered**:

- Best-effort support for all Git modes: rejected because recovery could
  overwrite or misrepresent state.
- Text-only publication: rejected because ordinary repositories contain
  binaries and executable files.
