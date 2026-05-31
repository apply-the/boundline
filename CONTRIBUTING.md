# Contributing to boundline

## Overview

Boundline is a bounded delivery orchestrator. Contributions should keep that bias:

- prefer small, inspectable changes over broad refactors
- preserve explicit session state, traces, and CLI guidance
- preserve the session-native `start -> capture -> plan -> run -> status -> next -> inspect` story as the default operator path
- treat workflow assistant surfaces and direct native commands as the same primary Boundline product story, with compatibility remaining explicit and subordinate
- keep direct `run --goal` native-first and require explicit `--compatibility` for manifest-backed execution
- keep bounded `bug-fix` and `change` completion credible: do not treat the end of a plan as success unless material change evidence and passed validation are both present, or the CLI reports an explicit stop instead
- keep negotiated delivery packets, acceptance boundaries, and blocking constraints explicit from capture through follow-up surfaces
- keep bounded context packs, credibility, and provenance explicit from planning through `run`, `status`, `next`, and `inspect`
- keep Canon capability snapshots, compact Canon-grounded memory, and governed next-action cues explicit whenever governed evidence is reused across planning or follow-up surfaces
- keep evidence-driven plan proposal state, confirmation boundaries, revision lineage, rationale, and verification strategy explicit from `plan` through `run`, `status`, `next`, and `inspect`
- keep runtime capability profiles, slot effort policies, and any resulting delegation packets explicit from `config show` and `plan` through `run`, `status`, `next`, and `inspect`
- keep clustered delivery sequential-first with one authoritative primary workspace session owner
- keep continuity explicit when a workspace moves from session-native state to compatibility-trace follow-up
- treat `boundline workflow` as a thin bounded layer over the same session-owned runtime, not as a generic workflow engine
- keep the local developer workflow deterministic
- update tests and docs together with behavior changes

## Prerequisites

- Rust `1.96.0` from [rust-toolchain.toml](rust-toolchain.toml)
- `rustfmt` and `clippy`
- `cargo-nextest` if you want the same test runner used by the repository pre-push hook and blocking CI workflows
- optional but recommended: `cargo-deny`
- `cargo-llvm-cov` if you install the repository pre-push hook
- `cargo-cyclonedx` if you want to generate the same CycloneDX SBOM artifacts used by the dedicated SBOM workflow

To install the repository git hooks:

```bash
./scripts/install-hooks.sh
```

## Repository Layout

- [src](src): library code, CLI, session-native runtime, and explicit compatibility execution support
- workspace-local `.boundline/workflows.toml`: optional named workflow registry compiled onto the existing session-native phases, with optional `summary` and `recommended_when` metadata surfaced by `boundline workflow list`
- [tests](tests): top-level Cargo test harnesses plus `unit`, `integration`, and `contract` modules
- [assistant](assistant): assistant command packs and shared assistant-facing docs
- [specs](specs): feature specs, plans, research notes, contracts, quickstarts, and task breakdowns
- [AGENTS.md](AGENTS.md): auto-generated development guidance derived from feature plans

## Working Style

### Small changes

For a bug fix or small improvement:

1. update the smallest possible code surface
2. add or adjust the nearest test coverage
3. update docs if the CLI, workflow, or operator guidance changes

### Feature-sized changes

For a non-trivial feature, work through the existing spec flow under [specs](specs):

1. create or update the numbered feature directory
2. keep `spec.md`, `plan.md`, and `tasks.md` aligned
3. implement in dependency order
4. close the loop with docs and validation

Avoid unrelated refactors while landing feature work unless the refactor is required to make the change safe or testable.

## Testing and Validation

Run these commands from the repository root before opening a PR:

