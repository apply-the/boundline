#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_root"

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "install-hooks.sh must be run inside a git repository" >&2
  exit 1
fi

git config core.hooksPath .githooks
find .githooks -maxdepth 1 -type f -exec chmod +x {} +
echo "Installed git hooks from $repo_root/.githooks"