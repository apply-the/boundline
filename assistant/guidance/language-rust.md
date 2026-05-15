# Rust Language Guidance

Use Rust to make invalid states difficult, resource ownership explicit, and failures observable without panic shortcuts.

- Propagate recoverable errors with `Result` and typed error variants instead of `unwrap`, `expect`, or hidden fallback branches.
- Prefer typed structs and enums over ad hoc maps or JSON assembly for stable serialized shapes.
- Keep ownership, borrowing, and mutability decisions local and explicit; do not widen lifetimes just to satisfy one call site.
- Model domain invariants with newtypes, enums, and builders when primitive values would blur meaning.
- Keep `unsafe`, FFI boundaries, and low-level mutation narrow, documented, and isolated from core domain logic.
- Respect existing crate boundaries and use the smallest compile or test slice that can falsify the change.
- Preserve compile-time guarantees before adding runtime indirection or compatibility fallbacks.
