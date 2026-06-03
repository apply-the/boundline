#!/usr/bin/env bash
set -euo pipefail

# Ensure we are in a rust workspace root with Cargo.toml
if [ ! -f Cargo.toml ]; then
  echo "Error: Cargo.toml not found in the current directory." >&2
  exit 1
fi

if [ ! -d docs ]; then
  echo "Warning: docs/ directory not found. Skipping."
  exit 0
fi

# Extract version from Cargo.toml
VERSION=$(grep -A 10 "\[workspace.package\]" Cargo.toml | grep -E '^version\s*=\s*' | cut -d '"' -f 2 || true)
if [ -z "$VERSION" ]; then
  # Fallback to standard [package]
  VERSION=$(grep -A 10 "\[package\]" Cargo.toml | grep -E '^version\s*=\s*' | cut -d '"' -f 2 || true)
fi

if [ -z "$VERSION" ]; then
  echo "Error: Could not extract version from Cargo.toml." >&2
  exit 1
fi

echo "Updating documentation references in docs/ to version: $VERSION"

# Find and update markdown files recursively using perl for portability across macOS and Linux
find docs -type f -name "*.md" | while read -r file; do
  # Check if the file contains any of our target patterns to avoid unnecessary writes
  if grep -q -E "blob/[0-9]+\.[0-9]+\.[0-9]+|tree/[0-9]+\.[0-9]+\.[0-9]+|Canon [0-9]+\.[0-9]+\.[0-9]+|Boundline [0-9]+\.[0-9]+\.[0-9]+" "$file"; then
    perl -pi -e "s|blob/\d+\.\d+\.\d+|blob/$VERSION|g" "$file"
    perl -pi -e "s|tree/\d+\.\d+\.\d+|tree/$VERSION|g" "$file"
    perl -pi -e "s|Canon \d+\.\d+\.\d+|Canon $VERSION|g" "$file"
    perl -pi -e "s|Boundline \d+\.\d+\.\d+|Boundline $VERSION|g" "$file"
    echo "  Updated: $file"
  fi
done

echo "Done!"
