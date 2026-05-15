# Systems Languages (C, C++, Zig)

Systems programming requires explicit memory management, defined lifetimes, and careful boundary validation. Safety mechanisms vary by language but the principles are consistent.

## Memory Safety

Every allocation must have a clear owner and defined lifetime. Prefer stack allocation for short-lived data. Avoid raw pointers when safer abstractions exist.

C: use clear ownership conventions, `free` in the same scope or explicit transfer documentation.
C++: use RAII, smart pointers (`unique_ptr`, `shared_ptr`), avoid raw `new`/`delete`.
Zig: use allocator-aware design, `defer` for cleanup, comptime where possible.

## Boundary Validation

Validate all external input: network data, file content, IPC messages, hardware registers. Never trust buffer sizes from external sources. Check bounds before indexing.

## Error Handling

C: return error codes consistently, check every return value, use errno or structured error contexts.
C++: prefer exceptions at high level, `std::expected` or error codes in performance-critical paths, never ignore return values.
Zig: use error unions, `try` for propagation, explicit error sets.

## Concurrency

Make synchronization explicit. Document which mutex protects which data. Avoid lock ordering violations. Prefer message passing when ownership transfer is clearer.

C/C++: use `std::mutex` with RAII guards, avoid manual lock/unlock pairs.
Zig: use explicit allocator and thread-local patterns.

## Build And Dependency Safety

Audit third-party dependencies carefully. Pin versions. Avoid build scripts that download from network without verification. Prefer vendored or audited dependencies.

## Recommended Ecosystem Tools

### C

| Category | Tool | Purpose |
|----------|------|---------|
| Build | CMake or Meson | Cross-platform build system |
| Memory analysis | Valgrind, AddressSanitizer | Runtime memory error detection |
| Testing | CMocka or Unity | Lightweight C test frameworks |
| Linting | `clang-tidy`, `cppcheck` | Static analysis |
| Formatting | `clang-format` | Consistent code style |

### C++

| Category | Tool | Purpose |
|----------|------|---------|
| Build | CMake | Standard build system |
| Testing | Catch2 or GoogleTest | Feature-rich test frameworks |
| Formatting | `fmt` (libfmt) | Type-safe format strings |
| JSON | `nlohmann/json` | Header-only JSON library |
| Package management | Conan or vcpkg | Dependency resolution |
| Sanitizers | ASan, TSan, UBSan | Runtime defect detection |

### Zig

| Category | Tool | Purpose |
|----------|------|---------|
| Build | `build.zig` (native) | Integrated build system |
| Testing | `zig test` (built-in) | First-class test support |
| C interop | `@cImport` | Direct C header inclusion |

## Anti-Patterns

- Raw pointers without clear ownership documentation
- Missing bounds checks on external input
- Ignored return values or error codes
- Manual lock/unlock without RAII
- Undefined behavior from uninitialized memory
- Buffer overflows from unchecked sizes
- Global mutable state without synchronization
- Build scripts with network access and no verification

## Guardian Hooks

Guardians that apply to this guidance:
- `security_boundary`: buffer overflow risks, unchecked external input
- `clean_code`: no-hidden-side-effects, no-primitive-obsession
- `supply_chain`: build script safety, dependency auditing
