# Coverage Helper Scripts

These scripts are the repository-local helpers for LCOV inspection and patch coverage triage.

## Scripts

- `parse_lcov.py`: print per-file coverage for selected repository files.
- `aggregate_lcov.py`: merge one or more LCOV reports and summarize coverage for selected files.
- `intersect_patch_coverage.py`: intersect changed diff lines with uncovered LCOV lines.

## Usage

Run commands from the repository root.

```bash
python3 scripts/common/coverage/parse_lcov.py lcov.info src/orchestrator/session_runtime.rs
```

```bash
python3 scripts/common/coverage/aggregate_lcov.py lcov.info lcov-partial.info src/domain/session.rs src/orchestrator/session_runtime.rs
```

To merge multiple LCOV reports into one file for downstream upload:

```bash
python3 scripts/common/coverage/aggregate_lcov.py \
  --output-lcov lcov.merged.info \
  lcov.workspace.info lcov.boundline-core.info lcov.boundline-adapters.info lcov.boundline-cli.info
```

```bash
git diff --unified=0 origin/main...HEAD -- src/domain/session.rs src/orchestrator/session_runtime.rs \
  | python3 scripts/common/coverage/intersect_patch_coverage.py --lcov lcov.info \
      src/domain/session.rs src/orchestrator/session_runtime.rs
```

For machine-readable output:

```bash
git diff --unified=0 origin/main...HEAD -- src/domain/session.rs \
  | python3 scripts/common/coverage/intersect_patch_coverage.py --lcov lcov.info --json src/domain/session.rs
```

## Notes

- These helpers are intentionally generic and can be mirrored into companion repositories.
- `intersect_patch_coverage.py` reads the diff from stdin unless `--diff-file` is provided.
- The scripts do not run coverage; they inspect existing LCOV and diff artifacts.
- `aggregate_lcov.py --output-lcov ...` writes a merged LCOV report and still prints the requested target summary.