```bash
sh scripts/check-no-local-paths.sh --tracked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
sh scripts/check-rust-no-panic.sh
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Additional validation when relevant:

```bash
cargo test --workspace --all-features
cargo deny check licenses advisories bans sources
```

After `./scripts/install-hooks.sh`, `pre-commit` runs `sh scripts/check-no-local-paths.sh --cached`
and `cargo fmt --all -- --check`; `pre-push` runs `sh scripts/check-no-local-paths.sh --tracked`,
`sh scripts/check-rust-no-panic.sh`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
That matches the blocking GitHub lint, test, and coverage workflows.

For the dedicated SBOM workflow, the matching local command is:

```bash
cargo install cargo-cyclonedx --locked
cargo cyclonedx --manifest-path Cargo.toml --format json --all-features --target all --override-filename workspace-sbom
```

## Reporting Issues

Use the GitHub issue templates in `.github/ISSUE_TEMPLATE/` so reports land
with the right structure:

- `bug-report.yaml` for reproducible defects in the CLI, session-native runtime, workflow surfaces, governed delivery flow, release surfaces, or assistant command packs
- `documentation.yaml` for README, guides, command-pack docs, examples, and contributor-doc issues
- `feature-request.md` for bounded product proposals
- `issue.md` only when the other templates do not fit

If the report is a vulnerability, do not open a public exploit report. Follow
`SECURITY.md` instead.

### Test Layout Notes

- Cargo discovers this repository's nested tests through the top-level harness files [tests/unit.rs](tests/unit.rs), [tests/integration.rs](tests/integration.rs), and [tests/contract.rs](tests/contract.rs).
- If you add a new test module under `tests/unit`, `tests/integration`, or `tests/contract`, you must also register it in the matching top-level harness file.
- Prefer narrow, behavior-scoped tests while iterating, then run the full suite before finalizing the change.

### Test Isolation: Process-Global State

Rust test binaries run cases in parallel within the same process.
Any test that mutates process-global state (environment variables, current
directory) must hold the appropriate shared lock for the duration of the
mutation:

| Shared resource | Lock | Location |
|-----------------|------|----------|
| Provider env vars (`OPENAI_API_KEY_ENV`, `ANTHROPIC_*`, `GITHUB_*`, etc.) | `SHARED_ENV_LOCK` | `src/adapters.rs` (root crate) and `crates/boundline-adapters/src/adapters.rs` |
| Config-path env vars (`XDG_CONFIG_HOME`, `HOME`) | Same `SHARED_ENV_LOCK` | Same locations |
| Current directory and `PWD` | `acquire_process_state_lock()` | `crates/boundline-cli/src/test_support.rs` |

Rules:

1. **One lock per binary for env mutations.** All source test modules compiled
   into the same library test binary must use `SHARED_ENV_LOCK` for any
   `std::env::set_var` / `std::env::remove_var` call. Do not introduce a local
   `static ENV_LOCK`; that serializes only within one module while other modules
   run concurrently.
2. **Poison-safe acquisition.** Always acquire with
   `.unwrap_or_else(|poisoned| poisoned.into_inner())` so a panic in one test
   does not cascade mutex-poison failures into unrelated tests.
3. **Save and restore.** Capture the prior value before mutation, restore it
   in a drop guard or explicit cleanup, and keep the lock held across both.
4. **Scope the lock broadly.** The lock must be held from before the first
   mutation through after the last assertion that depends on the mutated state.

## Docs Expectations

If you change a user-visible command, session workflow, or flow behavior, update the relevant docs in the same change. Common files include:

- [README.md](README.md)
- [docs/getting-started.md](docs/getting-started.md)
- [docs/configuration.md](docs/configuration.md)
- [assistant/README.md](assistant/README.md)
- [ROADMAP.md](ROADMAP.md)
- the relevant assistant command pack files under [assistant](assistant)
- the relevant feature quickstart under [specs](specs)

When a change affects routing, planning, or compatibility behavior, keep the docs explicit about which path is primary and which path is compatibility-only.

If you change `.boundline/workflows.toml` semantics or `boundline workflow ...` output, keep the docs explicit about workflow discovery guidance, bounded `review`/`govern` follow-through, and unsupported workflow-engine semantics.

If you change governed-stage behavior, keep the docs explicit about which stage now stops for governance, how packet lineage is reused on later stages, and how waiting or blocked guidance appears on both direct session and workflow-aware surfaces.

If you change Canon-grounded planning or governed follow-through, keep the docs explicit about capability snapshots, compact Canon memory, `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, `context_staleness_reason`, and `governance_next_action`, including when those fields are projected from persisted task context rather than an active goal plan.

If you change adaptive compatibility behavior, keep the docs explicit about
bounded mutation-family selection, validation-guided slice reselection,
bounded `read_targets`, candidate credibility and rejection wording, explicit
exhaustion behavior, and the fact that adaptive repair still remains on the
explicit compatibility path in this release.

If you change how `status`, `next`, or `inspect` choose the authoritative follow-up state, keep the docs explicit about `continuity_authority`, inspect-only compatibility follow-up, and when `boundline start` is or is not actually required.

If you change how `run`, `status`, `next`, or `inspect` align route-summary wording, keep the docs explicit about `route_owner`, any material `route_config_projection`, persisted `effective_routing`, `assistant_bindings`, `runtime_capabilities`, `slot_effort_policies`, `follow_through_guidance`, `follow_through_evidence_source`, and the rule that summary convergence must not hide the real owning route, continuity authority, or any explicit delegation boundary.

If you change selector-driven decision execution, keep the docs explicit about selector kind, selector rationale, evidence basis, verification intent, and explicit ask, replan, or stop outcomes on `run`, `status`, `next`, and `inspect`.

If you change negotiated capture, plan gating, or acceptance-boundary projection, keep the docs explicit about `negotiation_goal_summary`, `negotiation_resolution`, `negotiation_acceptance_boundary`, and whether the follow-up story comes from the native goal-plan route or an explicit compatibility trace.

If you change context assembly or plan gating, keep the docs explicit about
`context_summary`, `context_credibility`, `context_primary_inputs`,
`context_provenance`, `context_staleness_reason`, the evidence anchors that
admit primary inputs, and whether planning can continue or must stop
explicitly.

If you change dynamic planning, keep the docs explicit about
`goal_plan_state`, `goal_plan_revision`, `planning_rationale`,
`verification_strategy`, bounded proposal supersession, and when
`boundline plan --confirm` is required before native execution can continue.

If you change clustered delivery behavior, keep the docs explicit about the
primary workspace remaining authoritative, member-local trace persistence,
cluster participation or blocking cues, and the requirement that `--cluster
<primary-workspace>` stays a bounded sequential entrypoint rather than hidden
fan-out.

If the crate surface or release scope materially changed, update the crate version in [Cargo.toml](Cargo.toml).

## Pull Requests

PRs should make it easy to review the behavioral delta. Include:

- a short summary of what changed and why
- the validation commands you ran
- any updated specs or docs
- any known follow-up work that was intentionally left out

For CLI or trace-surface changes, include representative output snippets when that improves review clarity.

GitHub pre-fills `.github/PULL_REQUEST_TEMPLATE.md` for new pull requests.
Keep it accurate and include the exact validation you ran for the touched
surfaces.

## Code of Conduct

Participation in Boundline project spaces is governed by
`.github/CODE_OF_CONDUCT.md`.

## Versioning

Boundline follows Semantic Versioning.

Before `1.0.0`, breaking changes may still land in minor releases, but version bumps should remain intentional and consistent with the user-visible scope of the change.

When advancing the version or the supported Canon companion target, follow the checklist in
[docs/release-checklist.md](docs/release-checklist.md), which lists every file
that must change and the two contract tests that enforce alignment.