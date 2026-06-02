#!/usr/bin/env bash
set -e

# Navigate to the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "Starting Boundline VitePress dev server..."
npm run site:dev
