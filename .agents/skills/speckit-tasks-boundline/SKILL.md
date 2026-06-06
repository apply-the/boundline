---
name: "speckit-tasks-boundline"
description: "Generate Speckit tasks for Boundline by wrapping speckit-tasks and enforcing Boundline-specific release, docs, quality, and verification rules."
compatibility: "Requires spec-kit project structure with .specify/ directory and the standard speckit-tasks skill available."
metadata:
  author: "apply-the"
  wraps: "speckit-tasks"
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding, if it is not empty.

## Purpose

This skill wraps the standard `/speckit-tasks` workflow and then enforces Boundline-specific task-generation rules before `tasks.md` is considered complete.

The standard `/speckit-tasks` skill remains the baseline task generator. This wrapper adds mandatory Boundline release, documentation, quality, coverage, and verification requirements.

## Required Execution Model

1. Run or follow the standard `/speckit-tasks` workflow first.
2. Generate or update `tasks.md` according to Speckit task-generation rules.
3. Run a Boundline task-quality pass.
4. Repair `tasks.md` until all wrapper rules below are represented as concrete tasks.
5. Report any rule that cannot be satisfied with a clear reason and a blocking follow-up task.

Do not silently skip any wrapper rule.

## Boundline Wrapper Rules

### Rule 1: Cargo Version Bump

Every feature or bugfix task plan must include a task that updates the version in the root `Cargo.toml`.

Use this versioning policy:

- If the current version is `0.x.y`:
  - feature or breaking change: increment `x`, reset `y` to `0`
  - bugfix only: increment `y`
- If the current version is `x.y.z` where `x >= 1`:
  - feature: increment `y`, reset `z` to `0`
  - bugfix only: increment `z`
  - breaking change: increment `x`, reset `y` and `z` to `0`

Examples:

```text
0.72.0 feature  -> 0.73.0
0.72.3 bugfix   -> 0.72.4
0.72.3 breaking -> 0.73.0

1.4.2 feature   -> 1.5.0
1.4.2 bugfix    -> 1.4.3
1.4.2 breaking  -> 2.0.0
```

The task must name the exact file:

```text
Cargo.toml
```

If the feature type is ambiguous, add a task that requires explicit classification as one of:

- feature
- bugfix
- breaking change

Do not guess silently.

### Rule 2: Documentation And Roadmap Synchronization

If needed, generate tasks to update:

- `README.md`
- `CHANGELOG.md`
- markdown files under `tech-docs/`
- markdown files under `docs/`
- markdown files under `roadmap/`

If the spec is based on a roadmap file:

1. Copy the roadmap seed into the feature spec folder.
2. Rename it using the `feat-<slug>.md` convention.

Example:

```text
roadmap/features/01-example-feature.md
-> specs/<feature-dir>/feat-example-feature.md
```

3. Remove the original feature seed from the roadmap folder when the roadmap policy requires conversion rather than duplication.
4. Remove or update roadmap references that still point to the old seed.
5. Update any roadmap index, graph, or forward-roadmap file that references the old seed.

Tasks must name concrete files whenever known.

### Rule 3: Documentation Version Synchronization

Every task plan that changes the Cargo version or release-facing documentation must include a task to run:

```bash
./scripts/update-docs-versions.sh
```

The task must appear after the `Cargo.toml` version update and before final verification.

If the platform is Windows, mention the equivalent script only when the current environment requires it:

```powershell
scripts/update-docs-versions.ps1
```

### Rule 4: Rust Quality, Coverage, Tests, Clippy, And Formatting

A feature is not complete until all modified or created Rust files meet the quality bar.

Tasks must include verification steps for:

- all modified or created Rust files have at least 95% coverage
- all tests pass
- `cargo clippy` has no warnings
- `cargo fmt` has been run
- code duplication has been checked and reduced where practical

Use the repository scripts when available.

Preferred Linux/macOS scripts:

```bash
scripts/update-docs-versions.sh 
scripts/clippy.sh
scripts/test.sh
scripts/coverage.sh
scripts/check-no-local-paths.sh
scripts/check-rust-no-panic.sh
```

Preferred Windows scripts when applicable:

```powershell
scripts/update-docs-versions.sh
scripts/clippy.ps1
scripts/test.ps1
scripts/coverage.ps1
scripts/update-docs-versions.ps1
```

The generated `tasks.md` must include final verification tasks that run, at minimum:

```bash
cargo fmt
scripts/clippy.sh
scripts/test.sh
scripts/coverage.sh
scripts/check-no-local-paths.sh
scripts/check-rust-no-panic.sh
```

If assistant plugin metadata is touched, also include:

```bash
scripts/validate-assistant-plugins.sh
```

If distribution metadata is touched, also include:

```bash
scripts/sync-distribution-metadata.sh
```

If coverage cannot be measured for a modified Rust file, add a blocking task to fix the coverage setup or justify the exclusion explicitly.

## Script Reference

Use these scripts whenever they fit the task:

