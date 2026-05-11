#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo test --test assistant_plugin_packages

echo "PASS: Boundline assistant plugin packages are valid."
