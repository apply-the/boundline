# Rust Testing Guidance

Use Rust tests to prove behavior, serialized shape, and workspace integration without hiding behind broad snapshots.

- Run the narrowest executable check that can disconfirm the current hypothesis before widening validation.
- Use `cargo test --no-run --all-targets` when shared fixtures, inline `#[cfg(test)]` modules, or workspace wiring may be affected.
- Keep unit tests focused on local behavior and invariants; reserve integration tests for CLI, persistence, and cross-crate boundaries.
- Keep contract tests centered on stable serialized shapes, manifest fields, and operator-visible text surfaces.
- Prefer explicit evidence refs and targeted assertions over snapshot sprawl.
- Treat flaky timing, global temp state, and implicit environment coupling as test defects, not acceptable noise.
