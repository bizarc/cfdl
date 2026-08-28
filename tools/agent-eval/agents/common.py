"""Shared pieces for eval agent adapters: the task prompt and answer parsing.

Every adapter speaks the same contract to the model — the same framing, the
same final-answer shape — so a score difference between models is a model
difference, not a prompt difference.
"""

import json
import re

ANSWER_RE = re.compile(r"```json\s*(\{.*?\})\s*```", re.DOTALL)

PROMPT_HEADER = """You are a CFDL authoring agent. CFDL is a declarative cash-flow \
modeling language: contracts lower to streams through a pack's lowering rules — \
contracts are vocabulary, streams are the cash. You have ONLY the cfdl tools \
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


def extract_files(text: str) -> dict | None:
    """The last well-formed `{"files": {...}}` fenced block in the text."""
    for candidate in reversed(ANSWER_RE.findall(text or "")):
        try:
            parsed = json.loads(candidate)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed.get("files"), dict) and parsed["files"]:
            return {"files": parsed["files"]}
    return None
