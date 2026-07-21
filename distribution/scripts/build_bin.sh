#!/usr/bin/env bash
set -euo pipefail

# Generic release-binary builder: build_bin.sh <crate> <target-triple> <output-file>
# (build_lsp_binary.sh remains for the cfdl-lsp-specific path; new jobs use this.)

if [[ $# -lt 3 ]]; then
  echo "usage: $0 <crate> <target-triple> <output-file>" >&2
  exit 1
fi

CRATE="$1"
TARGET_TRIPLE="$2"
OUTPUT_FILE="$3"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Binary name matches the crate's [[bin]] name; for cfdl-cli the binary is `cfdl`.
BIN_NAME="${CRATE}"
if [[ "${CRATE}" == "cfdl-cli" ]]; then
  BIN_NAME="cfdl"
fi
if [[ "${TARGET_TRIPLE}" == *"windows"* ]]; then
  BIN_NAME="${BIN_NAME}.exe"
fi

pushd "${REPO_ROOT}" >/dev/null
cargo build -p "${CRATE}" --release --target "${TARGET_TRIPLE}"
SOURCE_PATH="${REPO_ROOT}/target/${TARGET_TRIPLE}/release/${BIN_NAME}"
if [[ ! -f "${SOURCE_PATH}" ]]; then
  echo "binary not found at ${SOURCE_PATH}" >&2
  exit 1
fi
mkdir -p "$(dirname "${OUTPUT_FILE}")"
cp "${SOURCE_PATH}" "${OUTPUT_FILE}"
chmod +x "${OUTPUT_FILE}"
popd >/dev/null

echo "built ${OUTPUT_FILE}"
