#!/usr/bin/env python3
"""Re-grade saved artifacts without calling a model.

    rescore.py <scorecard.json> [...]

An eval's scoring should be improvable after the fact. Because every run now
keeps the sources it produced, a change to how they are judged can be applied
to work already done — no re-spend, and old runs stay comparable with new
ones. This is what makes the name-independent `economics` score usable on
arms that were graded before it existed.
"""
import importlib.util
import json
import pathlib
import shutil
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("runner", HERE / "runner.py")
runner = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(runner)

CASE_FILES = ("case.toml", "run.json", "expected.csv", "expected_metrics.json",
              "expected_scenarios.json", "expected_monte_carlo.json")


def regrade(scorecard: pathlib.Path) -> dict:
    data = json.loads(scorecard.read_text(encoding="utf-8"))
    out = []
    missing = 0
    for row in data.get("results", []):
        artifact = resolve_artifact(scorecard, row.get("artifact"))
        if artifact is None:
            # Say so loudly: silently keeping the old score would report a
            # re-grade that never happened.
            missing += 1
            out.append({**row, "rescore": "no artifact found"})
            continue
        case_dir = runner.ROOT / "benchmarks" / row["id"]
        with tempfile.TemporaryDirectory() as tmp:
            target = pathlib.Path(tmp) / "case"
            target.mkdir()
            for source in artifact.glob("*.cfdl"):
                shutil.copy(source, target)
            for name in CASE_FILES:
                if (case_dir / name).exists():
                    shutil.copy(case_dir / name, target)
            task = {"tier": "transcribe", "id": row["id"], "_case_dir": str(case_dir)}
            files = {p.name: p.read_text(encoding="utf-8")
                     for p in artifact.glob("*.cfdl")}
            score = runner.grade_transcribe(task, {"files": files})
        out.append({**row, "score": score})
    if missing:
        print(f"  warning: {missing} of {len(out)} rows had no artifact to re-grade; "
              f"their original scores are unchanged", file=sys.stderr)
    tiers = sorted({r["tier"] for r in out})
    return {**data, "summary": runner.summarize(out, tiers), "results": out}


def resolve_artifact(scorecard, artifact):
    """Find a run's saved sources.

    Artifact paths are recorded as the runner saw them — relative to the
    directory the run was launched from. A scorecard is portable and may be
    re-graded from anywhere, so try the recorded path, then the same tail
    beside the scorecard itself.
    """
    if not artifact:
        return None
    tail = pathlib.Path(artifact)
    candidates = [tail]
    if len(tail.parts) >= 3:
        candidates.append(scorecard.parent / pathlib.Path(*tail.parts[-3:]))
    for candidate in candidates:
        if candidate.exists() and any(candidate.glob("*.cfdl")):
            return candidate
    return None


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    for arg in sys.argv[1:]:
        path = pathlib.Path(arg)
        report = regrade(path)
        rescored = path.with_name(path.stem + "-rescored.json")
        rescored.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        s = report["summary"].get("transcribe", {})
        print(f"{path.name}: matched {s.get('passed')}/{s.get('tasks')} | "
              f"partial {s.get('mean_partial')} | economics {s.get('mean_economics')} "
              f"-> {rescored.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
