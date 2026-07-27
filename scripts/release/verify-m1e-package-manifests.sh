#!/usr/bin/env bash
set -euo pipefail

readonly expected_version="0.90.0"
readonly expected_msrv="1.96.0"
readonly public_package="boundline-protocol"

metadata="$(cargo metadata --no-deps --format-version 1)"

jq -e \
  --arg version "$expected_version" \
  '[.packages[] | select(.name | startswith("boundline")) | .version]
   | length == 5 and all(. == $version)' \
  <<<"$metadata" >/dev/null

jq -e \
  --arg package "$public_package" \
  --arg version "$expected_version" \
  --arg msrv "$expected_msrv" \
  '.packages[]
   | select(.name == $package)
   | .version == $version
     and .edition == "2024"
     and .rust_version == $msrv
     and .license == "MIT"
     and (.description | length > 0)
     and .publish == null' \
  <<<"$metadata" >/dev/null

jq -e \
  --arg version "=$expected_version" \
  '[.packages[].dependencies[]
    | select(.name | startswith("boundline-"))
    | .req]
   | length > 0 and all(. == $version)' \
  <<<"$metadata" >/dev/null

package_list="$(cargo package --locked --list -p "$public_package" --allow-dirty)"
if grep -Eq '(^|/)(target|\\.boundline|lcov\\.info|coverage)(/|$)' <<<"$package_list"; then
  echo "package list contains excluded release output" >&2
  exit 1
fi

printf 'Boundline M1E package manifest checks passed.\n'
