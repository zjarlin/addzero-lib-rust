#!/usr/bin/env bash
set -euo pipefail

workspace_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest_path="${workspace_root}/Cargo.toml"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/set-workspace-date-version.sh
  ./scripts/set-workspace-date-version.sh --date 2026-05-10
  ./scripts/set-workspace-date-version.sh --version 2026.05.10
  ./scripts/set-workspace-date-version.sh --print --date 2026-05-10

Behavior:
  - Default output format is Cargo-safe YYYY.M.D without leading zeroes.
  - Updates [workspace.package].version.
  - Updates az-* entries in [workspace.dependencies] from =old to =new.
EOF
}

print_only=0
input_date=""
input_version=""

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --print)
      print_only=1
      shift
      ;;
    --date)
      [[ "$#" -ge 2 ]] || {
        echo "--date requires YYYY-MM-DD" >&2
        exit 1
      }
      input_date="$2"
      shift 2
      ;;
    --version)
      [[ "$#" -ge 2 ]] || {
        echo "--version requires X.Y.Z" >&2
        exit 1
      }
      input_version="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

normalize_date_version() {
  local year month day

  if [[ -n "$input_version" ]]; then
    [[ "$input_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
      echo "Version must look like X.Y.Z: $input_version" >&2
      exit 1
    }
    IFS=. read -r year month day <<< "$input_version"
  elif [[ -n "$input_date" ]]; then
    [[ "$input_date" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || {
      echo "Date must look like YYYY-MM-DD: $input_date" >&2
      exit 1
    }
    IFS=- read -r year month day <<< "$input_date"
  else
    year="$(date +%Y)"
    month="$(date +%m)"
    day="$(date +%d)"
  fi

  month=$((10#$month))
  day=$((10#$day))

  printf '%s.%s.%s\n' "$year" "$month" "$day"
}

next_version="$(normalize_date_version)"

if [[ "$print_only" -eq 1 ]]; then
  printf '%s\n' "$next_version"
  exit 0
fi

current_version="$(
  awk '
    /^\[workspace\.package\]$/ { in_section = 1; next }
    /^\[/ { if (in_section) exit }
    in_section && /^version = "/ {
      gsub(/^version = "/, "")
      gsub(/"$/, "")
      print
      exit
    }
  ' "$manifest_path"
)"

[[ -n "$current_version" ]] || {
  echo "Failed to locate [workspace.package].version in $manifest_path" >&2
  exit 1
}

if [[ "$current_version" == "$next_version" ]]; then
  printf 'Workspace version already at %s\n' "$next_version"
  exit 0
fi

python3 - "$manifest_path" "$next_version" <<'PY'
from __future__ import annotations

import re
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
next_version = sys.argv[2]
lines = manifest_path.read_text(encoding="utf-8").splitlines(keepends=True)

in_workspace_package = False
updated_workspace_version = False
updated_internal_versions = 0

for index, line in enumerate(lines):
    newline = "\n" if line.endswith("\n") else ""
    body = line[:-1] if newline else line
    stripped = body.strip()

    if stripped.startswith("[") and stripped.endswith("]"):
        in_workspace_package = stripped == "[workspace.package]"

    if in_workspace_package and body.startswith("version = \""):
        body = f'version = "{next_version}"'
        updated_workspace_version = True
        lines[index] = body + newline
        continue

    matched = re.match(
        r'^(az-[A-Za-z0-9-]+\s*=\s*\{\s*version\s*=\s*")=[^"]+(".*)$',
        body,
    )
    if matched:
        body = f"{matched.group(1)}={next_version}{matched.group(2)}"
        updated_internal_versions += 1
        lines[index] = body + newline

if not updated_workspace_version:
    raise SystemExit(f"Failed to update [workspace.package].version in {manifest_path}")

manifest_path.write_text("".join(lines), encoding="utf-8")
PY

printf 'Updated workspace version: %s -> %s\n' "$current_version" "$next_version"
