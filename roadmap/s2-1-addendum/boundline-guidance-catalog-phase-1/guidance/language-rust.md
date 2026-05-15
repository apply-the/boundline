# Rust Guidance

## Purpose

This guidance defines idiomatic Rust practices for AI-assisted implementation, review, testing, and refactoring.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy or Canon-governed standards.

## Version Posture

Active support window:
- Rust 1.70+
- modern stable toolchain

Target excellence:
- Rust 2024 Edition where repository constraints allow it

Legacy warnings:
- pre-2021 Edition layout
- old module layout using unnecessary `mod.rs`
- panic-heavy error handling
- raw primitive domain modeling

## Module Organization

Prefer modern module layout:

```text
src/domain.rs
src/domain/user.rs
```

Avoid unnecessary:

```text
src/domain/mod.rs
```

Keep visibility narrow:
- private by default
- `pub(crate)` for internal sharing
- `pub` only for stable public API

## Error Handling

Apps and binaries:
- `anyhow` is acceptable for top-level orchestration and command boundaries

Libraries and crates:
- prefer typed errors with `thiserror`
- avoid `anyhow` in public APIs
- expose matchable error variants where consumers need stable behavior

Large systems:
- consider `snafu` or structured error context when error provenance matters deeply

## Zero-Panic Policy

Forbidden in production library/domain code unless explicitly justified:

- `panic!`
- `todo!`
- `unimplemented!`
- `unwrap()`
- `expect()`
- `unwrap_err()`
- indexing with `slice[index]`

Prefer:
- `Result`
- `Option`
- `get`
- typed errors
- explicit impossible invariant comments where unavoidable

Allowed exceptions:
- tests
- CLI entry points where failure terminates the process intentionally
- documented impossible invariants

## Async Guidance

Avoid holding blocking locks across `await`.

Bad:
```text
std::sync::Mutex held across async boundary
```

Prefer:
- `tokio::sync::Mutex` when async coordination is needed
- narrower lock scope
- message passing when ownership is clearer
- avoiding shared mutable state where possible

## Domain Modeling

Use newtypes for domain IDs and quantities.

Bad:
```rust
fn rotate_token(user_id: String, ttl: i64)
```

Better:
```rust
fn rotate_token(user_id: UserId, ttl: TokenTtl)
```

Avoid primitive obsession in:
- IDs
- money
- quantities
- time windows
- permissions
- ownership references

## API Boundaries

For public crates:
- keep error types stable
- document invariants
- avoid leaking implementation-specific types
- avoid `anyhow` in public signatures
- avoid broad trait object errors unless the boundary is intentionally opaque

## Testing

Rust tests should:
- prefer meaningful domain assertions
- avoid `Box<dyn Error>` where typed failure helps
- use builders for complex data
- separate unit and integration tests
- use deterministic clocks where time matters

## Anti-Patterns

- `Box<dyn Error>` in production logic when typed errors are needed
- `anyhow` in public library APIs
- raw `String` domain IDs
- hidden panics from indexing
- excessive `Arc<Mutex<_>>`
- `clone()` to bypass ownership without understanding cost
- treating compiler appeasement as design quality

## Guardian Hooks

Recommended guardians:
- rust-zero-panic-guardian
- rust-error-boundary-guardian
- rust-async-boundary-guardian
- rust-domain-newtype-guardian
- rust-visibility-guardian

## Structured Finding Example

```json
{
  "guardian": "rust-zero-panic",
  "rule": "unwrap-in-domain-code",
  "disposition": "blocker",
  "summary": "Domain token rotation uses unwrap on parsed expiry data.",
  "evidence_refs": ["src/auth/token_rotation.rs"],
  "recommended_action": "Return a typed error and preserve invalid token context."
}
```
