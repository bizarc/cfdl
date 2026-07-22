#!/usr/bin/env bash
# Build the CFDL WASM playground bundle into docs-site/static/wasm.
# Requires the Rust toolchain, the wasm32-unknown-unknown target, and wasm-pack.
# The output is gitignored (built in CI, not committed).
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "[build-wasm] wasm-pack not found; skipping (playground will be unavailable)." >&2
  exit 0
fi

wasm-pack build "${repo_root}/crates/cfdl-wasm" \
  --target web \
  --out-dir "${repo_root}/docs-site/static/wasm" \
  --out-name cfdl_wasm

echo "[build-wasm] wrote docs-site/static/wasm"
