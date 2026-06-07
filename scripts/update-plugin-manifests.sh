#!/usr/bin/env bash
set -euo pipefail

if [ ! -f Cargo.toml ]; then
  echo "Error: Cargo.toml not found." >&2
  exit 1
fi

VERSION=$(grep -A 10 "\[workspace.package\]" Cargo.toml | grep -E '^version\s*=\s*' | cut -d '"' -f 2 || true)
if [ -z "$VERSION" ]; then
  VERSION=$(grep -A 10 "\[package\]" Cargo.toml | grep -E '^version\s*=\s*' | cut -d '"' -f 2 || true)
fi
if [ -z "$VERSION" ]; then
  echo "Error: Could not extract version from Cargo.toml." >&2
  exit 1
fi

echo "Synchronizing plugin and distribution manifests to version: $VERSION"

update_json_version() {
  local file="$1"
  if [ -f "$file" ]; then
    local old_version
    old_version=$(grep -E '"version"\s*:\s*"[0-9]+\.[0-9]+\.[0-9]+"' "$file" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
    if [ -n "$old_version" ] && [ "$old_version" != "$VERSION" ]; then
      perl -pi -e "s|\"version\"\s*:\s*\"\d+\.\d+\.\d+\"|\"version\": \"$VERSION\"|" "$file"
      echo "  Updated: $file  ($old_version -> $VERSION)"
    fi
  fi
}

update_json_version "assistant/plugin-metadata.json"
update_json_version "assistant/global/manifest.json"
update_json_version ".claude-plugin/manifest.json"
update_json_version ".codex-plugin/plugin.json"
update_json_version ".cursor-plugin/manifest.json"
update_json_version ".copilot-prompts/pack.json"

if [ -f "distribution/channel-metadata.toml" ]; then
  perl -pi -e "s|boundline_version\s*=\s*\"\d+\.\d+\.\d+\"|boundline_version = \"$VERSION\"|g" distribution/channel-metadata.toml
  perl -pi -e "s|version\s*=\s*\"\d+\.\d+\.\d+\"|version = \"$VERSION\"|g" distribution/channel-metadata.toml
  perl -pi -e "s|tag\s*=\s*\"\d+\.\d+\.\d+\"|tag = \"$VERSION\"|g" distribution/channel-metadata.toml
  perl -pi -e "s|/Boundline/\d+\.\d+\.\d+|/Boundline/$VERSION|g" distribution/channel-metadata.toml
  perl -pi -e "s|bundle-\d+\.\d+\.\d+|bundle-$VERSION|g" distribution/channel-metadata.toml
  echo "  Updated: distribution/channel-metadata.toml"
fi

echo ""
echo "Done! Files that may still need manual review after a version bump:"
echo "  tests/contract/canon_reasoning_posture_contract.rs"
echo "  tests/contract/distribution_release_surface_contract.rs"
echo "  specs/061-reasoning-profile-contracts/contracts/"
