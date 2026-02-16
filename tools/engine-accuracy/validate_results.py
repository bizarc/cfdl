#!/usr/bin/env python3
"""
Validate engine results: recompute model.total and model.npv from deterministic.series
and compare to deterministic.metrics. Optionally validate stream.*.total vs series sums.
Output: validation_report.json for the report generator.
"""
import json
from pathlib import Path
from typing import Any, Dict, List, Optional

SCRIPT_DIR = Path(__file__).resolve().parent
OUT_DIR = SCRIPT_DIR / "out"
REPORT_PATH = SCRIPT_DIR / "validation_report.json"

# Match engine round_amount (6 decimals). Use 1e-6 so rounded values compare equal.
TOLERANCE = 1e-6


def scalar_amount(metrics: Dict[str, Any], key: str) -> Optional[float]:
    """Get numeric amount for a metric. Handles Money { amount, currency } or Number."""
    if key not in metrics:
        return None
    val = metrics[key]
    if isinstance(val, dict) and "amount" in val:
        return float(val["amount"])
    if isinstance(val, (int, float)):
        return float(val)
    return None


def recompute_model_total(series: Dict[str, Any]) -> Optional[float]:
    """Sum of model.net_cash_flow.values[].amount."""
    net = series.get("model.net_cash_flow")
    if not net or "values" not in net:
        return None
    return sum(v.get("amount", 0) for v in net["values"])


def recompute_npv(series: Dict[str, Any], discount_rate: float) -> Optional[float]:
    """NPV = sum( cf[i] / (1+r)^i ) for i in 0..len-1. Matches engine (period-end)."""
    net = series.get("model.net_cash_flow")
    if not net or "values" not in net:
        return None
    total = 0.0
    for i, v in enumerate(net["values"]):
        amount = v.get("amount", 0)
        discount = (1.0 + discount_rate) ** i
        total += amount / discount
    return total


def recompute_stream_totals(series: Dict[str, Any]) -> Dict[str, float]:
    """For each series key stream.<name>, sum values[].amount."""
    out = {}
    for key, data in series.items():
        if not key.startswith("stream.") or key == "model.net_cash_flow":
            continue
        if not isinstance(data, dict) or "values" not in data:
            continue
        out[key] = sum(v.get("amount", 0) for v in data["values"])
    return out


def validate_one(path: Path) -> Dict[str, Any]:
    """Validate a single results JSON. Return entry for validation_report."""
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    det = data.get("deterministic") or {}
    metrics = det.get("metrics") or {}
    series = det.get("series") or {}

    discount_rate = scalar_amount(metrics, "run.discount_rate")
    if discount_rate is None:
        discount_rate = 0.0  # engine default when not in config

    expected_total = recompute_model_total(series)
    actual_total = scalar_amount(metrics, "model.total")
    expected_npv = recompute_npv(series, discount_rate)
    actual_npv = scalar_amount(metrics, "model.npv")

    issues = []
    if expected_total is not None and actual_total is not None:
        if abs(expected_total - actual_total) > TOLERANCE:
            issues.append({
                "metric": "model.total",
                "expected": expected_total,
                "actual": actual_total,
                "diff": expected_total - actual_total,
            })
    elif actual_total is not None and expected_total is None:
        issues.append({"metric": "model.total", "error": "missing model.net_cash_flow series"})
    elif expected_total is not None and actual_total is None:
        issues.append({"metric": "model.total", "error": "missing model.total in metrics"})

    if expected_npv is not None and actual_npv is not None:
        if abs(expected_npv - actual_npv) > TOLERANCE:
            issues.append({
                "metric": "model.npv",
                "expected": expected_npv,
                "actual": actual_npv,
                "diff": expected_npv - actual_npv,
            })
    elif actual_npv is not None and expected_npv is None:
        issues.append({"metric": "model.npv", "error": "missing model.net_cash_flow series"})
    elif expected_npv is not None and actual_npv is None:
        issues.append({"metric": "model.npv", "error": "missing model.npv in metrics"})

    # Optional: stream totals (series key is stream.<name>, metric is stream.<name>.total)
    stream_sums = recompute_stream_totals(series)
    for stream_key, expected_sum in stream_sums.items():
        total_key = stream_key + ".total"
        actual_sum = scalar_amount(metrics, total_key)
        if actual_sum is not None and abs(expected_sum - actual_sum) > TOLERANCE:
            issues.append({
                "metric": total_key,
                "expected": expected_sum,
                "actual": actual_sum,
                "diff": expected_sum - actual_sum,
            })

    return {
        "file": path.name,
        "path": str(path),
        "match": len(issues) == 0,
        "issues": issues,
        "checked": ["model.total", "model.npv"] + [k + ".total" for k in stream_sums],
    }


def main():
    if not OUT_DIR.exists():
        print(f"Out dir not found: {OUT_DIR}. Run run_examples.py first.")
        return 1
    results_files = sorted(OUT_DIR.glob("*.results.json"))
    if not results_files:
        print(f"No *.results.json in {OUT_DIR}")
        return 1

    report = {
        "tolerance": TOLERANCE,
        "validated_count": len(results_files),
        "match_count": 0,
        "mismatch_count": 0,
        "results": [],
    }
    for path in results_files:
        entry = validate_one(path)
        report["results"].append(entry)
        if entry["match"]:
            report["match_count"] += 1
        else:
            report["mismatch_count"] += 1

    with open(REPORT_PATH, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    print(f"Validated {report['validated_count']}, match {report['match_count']}, mismatch {report['mismatch_count']}. Report: {REPORT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
