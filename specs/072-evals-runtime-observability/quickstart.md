# Quickstart: Evals And Runtime Observability

**Feature**: 072-evals-runtime-observability
**Date**: 2026-06-05

## Prerequisites

- Rust 1.96.0 toolchain installed
- Boundline workspace built (`cargo build`)
- An isolated temporary workspace (NOT the Boundline repository root — use `mktemp -d`)

## Scenario 1: Run the Eval Suite Locally

1. Create an isolated temp workspace:

```bash
WORKSPACE=$(mktemp -d)
cd "$WORKSPACE"
git init && git commit --allow-empty -m "init"
```

2. Run the eval suite:

```bash
/path/to/target/debug/boundline evals run
```

3. Confirm output includes per-eval pass/fail status and a suite aggregate:

```text
Eval Suite Results
  planning-quality-01     PASS  312ms
  context-selection-01    PASS  245ms
  critical-omission-01    PASS  198ms
  guardian-finding-01     PASS  267ms
  council-rejection-01    PASS  301ms
  provider-failure-01     PASS  289ms
  compaction-decisions-01 PASS  156ms
  compaction-rejections-01 PASS  143ms
─────────────────────────────────────
  Suite: PASS  8/8  1911ms
```

4. Confirm CI-compatible JSON output with `--json`:

```bash
/path/to/target/debug/boundline evals run --json | jq '.suite_status'
# Expected: "pass"
```

5. Confirm CI exit code behavior: create a fixture that should fail, run the eval, and verify exit code 1.

## Scenario 2: Trace Compaction

1. In an isolated temp workspace, create a session that produces trace items (decisions, findings, transcripts):

```bash
WORKSPACE=$(mktemp -d)
cd "$WORKSPACE"
git init && git commit --allow-empty -m "init"
# Run a planning flow to populate traces...
```

2. Inspect the trace before compaction:

```bash
/path/to/target/debug/boundline trace inspect
```

3. Run compaction:

```bash
/path/to/target/debug/boundline trace compact
```

4. Confirm the compaction event is emitted:

```bash
/path/to/target/debug/boundline trace events | jq 'select(.event_type == "trace.compacted")'
```

5. Confirm accepted decisions and rejection reasons survived exactly:

```bash
# Verify that decision content is unchanged after compaction
/path/to/target/debug/boundline trace inspect --item decision-12
```

6. Confirm lossy summaries are marked:

```bash
/path/to/target/debug/boundline trace events | jq 'select(.event_type == "trace.compacted") | .payload.actions[] | select(.lossy == true)'
```

7. Confirm oversized trace (>50k items) is rejected with an actionable message:

```bash
# Simulate or construct a trace with >50k items, then:
/path/to/target/debug/boundline trace compact
# Expected: error message explaining the bound and suggesting --confirm or chunked processing
```

## Scenario 3: JSONL Export

1. Export structured events:

```bash
/path/to/target/debug/boundline trace export --format jsonl
```

2. Confirm every event has a `schema_version`:

```bash
/path/to/target/debug/boundline trace export --format jsonl | jq '.schema_version' | sort -u
```

3. Confirm no sensitive data leaks:

```bash
/path/to/target/debug/boundline trace export --format jsonl | grep -iE 'token|secret|password|key|credential'
# Expected: no matches
```

4. Confirm event types are recognized:

```bash
/path/to/target/debug/boundline trace export --format jsonl | jq -r '.event_type' | sort -u
# Expected: planning.analysis.completed, guardian.finding.emitted, provider.call.completed, trace.compacted, ...
```

## Scenario 4: Compatibility — Pre-Analysis Session Snapshot

1. Load a session snapshot that predates this feature (no structured events emitted):

```bash
# Use a fixture from tests/fixtures/ or create a minimal session
/path/to/target/debug/boundline trace export --format jsonl
# Expected: empty stream or events only from post-feature phases
```

2. Confirm that `boundline status` does not panic or show synthetic blocked state.

## Scenario 5: Conservative Tiebreaking

1. Construct a trace with an item whose type is not in the classification table:

```bash
# Add an unrecognized item type to a trace
# Run compaction
/path/to/target/debug/boundline trace compact
```

2. Confirm the item was classified by tiebreaking (stricter class won):

```bash
/path/to/target/debug/boundline trace events | jq 'select(.event_type == "trace.compacted") | .payload.actions[] | select(.tiebreak == true)'
# Expected: item present with tiebreak=true, classified at the stricter boundary
```

## Validation Commands

```bash
# Formatting
cargo fmt --check

# Clippy (strict)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Unit tests
cargo test --test unit

# Contract tests
cargo test --test contract

# Integration tests
cargo test --test integration

# Full suite
cargo test

# Coverage
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
scripts/common/coverage/intersect_patch_coverage.py
```

## Release Quality Gates

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] All unit, contract, and integration tests pass
- [ ] Changed-file coverage ≥ 95% for all touched Rust implementation files
- [ ] JSONL export produces valid JSON with correct `schema_version` per event type
- [ ] Compaction preserves accepted decisions and rejection reasons in 100% of regression cases
- [ ] No sensitive data in exported events or metrics
- [ ] Oversized-trace rejection works as specified
- [ ] CI eval runner produces machine-readable summary with correct exit code
