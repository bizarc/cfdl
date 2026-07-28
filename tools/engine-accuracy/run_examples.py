#!/usr/bin/env python3
"""
Run each example from example_runs.json: compile then run, capture results to out/.
On failure, record to run_failures.json for the report.
Run from repo root. Requires CFDL_BIN or builds target/debug/cfdl.
"""
import json
import os
import subprocess
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent
CATALOG = SCRIPT_DIR / "example_runs.json"
OUT_DIR = SCRIPT_DIR / "out"
FAILURES_PATH = SCRIPT_DIR / "run_failures.json"


def get_cfdl_bin():
    env_bin = os.environ.get("CFDL_BIN")
    if env_bin:
        return Path(env_bin)
    default = REPO_ROOT / "target" / "debug" / "cfdl"
    if not default.exists():
        subprocess.run(
            ["cargo", "build", "-p", "cfdl-cli"],
            cwd=REPO_ROOT,
            check=True,
        )
    return default


def main():
    with open(CATALOG, encoding="utf-8") as f:
        runs = json.load(f)
    cfdl = get_cfdl_bin()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    failures = []

    for run in runs:
        model_root = run["model_root"]
        run_label = run["run_label"]
        config_path = run.get("config_path")
        packs_dir = run.get("packs_dir")
        slug = run["slug"]
        out_name = f"{slug}_{run_label}.results.json"
        out_path = OUT_DIR / out_name
        run_id = f"{model_root} ({run_label})"

        compile_args = [str(cfdl), "compile", model_root, "--out", ""]
        run_args = [str(cfdl), "run", "", "--out", str(out_path)]

        if packs_dir:
            compile_args.extend(["--packs", packs_dir])
            run_args.extend(["--packs", packs_dir])
        if config_path:
            run_args.extend(["--config", config_path])
        else:
            run_args.extend(["--rate", "0.1"])

        with tempfile.NamedTemporaryFile(suffix=".ir.json", delete=False) as tmp:
            ir_path = tmp.name
        try:
            compile_args[4] = ir_path
            run_args[2] = ir_path

            cp = subprocess.run(
                compile_args,
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
            )
            if cp.returncode != 0:
                failures.append({
                    "run_id": run_id,
                    "model_root": model_root,
                    "run_label": run_label,
                    "stage": "compile",
                    "exit_code": cp.returncode,
                    "stderr": cp.stderr or "(none)",
                    "stdout": cp.stdout or "(none)",
                })
                continue

            rp = subprocess.run(
                run_args,
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
            )
            if rp.returncode != 0:
                failures.append({
                    "run_id": run_id,
                    "model_root": model_root,
                    "run_label": run_label,
                    "stage": "run",
                    "exit_code": rp.returncode,
                    "stderr": rp.stderr or "(none)",
                    "stdout": rp.stdout or "(none)",
                })
                if out_path.exists():
                    out_path.unlink()
        finally:
            if os.path.exists(ir_path):
                os.unlink(ir_path)

    with open(FAILURES_PATH, "w", encoding="utf-8") as f:
        json.dump(failures, f, indent=2)
    print(f"Runs: {len(runs)}, failures: {len(failures)}. Results in {OUT_DIR}, failures in {FAILURES_PATH}")
    return 0  # always succeed so pipeline continues to validate and report


if __name__ == "__main__":
    raise SystemExit(main())
