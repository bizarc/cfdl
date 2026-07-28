#!/usr/bin/env python3
"""Prove that documentation examples do what their prose claims.

The benchmark models are held to an independent reference on every commit.
The examples in the pack guides — which is what a reader actually meets first —
were held to nothing, and it showed: the CRE quick start advertised
"lease-by-lease with recoveries and rollover", contained one tenant, and
computed recoveries of exactly zero against a property that declared no
expenses. It compiled and ran, so nothing objected.

This compiles and runs every complete model in the documentation and fails on:

  * a model that does not compile or run;
  * a stream whose total is zero — a feature shown but not exercised, which
    reads to a newcomer as a broken feature;
  * a stream whose total contradicts its declared direction — an `outflow`
    that nets positive is the pmt sign error that once reported debt service
    as income and overstated an example's NPV tenfold.

Both stream checks can be waived per-stream where the zero or the sign is the
point being made, using the repo's existing escape-hatch convention:

    // examples-allow: <stream substring> — <reason>

Usage:
    python3 tools/check-doc-examples.py            # check every source
    python3 tools/check-doc-examples.py --verbose  # list every stream total
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
import tempfile

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
CLI = REPO_ROOT / "target" / "debug" / "cfdl"
PACKS = REPO_ROOT / "packs"

# Documentation whose fenced cfdl blocks are meant to be complete models.
SOURCES = sorted(REPO_ROOT.glob("packs/*/README.md"))

FENCE = re.compile(r"```cfdl\n(.*?)```", re.S)
ALLOW = re.compile(r"//\s*examples-allow:\s*(\S+)")
USE_PACK = re.compile(r'^\s*use\s+pack\s+"([^"]+)"', re.M)


def complete_models(path: pathlib.Path) -> list[tuple[int, str]]:
    """Fenced cfdl blocks that are whole models, with their line numbers.

    A fragment — the single contract a "Recipes" section shows — has no
    `version` line and cannot be compiled on its own.
    """
    text = path.read_text()
    found = []
    for match in FENCE.finditer(text):
        body = match.group(1)
        if not re.match(r"\s*version\s", body):
            continue
        line = text[: match.start()].count("\n") + 1
        found.append((line, body))
    return found


def run_model(source: str, workdir: pathlib.Path) -> tuple[dict, dict]:
    """Compile and run one model; returns (ir, results)."""
    (workdir / "model.cfdl").write_text(source)
    (workdir / "run.json").write_text(
        json.dumps({"deterministic": {"annual_discount_rate": 0.08}})
    )
    ir_path = workdir / "ir.json"
    res_path = workdir / "results.json"

    compile_cmd = [str(CLI), "compile", str(workdir), "--packs", str(PACKS),
                   "--out", str(ir_path)]
    done = subprocess.run(compile_cmd, capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(f"compile failed:\n{indent(done.stderr or done.stdout)}")

    run_cmd = [str(CLI), "run", str(ir_path), "--config", str(workdir / "run.json"),
               "--packs", str(PACKS), "--out", str(res_path)]
    pack = USE_PACK.search(source)
    if pack:
        run_cmd += ["--pack", pack.group(1)]
    done = subprocess.run(run_cmd, capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(f"run failed:\n{indent(done.stderr or done.stdout)}")

    return json.loads(ir_path.read_text()), json.loads(res_path.read_text())


def indent(text: str) -> str:
    return "\n".join(f"      {line}" for line in text.strip().splitlines()[:6])


def stream_totals(results: dict) -> dict[str, float]:
    series = results["deterministic"]["series"]
    return {
        name: sum(point["amount"] for point in block["values"])
        for name, block in series.items()
        if name != "model.net_cash_flow"
    }


def directions(ir: dict) -> dict[str, str]:
    """Declared direction per stream name, from the IR."""
    return {s["name"]: s.get("direction", "") for s in ir.get("streams", [])}


def check(path: pathlib.Path, line: int, source: str, verbose: bool) -> list[str]:
    waived = ALLOW.findall(source)
    problems: list[str] = []

    with tempfile.TemporaryDirectory() as tmp:
        try:
            ir, results = run_model(source, pathlib.Path(tmp))
        except RuntimeError as err:
            return [f"{path.relative_to(REPO_ROOT)}:{line}: {err}"]

    totals = stream_totals(results)
    if not totals:
        return [f"{path.relative_to(REPO_ROOT)}:{line}: model emits no streams"]

    declared = directions(ir)
    for name, total in sorted(totals.items()):
        if verbose:
            print(f"      {name:52} {total:16,.2f}")
        if any(token in name for token in waived):
            continue

        if abs(total) < 0.005:
            problems.append(
                f"{path.relative_to(REPO_ROOT)}:{line}: stream `{name}` totals zero — "
                "the example shows a feature it never exercises"
            )
            continue

        # `stream.<name>` in results maps to `<name>` in the IR.
        bare = name[len("stream.") :] if name.startswith("stream.") else name
        direction = declared.get(bare, "")
        if direction == "outflow" and total > 0:
            problems.append(
                f"{path.relative_to(REPO_ROOT)}:{line}: stream `{name}` is declared "
                f"`outflow` but totals +{total:,.2f} — check the sign of its amount"
            )
        elif direction == "inflow" and total < 0:
            problems.append(
                f"{path.relative_to(REPO_ROOT)}:{line}: stream `{name}` is declared "
                f"`inflow` but totals {total:,.2f} — check the sign of its amount"
            )

    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verbose", action="store_true",
                        help="print every stream total as it is checked")
    args = parser.parse_args()

    if not CLI.exists():
        print(f"check-doc-examples: {CLI} not found — run `cargo build -p cfdl-cli`",
              file=sys.stderr)
        return 1

    problems: list[str] = []
    checked = 0
    for path in SOURCES:
        for line, source in complete_models(path):
            checked += 1
            if args.verbose:
                print(f"  {path.relative_to(REPO_ROOT)}:{line}")
            problems += check(path, line, source, args.verbose)

    if problems:
        print(f"\ncheck-doc-examples: {len(problems)} problem(s) in {checked} example(s):\n",
              file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        print(
            "\nIf a zero or a sign is deliberate, waive that stream in the model:\n"
            "  // examples-allow: <stream substring> — <reason>",
            file=sys.stderr,
        )
        return 1

    print(f"check-doc-examples: OK ({checked} documentation examples compile, run, "
          "and exercise every stream they declare)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
