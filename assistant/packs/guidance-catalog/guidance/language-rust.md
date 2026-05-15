# Rust

Idiomatic Rust practices for implementation, review, testing, and refactoring. Rust rewards explicit design, clear ownership, and precise state modeling.

## Module Organization

Prefer modern module layout without `mod.rs`:

```text
src/domain.rs
src/domain/user.rs
```

Keep visibility narrow: private by default, `pub(crate)` for internal sharing, `pub` only for stable public API.

## Zero-Panic Policy

Forbidden in production library/domain code:
- `panic!`, `todo!`, `unimplemented!`
- `unwrap()`, `expect()`, `unwrap_err()`
- Direct indexing `slice[index]`

Allowed exceptions: tests, CLI entry points where failure terminates the process intentionally, documented impossible invariants.

Prefer `Result`, `Option`, `.get()`, typed errors, and explicit propagation with `?`.

## Error Handling

Apps and binaries: `anyhow` is acceptable for top-level orchestration and command boundaries.

Libraries and crates: prefer typed errors with `thiserror`. Avoid `anyhow` in public APIs. Expose matchable error variants where consumers need stable behavior.

```rust
#[derive(Debug, thiserror::Error)]
pub enum CreateOrderError {
    #[error("customer not found")]
    CustomerNotFound,
    #[error("repository error")]
    Repository(#[from] RepositoryError),
}
```

## Domain Modeling

Use newtypes for domain IDs and quantities:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderId(String);
```

Model states with enum:

```rust
pub enum PaymentStatus {
    Pending,
    Completed { transaction_id: TransactionId },
    Failed { reason: String },
}
```

Make objects valid at construction: do not expose constructors that allow invalid states.

## Dependency Injection

Use traits and generics. Static dispatch by default; dynamic dispatch (`Box<dyn Trait>`) when heterogeneity or boundary stability is needed.

```rust
pub struct OrderService<R: OrderRepository> {
    repository: R,
}
```

## Async

Do not block inside async functions. Do not hold `std::sync::Mutex` across `.await` points; use `tokio::sync::Mutex` when async coordination is needed.

Propagate cancellation where possible. Avoid detached tasks without error observation.

## Logging And Tracing

Prefer `tracing` over unstructured logging. Use `#[tracing::instrument]` for span creation. Do not log sensitive data.

## Testing

Prefer meaningful domain assertions. Use builders for complex test data. Separate unit and integration tests. Use deterministic clocks where time matters.

```rust
#[test]
fn rejects_empty_order() {
    let result = Order::new(vec![]);
    assert!(matches!(result, Err(OrderError::Empty)));
}
```

## Recommended Ecosystem Libraries

| Category | Crate | Purpose |
|----------|-------|---------|
| Error handling | `thiserror` | Typed error derives for libraries |
| Error handling | `anyhow` | Ergonomic errors for binaries/CLI |
| Serialization | `serde`, `serde_json` | Derive-based (de)serialization |
| Async runtime | `tokio` | Default async executor and I/O |
| HTTP client | `reqwest` | Async HTTP with connection pooling |
| CLI | `clap` | Derive-based argument parsing |
| Enum utilities | `strum` | Display, EnumString, EnumIter derives |
| Database | `sqlx` | Compile-time checked async SQL |
| Identifiers | `uuid` | UUID generation and parsing |
| Time | `time` or `chrono` | Date/time handling (prefer `time` for new code) |
| Observability | `tracing` | Structured, span-based instrumentation |
| Testing | `mockall` | Trait-based mock generation |
| Testing | `proptest` | Property-based / fuzz testing |
| Test runner | `cargo-nextest` | Parallel, filtered test execution |
| Assertions | `pretty_assertions`, `assert_matches` | Readable diff output |

Prefer well-maintained crates with minimal transitive dependency trees. Audit before adding.

## Anti-Patterns

- `Box<dyn Error>` in production logic when typed errors are needed
- `anyhow` in public library APIs
- Raw `String` domain IDs
- Hidden panics from indexing
- Excessive `Arc<Mutex<_>>`
- `clone()` to bypass ownership without understanding cost
- Holding blocking locks across await points
- Treating compiler appeasement as design quality

## Guardian Hooks

Guardians that apply to this guidance:
- `rust_zero_panic`: no-unwrap-in-domain-code, no-expect-in-library-code, no-panic-in-production-path
- `clean_code`: no-primitive-obsession, no-mixed-responsibilities
- `architecture_boundary`: dependency-direction
