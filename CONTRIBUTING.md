# Contributing to synod

## Overview

Synod is a bounded delivery orchestrator. Contributions should keep that bias:

- prefer small, inspectable changes over broad refactors
- preserve explicit session state, traces, and CLI guidance
- preserve the session-native `start -> capture -> plan -> run -> status -> next -> inspect` story as the default operator path
- keep continuity explicit when a workspace moves from session-native state to compatibility-trace follow-up
- treat `synod workflow` as a thin bounded layer over the same session-owned runtime, not as a generic workflow engine
- keep the local developer workflow deterministic
- update tests and docs together with behavior changes

## Prerequisites

- Rust `1.95.0` from [rust-toolchain.toml](rust-toolchain.toml)
- `rustfmt` and `clippy`
- `cargo-nextest` if you want the same test runner used by the repository pre-push hook and blocking CI workflows
- optional but recommended: `cargo-deny`
- `cargo-llvm-cov` if you install the repository pre-push hook

To install the repository git hooks:

```bash
./scripts/install-hooks.sh
```

## Repository Layout

- [src](src): library code, CLI, session-native runtime, and explicit compatibility execution support
- workspace-local `.synod/workflows.toml`: optional named workflow registry compiled onto the existing session-native phases, with optional `summary` and `recommended_when` metadata surfaced by `synod workflow list`
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
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Additional validation when relevant:

```bash
cargo test --workspace --all-features
cargo deny check licenses advisories bans sources
```

After `./scripts/install-hooks.sh`, `pre-commit` runs `cargo fmt --all --
--check`; `pre-push` runs `cargo clippy --workspace --all-targets
--all-features -- -D warnings`, `cargo nextest run --workspace --all-features`, and
`cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
That matches the blocking GitHub lint, test, and coverage workflows.

### Test Layout Notes

- Cargo discovers this repository's nested tests through the top-level harness files [tests/unit.rs](tests/unit.rs), [tests/integration.rs](tests/integration.rs), and [tests/contract.rs](tests/contract.rs).
- If you add a new test module under `tests/unit`, `tests/integration`, or `tests/contract`, you must also register it in the matching top-level harness file.
- Prefer narrow, behavior-scoped tests while iterating, then run the full suite before finalizing the change.

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

If you change `.synod/workflows.toml` semantics or `synod workflow ...` output, keep the docs explicit about workflow discovery guidance, bounded `review`/`govern` follow-through, and unsupported workflow-engine semantics.

If you change governed-stage behavior, keep the docs explicit about which stage now stops for governance, how packet lineage is reused on later stages, and how waiting or blocked guidance appears on both direct session and workflow-aware surfaces.

If you change adaptive compatibility behavior, keep the docs explicit about
bounded mutation-family selection, validation-guided slice reselection,
bounded `read_targets`, candidate credibility and rejection wording, explicit
exhaustion behavior, and the fact that adaptive repair still remains on the
explicit compatibility path in this release.

If you change how `status`, `next`, or `inspect` choose the authoritative follow-up state, keep the docs explicit about `continuity_authority`, inspect-only compatibility follow-up, and when `synod start` is or is not actually required.

If you change how `run`, `status`, `next`, or `inspect` align route-summary wording, keep the docs explicit about `route_owner`, any material `route_config_projection`, and the rule that summary convergence must not hide the real owning route.

If the crate surface or release scope materially changed, update the crate version in [Cargo.toml](Cargo.toml).

## Pull Requests

PRs should make it easy to review the behavioral delta. Include:

- a short summary of what changed and why
- the validation commands you ran
- any updated specs or docs
- any known follow-up work that was intentionally left out

For CLI or trace-surface changes, include representative output snippets when that improves review clarity.

## Versioning

Synod follows Semantic Versioning.

Before `1.0.0`, breaking changes may still land in minor releases, but version bumps should remain intentional and consistent with the user-visible scope of the change.