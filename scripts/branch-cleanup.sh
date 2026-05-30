#!/usr/bin/env bash
set -euo pipefail

current_branch="$(git branch --show-current)"

branches_to_delete="$(
  git branch --format='%(refname:short)' \
    | grep -v -x "$current_branch" \
    | grep -v -x "main" || true
)"

if [[ -z "$branches_to_delete" ]]; then
  echo "No local branches to delete."
  exit 0
fi

echo "Branches to delete:"
echo "$branches_to_delete"

read -r -p "Delete these branches? [y/N] " confirm

if [[ "$confirm" == "y" || "$confirm" == "Y" ]]; then
  echo "$branches_to_delete" | xargs -r git branch -d
else
  echo "Aborted."
fi