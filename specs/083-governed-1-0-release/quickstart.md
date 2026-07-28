# Quickstart: Governed 1.0 Release

This feature is delivered milestone by milestone. Do not run Boundline itself
against the Boundline repository root; use isolated fixture repositories for
all mutation and publication tests.

## 1. Confirm aligned feature branches

In each of `boundline`, `canon`, and `boundline-adapter-speckit`:

```bash
git branch --show-current
git status --short
```

Expected branch:

```text
083-governed-1-0-release
```

## 2. Establish M0 before implementation

Record exact commits, toolchain output, lockfile digests, platform and
filesystem properties, and all command results. No green baseline is assumed.

Required repository checks:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo nextest run
cargo deny check licenses advisories bans sources
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Run the repository patch-coverage helper against the admitted diff. Freeze the
line threshold at the greater of 80% or the measured M0 baseline and patch
coverage at 90% before 0.95.

## 3. Implement in milestone order

```text
M0 -> M1 -> M2a
             ├── M2b ─┐
             ├── M3  ─┼── M5 -> M6
             └── M4  ─┘
```

M2b, M3, and M4 may proceed in parallel only after their frozen M1 contract
dependencies exist.

## 4. Exercise the M2a Canon boundary

Canon admits exactly these stable profiles, in this order:

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

`implementation` and the earlier extended mode taxonomy are not stable
admission aliases. Historical records remain inspectable only through the
explicit legacy boundary. `change` governs intent and constraints; it does not
execute a repository mutation.

Use `canon verify --run <RUN_ID>` only as a deterministic inspection of the
persisted evidence projection. It reports structural validity separately from
`external_semantic_judgment: required_missing` and exits nonzero while required
external evidence is absent. Canon does not run a semantic reviewer, invoke a
model, or manufacture review evidence. MCP transport and handlers remain
absent until their later owner task.

The complete M2a acceptance record is
[`evidence/m2a-canon-profiles-verification.md`](evidence/m2a-canon-profiles-verification.md).

## 5. Keep M1C help honest

At 0.90, stable help and completion metadata list only StableOperational
commands: commands with a parser, real operational handler, fail-closed
behavior, contract tests, and matching documentation. `StableTargetPending`
is a planning/inventory state only and is never a public parser or runtime
classification.

The 1.0 target inventory may reserve `approve`, `session abort`, `session
cleanup`, recovery commands, `rpc`, and `serve --transport mcp-stdio`, but
M1C must leave each hidden until its owner promotes it atomically with the
real handler. Do not accept a parser-only command, placeholder handler, or
generic not-yet-implemented response as a stable command. M1C removes legacy
lifecycle entrypoints without aliases, keeps preview commands visibly preview,
and keeps internal commands hidden.

Use the consolidated lifecycle spellings:

```text
boundline run --until [intent]
boundline run --one-step
boundline run --resume
```

Use structured `next_actions` from `boundline status` instead of `next`.
Use `boundline doctor` or `boundline status` instead of `probe` and
`help-next`; use `boundline plan` or `boundline run` instead of `govern`.

Preview capabilities are available only through the explicit gateway:

```text
boundline preview flow ...
boundline preview workflow ...
boundline preview cluster ...
boundline preview council ...
boundline preview evals ...
boundline preview override ...
boundline preview trace ...
```

The preview gateway is intentionally absent from stable root help and stable
completion metadata. Use `boundline preview --help` to inspect it.

## 6. Inspect and exercise the M1D bridge

M1D adds repository-local library APIs; it does not add a stable CLI command:

```text
inspect_legacy_state(source)
plan_migration(inspection)
apply_migration(plan)
recover_migration(source)
```

Only Boundline 0.82.0 and Canon 0.72.6 are admitted. Inspection and planning
are byte-preserving and create no lock, backup, staging, journal, or completion
record. Apply binds the plan to the inspected source digest, creates and
verifies a durable backup, builds and verifies a separate staged tree, then
performs the journaled same-filesystem replacement. Recovery proceeds only
when the matching journal explains the state and no live process owns the OS
lock.

Use the compact historical fixture trees under each repository's
`tests/fixtures/migration/` directory. Their `PROVENANCE.toml` files bind the
fixture digest to the exact historical tag and record that the fixtures were
hand-authored from the tagged types, never serialized by the new migrator.
Active, partially mutated, or semantically ambiguous pre-0.90 state is archived
read-only and requires a new admitted session or run; it is never resumed.

Run the bridge and forced-termination matrix with:

```bash
cargo test --test bridge_090 --all-features
```

## 7. Verify the M1E package candidates

T019 aligns the Boundline and Canon workspaces to `0.90.0`, qualifies exact
local candidates for `boundline-protocol` and `canon-contracts`, and replaces
the adapter's full-runtime `0.66.0` Git bridge with:

```toml
boundline-protocol = "=0.90.0"
```

The adapter's separately tested host range is `>=0.90.0,<1.0.0`; versions
outside that range fail preflight closed. Registry-shaped local consumers use
only exact package requirements and an isolated Cargo home, with no path, Git,
workspace, or patch source. This is package-readiness evidence, not proof of
public-registry publication.

T020 is `READY_TO_PUBLISH` but incomplete. A separately approved run must
publish the exact reviewed contract artifacts, retrieve and test them from the
real registry, create annotated cryptographically signed `0.90.0` tags at the
reviewed source commits, verify the signatures, and push without force.

## 8. Exercise mutation in disposable fixtures

Each transaction fixture must create:

- A clean authoritative Git checkout.
- A separate managed session worktree.
- A protected external state root.
- An admitted mutation boundary.
- A deterministic executor or controlled test executor.

At minimum, prove:

```text
same request ID + same payload -> recorded result
same request ID + different payload -> idempotency_conflict
second executor -> rejected
stale fencing token -> rejected
surviving child -> reconciliation waits
changed crash state -> uncommitted_candidate
```

## 9. Exercise publication recovery

Inject termination:

```text
before durable publication-start
after publication-start and before completion
after durable completion and before lock release
between each per-path validation, replacement, index update, and ref update
```

Every case must complete, restore the prior authoritative state, or preserve
unexplained state without destructive action.

## 10. Qualify cross-repository completion

The RC vertical slice must:

1. Produce a Canon governance bundle.
2. Admit and execute a change in a persistent Boundline session worktree.
3. Exercise Tier 2 or Tier 3 independent challenge.
4. Publish a verified commit into a clean authoritative checkout.
5. Persist the terminal Canon outcome event.
6. Demonstrate idempotent retry during temporary Canon unavailability.
7. Verify exactly one decision-memory outcome.
8. Compare normalized CLI, JSON-RPC, MCP, and five host-pack projections.

## 11. Freeze the RC

After `1.0.0-rc.1`, accept only corrections required to satisfy an already
frozen stable contract. Run the complete Linux, macOS, Windows, migration,
fault-injection, coverage, dependency, packaging, and host-compatibility
matrices through the minimum two-week soak.
