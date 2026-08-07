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

# THE BUNDLE IS SERVED TO EVERY PLAYGROUND VISITOR, so its size is worth
# watching. What is NOT worth doing is failing the deploy every time the
# language legitimately grows.
#
# An absolute ceiling was tried and did not hold: 640 -> 680 -> 740, on top of
# four earlier raises. Every one was deliberate and documented, which is the
# tripwire working exactly as designed — and is also the evidence that the
# number cannot stay right in a language still gaining surface. A ceiling that
# always moves is a speed bump, not a limit, and each move cost a failed deploy
# and a commit.
#
# It also aimed at the wrong risk. Steady growth from real work — an ontology
# loader, fifteen diagnostics, a transition log — is expected and fine. The
# failure worth catching is a JUMP: a large table embedded by accident, or a
# dependency that quietly pulls in something huge. An absolute number cannot
# tell those apart. A delta can.
#
# So: record the size, compare against the recorded baseline, and fail only on
# a jump. Steady growth passes with the number printed every time.
#
# THE COMPARISON IS CI-ONLY, and that is not caution — it is a measurement
# fact. The same commit built here and on the runner differs by ~8% gzipped
# (778 KB local against 717 KB in CI), because rustc, binaryen and the debug
# info they emit vary by machine. That is over half the jump threshold before
# anything has changed. A local build therefore REPORTS its size and stops; the
# baseline is a CI number and only CI is entitled to compare against it.
#
# Worth recording what does NOT shrink it, since it is the obvious first guess:
# the pack TOMLs are include_str!-embedded, so their comments do ship — but
# cutting ~2 KB of comment prose recovered 0 KB gzipped, because gzip was
# already collapsing repetitive text, and all four pack ontologies together are
# only 7 KB gzipped. The lever is the module, not the documentation.
JUMP_PCT=15
BASELINE_FILE="${REPO_ROOT}/site/.wasm-size-baseline"

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
BASELINE_KB=""
if [[ -f "${BASELINE_FILE}" ]]; then
  BASELINE_KB="$(tr -cd '0-9' < "${BASELINE_FILE}")"
fi

if [[ "${CI:-}" != "true" ]]; then
  echo "build-wasm: cfdl_wasm_bg.wasm  ${RAW_KB} KB raw / ${GZIP_KB} KB gzipped (local build; baseline ${BASELINE_KB:-unset} KB is a CI figure and not comparable)"
  exit 0
fi

if [[ -z "${BASELINE_KB}" ]]; then
  # First run, or the baseline was removed. Record and carry on rather than
  # failing on a comparison there is nothing to compare against.
  echo "${GZIP_KB}" > "${BASELINE_FILE}"
  echo "build-wasm: cfdl_wasm_bg.wasm  ${RAW_KB} KB raw / ${GZIP_KB} KB gzipped (baseline recorded)"
  exit 0
fi

DELTA_KB=$(( GZIP_KB - BASELINE_KB ))
# Integer percentage, rounded toward zero; the threshold is coarse on purpose.
DELTA_PCT=$(( DELTA_KB * 100 / BASELINE_KB ))
SIGN=""
if (( DELTA_KB >= 0 )); then SIGN="+"; fi

echo "build-wasm: cfdl_wasm_bg.wasm  ${RAW_KB} KB raw / ${GZIP_KB} KB gzipped (baseline ${BASELINE_KB} KB, ${SIGN}${DELTA_PCT}%)"

if (( DELTA_PCT > JUMP_PCT )); then
  echo "build-wasm: SIZE JUMP — ${GZIP_KB} KB gzipped is ${SIGN}${DELTA_PCT}% over the ${BASELINE_KB} KB baseline (threshold ${JUMP_PCT}%)." >&2
  echo "  A jump this size is usually an accident: a large table embedded by" >&2
  echo "  mistake, or a dependency pulling something in. Check what grew." >&2
  echo "  If it is deliberate, update site/.wasm-size-baseline in the same commit." >&2
  exit 1
fi
