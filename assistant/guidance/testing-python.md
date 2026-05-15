# Python Testing Guidance

Use pytest, unittest, and framework test clients to keep Python feedback fast, explicit, and deterministic.

- Prefer readable fixtures and builders over deep autouse magic or hidden module state.
- Unit test domain and application services without the database unless persistence behavior is the subject.
- Use integration tests for ORM mappings, migrations, auth, serialization, and framework request boundaries.
- Parameterize behavior-heavy cases instead of duplicating setup across many similar tests.
- Keep async tests explicit about event loops, client lifetimes, and awaited work.
- Avoid clock, timezone, filesystem, and network coupling unless the test is explicitly about those boundaries.
