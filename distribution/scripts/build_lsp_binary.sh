#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <target-triple> <output-file>" >&2
  exit 1
fi

TARGET_TRIPLE="$1"
OUTPUT_FILE="$2"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BIN_NAME="cfdl-lsp"

if [[ "${TARGET_TRIPLE}" == *"windows"* ]]; then
  BIN_NAME="cfdl-lsp.exe"
fi

pushd "${REPO_ROOT}" >/dev/null
cargo build -p cfdl-lsp --release --target "${TARGET_TRIPLE}"
SOURCE_PATH="${REPO_ROOT}/target/${TARGET_TRIPLE}/release/${BIN_NAME}"
if [[ ! -f "${SOURCE_PATH}" ]]; then
  echo "binary not found at ${SOURCE_PATH}" >&2
  exit 1
fi
mkdir -p "$(dirname "${OUTPUT_FILE}")"
cp "${SOURCE_PATH}" "${OUTPUT_FILE}"
popd >/dev/null

echo "wrote ${OUTPUT_FILE}"
