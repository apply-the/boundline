#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

export DISABLE_AUTO_UPDATE=true

cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.workspace.info "$@"
cargo llvm-cov -p boundline-core --lib --all-features --lcov --output-path lcov.boundline-core.info -- --test-threads=1
cargo llvm-cov -p boundline-adapters --lib --all-features --lcov --output-path lcov.boundline-adapters.info -- --test-threads=1
cargo llvm-cov -p boundline-cli --lib --all-features --lcov --output-path lcov.boundline-cli.info -- --test-threads=1
python3 scripts/common/coverage/aggregate_lcov.py \
  --output-lcov lcov.info \
  lcov.workspace.info lcov.boundline-core.info lcov.boundline-adapters.info lcov.boundline-cli.info
