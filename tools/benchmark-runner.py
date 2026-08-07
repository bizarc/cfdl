#!/usr/bin/env python3
"""CFDL benchmark harness.

Each case directory (benchmarks/<pack>/<case>/) contains:
  model.cfdl            the CFDL model
  run.json              run configuration
  case.toml             pack name + per-period tolerance
  expected.csv          per-period expectations from an independent reference
  expected_metrics.json  metric -> {value, tolerance}
  expected_scenarios.json  scenario -> metric -> {value, tolerance}   (optional)

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

# These tools print prose. A Windows console defaults to cp1252, which
# cannot encode every character the check names use, so pin stdout to UTF-8.
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

try:
    import tomllib
except ImportError:  # pragma: no cover
    print("python >= 3.11 required (tomllib)", file=sys.stderr)
    sys.exit(2)

ROOT = pathlib.Path(__file__).resolve().parents[1]
# Windows names the binary cfdl.exe. subprocess resolves that on its own, so
# this ran green for as long as it never checked the path itself — which is
# exactly the trap analytic-checks fell into. Be explicit.
_CLI_NAME = "cfdl.exe" if os.name == "nt" else "cfdl"
CFDL = os.environ.get("CFDL_BIN", str(ROOT / "target" / "debug" / _CLI_NAME))


def scalar(value):
    """A series point as a number, or None where it is genuinely undefined.

    `None` is not zero. A coverage ratio in a period with no debt service has no
    value, and the engine publishes JSON null to say so; coercing that to 0.0
    would let a case "match" an expectation it never computed.
    """
    if value is None:
        return None
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
        # A header naming a result series verbatim wins. That is what lets a
        # case assert `domain.cre.dscr`, `domain.cre.noi` or a `state.` series —
        # the reconciliations that previously could only live in NOTES.md as
        # prose, because the harness could reach per-stream cash and nothing
        # else.
        if column in series:
            key, label = column, column
        elif column == "net_cash_flow":
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
    case = tomllib.loads((case_dir / "case.toml").read_text(encoding="utf-8"))

    # Every case declares one sentence describing the DEAL, which is what the
    # documentation site publishes for it.
    #
    # The site used to scrape this file's leading comments instead. Those are
    # maintainer's notes — why a tolerance moved, which line was wrong and for
    # how long — and publishing them put a wall of engineering narrative at the
    # top of the strongest evidence pages there are. Requiring the field here
    # rather than only in the site generator means a new case cannot be added
    # without one, and the failure names the case rather than appearing later
    # as a stale page.
    summary = case.get("summary", "").strip()
    if not summary:
        failures.append(
            f"{case_dir.name}: case.toml has no `summary`. Add one sentence "
            f"describing the deal — it is what the docs page shows. Comments in "
            f"this file are notes for maintainers and are not published."
        )
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
        results = json.loads(results_path.read_text(encoding="utf-8"))

    if results.get("warnings"):
        failures.append(f"engine warnings: {results['warnings'][:3]}")

    series = results["deterministic"]["series"]
    # One tolerance cannot serve a whole case. HUD publishes money to whole
    # dollars, so its lines need ~1.0; its DSCR is quoted to sixteen figures and
    # agrees to five decimals, so a shared 1.0 would assert nothing about it
    # while a shared 1e-4 would fail every money line. `period_tolerance` stays
    # the default; `[tolerance]` overrides it per column.
    default_tolerance = float(case.get("period_tolerance", 0.01))
    per_column = {k: float(v) for k, v in (case.get("tolerance") or {}).items()}
    with open(case_dir / "expected.csv", encoding="utf-8") as fh:
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
                if got is None:
                    # The CSV states a value the results say is undefined. A
                    # blank cell means "not asserted"; a stated one means the
                    # case expected a number and did not get one.
                    failures.append(
                        f"{label} period {t}: series is null (undefined here) "
                        f"but {expected:.6f} was expected"
                    )
                    if len(failures) > 5:
                        failures.append("... (truncated)")
                        return failures
                    continue
                tol = per_column.get(column, default_tolerance)
                if abs(got - expected) > tol:
                    failures.append(
                        f"{label} period {t}: {got:.6f} vs expected {expected:.6f} "
                        f"(|diff| {abs(got - expected):.6f} > {tol})"
                    )
                    if len(failures) > 5:
                        failures.append("... (truncated)")
                        return failures

    # Scenario metrics, when the case declares them. A scenario is a full
    # deterministic run under different parameters, so a case whose subject is
    # how a number moves with an input asserts it here rather than needing one
    # case directory per variant.
    expected_scenarios_path = case_dir / "expected_scenarios.json"
    if expected_scenarios_path.exists():
        summaries = {
            s["name"]: s["metrics"]
            for s in (results.get("scenarios") or {}).get("summaries", [])
        }
        expected_scenarios = json.loads(expected_scenarios_path.read_text(encoding="utf-8"))
        for name, wanted in expected_scenarios.items():
            if name not in summaries:
                failures.append(f"scenario {name!r}: not in results")
                continue
            for key, spec in wanted.items():
                if key not in summaries[name]:
                    failures.append(f"scenario {name}: metric {key} missing from results")
                    continue
                got = scalar(summaries[name][key])
                if abs(got - spec["value"]) > spec["tolerance"]:
                    failures.append(
                        f"scenario {name} metric {key}: {got} vs expected "
                        f"{spec['value']} (tolerance {spec['tolerance']})"
                    )

    metrics = dict(results["deterministic"]["metrics"])
    domain = results.get("domain_metrics") or {}
    metrics.update(domain.get("metrics", {}))
    expected_metrics = json.loads((case_dir / "expected_metrics.json").read_text(encoding="utf-8"))
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
