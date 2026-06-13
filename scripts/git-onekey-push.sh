#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

if ! git diff --quiet || ! git diff --cached --quiet || [ -n "$(git ls-files --others --exclude-standard)" ]; then
  :
else
  echo "No changes to commit."
  exit 0
fi

branch="$(git branch --show-current)"
if [ -z "$branch" ]; then
  echo "Cannot commit from detached HEAD." >&2
  exit 1
fi

remote="${GIT_ONEKEY_REMOTE:-origin}"

echo "Repository: $repo_root"
echo "Branch: $branch"
echo
echo "Changes:"
git status --short
echo

git add -A

changed_files="$(git diff --cached --name-only)"
changed_count="$(printf '%s\n' "$changed_files" | sed '/^$/d' | wc -l | tr -d ' ')"

scope="$(
  printf '%s\n' "$changed_files" |
    awk -F/ '
      NF >= 2 { print $1 "/" $2; next }
      NF == 1 && length($1) > 0 { print $1 }
    ' |
    sort |
    uniq |
    head -n 3 |
    paste -sd ", " -
)"

if [ -z "$scope" ]; then
  scope="workspace"
fi

message="${GIT_ONEKEY_MESSAGE:-chore: update ${scope} (${changed_count} files)}"

echo "Commit message: $message"
git commit -m "$message"

if git rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
  git push
else
  git push -u "$remote" "$branch"
fi
