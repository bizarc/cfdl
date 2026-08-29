#!/usr/bin/env python3
"""The agent-eval harness (docs/32 Phase 3).

The benchmark suite becomes the grader. Three task tiers:

  repair      a minimal failing model plus its structured diagnostics; the
              agent returns fixed sources. Scored: compiles. Tasks come from
              fixtures/invalid/ + gold/diag/ (70 tasks); the recorded fixes
              in fixtures/repairs/ are the replay agent's solutions.
  transcribe  a benchmark case's CASE.md specification and the reference
              material the case is allowed to see — never expected.csv — plus
              its run configuration; the agent returns model sources. Scored:
              compiles / runs / matches, with the benchmark runner's own
              tolerance discipline; partial credit by asserted column and
              metric, not prose similarity.
  extend      an existing model plus a change request, graded by targeted
              assertions. Task format defined below; the public set ships
              empty until assertions with independent derivations exist.

Agents under test:

  --agent replay          the scripted agent: returns the known-good sources
                          (benchmarks/<case>/ for transcribe,
                          fixtures/repairs/<name>/ for repair). Must score
                          100% — the self-test that separates harness bugs
                          from model failures.
  --agent cmd:<command>   runs <command>; task JSON on stdin, and expects
                          {"files": {"model.cfdl": "..."}} on stdout.
  --agent http:<url>      POSTs the task JSON; expects the same response
                          shape. This is the provider-agnostic seam: the
                          agent behind the endpoint drives its own Phase-1
                          MCP loop and answers with its final sources.

Determinism: scores carry no timestamps and every comparison is the
benchmark runner's, so a score is reproducible byte-for-byte.

Held-out split: --benchmarks-dir accepts any directory with the registered
case layout, so a private case set (docs/31 W2 engagements) runs unchanged;
the honest headline number comes from a directory this repository never sees.

Usage:
  python3 tools/agent-eval/runner.py --tier all --agent replay
  python3 tools/agent-eval/runner.py --self-test          # sampled, CI gate
  python3 tools/agent-eval/runner.py --tier transcribe \\
      --agent http://localhost:8088/solve --out scores.json
"""

import argparse
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import urllib.request

# line_buffering so a redirected long run (nohup ... > log) streams per-task
# progress instead of flushing in 8 KB blocks.
sys.stdout.reconfigure(encoding="utf-8", line_buffering=True)
sys.stderr.reconfigure(encoding="utf-8")

try:
    import tomllib
except ImportError:  # pragma: no cover
    print("python >= 3.11 required (tomllib)", file=sys.stderr)
    sys.exit(2)

ROOT = pathlib.Path(__file__).resolve().parents[2]
_CLI_NAME = "cfdl.exe" if os.name == "nt" else "cfdl"
CFDL = os.environ.get("CFDL_BIN", str(ROOT / "target" / "debug" / _CLI_NAME))

# The comparison IS the benchmark runner's — imported, not restated, so the
# eval and `make bench` can never grade differently.
_spec = importlib.util.spec_from_file_location(
    "benchmark_runner", ROOT / "tools" / "benchmark-runner.py"
)
benchmark_runner = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(benchmark_runner)

# The sampled self-test set: one exemplar per domain plus three repair
# families. Small because ci-gates runs it on every push; `--tier all
# --agent replay` is the full 100% gate, run before a release claim.
SELF_TEST_TRANSCRIBE = [
    "cre/office_two_tenant",
    "credit/level_pay_pool",
    "opco/lbo_buyout",
]
SELF_TEST_REPAIR = ["missing_time", "stream_unknown_category", "term_expr_invalid"]

# Reference files an agent may read: text formats only; binaries are named
# but not inlined.
TEXT_SUFFIXES = {".md", ".py", ".txt", ".csv", ".json", ".toml"}
MAX_REFERENCE_BYTES = 200_000


# --- Tasks -------------------------------------------------------------------

