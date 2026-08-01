#!/usr/bin/env bash
#
# Builds the CFDL engine to WebAssembly for the playground and the runnable
# docs cells, into site/public/wasm/.
#
# The output is COMMITTED. Vercel's build image has no Rust toolchain, so the
# bundle has to be in the repo; CI rebuilds it and fails on drift, which keeps
# the committed artifact honest.
#
# SKIP_WASM=1 skips the build (for docs-only work when the bundle is already
# present). Missing tooling is a hard failure otherwise — a silently absent
# engine would ship a playground that loads forever.
set -euo pipefail

cd "$(dirname "$0")/.."
SITE_DIR="$(pwd)"
REPO_ROOT="$(cd .. && pwd)"
OUT_DIR="${SITE_DIR}/public/wasm"

# Gzipped budget for the engine module. Raise deliberately, with a note in the
# commit message explaining what grew.
BUDGET_KB=600

if [[ "${SKIP_WASM:-0}" == "1" ]]; then
  echo "build-wasm: SKIP_WASM=1 — using the committed bundle"
  exit 0
fi

# Pinned so every build of the committed bundle comes from one toolchain.
# Bump deliberately and rebuild in the same commit.
WASM_PACK_VERSION=0.13.1

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "build-wasm: wasm-pack not found." >&2
  echo "  Install:  cargo install wasm-pack --version ${WASM_PACK_VERSION}" >&2
  echo "  Or skip:  SKIP_WASM=1 npm run build:wasm" >&2
  exit 1
fi

HAVE_WASM_PACK="$(wasm-pack --version 2>/dev/null | awk '{print $2}')"
if [[ "${HAVE_WASM_PACK}" != "${WASM_PACK_VERSION}" ]]; then
  # A warning, not an error. wasm-pack installs globally, so failing here would
  # push contributors into `cargo install --force`, downgrading the copy every
  # other project on their machine uses — disproportionate for a difference
  # that shows up as bundle size and wasm-bindgen glue, not as wrong numbers.
  # The freshness gates hash SOURCES, so they stay correct either way.
  echo "build-wasm: NOTE — wasm-pack ${HAVE_WASM_PACK} in use; this repo builds with ${WASM_PACK_VERSION}." >&2
  echo "  The bundle will differ in size and glue from the committed one." >&2
  echo "  To match exactly:  cargo install wasm-pack --version ${WASM_PACK_VERSION}" >&2
fi

if ! rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
  echo "build-wasm: the wasm32-unknown-unknown target is missing." >&2
  echo "  Install:  rustup target add wasm32-unknown-unknown" >&2
  exit 1
fi

echo "build-wasm: building cfdl-wasm (release)…"
wasm-pack build "${REPO_ROOT}/crates/cfdl-wasm" \
  --release \
  --target web \
  --out-dir "${OUT_DIR}" \
  --out-name cfdl_wasm

# wasm-pack drops packaging files we don't serve.
rm -f "${OUT_DIR}/package.json" "${OUT_DIR}/README.md" "${OUT_DIR}/.gitignore"

WASM_FILE="${OUT_DIR}/cfdl_wasm_bg.wasm"
if [[ ! -f "${WASM_FILE}" ]]; then
  echo "build-wasm: expected ${WASM_FILE} to exist after the build." >&2
  exit 1
fi

# Record what the bundle was built from. Hashing the *sources* rather than the
# output keeps the check reproducible: wasm-pack, binaryen and the rustc path
# prefixes baked into the module all vary by machine, so byte-comparing two
# honest builds produces false failures. Mirrors write_render_stamp in
# tools/render-notebooks.py.
node "${SITE_DIR}/scripts/wasm-stamp.mjs" --write

RAW_KB=$(( $(wc -c < "${WASM_FILE}") / 1024 ))
GZIP_KB=$(( $(gzip -c "${WASM_FILE}" | wc -c) / 1024 ))
echo "build-wasm: cfdl_wasm_bg.wasm  ${RAW_KB} KB raw / ${GZIP_KB} KB gzipped (budget ${BUDGET_KB} KB)"

if (( GZIP_KB > BUDGET_KB )); then
  echo "build-wasm: OVER BUDGET — ${GZIP_KB} KB gzipped exceeds ${BUDGET_KB} KB." >&2
  echo "  Shrink the module, or raise BUDGET_KB deliberately in this script." >&2
  exit 1
fi
