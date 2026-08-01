#!/usr/bin/env python3
"""CFDL benchmark harness.

Each case directory (benchmarks/<pack>/<case>/) contains:
  model.cfdl            the CFDL model
  run.json              run configuration
  case.toml             pack name + per-period tolerance
  expected.csv          per-period expectations from an independent reference
  expected_metrics.json metric -> {value, tolerance}

The harness compiles and runs each case with the cfdl CLI and fails if any
period-level value or summary metric drifts outside its tolerance.

expected.csv is indexed by either `period` (the 0-based period number) or
`year` (any label; rows are taken in order, so the first row is period 0).
Every other column names a series to check: `net_cash_flow` is the model
total, and anything else is a stream id, matched against `stream.<column>`.
A case may check the total alone, or every stream line individually — the
latter localises a break to one stream instead of only reporting that the
total moved. Blank cells are skipped, so a column need not span every row.
"""
import csv
import json
import os
import pathlib
import subprocess
import sys
import tempfile

try:
    import tomllib
except ImportError:  # pragma: no cover
    print("python >= 3.11 required (tomllib)", file=sys.stderr)
    sys.exit(2)

ROOT = pathlib.Path(__file__).resolve().parents[1]
CFDL = os.environ.get("CFDL_BIN", str(ROOT / "target/debug/cfdl"))


def scalar(value):
    if isinstance(value, dict) and "amount" in value:
        return float(value["amount"])
    return float(value)


def resolve_columns(fieldnames, series, failures):
    """Map expected.csv headers onto result series.

    Returns (index_column, [(csv_column, series_key, label)]) or None if the
    header cannot be resolved, in which case `failures` says why.
    """
    fields = list(fieldnames or [])
    index_col = next((c for c in ("period", "year") if c in fields), None)
    if index_col is None:
        failures.append("expected.csv: need a 'period' or 'year' column")
        return None

    columns = []
    for column in fields:
        if column == index_col:
            continue
        if column == "net_cash_flow":
            key, label = "model.net_cash_flow", "net"
        else:
            key, label = f"stream.{column}", column
        if key not in series:
            failures.append(f"expected.csv column {column!r}: no series {key!r} in results")
            continue
        columns.append((column, key, label))
    if not columns:
        failures.append("expected.csv: no columns to check")
        return None
    return index_col, columns


def run_case(case_dir: pathlib.Path) -> list[str]:
    failures = []
    case = tomllib.loads((case_dir / "case.toml").read_text())
    with tempfile.TemporaryDirectory() as tmp:
        ir = pathlib.Path(tmp) / "model.ir.json"
        results_path = pathlib.Path(tmp) / "results.json"
        subprocess.run(
            [CFDL, "compile", str(case_dir), "--out", str(ir), "--packs", str(ROOT / "packs")],
            check=True,
        )
        subprocess.run(
            [
                CFDL, "run", str(ir),
                "--out", str(results_path),
                "--config", str(case_dir / "run.json"),
                "--packs", str(ROOT / "packs"),
                "--pack", case["pack"],
            ],
            check=True,
        )
        results = json.loads(results_path.read_text())

    if results.get("warnings"):
        failures.append(f"engine warnings: {results['warnings'][:3]}")

    series = results["deterministic"]["series"]
    tolerance = float(case.get("period_tolerance", 0.01))
    with open(case_dir / "expected.csv") as fh:
        reader = csv.DictReader(fh)
        resolved = resolve_columns(reader.fieldnames, series, failures)
        if resolved is None:
            return failures
        index_col, columns = resolved
        values = {
            key: [scalar(v) for v in series[key]["values"]] for _, key, _ in columns
        }
        for row_no, row in enumerate(reader):
            t = int(row[index_col]) if index_col == "period" else row_no
            for column, key, label in columns:
                cell = (row.get(column) or "").strip()
                if not cell:
                    continue
                actual = values[key]
                if t >= len(actual):
                    failures.append(
                        f"{label} period {t}: beyond the {len(actual)}-period timeline"
                    )
                    return failures
                got, expected = actual[t], float(cell)
                if abs(got - expected) > tolerance:
                    failures.append(
                        f"{label} period {t}: {got:.6f} vs expected {expected:.6f} "
                        f"(|diff| {abs(got - expected):.6f} > {tolerance})"
                    )
                    if len(failures) > 5:
                        failures.append("... (truncated)")
                        return failures

    metrics = dict(results["deterministic"]["metrics"])
    domain = results.get("domain_metrics") or {}
    metrics.update(domain.get("metrics", {}))
    expected_metrics = json.loads((case_dir / "expected_metrics.json").read_text())
    for key, spec in expected_metrics.items():
        if key not in metrics:
            failures.append(f"metric {key}: missing from results")
            continue
        got = scalar(metrics[key])
        if abs(got - spec["value"]) > spec["tolerance"]:
            failures.append(
                f"metric {key}: {got} vs expected {spec['value']} "
                f"(tolerance {spec['tolerance']})"
            )
    return failures


def main() -> int:
    cases = sorted(p.parent for p in (ROOT / "benchmarks").glob("*/*/model.cfdl"))
    if not cases:
        print("[bench] no benchmark cases found")
        return 1
    failed = 0
    for case_dir in cases:
        rel = case_dir.relative_to(ROOT)
        failures = run_case(case_dir)
        if failures:
            failed += 1
            print(f"[bench][FAIL] {rel}")
            for failure in failures:
                print(f"    {failure}")
        else:
            print(f"[bench][PASS] {rel}")
    print(f"[bench] Done. PASS={len(cases) - failed} FAIL={failed}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