def transcribe_tasks(benchmarks_dir: pathlib.Path, only: set[str] | None) -> list[dict]:
    tasks = []
    for model in sorted(benchmarks_dir.glob("*/*/model.cfdl")):
        case_dir = model.parent
        case_id = f"{case_dir.parent.name}/{case_dir.name}"
        if only is not None and case_id not in only:
            continue
        case = tomllib.loads((case_dir / "case.toml").read_text(encoding="utf-8"))
        reference: dict[str, str] = {}
        withheld: list[str] = []
        for path in sorted(case_dir.glob("reference/*")):
            rel = f"reference/{path.name}"
            if (
                path.suffix in TEXT_SUFFIXES
                and path.stat().st_size <= MAX_REFERENCE_BYTES
            ):
                reference[rel] = path.read_text(encoding="utf-8", errors="replace")
            else:
                withheld.append(rel)
        tasks.append(
            {
                "id": case_id,
                "tier": "transcribe",
                "spec": (case_dir / "CASE.md").read_text(encoding="utf-8"),
                "summary": case.get("summary", ""),
                "pack": case.get("pack"),
                "run_config": json.loads(
                    (case_dir / "run.json").read_text(encoding="utf-8")
                ),
                "reference": reference,
                "reference_binaries": withheld,
                "_case_dir": str(case_dir),
            }
        )
    return tasks


def repair_tasks(only: set[str] | None) -> list[dict]:
    tasks = []
    for fixture in sorted((ROOT / "fixtures" / "invalid").iterdir()):
        if not fixture.is_dir():
            continue
        if only is not None and fixture.name not in only:
            continue
        golden = ROOT / "gold" / "diag" / f"{fixture.name}.diag.json"
        tasks.append(
            {
                "id": fixture.name,
                "tier": "repair",
                "files": {
                    p.name: p.read_text(encoding="utf-8")
                    for p in sorted(fixture.glob("*.cfdl"))
                },
                "diagnostics": json.loads(golden.read_text(encoding="utf-8"))
                if golden.exists()
                else [],
                "instructions": (
                    "The model fails to compile with the diagnostics shown. "
                    "Return the minimal change that fixes every diagnosed problem "
                    "while preserving the model's identity."
                ),
            }
        )
    return tasks


def extend_tasks(only: set[str] | None) -> list[dict]:
    """Extend tasks: tools/agent-eval/tasks/extend/<id>.toml, each declaring
    `base_case`, `request` (prose), and `[assertions]` (metric -> {value,
    tolerance}, derived independently of this engine). None ship yet."""
    tasks = []
    tasks_dir = ROOT / "tools" / "agent-eval" / "tasks" / "extend"
    if not tasks_dir.is_dir():
        return tasks
    for spec_path in sorted(tasks_dir.glob("*.toml")):
        if only is not None and spec_path.stem not in only:
            continue
        spec = tomllib.loads(spec_path.read_text(encoding="utf-8"))
        base = ROOT / "benchmarks" / spec["base_case"]
        tasks.append(
            {
                "id": spec_path.stem,
                "tier": "extend",
                "request": spec["request"],
                "files": {
                    p.name: p.read_text(encoding="utf-8")
                    for p in sorted(base.glob("*.cfdl"))
                },
                "run_config": json.loads((base / "run.json").read_text(encoding="utf-8")),
                "pack": tomllib.loads((base / "case.toml").read_text(encoding="utf-8")).get("pack"),
                "_assertions": spec.get("assertions", {}),
                "_base_case": spec["base_case"],
            }
        )
    return tasks


# --- Agents ------------------------------------------------------------------

