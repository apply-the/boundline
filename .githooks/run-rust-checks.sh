#!/bin/sh
set -eu

hook_name=${1:-hook}

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

if [ -f rust-toolchain.toml ]; then
  repo_toolchain=$(awk -F'"' '/^channel = / { print $2; exit }' rust-toolchain.toml)
  if [ -n "$repo_toolchain" ]; then
    export RUSTUP_TOOLCHAIN="$repo_toolchain"
  fi
fi

step_index=1
step_total=4
case "$hook_name" in
  pre-commit)
    step_total=1
    ;;
  pre-push)
    step_total=5
    ;;
  *)
    step_total=5
    ;;
esac

run_step() {
  step_label=$1
  failure_hint=$2
  shift 2

  printf '%s\n' "[$hook_name] [$step_index/$step_total] Running: $step_label"
  if "$@"; then
    printf '%s\n' "[$hook_name] [$step_index/$step_total] OK: $step_label"
  else
    status=$?
    printf '%s\n' "[$hook_name] [$step_index/$step_total] FAILED: $step_label" >&2
    printf '%s\n' "[$hook_name] Hint: $failure_hint" >&2
    exit "$status"
  fi

  step_index=$((step_index + 1))
}

printf '%s\n' "[$hook_name] Running Rust quality checks in $repo_root"

run_step \
  "cargo fmt --all -- --check" \
  "Run 'cargo fmt', restage any formatting changes, then retry." \
  cargo fmt --all -- --check

if [ "$hook_name" != "pre-commit" ]; then
  run_step \
    "sh scripts/check-rust-no-panic.sh" \
    "Run 'sh scripts/check-rust-no-panic.sh' and replace the reported panic-prone calls outside main.rs and test code." \
    sh scripts/check-rust-no-panic.sh

  run_step \
    "cargo clippy --workspace --all-targets --all-features -- -D warnings" \
    "Run 'cargo clippy --workspace --all-targets --all-features -- -D warnings' and fix the reported warnings." \
    cargo clippy --workspace --all-targets --all-features -- -D warnings

  run_step \
    "cargo nextest run --workspace --all-features" \
    "Install cargo-nextest if needed with 'cargo install cargo-nextest', then rerun 'cargo nextest run --workspace --all-features'." \
    cargo nextest run --workspace --all-features

  run_step \
    "cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info" \
    "Install cargo-llvm-cov if needed with 'cargo install cargo-llvm-cov', then rerun 'cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info'." \
    cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
fi

printf '%s\n' "[$hook_name] All Rust quality checks passed."