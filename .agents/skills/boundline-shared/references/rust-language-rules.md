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

## No Magic Literals In Owned Logic

- In Rust code outside `main.rs`, `#[cfg(test)]` modules, and files under
  `tests/`, do not introduce magic strings or magic numbers in domain logic,
  protocol handling, persistence, configuration, CLI contracts, or
  serialization paths.
- Promote reusable literals, wire-format keys, status strings, exit codes,
  schema versions, filenames, and sentinel numeric values into named `const`
  items or typed enums/newtypes scoped to the owning module or type.
- Prefer typed wrappers or enums when the literal represents a constrained
  domain value rather than a generic scalar.

## Typed Serialization For Stable Shapes

- When a serialized or deserialized shape is stable, model it with typed
  `struct` or `enum` definitions plus `serde` derives rather than assembling
  `serde_json::Map`, relying on repeated raw key strings, or constructing
  stable payloads with ad hoc `json!` objects.
- Use map- or value-oriented assembly only when the payload shape is genuinely
  dynamic or partially open-ended, and keep the dynamic boundary explicit in
  the owning type or function.

## Allowed Non-Panicking Helpers

- This rule does not ban non-panicking combinators such as
  `unwrap_or_default`, `unwrap_or_else`, `unwrap_or`, or `Option`/`Result`
  handling that returns explicit errors instead of panicking.

## Review Expectation

- Reviewers and implementers should treat newly introduced panic-prone calls
  outside `main.rs` and test code as policy violations.
- Reviewers and implementers should treat newly introduced magic literals or
  stable-shape ad hoc map/json serialization outside `main.rs` and test code
  as policy violations.