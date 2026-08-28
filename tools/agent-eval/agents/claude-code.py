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

import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[3]

ANSWER_RE = re.compile(r"```json\s*(\{.*?\})\s*```", re.DOTALL)

PROMPT_HEADER = """You are a CFDL authoring agent. CFDL is a declarative cash-flow \
modeling language: contracts lower to streams through a pack's lowering rules — \
contracts are vocabulary, streams are the cash. You have ONLY the cfdl MCP tools \
(compile, run, lookup, skeleton, explain). Do not use any file, shell, or web tool. \
Never reference filesystem paths; pass sources inline via the tools' `files` \
parameter.

Work the loop: draft sources, `compile` them (diagnostics are the repair signal), \
fix, and for a full model also `run` it (pass the run configuration inline as \
`config`, and the pack name as `pack`) and sanity-check the numbers with the \
summary and `explain`. Use `lookup` for pack contract rosters and terminology, \
and `skeleton` for a valid starting point.

When you are done, output your final answer as EXACTLY ONE fenced json block of \
the shape:

```json
{"files": {"model.cfdl": "<full source>"}}
```

No prose after the block. The files must be your best, compile-verified attempt.
"""


def build_prompt(task: dict) -> str:
    parts = [PROMPT_HEADER]
    tier = task.get("tier")
    if tier == "repair":
        parts.append(
            "\n## Task: repair\n\nThe model below fails to compile with the "
            "diagnostics shown. Make the MINIMAL change that fixes every diagnosed "
            "problem while preserving the model's identity, verify with `compile`, "
            "and return every source file.\n"
        )
        for name, source in task.get("files", {}).items():
            parts.append(f"\n### {name}\n```cfdl\n{source}\n```\n")
        parts.append(
            "\n### Diagnostics\n```json\n"
            + json.dumps(task.get("diagnostics", []), indent=1)
            + "\n```\n"
        )
    elif tier == "transcribe":
        parts.append(
            "\n## Task: transcribe\n\nAuthor a CFDL model from the case "
            "specification below. The reference material is what the case allows "
            "you to see; the expected results are withheld and you will be graded "
            "against them with the benchmark suite's tolerances. Verify your model "
            "compiles and runs under the given run configuration before answering.\n"
        )
        if task.get("pack"):
            parts.append(f"\nDomain pack: `{task['pack']}` (pass as `pack` to `run`).\n")
        parts.append(f"\n### Specification (CASE.md)\n\n{task.get('spec', '')}\n")
        parts.append(
            "\n### Run configuration (pass inline as `config`)\n```json\n"
            + json.dumps(task.get("run_config", {}), indent=1)
            + "\n```\n"
        )
        for name, content in task.get("reference", {}).items():
            parts.append(f"\n### Reference: {name}\n\n````\n{content}\n````\n")
        if task.get("reference_binaries"):
            parts.append(
                "\n(Reference files not shown: "
                + ", ".join(task["reference_binaries"])
                + ")\n"
            )
    else:  # extend
        parts.append(
            "\n## Task: extend\n\nModify the model below per the change request, "
            "keeping everything else identical. Verify with `compile` and `run`.\n"
            f"\n### Change request\n\n{task.get('request', '')}\n"
        )
        for name, source in task.get("files", {}).items():
            parts.append(f"\n### {name}\n```cfdl\n{source}\n```\n")
        parts.append(
            "\n### Run configuration\n```json\n"
            + json.dumps(task.get("run_config", {}), indent=1)
            + "\n```\n"
        )
    return "".join(parts)


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
    answers = ANSWER_RE.findall(text)
    for candidate in reversed(answers):
        try:
            parsed = json.loads(candidate)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed.get("files"), dict) and parsed["files"]:
            print(json.dumps({"files": parsed["files"]}))
            return 0
    print(f"no files block in agent answer; tail: {text[-400:]}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