def replay_agent(task: dict, benchmarks_dir: pathlib.Path) -> dict:
    if task["tier"] == "transcribe":
        case_dir = benchmarks_dir / task["id"]
        return {
            "files": {
                p.name: p.read_text(encoding="utf-8")
                for p in sorted(case_dir.glob("*.cfdl"))
            }
        }
    if task["tier"] == "repair":
        repair_dir = ROOT / "fixtures" / "repairs" / task["id"]
        return {
            "files": {
                p.name: p.read_text(encoding="utf-8")
                for p in sorted(repair_dir.glob("*.cfdl"))
            }
        }
    # extend: replay returns the base model unchanged — deliberately NOT a
    # 100% agent for this tier; the tier's self-test is its assertion format.
    return {"files": task["files"]}


def public_task(task: dict) -> dict:
    """What the agent under test sees: every key except the withheld ones."""
    return {k: v for k, v in task.items() if not k.startswith("_")}


def call_agent(agent: str, task: dict, benchmarks_dir: pathlib.Path) -> dict:
    if agent == "replay":
        return replay_agent(task, benchmarks_dir)
    payload = json.dumps(public_task(task)).encode("utf-8")
    if agent.startswith("cmd:"):
        result = subprocess.run(
            agent[4:],
            shell=True,
            input=payload,
            capture_output=True,
            timeout=2700,
        )
        if result.returncode != 0:
            raise RuntimeError(
                f"agent command failed ({result.returncode}): "
                f"{result.stderr.decode('utf-8', 'replace')[-400:]}"
            )
        return json.loads(result.stdout.decode("utf-8"))
    if agent.startswith("http://") or agent.startswith("https://"):
        request = urllib.request.Request(
            agent, data=payload, headers={"Content-Type": "application/json"}
        )
        with urllib.request.urlopen(request, timeout=1800) as response:
            return json.loads(response.read().decode("utf-8"))
    raise SystemExit(f"unknown agent '{agent}' (use replay, cmd:<command>, or an http url)")


# --- Grading -----------------------------------------------------------------

def compile_files(files: dict[str, str], tmp: pathlib.Path) -> tuple[bool, list[dict]]:
    model_dir = tmp / "model"
    model_dir.mkdir()
    for name, source in files.items():
        (model_dir / name).write_text(source, encoding="utf-8")
    out = tmp / "ir.json"
    result = subprocess.run(
        [CFDL, "--json", "compile", str(model_dir), "--out", str(out), "--packs", str(ROOT / "packs")],
        capture_output=True,
        encoding="utf-8",
    )
    if result.returncode == 0:
        return True, []
    try:
        return False, json.loads(result.stdout)
    except json.JSONDecodeError:
        return False, [{"code": "HARNESS", "message": result.stderr[:400]}]


def grade_repair(task: dict, submission: dict) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        ok, diags = compile_files(submission.get("files", {}), pathlib.Path(tmp))
    return {
        "compiles": ok,
        "runs": None,
        "matches": None,
        "partial": 1.0 if ok else 0.0,
        "failures": [d.get("code", "?") for d in diags],
    }


def asserted_labels(case_dir: pathlib.Path) -> set[str]:
    """The independent assertion groups a case carries, by the same labels
    the benchmark runner's structured failures use: each expected.csv column,
    each metric key, `name.metric` per scenario, `key.aggregate` per Monte
    Carlo assertion."""
    labels: set[str] = set()
    csv_path = case_dir / "expected.csv"
    if csv_path.exists():
        header = csv_path.read_text(encoding="utf-8").splitlines()[0]
        labels |= {
            c.strip() for c in header.split(",") if c.strip() not in ("period", "year")
        }
    metrics_path = case_dir / "expected_metrics.json"
    if metrics_path.exists():
        labels |= set(json.loads(metrics_path.read_text(encoding="utf-8")))
    for name in ("expected_scenarios.json", "expected_monte_carlo.json"):
        path = case_dir / name
        if path.exists():
            data = json.loads(path.read_text(encoding="utf-8"))
            labels |= {
                f"{outer}.{inner}" for outer, wanted in data.items() for inner in wanted
            }
    return labels


