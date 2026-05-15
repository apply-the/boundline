# Cargo Workspace Guidance

Treat the Cargo workspace as the delivery boundary for Rust code, tooling, and release metadata.

- Keep crate responsibilities explicit; add behavior in the owning crate before creating new bridge layers.
- Validate the narrowest crate, target, or test slice that can disconfirm the current hypothesis, then widen only when needed.
- Keep workspace package metadata, changelog, and release-aligned docs in sync when a versioned feature lands.
- Prefer repository-managed assets, configuration, and manifests over ad hoc generated state.
- Avoid leaking test-only shortcuts or panic-prone helpers into production modules.
- Respect serialized shape stability across CLI, session, trace, and config surfaces.
