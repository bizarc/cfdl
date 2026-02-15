#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <output-vsix-path>" >&2
  exit 1
fi

OUTPUT_VSIX="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
EXT_DIR="${REPO_ROOT}/editors/vscode"

pushd "${EXT_DIR}" >/dev/null
npm ci
npm run lint
npx @vscode/vsce package --out "${OUTPUT_VSIX}"
popd >/dev/null

echo "wrote ${OUTPUT_VSIX}"
