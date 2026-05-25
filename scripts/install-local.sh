#!/usr/bin/env bash
# install-local.sh — Build a release binary and install it into the active brew
# keg so that /opt/homebrew/bin/boundline resolves to the locally-built version.
#
# Usage:
#   ./scripts/install-local.sh          # build + install
#   ./scripts/install-local.sh --check  # print current binary source and exit

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BREW_KEG="$(brew --prefix boundline 2>/dev/null || true)"
TARGET_BIN="${REPO_ROOT}/target/release/boundline"
KEG_BIN="${BREW_KEG}/bin/boundline"

if [[ "${1:-}" == "--check" ]]; then
  echo "PATH binary : $(which boundline)"
  echo "PATH version: $(boundline --version 2>/dev/null || echo '(unknown)')"
  echo "Keg path    : ${KEG_BIN}"
  echo "Local build : ${TARGET_BIN}"
  if [[ -f "${TARGET_BIN}" ]]; then
    echo "Local built : $(${TARGET_BIN} --version 2>/dev/null || echo '(unknown)')"
  else
    echo "Local built : (not built yet — run without --check)"
  fi
  exit 0
fi

if [[ -z "${BREW_KEG}" || ! -d "${BREW_KEG}" ]]; then
  echo "error: boundline is not installed via brew (brew --prefix boundline failed)" >&2
  echo "       Install it first with: brew install apply-the/tap/boundline" >&2
  exit 1
fi

echo "==> Building release binary…"
cd "${REPO_ROOT}"
cargo build --release --bin boundline

echo "==> Installing into brew keg: ${KEG_BIN}"
sudo cp "${TARGET_BIN}" "${KEG_BIN}"

echo "==> Done. Active version: $(boundline --version 2>/dev/null)"
