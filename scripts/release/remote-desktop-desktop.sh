#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <version>"
  echo "example: $0 0.1.0"
  exit 1
fi

VERSION="$1"
TAG="remote-desktop-desktop-v${VERSION}"

git tag "${TAG}"
git push origin "${TAG}"

echo "Triggered GitHub Actions release for ${TAG}"
echo "Check: https://github.com/zjarlin/addzero-lib-rust/actions/workflows/release-remote-desktop.yml"
