#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: sync-homebrew-tap.sh --formula <PATH> [--tap-root <DIR>] [--artifact-output <PATH>]
EOF
}

formula=""
tap_root=""
artifact_output=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --formula)
      formula="${2:-}"
      shift 2
      ;;
    --tap-root)
      tap_root="${2:-}"
      shift 2
      ;;
    --artifact-output)
      artifact_output="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 64
      ;;
  esac
done

if [[ -z "$formula" ]]; then
  usage >&2
  exit 64
fi

if [[ ! -f "$formula" ]]; then
  echo "Formula file not found: $formula" >&2
  exit 65
fi

if [[ -n "$tap_root" ]]; then
  destination_dir="$tap_root/Formula"
  destination_path="$destination_dir/boundline.rb"
  mkdir -p "$destination_dir"

  status="updated"
  if [[ -f "$destination_path" ]] && cmp -s "$formula" "$destination_path"; then
    status="noop"
  else
    cp "$formula" "$destination_path"
  fi

  printf 'STATUS=%s\n' "$status"
  printf 'FORMULA_PATH=%s\n' "$destination_path"
  exit 0
fi

reported_path="$formula"
if [[ -n "$artifact_output" ]]; then
  mkdir -p "$(dirname "$artifact_output")"
  cp "$formula" "$artifact_output"
  reported_path="$artifact_output"
fi

printf 'STATUS=artifact-only\n'
printf 'FORMULA_PATH=%s\n' "$reported_path"