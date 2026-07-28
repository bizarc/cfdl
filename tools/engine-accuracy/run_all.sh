#!/usr/bin/env bash
# One-shot: discover runs -> run examples -> validate -> generate report.
# Run from repo root. Optional: CFDL_BIN=/path/to/cfdl
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
echo "[engine-accuracy] Discovering example runs..."
python3 tools/engine-accuracy/discover_runs.py
echo "[engine-accuracy] Running all example models..."
python3 tools/engine-accuracy/run_examples.py
echo "[engine-accuracy] Validating results..."
python3 tools/engine-accuracy/validate_results.py
echo "[engine-accuracy] Generating report..."
python3 tools/engine-accuracy/generate_report.py
echo "[engine-accuracy] Done. Report: tools/engine-accuracy/ENGINE_ACCURACY_REPORT.md"
