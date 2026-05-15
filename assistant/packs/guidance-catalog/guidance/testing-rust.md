# Testing Rust

Testing conventions specific to Rust projects using cargo test, nextest, and common testing patterns.

## Test Organization

- Unit tests: `#[cfg(test)] mod tests` in source files for private behavior
- Integration tests: `tests/` directory for public API contracts
- Doc tests: `///` examples that verify documentation accuracy

## Assertions

Use `assert_eq!`, `assert_ne!`, and `assert!` with descriptive messages. For complex assertions, use dedicated assertion crates (`assert_matches`, `pretty_assertions`).

## Error Testing

Test both success and error paths. Use `#[should_panic]` sparingly; prefer matching on `Result::Err` variants for precise error assertions.

```rust
#[test]
fn rejects_empty_name() {
    let result = Name::new("");
    assert!(matches!(result, Err(DomainError::EmptyName)));
}
```

## Test Isolation

Each test must be independent. Avoid shared mutable state. Use fresh instances. For filesystem tests, use temp directories cleaned up on drop.

## Mocking

Prefer trait-based dependency injection over mock frameworks. Define traits for external boundaries, inject test implementations. Use `mockall` only when trait boundaries are already established.

## Performance

Use `#[ignore]` for slow tests. Run fast tests frequently during development. Use `cargo nextest` for parallel execution.

## Recommended Tools

| Tool | Purpose |
|------|---------|
| `cargo-nextest` | Parallel test runner with filtering and retries |
| `mockall` | Auto-generated trait mocks |
| `proptest` | Property-based / fuzz testing |
| `assert_cmd` | CLI binary integration testing |
| `wiremock` | HTTP mock server for integration tests |
| `pretty_assertions` | Readable diff output on failure |
| `insta` | Snapshot testing with review workflow |
| `test-case` | Parameterized tests via proc-macro |

## Anti-Patterns

- Tests that depend on execution order
- `unwrap()` in tests without clear context on what failed
- Mocking implementation details instead of boundaries
- Tests that require network or filesystem access without isolation
- Missing error path testing
- Large test fixtures that obscure what is being tested

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: untestable-design, test-isolation
- `rust_zero_panic`: tests should still demonstrate explicit error handling patterns
