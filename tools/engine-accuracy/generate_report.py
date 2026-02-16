#!/usr/bin/env python3
"""
Generate ENGINE_ACCURACY_REPORT.md from validation_report.json and run_failures.json.
"""
import json
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
VALIDATION_REPORT = SCRIPT_DIR / "validation_report.json"
FAILURES_PATH = SCRIPT_DIR / "run_failures.json"
CATALOG = SCRIPT_DIR / "example_runs.json"
OUT_REPORT = SCRIPT_DIR / "ENGINE_ACCURACY_REPORT.md"


def main():
    with open(CATALOG, encoding="utf-8") as f:
        catalog = json.load(f)
    total_runs = len(catalog)

    failures = []
    if FAILURES_PATH.exists():
        with open(FAILURES_PATH, encoding="utf-8") as f:
            failures = json.load(f)
    runs_succeeded = total_runs - len(failures)

    if not VALIDATION_REPORT.exists():
        print(f"Missing {VALIDATION_REPORT}. Run validate_results.py first.")
        return 1
    with open(VALIDATION_REPORT, encoding="utf-8") as f:
        vreport = json.load(f)
    validated_count = vreport["validated_count"]
    match_count = vreport["match_count"]
    mismatch_count = vreport["mismatch_count"]
    tolerance = vreport.get("tolerance", 1e-6)

    mismatches = [r for r in vreport["results"] if not r["match"]]

    lines = [
        "# CFDL Engine Accuracy Report",
        "",
        "## Summary",
        "",
        f"- **Total runs attempted**: {total_runs}",
        f"- **Compile/run succeeded**: {runs_succeeded}",
        f"- **Compile/run failed**: {len(failures)}",
        f"- **Validated** (comparison run): {validated_count}",
        f"- **Match**: {match_count}",
        f"- **Mismatch**: {mismatch_count}",
        "",
        "## Methodology",
        "",
        "- **Recomputed metrics**:",
        "  - `model.total` = sum of `deterministic.series[\"model.net_cash_flow\"].values[].amount`",
        "  - `model.npv` = sum of `cf[i] / (1 + discount_rate)^i` for each period (period-end discounting, matching engine).",
        "- **Optional**: For each `stream.<name>` series, sum of `values[].amount` compared to `stream.<name>.total` in metrics.",
        f"- **Tolerance**: `{tolerance}` (consistent with engine 6-decimal rounding).",
        "",
        "## Accuracy",
        "",
    ]
    if validated_count > 0:
        pct = 100.0 * match_count / validated_count
        lines.append(f"- **Runs where all checked metrics matched**: {match_count} / {validated_count} ({pct:.1f}%).")
    else:
        lines.append("- No results were validated (no successful run outputs in `out/`).")
    lines.extend(["", "## Issues", ""])
    if not mismatches:
        lines.append("No mismatches. All validated runs matched recomputed metrics.")
    else:
        lines.append("| Example (file) | Metric | Expected | Actual | Diff |")
        lines.append("|----------------|--------|----------|--------|------|")
        for r in mismatches:
            fname = r["file"]
            for issue in r.get("issues", []):
                if "error" in issue:
                    lines.append(f"| {fname} | {issue.get('metric', '?')} | — | — | {issue['error']} |")
                else:
                    exp = issue.get("expected", "")
                    act = issue.get("actual", "")
                    diff = issue.get("diff", "")
                    lines.append(f"| {fname} | {issue.get('metric', '?')} | {exp} | {act} | {diff} |")
    lines.extend(["", "## Failures", ""])
    if not failures:
        lines.append("No compile or run failures.")
    else:
        lines.append("Runs that failed to compile or run:")
        lines.append("")
        for f in failures:
            lines.append(f"- **{f['run_id']}** — stage: {f['stage']}, exit_code: {f['exit_code']}")
            lines.append(f"  - stderr: {f.get('stderr', '')[:200].strip() or '(none)'}")
        lines.append("")
        lines.append("Full details are in `run_failures.json`.")
    lines.extend(["", "## Recommendations", ""])
    if mismatches:
        lines.append("- Investigate each mismatch using the root-cause checklist (rounding, NPV formula, engine bug).")
    if failures:
        lines.append("- Fix or document compile/run failures (e.g. missing language support, model errors).")
    lines.append("- Consider adding unit tests in cfdl-engine for NPV and model.total from series.")
    lines.append("- Optionally lock golden results for example runs to catch regressions.")
    lines.append("")

    OUT_REPORT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {OUT_REPORT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
