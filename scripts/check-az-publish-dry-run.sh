#!/usr/bin/env bash
set -euo pipefail

workspace_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$workspace_root"

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 1
fi

packages=()
if [[ "$#" -gt 0 ]]; then
  while [[ "$#" -gt 0 ]]; do
    packages+=("$1")
    shift
  done
else
  while IFS= read -r package; do
    packages+=("$package")
  done < <(
    cargo metadata --no-deps --format-version 1 |
      jq -r '
        .packages[]
        | select(.name | startswith("az-"))
        | select(.manifest_path | contains("/crates/"))
        | .name
      ' |
      sort
  )
fi

failed=0
for package in "${packages[@]}"; do
  echo "==> cargo publish -p ${package} --dry-run --allow-dirty"
  if ! cargo publish -p "$package" --dry-run --allow-dirty; then
    failed=1
  fi
done

exit "$failed"
