# CFDL Engine Accuracy Validation

This directory contains tooling to run each example model via the CLI, capture results, recompute key metrics from the engine output, and compare to detect discrepancies. The output is a report summarizing accuracy and any issues.

For recommended next steps to improve engine accuracy (tests, golden locking, independent recomputation, scenarios/Monte Carlo, CI), see [docs/engine_accuracy_next_steps.md](../docs/engine_accuracy_next_steps.md).

## Quick run

From the repository root:

```bash
./tools/engine-accuracy/run_all.sh
```

This will:

1. Discover all example runs (from `examples/` and `examples/language_tutorial/`) and write `example_runs.json`.
2. For each run: compile the model, run the engine, and save results to `out/<slug>_<run_label>.results.json`. Failures are recorded in `run_failures.json`.
3. Validate each results file by recomputing `model.total` and `model.npv` from `deterministic.series` and comparing to `deterministic.metrics` (tolerance 1e-6). Stream totals are also checked. Output: `validation_report.json`.
4. Generate `ENGINE_ACCURACY_REPORT.md` with summary, methodology, accuracy, issues, failures, and recommendations.

## Prerequisites

- Python 3 (stdlib only for scripts).
- CFDL CLI: built with `cargo build -p cfdl-cli` or set `CFDL_BIN` to your binary.

## Steps (manual)

- **Discover runs**: `python3 tools/engine-accuracy/discover_runs.py` → updates `example_runs.json`.
- **Run examples**: `python3 tools/engine-accuracy/run_examples.py` → populates `out/`, writes `run_failures.json` on any compile/run failure.
- **Validate**: `python3 tools/engine-accuracy/validate_results.py` → writes `validation_report.json`.
- **Report**: `python3 tools/engine-accuracy/generate_report.py` → writes `ENGINE_ACCURACY_REPORT.md`.

## Interpreting the report

- **Summary**: How many runs were attempted, succeeded, validated, and matched.
- **Methodology**: What is recomputed (model.total from net cash flow series, model.npv with period-end discounting) and the tolerance used.
- **Accuracy**: Percentage of validated runs where all checked metrics matched.
- **Issues**: Table of mismatches (expected vs actual) for follow-up; root causes may be rounding, formula alignment, or engine bugs.
- **Failures**: Runs that failed to compile or run (e.g. missing CEL support, model errors), with pointers to `run_failures.json`.
- **Recommendations**: Suggestions for tests or fixes.

## Adding new examples

1. Add the example under `examples/` (with `model.cfdl` and optionally `run.json` or `run.base.json`/`run.stress.json`).
2. If the example uses pack contracts, ensure its path is listed in `discover_runs.py` in `PACKS_REQUIRED` (paths under `examples/` that need `--packs packs`).
3. Re-run `discover_runs.py` (or `run_all.sh`); the new run(s) will be included automatically.

## Files

| File | Purpose |
|------|--------|
| `discover_runs.py` | Scans examples, emits `example_runs.json`. |
| `example_runs.json` | Catalog of (model_root, run_label, config_path, packs_dir). |
| `run_examples.py` | Runs CLI for each catalog entry; writes `out/*.results.json`, `run_failures.json`. |
| `validate_results.py` | Recomputes model.total and model.npv (and stream totals), compares to metrics; writes `validation_report.json`. |
| `generate_report.py` | Reads validation and failures, writes `ENGINE_ACCURACY_REPORT.md`. |
| `run_all.sh` | Single entry: discover → run → validate → report. |
| `out/` | Captured results JSON per run. |
| `run_failures.json` | Compile/run failures (run_id, stage, exit_code, stderr). |
| `validation_report.json` | Per-file validation result (match/mismatch, issues). |
| `ENGINE_ACCURACY_REPORT.md` | Human-readable accuracy report. |
