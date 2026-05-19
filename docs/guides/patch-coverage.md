# Patch Coverage Helpers

Boundline keeps LCOV inspection and patch-coverage triage helpers under `scripts/common/coverage/`.

## Available Helpers

- `scripts/common/coverage/parse_lcov.py`: print per-file coverage for selected repository files.
- `scripts/common/coverage/aggregate_lcov.py`: merge one or more LCOV reports and summarize coverage for selected files.
- `scripts/common/coverage/intersect_patch_coverage.py`: intersect changed diff lines with uncovered LCOV lines.

## Typical Workflow

1. Refresh or collect `lcov.info`.
2. Generate a zero-context diff for the files under review.
3. Intersect the diff with uncovered LCOV lines.
4. Use the reported line numbers and contexts to add the smallest targeted tests.

Example:

```bash
git diff --unified=0 origin/main...HEAD -- src/domain/session.rs src/orchestrator/session_runtime.rs \
  | python3 scripts/common/coverage/intersect_patch_coverage.py --lcov lcov.info \
      src/domain/session.rs src/orchestrator/session_runtime.rs
```

For machine-readable output:

```bash
git diff --unified=0 origin/main...HEAD -- src/domain/session.rs \
  | python3 scripts/common/coverage/intersect_patch_coverage.py --lcov lcov.info --json \
      src/domain/session.rs
```

## Availability

These helpers are intentionally generic. Boundline keeps the canonical copies in the same repository-local path used by Canon:

- `scripts/common/coverage/`

When the scripts evolve, keep the command-line contract aligned across both repositories.

## Guidance For Future Agents

- Treat these helpers as analysis utilities, not as validation substitutes.
- Prefer `intersect_patch_coverage.py` when the question is about patch coverage rather than full-file coverage.
- If `lcov.info` is stale or missing, fix the coverage generation flow first; do not trust the helper output blindly.
