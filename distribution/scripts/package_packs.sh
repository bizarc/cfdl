#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <output-packs-tar-gz-path>" >&2
  exit 1
fi

OUTPUT_ARCHIVE="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

mkdir -p "$(dirname "${OUTPUT_ARCHIVE}")"

TAR_OPTS=(-czf "${OUTPUT_ARCHIVE}")
if tar --help 2>/dev/null | grep -q -- "--sort"; then
  TAR_OPTS=(
    --sort=name
    --mtime='UTC 1970-01-01'
    --owner=0
    --group=0
    --numeric-owner
    -czf "${OUTPUT_ARCHIVE}"
  )
fi

# Ship every pack that has a manifest, discovered rather than listed: the
# previous hardcoded list named only cre and opco, so the energy and credit
# packs were absent from every release while the docs promised all four.
# testpack is a compiler fixture, not a shipped pack.
PACK_DIRS=()
for manifest in "${REPO_ROOT}"/packs/*/pack.toml; do
  pack_dir="$(basename "$(dirname "${manifest}")")"
  [[ "${pack_dir}" == "testpack" ]] && continue
  PACK_DIRS+=("packs/${pack_dir}")
done

if [[ ${#PACK_DIRS[@]} -eq 0 ]]; then
  echo "no packs found under ${REPO_ROOT}/packs" >&2
  exit 1
fi

tar \
  --exclude=".DS_Store" \
  "${TAR_OPTS[@]}" \
  -C "${REPO_ROOT}" \
  "${PACK_DIRS[@]}"

echo "wrote ${OUTPUT_ARCHIVE}"
