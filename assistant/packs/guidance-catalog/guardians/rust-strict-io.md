# Rust Strict IO Guardian

Enforce strict separation between the presentation layer and the core logic layer regarding standard I/O and logging.

## Rules

### no-println-in-core
Standard I/O macros (`println!`, `print!`, `eprintln!`, `dbg!`) are strictly prohibited in core logic, orchestrators, adapters, and libraries.

**Rationale:** Prevents uncontrolled I/O side effects, avoids "polluting" standard output when the code is consumed as a library, and ensures a consistent user experience by centralizing output formatting in the presentation layer.

**Remediation:** Replace console prints with structured logging frameworks (e.g., `tracing` or `log`). Tracing should capture execution flow events, while any user-facing message must be returned up the call stack via return types (`Result<T, Error>`) to the CLI/Presentation layer, which is exclusively responsible for rendering output to stdout/stderr. Direct I/O is only permitted in entrypoints or explicitly designated presentation modules (e.g., `main.rs`, `cli.rs`, `presentation/`).

## Disposition

Default: `warning` (block merging, requires remediation).

## Scope

Applies to Rust code. Language-specific guardian. Enforced by code review and bounded execution checks.