def grade_transcribe(task: dict, submission: dict) -> dict:
    files = submission.get("files", {})
    source_case = pathlib.Path(task["_case_dir"])
    with tempfile.TemporaryDirectory() as tmp:
        compiled, diags = compile_files(files, pathlib.Path(tmp))
        if not compiled:
            return {
                "compiles": False,
                "runs": False,
                "matches": False,
                "partial": 0.0,
                "failures": [d.get("code", "?") for d in diags],
            }
        # A graded case directory: the submission plus the case's own
        # expectations — then the benchmark runner's own run_case grades it.
        case_dir = pathlib.Path(tmp) / "case"
        case_dir.mkdir()
        for name, source in files.items():
            (case_dir / name).write_text(source, encoding="utf-8")
        for name in (
            "case.toml",
            "run.json",
            "expected.csv",
            "expected_metrics.json",
            "expected_scenarios.json",
            "expected_monte_carlo.json",
        ):
            src = source_case / name
            if src.exists():
                (case_dir / name).write_text(
                    src.read_text(encoding="utf-8"), encoding="utf-8"
                )
        try:
            failures = benchmark_runner.run_case(case_dir, structured=True)
        except subprocess.CalledProcessError:
            return {
                "compiles": True,
                "runs": False,
                "matches": False,
                "partial": 0.0,
                "failures": ["run failed"],
            }
    labels = asserted_labels(source_case)
    # Partial credit by asserted group, on the runner's structured labels —
    # no prose parsing. A group with any failure counts as missed; meta
    # failures (warnings, resolution) fail the match without eating a group.
    missed = len({group for group, _ in failures} & labels)
    matched = max(len(labels) - missed, 0)
    return {
        "compiles": True,
        "runs": True,
        "matches": not failures,
        "partial": round(matched / len(labels), 4) if labels else 0.0,
        "failures": [text for _, text in failures],
    }


def grade_extend(task: dict, submission: dict) -> dict:
    files = submission.get("files", {})
    with tempfile.TemporaryDirectory() as tmp:
        compiled, diags = compile_files(files, pathlib.Path(tmp))
        if not compiled:
            return {
                "compiles": False,
                "runs": False,
                "matches": False,
                "partial": 0.0,
                "failures": [d.get("code", "?") for d in diags],
            }
        case_dir = pathlib.Path(tmp) / "case"
        case_dir.mkdir()
        for name, source in files.items():
            (case_dir / name).write_text(source, encoding="utf-8")
        base = ROOT / "benchmarks" / task["_base_case"]
        (case_dir / "run.json").write_text(
            (base / "run.json").read_text(encoding="utf-8"), encoding="utf-8"
        )
        (case_dir / "case.toml").write_text(
            (base / "case.toml").read_text(encoding="utf-8"), encoding="utf-8"
        )
        (case_dir / "expected_metrics.json").write_text(
            json.dumps(task["_assertions"]), encoding="utf-8"
        )
        (case_dir / "expected.csv").write_text("period\n", encoding="utf-8")
        try:
            failures = benchmark_runner.run_case(case_dir, structured=True)
        except subprocess.CalledProcessError:
            return {
                "compiles": True,
                "runs": False,
                "matches": False,
                "partial": 0.0,
                "failures": ["run failed"],
            }
    labels = set(task["_assertions"])
    missed = len({group for group, _ in failures} & labels)
    return {
        "compiles": True,
        "runs": True,
        "matches": not failures,
        "partial": round(max(len(labels) - missed, 0) / len(labels), 4) if labels else 0.0,
        "failures": [text for _, text in failures],
    }


GRADERS = {"repair": grade_repair, "transcribe": grade_transcribe, "extend": grade_extend}


# --- Runner ------------------------------------------------------------------

