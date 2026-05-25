# Implementation Plan: Distribution & Bundling

**Branch**: `039-distribution-bundling` | **Date**: 2026-05-03 | **Spec**: [/Users/rt/workspace/boundline/specs/039-distribution-bundling/spec.md](/Users/rt/workspace/boundline/specs/039-distribution-bundling/spec.md)
**Input**: Feature specification from `/specs/039-distribution-bundling/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Ship Boundline `0.39.0` as a real end-user install surface instead of a source-only
tool by adding repo-managed Homebrew and winget distribution metadata,
release-bundle automation that carries a compatible Canon companion, install or
repair verification through the existing `doctor` story, and a documentation
split between a brutal quick path and a separate advanced architecture layer
that keeps Boundline and Canon clearly separated. Keep the product session-native,
make unsupported or partial distribution states explicit, and close the slice
with release docs, roadmap updates, assistant guidance, and >95% coverage for
all modified Rust files.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024 for the CLI plus repository-managed shell scripts, YAML manifests, and GitHub Actions workflows for release packaging  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`; no new Rust runtime dependencies planned for the first slice  
**Storage**: Existing workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, plus new repository-managed distribution metadata under `distribution/` and release automation in `.github/workflows/`  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, integration, and contract tests for distribution diagnostics plus release metadata, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS and Windows end-user install surfaces, plus Linux/macOS/Windows CI builders for release assets and validation  
**Project Type**: Single Rust CLI/library crate with file-backed session state, repo-managed packaging assets, and GitHub-hosted build automation  
**Execution Model**: Session-native bounded delivery remains the primary runtime; install and repair verification extends the existing diagnostics path, while release preparation stays a bounded maintainer workflow that emits explicit package metadata and bundle state  
**Observability Surface**: `doctor` install diagnostics, release workflow logs, repository-managed formula and winget manifests, README plus getting-started quick path, advanced architecture docs, assistant guidance, roadmap, and changelog  
**Performance Goals**: Supported macOS and Windows users should reach a runnable CLI plus explicit Canon pairing state in under 10 minutes; new readers should find the quick path in under 2 minutes; release packaging validation for the slice should complete in under 20 minutes  
**Constraints**: No hosted service, no GUI installer, no Linux package-manager expansion beyond source install, no new long-running runtime, no Canon-owned control flow, no additional package managers beyond Homebrew and winget in this slice  
**Scale/Scope**: One Boundline release at a time, two official distribution channels, one bounded Canon companion version window per release, and one repo-owned documentation split between onboarding and advanced architecture

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice improves bounded software delivery by making Boundline installable, updatable, and diagnosable without source builds, which directly reduces operator friction before the first session-native run. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes install verification, official channel metadata, release bundles, and the primary docs path ahead of polish-only copy edits. See Summary, Technical Context, and research.
- **PASS** Primary workflow: The main operator path remains session-native `goal -> plan -> run -> status -> next -> inspect`; explicit compatibility behavior stays documented but subordinate after installation. See Summary, spec, and quickstart.
- **PASS** Bounded execution: Install and release surfaces resolve into explicit ready, blocked, or repair-needed states rather than hidden background repair logic. Session execution limits remain unchanged. See Technical Context, research, and contracts.
- **PASS** Stateful execution: The feature reuses existing workspace state surfaces and adds repo-managed release metadata plus explicit diagnostics output instead of hidden installer state. See Technical Context and data model.
- **PASS** Mutable planning: The slice does not replace the planning model; it adds bounded release and install verification inputs while leaving native replanning behavior intact. See Summary and research.
- **PASS** Sequential-first design: Install verification and release preparation stay sequential and explicit; no background workers or hidden fan-out are introduced beyond CI jobs that already build release assets. See Technical Context and structure decision.
- **PASS** Tool-agent symmetry: The feature keeps reasoning and action explicit through diagnostics output, release metadata, and install/update instructions rather than hidden package-manager assumptions. See contracts and quickstart.
- **PASS** Observability and explicit intelligence: Package readiness, Canon pairing state, and install or repair guidance are surfaced through `doctor`, docs, workflows, and release metadata. See Technical Context, contracts, and quickstart.
- **PASS** Non-goals and external separation: Canon remains a companion runtime with an explicit version window; package channels and workflows support Boundline but do not redefine session control flow or introduce unrelated UI or hosted deployment scope. See Constraints and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is one end-to-end official distribution surface with install verification, bounded Canon pairing, and a clearer operator-facing docs split. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/039-distribution-bundling/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Keep the structure minimal, delivery-focused, and sequential-
  first. Do not introduce extra top-level projects or UI/runtime surfaces unless
  the Constitution Check explicitly justifies them.
-->

```text
src/
├── cli.rs
├── cli/
│   ├── diagnostics.rs
│   └── output.rs
├── domain/
│   └── distribution.rs
└── lib.rs

tests/
├── contract/
│   ├── distribution_cli_contract.rs
│   └── distribution_metadata_contract.rs
├── integration/
│   ├── distribution_doctor_flow.rs
│   └── release_metadata_flow.rs
└── unit/
  ├── distribution_diagnostics.rs
  └── distribution_metadata.rs

distribution/
├── canon-bundle.toml
├── homebrew/
│   └── Formula/
│       └── boundline.rb
└── winget/
  └── manifests/

.github/workflows/
├── ci.yml
└── release-distribution.yml

scripts/
└── sync-distribution-metadata.sh

README.md
docs/
assistant/
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the runtime change narrow by extending the
existing diagnostics and output surfaces for install verification, add one
small domain module for distribution metadata and Canon pairing rules, and keep
package-channel assets plus release automation as repository-managed files
rather than a new service or runtime. The only new top-level directory is
`distribution/`, which is justified because Homebrew and winget metadata are
release artifacts, not session runtime state.

## Complexity Tracking

No constitution violations are expected for this slice.
