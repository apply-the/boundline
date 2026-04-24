# synod Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-23

## Active Technologies

- Rust 1.95.0, edition 2024 + Rust standard library plus `serde`, `serde_json`, `thiserror`, `tracing`, and `uuid` for structured state, trace serialization, error handling, instrumentation, and stable identifiers (001-delivery-orchestrator-core)

## Project Structure

```text
src/
tests/
```

## Commands

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo nextest run`
- `cargo deny check licenses advisories bans sources`

## Code Style

Rust 1.95.0, edition 2024: Follow standard conventions

## Recent Changes

- 001-delivery-orchestrator-core: Added Rust 1.95.0, edition 2024 + Rust standard library plus `serde`, `serde_json`, `thiserror`, `tracing`, and `uuid` for structured state, trace serialization, error handling, instrumentation, and stable identifiers

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
