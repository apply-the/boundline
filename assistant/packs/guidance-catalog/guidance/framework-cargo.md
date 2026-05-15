# Cargo And Rust Workspaces

Conventions for Rust projects using Cargo workspaces, including crate organization, dependency management, and build configuration.

## Workspace Organization

Use a workspace for multi-crate projects. Keep each crate focused on one responsibility:
- `core` or `domain`: pure domain logic, no framework dependencies
- `adapters` or `infrastructure`: external integrations, persistence
- `cli` or `api`: entry points, transport, presentation

Share dependencies through `[workspace.dependencies]` to keep versions consistent.

## Dependency Management

Pin dependencies through `Cargo.lock` (committed for binaries and applications). Use `[workspace.dependencies]` for shared version declarations. Audit new dependencies before adding.

Prefer:
- Well-maintained crates with stable APIs
- Crates with minimal transitive dependencies
- `cargo deny` for license and vulnerability checks

## Feature Flags

Use feature flags for optional functionality. Keep the default feature set minimal. Document what each feature enables.

## Build Configuration

Use `rust-toolchain.toml` to pin the Rust version. Use `rustfmt.toml` for consistent formatting. Use `clippy.toml` or workspace-level `#![deny(clippy::...)]` for lint configuration.

## Testing Organization

Separate unit tests (in `src/` with `#[cfg(test)]`), integration tests (`tests/`), and benchmarks (`benches/`). Use test utilities crates for shared fixtures.

## Anti-Patterns

- Workspace without `[workspace.dependencies]` leading to version drift
- Binary crate with application logic that should be in a library crate
- Missing `Cargo.lock` for applications
- Circular dependencies between workspace members
- Feature flags that change public API shape unexpectedly

## Guardian Hooks

Guardians that apply to this guidance:
- `supply_chain`: dependency-manifest-without-lockfile-update, vulnerability-untriaged
- `architecture_boundary`: dependency-direction between workspace members
