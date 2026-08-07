#!/usr/bin/env python3
"""Every shipped example runs with the run config that ships beside it.

WHY THIS EXISTS. `examples/language_tutorial/uncertainty/` — the lesson whose
entire subject is Monte Carlo — carried a `run.json` that said `"trials": 500`
where the field is `trial_count`. The engine rejected the config outright, so
the lesson had never produced a distribution. Nothing caught it: the golden
suite runs fixtures, the benchmark runner runs benchmarks, and the doc-example
gate compiles the model from the page but supplies its own run config.

An example is not correct because it compiles. It is correct when it runs the
way a reader will run it, which means with its own `run.json`.

Checked for every example directory holding a `model.cfdl`:

  * it compiles;
  * it runs under each run config beside it;
  * a config declaring `monte_carlo` produces a Monte Carlo block with the
    trial count it asked for — an example that silently ran deterministic is
    the failure this gate was written for.

Usage: python3 tools/check-shipped-examples.py
"""

from __future__ import annotations

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


def example_dirs() -> list[pathlib.Path]:
    roots = [REPO_ROOT / "examples"]
    found = []
    for root in roots:
        if not root.exists():
            continue
        for model in sorted(root.rglob("model.cfdl")):
            found.append(model.parent)
    return found


def run_configs(directory: pathlib.Path) -> list[pathlib.Path]:
    return sorted(
        p for p in directory.glob("run*.json") if p.is_file()
    )


def check(directory: pathlib.Path) -> list[str]:
    rel = directory.relative_to(REPO_ROOT)
    problems: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        ir = pathlib.Path(tmp) / "ir.json"
        done = subprocess.run(
            [str(CLI), "compile", str(directory), "--packs", str(PACKS), "--out", str(ir)],
            capture_output=True, text=True, encoding="utf-8",
        )
        if done.returncode != 0:
            return [f"{rel} does not compile:\n{indent(done.stderr or done.stdout)}"]

        for config in run_configs(directory):
            out = pathlib.Path(tmp) / "results.json"
            done = subprocess.run(
                [str(CLI), "run", str(ir), "--config", str(config),
                 "--packs", str(PACKS), "--out", str(out)],
                capture_output=True, text=True, encoding="utf-8",
            )
            if done.returncode != 0:
                problems.append(
                    f"{rel} does not run with {config.name}:\n"
                    f"{indent(done.stderr or done.stdout)}"
                )
                continue

            declared = json.loads(config.read_text(encoding="utf-8")).get("monte_carlo")
            if not declared:
                continue
            results = json.loads(out.read_text(encoding="utf-8"))
            monte_carlo = results.get("monte_carlo")
            if not monte_carlo or monte_carlo.get("status") != "ok":
                problems.append(
                    f"{rel}/{config.name} declares monte_carlo but the run "
                    f"produced none — the example teaches a distribution and "
                    f"returns a point estimate"
                )
            elif monte_carlo.get("trials") != declared.get("trial_count"):
                problems.append(
                    f"{rel}/{config.name} asked for "
                    f"{declared.get('trial_count')} trials and got "
                    f"{monte_carlo.get('trials')}"
                )
    return problems


def indent(text: str) -> str:
    return "\n".join(f"      {line}" for line in text.strip().splitlines()[:6])


def main() -> int:
    if not CLI.exists():
        print(f"check-shipped-examples: {CLI} not found — run `cargo build -p cfdl-cli`",
              file=sys.stderr)
        return 1

    problems: list[str] = []
    directories = example_dirs()
    runs = 0
    for directory in directories:
        runs += max(1, len(run_configs(directory)))
        problems += check(directory)

    if problems:
        print(f"\ncheck-shipped-examples: {len(problems)} problem(s):\n", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        return 1

    print(f"check-shipped-examples: OK ({len(directories)} examples compile and run "
          f"under {runs} shipped run configs)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
