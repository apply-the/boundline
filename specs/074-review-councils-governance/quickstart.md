# Quickstart: Review Councils And Role-Gated Governance

**Feature**: 074-review-councils-governance
**Date**: 2026-06-06

## Prerequisites

- Rust 1.96.0 toolchain
- Boundline built (`cargo build`)
- Isolated temp workspace (NOT the repo root)

## Scenario 1: Rust Runtime Change Activates Correct Guardians

```bash
WORKSPACE=$(mktemp -d)
cd "$WORKSPACE"
git init && git commit --allow-empty -m "init"
/path/to/boundline init
mkdir -p src/domain
echo 'fn main() {}' > src/domain/lib.rs
git add . && git commit -m "rust change"
/path/to/boundline goal --goal "test"
/path/to/boundline council adjudicate
```

Expected: `rust-guardian` activated, `docs-consistency-guardian` skipped.

## Scenario 2: Documentation Change Skips Runtime Guardians

```bash
mkdir -p docs
echo '# Test' > docs/test.md
git add . && git commit -m "docs change"
/path/to/boundline council adjudicate
```

Expected: Only `docs-consistency-guardian` and `release-surface-guardian` activated.

## Scenario 3: Invalid Ruleset Fails Closed

```bash
echo '[[rules]]
id = "bad"
stages = ["run"]
files = ["**/*.rs"]
activate = ["rust-guardian"]
skip = ["rust-guardian"]' > .boundline/guardian-rules.toml
/path/to/boundline council adjudicate
```

Expected: Error explaining contradictory rule for `rust-guardian`.

## Scenario 4: Missing Mandatory Guardian Blocks Council

> **Note**: This scenario requires a guardian implementation to be available at runtime.
> Mark this test `#[ignore]` until guardian implementations are wired.

```bash
# Use built-in rules; simulate unavailable mandatory guardian
/path/to/boundline council adjudicate
```

Expected: If a mandatory guardian is unavailable, outcome is `blocked`.

## Scenario 5: JSON Output

```bash
/path/to/boundline council adjudicate --json | jq '.outcome'
```

Expected: Valid JSON with `outcome` field.

## Validation Commands

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --test unit
cargo test --test contract
cargo test --test integration
```
