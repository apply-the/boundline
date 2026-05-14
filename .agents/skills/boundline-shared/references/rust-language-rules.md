# Boundline Rust Language Rules

These rules are normative for Rust code changes in this repository. They are
part of the repository's AI-visible engineering surface and apply to both human
and AI-authored changes.

## No Panic Outside Entrypoints And Tests

- In Rust code outside `main.rs`, `#[cfg(test)]` modules, and files under
  `tests/`, do not introduce panic-prone control flow.
- Treat `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`,
  `unreachable!()`, and assert-family macros used as runtime guards as banned
  outside `main.rs` and test code.
- When a failure can arise from workspace state, user input, IO, parsing,
  serialization, validation, configuration, session projection, governance
  integration, or delivery-path selection, surface it with explicit error
  propagation or a typed blocked, stale, unsupported, or invalid state.
- `main.rs` may still panic when immediate process termination is the intended
  behavior of the executable entrypoint, but explicit exit handling remains
  preferred when practical.
- `#[cfg(test)]` code and files under `tests/` may use panicking helpers
  freely.

## Allowed Non-Panicking Helpers

- This rule does not ban non-panicking combinators such as
  `unwrap_or_default`, `unwrap_or_else`, `unwrap_or`, or `Option`/`Result`
  handling that returns explicit errors instead of panicking.

## Review Expectation

- Reviewers and implementers should treat newly introduced panic-prone calls
  outside `main.rs` and test code as policy violations.