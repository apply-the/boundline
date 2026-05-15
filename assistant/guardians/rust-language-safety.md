# Rust Language Safety Guardian

Check changed Rust files for `unwrap` and `expect` shortcuts that bypass explicit error handling in production code.

- Treat `unwrap` and `expect` as verification failures unless the code is clearly test-only or the invariant is enforced at the entry point.
- Prefer typed errors, explicit propagation, and local invariant modeling over panic-driven flow.
- Keep the finding evidence tied to the changed file so the repair remains bounded.
