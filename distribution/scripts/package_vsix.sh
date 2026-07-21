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
BUNDLED_DIR="${EXT_DIR}/bundled"

cleanup() {
  rm -rf "${BUNDLED_DIR}"
}

trap cleanup EXIT

mkdir -p "${BUNDLED_DIR}/docs"
mkdir -p "${BUNDLED_DIR}/packs"
cp "${REPO_ROOT}/docs/09_user_guide.md" "${BUNDLED_DIR}/docs/LANGUAGE_GUIDE.md"
cp "${REPO_ROOT}/distribution/install-configure.md" "${BUNDLED_DIR}/docs/install-configure.md"
cp -R "${REPO_ROOT}/examples/language_tutorial" "${BUNDLED_DIR}/docs/language_tutorial"
cp -R "${REPO_ROOT}/packs/cre" "${BUNDLED_DIR}/packs/cre"
cp -R "${REPO_ROOT}/packs/opco" "${BUNDLED_DIR}/packs/opco"

pushd "${EXT_DIR}" >/dev/null
npm ci
npm run lint
npx @vscode/vsce package --out "${OUTPUT_VSIX}"
popd >/dev/null

echo "wrote ${OUTPUT_VSIX}"
