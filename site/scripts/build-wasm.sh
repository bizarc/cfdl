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
#
# 600 -> 640 when stream categories landed. The bundle had been sitting at
# exactly 600/600, so the next addition of any kind was going to trip this;
# categories added ~9 KB raw / 3 KB gzipped between the category data itself,
# the load-time validation and the parser arm.
#
# Worth recording what did NOT work, since it is the obvious first guess: the
# pack TOMLs are include_str!-embedded, so their comments do ship — but cutting
# ~2 KB of comment prose out of them recovered 0 KB gzipped, because gzip was
# already collapsing repetitive text. If this needs shrinking for real, the
# lever is the module, not the documentation.
#
# 640 -> 680 when statement declarations landed. That is the SECOND raise in one
# working session, which is the tripwire doing its job: the reporting work has
# added ~50 KB gzipped across categories, subtotals, provenance and statements,
# and raising the number each time is not a strategy. The structural answer is
# site/DEPLOY.md — build the bundle in CI and stop committing it, which makes
# the budget a CI concern rather than something a developer trips over. Until
# that lands, raise it deliberately and keep noticing.
BUDGET_KB=680

if [[ "${SKIP_WASM:-0}" == "1" ]]; then
  echo "build-wasm: SKIP_WASM=1 — using the committed bundle"
  exit 0
fi

# Pinned so every build of the committed bundle comes from one toolchain.
#
# Held at 0.13.1 on purpose, not out of neglect. wasm-pack 0.15.0 needs rustc
# 1.88 or newer (sysinfo and time both require it), and this repo tracks the
# `stable` channel rather than pinning a release. Moving to 0.15.0 therefore
# means raising the compiler for the whole workspace, which is a separate
# decision with its own blast radius — CI runs clippy with `-D warnings`, so a
# compiler jump brings new lints with it.
#
# Bump both together, deliberately, and rebuild the bundle in the same commit.
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
# wasm-pack nags about a newer release on every single build. We know, and the
# comment on WASM_PACK_VERSION says why we are not taking it.
#
# Filtered by capturing the log rather than piping through grep. Piping needs a
# `|| true` to tolerate grep matching nothing, and that swallows a failed build
# — an earlier revision of this script exited 0 on a compile error because of
# exactly that. On failure the log is replayed untouched; only a successful
# build is filtered, and only that one line. `--log-level error` was the other
# option and would have hidden real warnings too.
WASM_LOG="$(mktemp)"
trap 'rm -f "${WASM_LOG}"' EXIT

if ! wasm-pack build "${REPO_ROOT}/crates/cfdl-wasm" \
  --release \
  --target web \
  --out-dir "${OUT_DIR}" \
  --out-name cfdl_wasm >"${WASM_LOG}" 2>&1
then
  cat "${WASM_LOG}" >&2
  echo "build-wasm: wasm-pack failed." >&2
  exit 1
fi
grep -v "There's a newer version of wasm-pack available" "${WASM_LOG}" || true

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
