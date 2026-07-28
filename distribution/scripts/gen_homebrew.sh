#!/usr/bin/env bash
# Generate a filled Homebrew formula from built release assets.
#
# Usage: gen_homebrew.sh <version> <release-assets-dir> <output.rb>
# Expects the CLI binaries in <release-assets-dir>:
#   cfdl-darwin-arm64, cfdl-darwin-x64, cfdl-linux-x64
#
# Build-only: this writes a formula and (optionally) you run `brew audit` /
# `brew install --formula <output.rb>` locally. Pushing to a Homebrew tap is a
# separate, human-approved step.
set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "usage: $0 <version> <release-assets-dir> <output.rb>" >&2
  exit 1
fi

VERSION="$1"
ASSETS="$2"
OUT="$3"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE="${SCRIPT_DIR}/../homebrew/cfdl.rb"

sha() {
  local file="${ASSETS}/$1"
  if [[ ! -f "${file}" ]]; then
    echo "missing release asset: ${file}" >&2
    exit 1
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file}" | awk '{print $1}'
  else
    shasum -a 256 "${file}" | awk '{print $1}'
  fi
}

SHA_DARWIN_ARM64="$(sha cfdl-darwin-arm64)"
SHA_DARWIN_X64="$(sha cfdl-darwin-x64)"
SHA_LINUX_X64="$(sha cfdl-linux-x64)"

sed \
  -e "s/__VERSION__/${VERSION}/g" \
  -e "s/__SHA_DARWIN_ARM64__/${SHA_DARWIN_ARM64}/g" \
  -e "s/__SHA_DARWIN_X64__/${SHA_DARWIN_X64}/g" \
  -e "s/__SHA_LINUX_X64__/${SHA_LINUX_X64}/g" \
  "${TEMPLATE}" > "${OUT}"

echo "wrote ${OUT}"
