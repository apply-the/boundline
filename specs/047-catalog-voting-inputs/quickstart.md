# Quickstart: Catalog Currency, Independent Voting, and File-Backed Inputs

## Preconditions

- Use the repository root as the working directory.
- Ensure the workspace has a writable temporary directory for test fixtures.

## 1. Verify bundled catalog refresh landed

Run:

```bash
rg -n "gpt-5.5|opus-4.7|sonnet-4.6|gemini-3.1-pro-preview" assistant/catalog/model-catalog.toml
```

Expected result:

- Each listed model appears in the bundled catalog.
- The catalog metadata shows the refreshed date and version for the current slice.

## 2. Verify prompt-style path input normalizes to file-backed sources

Run the focused tests:

```bash
cargo test -p boundline-core normalizes_path_only_goal_as_referenced_markdown
cargo test -p boundline-core normalizes_markdown_reference_array_as_ordered_file_backed_input
```

Expected result:

- Both tests pass.
- A single path or array-of-path shorthand is stored as referenced Markdown input without redundant direct goal text.

## 3. Verify collapsed review councils are rejected explicitly

Run the focused adapter tests:

```bash
cargo test -p boundline-adapters resolve_review_vote_rejects_non_independent_review_council
cargo test -p boundline-adapters build_task_request_copies_routing_projection_into_initial_context
```

Expected result:

- The non-independent council test fails before a vote is recorded unless reviewer routes are distinct.
- Task creation persists routing projection into task state for downstream review-route resolution.

## 4. Run release-facing validation

Run:

```bash
cargo test --no-run --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
```

Optional coverage refresh:

```bash
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected result:

- Compilation, lint, and formatting checks succeed.
- Coverage artifacts can be refreshed after the focused tests and broader test selection complete.