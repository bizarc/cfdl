#!/usr/bin/env python3
"""CFDL benchmark harness (LAUNCH_PLAN §6D).

Each case directory (benchmarks/<pack>/<case>/) contains:
  model.cfdl            the CFDL model
  run.json              run configuration
  case.toml             pack name + per-period tolerance
  expected.csv          period,net_cash_flow from an independent reference
  expected_metrics.json metric -> {value, tolerance}

The harness compiles and runs each case with the cfdl CLI and fails if any
period-level net cash flow or summary metric drifts outside its tolerance.
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

    actual = [
        scalar(v)
        for v in results["deterministic"]["series"]["model.net_cash_flow"]["values"]
    ]
    tolerance = float(case.get("period_tolerance", 0.01))
    with open(case_dir / "expected.csv") as fh:
        for row in csv.DictReader(fh):
            t = int(row["period"])
            expected = float(row["net_cash_flow"])
            got = actual[t]
            if abs(got - expected) > tolerance:
                failures.append(
                    f"period {t}: net {got:.6f} vs expected {expected:.6f} "
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
