#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <output-docs-tar-gz-path>" >&2
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

# Ship the whole docs tree rather than a hand-listed subset: the previous list
# named three files that were renamed in the docs restructure, and the archive
# had been failing to build on every tagged release since.
tar \
  "${TAR_OPTS[@]}" \
  --exclude=".DS_Store" \
  -C "${REPO_ROOT}" \
  docs \
  distribution/install-configure.md \
  examples/language_tutorial

echo "wrote ${OUTPUT_ARCHIVE}"