def run_eval(
    tiers: list[str],
    agent: str,
    benchmarks_dir: pathlib.Path,
    only: set[str] | None,
) -> dict:
    tasks: list[dict] = []
    if "repair" in tiers:
        tasks += repair_tasks(only)
    if "transcribe" in tiers:
        tasks += transcribe_tasks(benchmarks_dir, only)
    if "extend" in tiers:
        tasks += extend_tasks(only)
    results = []
    for task in tasks:
        try:
            submission = call_agent(agent, task, benchmarks_dir)
            score = GRADERS[task["tier"]](task, submission)
        except Exception as err:  # an agent crash scores zero, named
            score = {
                "compiles": False,
                "runs": False,
                "matches": False,
                "partial": 0.0,
                "failures": [f"agent error: {err}"],
            }
        results.append({"id": task["id"], "tier": task["tier"], "score": score})
        marker = "PASS" if score.get("matches") or (
            task["tier"] == "repair" and score["compiles"]
        ) else "fail"
        print(f"[eval][{marker}] {task['tier']}/{task['id']} partial={score['partial']}")
    summary = {}
    for tier in tiers:
        rows = [r for r in results if r["tier"] == tier]
        if not rows:
            continue
        key = "compiles" if tier == "repair" else "matches"
        summary[tier] = {
            "tasks": len(rows),
            "passed": sum(1 for r in rows if r["score"][key]),
            "mean_partial": round(
                sum(r["score"]["partial"] for r in rows) / len(rows), 4
            ),
        }
    return {"agent": agent, "summary": summary, "results": results}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--tier", default="all", help="repair|transcribe|extend|all")
    parser.add_argument("--agent", default="replay", help="replay | cmd:<command> | http(s) url")
    parser.add_argument("--cases", help="comma-separated task ids to run")
    parser.add_argument("--benchmarks-dir", default=str(ROOT / "benchmarks"),
                        help="case-set root; point at a private split for held-out runs")
    parser.add_argument("--out", help="write scored JSON here")
    parser.add_argument("--self-test", action="store_true",
                        help="sampled replay run that must score 100% (the CI gate)")
    args = parser.parse_args()

    if not pathlib.Path(CFDL).exists():
        raise SystemExit(f"cfdl binary not found at {CFDL}; run `cargo build -p cfdl-cli`")

    benchmarks_dir = pathlib.Path(args.benchmarks_dir)
    if args.self_test:
        only = set(SELF_TEST_TRANSCRIBE) | set(SELF_TEST_REPAIR)
        report = run_eval(["repair", "transcribe"], "replay", benchmarks_dir, only)
        expected = len(SELF_TEST_TRANSCRIBE) + len(SELF_TEST_REPAIR)
        graded = sum(t["tasks"] for t in report["summary"].values())
        passed = sum(t["passed"] for t in report["summary"].values())
        if graded != expected or passed != expected:
            print(
                f"agent-eval self-test FAILED: {passed}/{graded} passed "
                f"(expected {expected}/{expected})",
                file=sys.stderr,
            )
            return 1
        print(f"agent-eval self-test: OK (replay scores {passed}/{expected})")
        return 0

    tiers = ["repair", "transcribe", "extend"] if args.tier == "all" else [args.tier]
    only = set(args.cases.split(",")) if args.cases else None
    report = run_eval(tiers, args.agent, benchmarks_dir, only)
    print(json.dumps(report["summary"], indent=2))
    if args.out:
        pathlib.Path(args.out).write_text(
            json.dumps(report, indent=2) + "\n", encoding="utf-8"
        )
        print(f"wrote {args.out}")
    # The replay agent must be perfect on repair + transcribe; anything else
    # exits by its own judgment.
    if args.agent == "replay":
        for tier in ("repair", "transcribe"):
            if tier in report["summary"]:
                tier_summary = report["summary"][tier]
                if tier_summary["passed"] != tier_summary["tasks"]:
                    print(f"replay agent below 100% on {tier}", file=sys.stderr)
                    return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
