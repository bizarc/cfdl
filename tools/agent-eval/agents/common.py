"""Shared pieces for eval agent adapters: the task prompt and answer parsing.

Every adapter speaks the same contract to the model — the same framing, the
same final-answer shape — so a score difference between models is a model
difference, not a prompt difference.
"""

import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[3]

ANSWER_RE = re.compile(r"```json\s*(\{.*?\})\s*```", re.DOTALL)
RESULT_SECTION = re.compile(r"## The result\n(.*?)(?=\n## )", re.DOTALL)

# How many worked models to show. Two is enough to convey the idiom without
# crowding out the specification; the agent can always ask for more structure
# through `lookup`.
EXAMPLE_COUNT = 2

# The bare loop, for A/B comparison. With CFDL_EVAL_BARE=1 the prompt carries
# only the specification — no worked examples, no convergence target, no pack
# reference. That is the control arm, and it has to remain runnable after the
# enrichments land or the comparison stops being reproducible.
def bare_mode() -> bool:
    import os

    return os.environ.get("CFDL_EVAL_BARE", "").strip() not in ("", "0", "false")

# Examples are chosen by domain, shortest first: a small complete model teaches
# the shape of the language better than a long one, which reads as a wall.
def worked_examples(task: dict, limit: int = EXAMPLE_COUNT) -> list[tuple[str, str]]:
    """Compiling models from the SAME domain as the task, never the task's own.

    An agent asked to write in a language it has only read about is being set
    an unfair problem. These are the language being used — the single most
    valuable thing to put in front of it.
    """
    case_id = task.get("id") or ""
    domain = case_id.split("/")[0] if "/" in case_id else None
    if not domain:
        return []
    own = case_id.split("/")[-1]
    candidates = []
    for model in sorted((ROOT / "benchmarks" / domain).glob("*/model.cfdl")):
        if model.parent.name == own:
            continue  # never show the answer to the task being graded
        candidates.append(model)
    candidates.sort(key=lambda p: p.stat().st_size)
    out = []
    for model in candidates[:limit]:
        summary = ""
        case_toml = model.parent / "case.toml"
        if case_toml.exists():
            for line in case_toml.read_text(encoding="utf-8").splitlines():
                if line.startswith("summary"):
                    summary = line.split("=", 1)[1].strip().strip('"')
                    break
        out.append((f"{domain}/{model.parent.name} — {summary}",
                    model.read_text(encoding="utf-8")))
    return out


def pack_reference(pack: str | None) -> str:
    """The pack's contract types and the terms each rule reads.

    Available through `lookup`, but an agent has to know to ask; putting it in
    the prompt removes a discovery step from every task.
    """
    if not pack:
        return ""
    rules = ROOT / "packs" / pack / "lowering" / "rules.toml"
    if not rules.exists():
        return ""
    # A COMPACT reference: each contract with the terms it reads and the
    # streams it emits. The full TOML is tens of thousands of tokens and is
    # re-sent on every turn of the loop; what an author needs from it is the
    # vocabulary, which `lookup` can expand on demand.
    import tomllib

    data = tomllib.loads(rules.read_text(encoding="utf-8"))
    # One rule emits one stream, so a contract's rules are collected together:
    # what an author needs is the contract, its terms, and every line it makes.
    by_contract: dict[str, dict[str, set]] = {}
    for rule in data.get("rules", []):
        name = rule.get("contract_name")
        if not name:
            continue
        entry = by_contract.setdefault(name, {"terms": set(), "streams": set()})
        for expr in _rule_expressions(rule):
            entry["terms"].update(re.findall(r"terms\.([a-z_][a-z0-9_]*)", expr))
        entry["terms"].update(rule.get("defaults", {}).keys())
        stream = rule.get("stream_name")
        if stream:
            entry["streams"].add(stream.replace("{{contract.dot_suffix}}", "[.instance]"))
    lines = [f"Contracts in pack `{pack}` — the terms each reads and the streams "
             f"it lowers to. `lookup` expands any of them."]
    for name in sorted(by_contract):
        entry = by_contract[name]
        lines.append(f"\n- `{name}`")
        if entry["terms"]:
            lines.append(f"    terms: {', '.join(sorted(entry['terms']))}")
        if entry["streams"]:
            lines.append(f"    streams: {', '.join(sorted(entry['streams']))}")
    return "\n".join(lines)


def _rule_expressions(node) -> list:
    """Every string in a lowering rule, so term reads can be found wherever
    the rule spells them."""
    if isinstance(node, str):
        return [node]
    if isinstance(node, dict):
        return [s for v in node.values() for s in _rule_expressions(v)]
    if isinstance(node, list):
        return [s for v in node for s in _rule_expressions(v)]
    return []


def stated_result(spec: str) -> str:
    """The figures the specification publishes for its own deal.

    Every CASE.md states its answer. An agent that never compares its run
    against them is flying blind on exactly the thing being graded.
    """
    match = RESULT_SECTION.search(spec or "")
    return match.group(1).strip() if match else ""

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
            "you to see; the per-period expectations are withheld and you will be "
            "graded against them with the benchmark suite's tolerances.\n"
        )
        target = "" if bare_mode() else stated_result(task.get("spec", ""))
        if target:
            parts.append(
                "\n### Verify before you answer\n\nThe specification publishes "
                "the figures this deal produces:\n\n"
                f"> {target}\n\n"
                "Treat them as your convergence target. `run` your model, compare "
                "its metrics against these, and if they disagree, use `explain` on "
                "the series that drive them to find where — then fix and re-run. "
                "Do not answer with a model whose own published figures you have "
                "not checked. If you cannot reach them, say in a comment which "
                "figure is off and by how much.\n"
            )
        examples = [] if bare_mode() else worked_examples(task)
        if examples:
            parts.append(
                "\n### Worked models in this domain\n\nThese compile, run, and "
                "match their own references. They are the idiom to follow — the "
                "shape of a model, how contracts are declared, how terms are "
                "written. Do not copy their numbers; this is a different deal.\n"
            )
            for title, source in examples:
                parts.append(f"\n#### {title}\n\n```cfdl\n{source.rstrip()}\n```\n")
        if task.get("pack"):
            parts.append(f"\nDomain pack: `{task['pack']}` (pass as `pack` to `run`).\n")
            reference = "" if bare_mode() else pack_reference(task["pack"])
            if reference:
                parts.append(f"\n### Pack reference\n\n{reference}\n")
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
