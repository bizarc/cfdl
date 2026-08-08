#!/usr/bin/env python3
"""Prove that training exercises do what their prompts claim.

Every exercise under training/exercises/<chapter>/<name>/ carries a starter
(model.cfdl), a solution (solution.cfdl), a run config (run.json), and the
metrics the solution must reproduce (expected.json). The fenced models inside
the chapters themselves are covered by check-doc-examples; this gate covers
what a learner actually opens and edits:

  * the starter must PARSE — it is allowed to be an intentionally incomplete
    scaffold (a bare time grid the learner populates), so compilation is not
    required, but a starter with a syntax error teaches the wrong lesson on
    arrival;
  * the solution must compile and run under the exercise's own run config;
  * the solution's metrics must match expected.json within its tolerance —
    the same discipline the chapters preach: a stated number is a checked
    number.

expected.json shape:

    { "metrics": { "model.total": 194400.0, ... }, "tolerance": 0.01 }

Regenerate an expected.json by running the solution through the CLI and
recording the metrics it reports — never by editing the number to match.

Usage:
    python3 tools/check-training-examples.py            # check every exercise
    python3 tools/check-training-examples.py --verbose  # list every metric
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import subprocess
import sys
import tempfile

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
CLI = REPO_ROOT / "target" / "debug" / ("cfdl.exe" if os.name == "nt" else "cfdl")
PACKS = REPO_ROOT / "packs"
EXERCISES = REPO_ROOT / "training" / "exercises"


def metric_number(value) -> float:
    """Metrics are either bare numbers or {amount, currency} money objects."""
    if isinstance(value, dict):
        return float(value["amount"])
    return float(value)


def run_cli(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [str(CLI), *args], capture_output=True, text=True, encoding="utf-8"
    )


def check_exercise(ex_dir: pathlib.Path, verbose: bool) -> list[str]:
    rel = ex_dir.relative_to(REPO_ROOT)
    problems: list[str] = []

    starter = ex_dir / "model.cfdl"
    solution = ex_dir / "solution.cfdl"
    run_json = ex_dir / "run.json"
    expected_json = ex_dir / "expected.json"

    for required in (starter, solution, run_json, expected_json):
        if not required.exists():
            problems.append(f"{rel}: missing {required.name}")
    if problems:
        return problems

    # 1. The starter parses.
    with tempfile.TemporaryDirectory() as tmp:
        work = pathlib.Path(tmp)
        (work / "model.cfdl").write_text(starter.read_text(encoding="utf-8"), encoding="utf-8")
        done = run_cli("parse", str(work))
        if done.returncode != 0:
            problems.append(f"{rel}: starter does not parse:\n{done.stderr or done.stdout}")

    # 2. The solution compiles and runs under the exercise's config.
    with tempfile.TemporaryDirectory() as tmp:
        work = pathlib.Path(tmp)
        (work / "model.cfdl").write_text(solution.read_text(encoding="utf-8"), encoding="utf-8")
        ir = work / "ir.json"
        done = run_cli("compile", str(work), "--out", str(ir), "--packs", str(PACKS))
        if done.returncode != 0:
            problems.append(f"{rel}: solution does not compile:\n{done.stderr or done.stdout}")
            return problems

        results = work / "results.json"
        done = run_cli("run", str(ir), "--out", str(results), "--config", str(run_json))
        if done.returncode != 0:
            problems.append(f"{rel}: solution does not run:\n{done.stderr or done.stdout}")
            return problems

        reported = json.loads(results.read_text(encoding="utf-8"))["deterministic"]["metrics"]

    # 3. The metrics match what the exercise promises.
    expected = json.loads(expected_json.read_text(encoding="utf-8"))
    tolerance = float(expected.get("tolerance", 0.01))
    for name, want in expected["metrics"].items():
        if name not in reported:
            problems.append(f"{rel}: solution reports no metric {name}")
            continue
        got = metric_number(reported[name])
        if abs(got - float(want)) > tolerance:
            problems.append(
                f"{rel}: {name} is {got}, expected {want} (±{tolerance})"
            )
        elif verbose:
            print(f"  {rel}: {name} = {got}")

    return problems


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args()

    if not CLI.exists():
        print(f"check-training-examples: build the CLI first ({CLI} is missing)", file=sys.stderr)
        return 1

    exercise_dirs = sorted(
        d for d in EXERCISES.glob("*/*") if d.is_dir()
    ) if EXERCISES.exists() else []

    problems: list[str] = []
    for ex_dir in exercise_dirs:
        problems += check_exercise(ex_dir, args.verbose)

    if problems:
        print("check-training-examples: FAIL", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        return 1

    print(
        f"check-training-examples: OK ({len(exercise_dirs)} exercises: starters parse, "
        "solutions compile, run, and match their expected metrics)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
