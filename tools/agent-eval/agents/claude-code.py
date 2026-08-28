#!/usr/bin/env python3
"""A real agent for the eval: headless Claude Code driving the cfdl-mcp loop.

Adapter contract (runner.py `--agent cmd:`): task JSON on stdin,
`{"files": {...}}` on stdout. Everything else goes to stderr.

The agent is sandboxed the way an honest transcribe run requires:

  - it runs in an empty temporary directory, not the repository;
  - its ONLY allowed tools are the cfdl-mcp verifier loop (compile, run,
    lookup, skeleton, explain) — no file tools, no shell, and no `diff`,
    which could otherwise be pointed at a benchmark case directory and
    read the expectations it is being graded against;
  - the MCP server gets `--packs` only, no benchmarks directory.

Knobs (environment): CFDL_EVAL_MODEL (default "sonnet").

Contamination caveat: this repository is public; a public-split score for
any trained model is an upper bound, not the headline. The headline comes
from the private split (docs/32 Phase 3).
"""

import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[3]

_spec = importlib.util.spec_from_file_location(
    "common", pathlib.Path(__file__).parent / "common.py"
)
common = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(common)

build_prompt = common.build_prompt


def main() -> int:
    task = json.load(sys.stdin)
    model = os.environ.get("CFDL_EVAL_MODEL", "sonnet")
    mcp_binary = ROOT / "target" / "debug" / "cfdl-mcp"
    if not mcp_binary.exists():
        print("cfdl-mcp not built; run `cargo build -p cfdl-mcp`", file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory() as tmp:
        workdir = pathlib.Path(tmp)
        mcp_config = {
            "mcpServers": {
                "cfdl": {
                    "command": str(mcp_binary),
                    "args": ["--packs", str(ROOT / "packs")],
                }
            }
        }
        config_path = workdir / "mcp.json"
        config_path.write_text(json.dumps(mcp_config), encoding="utf-8")
        allowed = ",".join(
            f"mcp__cfdl__{tool}"
            for tool in ("compile", "run", "lookup", "skeleton", "explain")
        )
        result = subprocess.run(
            [
                "claude",
                "-p",
                "--output-format", "json",
                "--model", model,
                "--mcp-config", str(config_path),
                "--allowedTools", allowed,
            ],
            input=build_prompt(task).encode("utf-8"),
            capture_output=True,
            cwd=workdir,
            timeout=1500,
        )
    if result.returncode != 0:
        print(
            f"claude exited {result.returncode}: "
            f"{result.stderr.decode('utf-8', 'replace')[:400]}",
            file=sys.stderr,
        )
        return 1
    payload = json.loads(result.stdout.decode("utf-8"))
    text = payload.get("result", "")
    answer = common.extract_files(text)
    if answer:
        print(json.dumps(answer))
        return 0
    print(f"no files block in agent answer; tail: {text[-400:]}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
