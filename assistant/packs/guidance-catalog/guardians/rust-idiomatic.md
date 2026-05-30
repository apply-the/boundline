# Rust Idiomatic Guardian

Enforce idiomatic Rust patterns, especially regarding concurrency and safety across async boundaries.

## Rules

### prefer-channels-over-locks
Avoid sharing mutable state through highly-contended locks like `Arc<Mutex<T>>`. Prefer message-passing (channels) or immutable data snapshots to share state across concurrent boundaries.

Triggers: excessive use of nested locks, returning locked references across await points, architectures that rely heavily on global synchronized state instead of worker queues or actor models in Rust.

### no-wildcard-imports
Avoid wildcard imports (e.g., `use std::io::*;`) in favor of explicit imports. Wildcard imports pollute the namespace, make it difficult to track where symbols come from, and can break the build if a dependency adds a colliding name.

Triggers: `use module::*;` declarations, especially from external crates or the standard library.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to Rust code only. Language-specific guardian. Enforced by code review and bounded execution checks.
