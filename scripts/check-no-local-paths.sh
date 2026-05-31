#!/bin/sh
set -eu

usage() {
  printf '%s\n' "Usage: $0 [--cached|--tracked]" >&2
  exit 2
}

mode=${1:---tracked}
case "$mode" in
  --cached|--staged|--tracked)
    ;;
  *)
    usage
    ;;
esac

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  printf '%s\n' "check-no-local-paths.sh must run inside a git repository" >&2
  exit 1
fi

local_path_pattern='(/Users/[[:alnum:]._-]+/[^[:space:]]+|/home/[[:alnum:]._-]+/[^[:space:]]+|C:\\Users\\[[:alnum:]._-]+\\[^[:space:]]+|C:/Users/[[:alnum:]._-]+/[^[:space:]]+|/private/var/folders/[[:alnum:]._/-]+|/var/folders/[[:alnum:]._/-]+)'

tmp_matches=$(mktemp "${TMPDIR:-/tmp}/boundline-local-paths.XXXXXX")
cleanup() {
  rm -f "$tmp_matches"
}
trap cleanup EXIT HUP INT TERM

search_cached_paths() {
  old_ifs=$IFS
  IFS='
'
  set -f
  set -- $(git diff --cached --name-only --diff-filter=ACMR)
  set +f
  IFS=$old_ifs

  : >"$tmp_matches"

  if [ "$#" -eq 0 ]; then
    return 1
  fi

  found_match=1
  for path in "$@"; do
    set +e
    git grep --cached -nI -E "$local_path_pattern" -- "$path" >>"$tmp_matches" 2>/dev/null
    path_exit=$?
    set -e
    case "$path_exit" in
      0)
        found_match=0
        ;;
      1)
        ;;
      *)
        return "$path_exit"
        ;;
    esac
  done

  if [ "$found_match" -eq 0 ]; then
    cat "$tmp_matches"
    return 0
  fi

  return 1
}

set +e
case "$mode" in
  --cached|--staged)
    matches=$(search_cached_paths)
    grep_exit=$?
    ;;
  --tracked)
    matches=$(git grep -nI -E "$local_path_pattern" 2>/dev/null)
    grep_exit=$?
    ;;
esac
set -e

case "$grep_exit" in
  0)
    printf '%s\n' 'Found machine-local absolute paths in tracked content:' >&2
    printf '%s\n' "$matches" >&2
    printf '%s\n' 'Replace them with repo-relative paths, sibling repo names, URLs, or stable placeholders before committing.' >&2
    exit 1
    ;;
  1)
    exit 0
    ;;
  *)
    printf '%s\n' 'Failed to scan tracked content for machine-local absolute paths.' >&2
    exit "$grep_exit"
    ;;
esac