| Script | Platform | Purpose |
|--------|----------|---------|
| `clippy.sh` / `clippy.ps1` | Linux/macOS / Windows | Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `test.sh` / `test.ps1` | Linux/macOS / Windows | Run `cargo nextest run --workspace --all-features` |
| `coverage.sh` / `coverage.ps1` | Linux/macOS / Windows | Run `cargo llvm-cov` with per-crate merge: workspace + `boundline-core` + `boundline-adapters` + `boundline-cli` to `lcov.info` |
| `update-docs-versions.sh` / `update-docs-versions.ps1` | Linux/macOS / Windows | Synchronize version references across the docs directory |
| `sync-distribution-metadata.sh` | Linux/macOS | Sync Homebrew formula and Winget manifests from the Cargo.toml version |
| `validate-assistant-plugins.sh` | Linux/macOS | Validate assistant plugin pack structure and metadata |
| `check-no-local-paths.sh` | Linux/macOS | Ensure no local filesystem paths are committed |
| `check-rust-no-panic.sh` | Linux/macOS | Audit Rust code for panic-prone patterns such as `unwrap` and `expect` outside `main.rs` |

## Boundline Task Quality Rules

In addition to standard Speckit rules, enforce these rules:

- Every task must map to a Functional Requirement, User Story, Success Criterion, Edge Case, analysis finding, release requirement, or wrapper rule.
- Convert all unresolved HIGH and MEDIUM `speckit.analyze` findings into concrete remediation tasks before implementation tasks.
- Do not allow implementation to proceed while unresolved HIGH or MEDIUM findings remain unless explicitly deferred in the spec.
- Every behavior-changing task must include a corresponding test task.
- Every runtime feature must include status, inspect, and trace projection tasks when applicable.
- Every fail-closed, blocked, degraded, unavailable, invalid-config, or malformed-input path must have a test.
- Provider output must never be treated as truth without validation.
- Canon changes must not be introduced unless the spec explicitly declares a Canon companion dependency.
- Process-only requirements must become checklist, CI, or review tasks, not fake runtime code tasks.
- Tasks must name concrete files or modules when known.
- Tasks must include dependency notes where ordering matters.
- Avoid vague tasks such as "update logic", "improve handling", "add support", or "fix issue".
- Keep tasks grouped by setup, foundational, user story, integration, polish, and release validation.
- If a roadmap file was converted into a spec artifact, include tasks to remove duplication and update references.
- If a release-facing change is made, include CHANGELOG and version synchronization tasks.

## Required Final Verification Phase

The final phase of `tasks.md` must include a section named:

```md
## Final Phase: Release, Quality, And Verification
```

This phase must include tasks equivalent to:

```text
- [ ] TXXX Update Cargo.toml version according to Boundline versioning policy in Cargo.toml
- [ ] TXXX Update README.md and CHANGELOG.md for this feature if release-facing behavior changed
- [ ] TXXX Update docs, tech-docs, and roadmap markdown references affected by this feature
- [ ] TXXX Run ./scripts/update-docs-versions.sh
- [ ] TXXX Run cargo fmt
- [ ] TXXX Run scripts/clippy.sh and fix all warnings
- [ ] TXXX Run scripts/test.sh and fix failing tests
- [ ] TXXX Run scripts/coverage.sh and confirm at least 95% coverage for every modified or created Rust file
- [ ] TXXX Run scripts/check-no-local-paths.sh
- [ ] TXXX Run scripts/check-rust-no-panic.sh
```

Only include `scripts/validate-assistant-plugins.sh` when assistant plugin assets are touched.

Only include `scripts/sync-distribution-metadata.sh` when distribution metadata is touched or the release workflow requires it.

## Roadmap Conversion Rules

When the active spec was created from a roadmap seed:

1. Preserve the seed content under the feature spec folder as `feat-<slug>.md`.
2. Remove the roadmap seed file if the roadmap uses move-on-conversion semantics.
3. Update roadmap index files and graph files to point to the real spec.
4. Remove duplicate references that keep both the old roadmap seed and the new spec as active sources of truth.
5. Ensure the spec folder is now the source of truth.

If unsure whether the roadmap uses move-on-conversion semantics, add a task to inspect the roadmap README or conversion policy before deleting the seed.

## Output Requirements

After generating or repairing `tasks.md`, report:

- path to generated `tasks.md`
- total task count
- task count per user story
- task IDs added by this wrapper
- version bump task ID
- docs synchronization task ID
- final quality verification task IDs
- any wrapper rule that could not be fully enforced

## Completion Gate

Do not present the task plan as complete unless:

- standard Speckit task format validation passes
- all wrapper rules are represented in `tasks.md`
- the final verification phase exists
- version bump policy is represented
- docs synchronization is represented
- quality scripts are represented
- coverage target is represented
- roadmap duplication handling is represented when applicable

The expected completion bar is:

```text
tests green
coverage >= 95% for modified or created Rust files
clippy clean
cargo fmt applied
no unnecessary duplication
docs versions synchronized
roadmap/spec duplication resolved when applicable
```