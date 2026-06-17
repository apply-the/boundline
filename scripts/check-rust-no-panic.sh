#!/bin/sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

if [ -f rust-toolchain.toml ]; then
  repo_toolchain=$(awk -F'"' '/^channel = / { print $2; exit }' rust-toolchain.toml)
  if [ -n "$repo_toolchain" ]; then
    export RUSTUP_TOOLCHAIN="$repo_toolchain"
  fi
fi

cargo clippy --workspace --lib --bins --all-features -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::todo \
  -D clippy::unimplemented \
  -D clippy::unreachable