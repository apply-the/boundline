# Systems Language Guidance

Use C, C++, Zig, and similar systems languages with explicit ownership, constrained unsafe boundaries, and documented invariants.

- Model ownership, lifetime, and valid state explicitly instead of relying on comments or sentinel values alone.
- Keep low-level memory, FFI, and unsafe sections narrow, reviewed, and isolated from domain decisions.
- Prefer typed enums, structs, and state machines over integer flags or loosely coupled primitives.
- Make cleanup deterministic on every error path.
- Keep global mutable state and hidden initialization out of core logic unless the platform boundary demands it.
- Document performance-sensitive assumptions and failure behavior where the type system cannot express them alone.
