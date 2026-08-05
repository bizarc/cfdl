#!/usr/bin/env python3
"""Keep internal engineering narrative off the documentation site.

WHY THIS EXISTS. The site is largely generated from files in this repository —
pack READMEs, benchmark models, JSON schema descriptions, the language
specifications. Those files are written by and for people building CFDL, and
that is correct: recording why a design was rejected, which figure was wrong and
for how long, and what is still missing is how the work stays honest.

None of it is documentation. A reader evaluating CFDL met a wall of
shouty-capitals tolerance archaeology at the top of the HUD benchmark page, and
`which is backlog 1.3` inside the JSON schema served publicly at /schemas. Both
arrived by the same route: a maintainer wrote a note in a source file, and a
generator published it.

The split this gate protects is not "stop writing rationale". It is: rationale
belongs in the repository, conclusions belong on the site. Engineers keep
writing the way they do, and it stops reaching readers.

SCOPE. Only files that FEED the site are checked. `docs/13_feature_backlog.md`
and the design notes are not published and are deliberately not read here — this
is a gate on what escapes, not a style rule for the repository.

The specification pages are exempt for the internal-cross-reference rule: they
are normative documents published as a labelled section, and a spec citing
another spec by filename is legitimate.

ESCAPE HATCH. Append `site-allow: <reason>` on the offending line, following the
convention `tools/check-doc-examples.py` uses.

Usage: python3 tools/check-site-voice.py
"""

from __future__ import annotations

import json
import pathlib
import re
import sys

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
ALLOW = re.compile(r"site-allow:")

# Each pattern is a thing that reads as development process rather than as
# documentation. Kept narrow on purpose: a gate that cries wolf gets disabled.
PATTERNS = [
    (re.compile(r"\bdocs/\d{2}_[a-z_]+\.md"), "cites an internal document by filename"),
    (re.compile(r"\bbacklog\b", re.I), "cites the feature backlog"),
    (re.compile(r"\bSUPERSEDED\b"), "carries a supersession banner"),
    (re.compile(r"\boriginally (said|gave|stated|claimed)\b", re.I), "narrates a past mistake"),
    (re.compile(r"\bTODO\b|\bFIXME\b"), "carries a TODO"),
]


def sources() -> list[pathlib.Path]:
    """Every file whose bytes can reach a site page."""
    found: list[pathlib.Path] = []
    found += sorted(REPO_ROOT.glob("packs/*/README.md"))
    found += sorted(REPO_ROOT.glob("benchmarks/*/*/model.cfdl"))
    found += sorted(REPO_ROOT.glob("benchmarks/*/*/case.toml"))
    found += sorted(REPO_ROOT.glob("examples/*/README.md"))
    found += sorted(REPO_ROOT.glob("examples/language_tutorial/*/README.md"))
    found.append(REPO_ROOT / "docs" / "09_user_guide.md")
    # The diagnostic register feeds a generated table on an authored Reference
    # page, so what is written there reaches a reader who is not looking at a
    # specification. The rest of docs/08 is published as Specification, where an
    # internal cross-reference is legitimate — only the register is read here.
    found.append(REPO_ROOT / "docs" / "08_diagnostics.md")
    return [p for p in found if p.exists()]


def check_text_file(path: pathlib.Path) -> list[str]:
    findings = []
    rel = path.relative_to(REPO_ROOT)
    # A case.toml's COMMENTS are maintainer's notes and are no longer published;
    # only its declared `summary` reaches a page.
    only_summary = path.name == "case.toml"
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if ALLOW.search(line):
            continue
        if only_summary and not line.lstrip().startswith("summary"):
            continue
        for pattern, why in PATTERNS:
            if pattern.search(line):
                findings.append(f"  {rel}:{n}  {why}\n      {line.strip()[:100]}")
                break
    return findings


def check_schema(path: pathlib.Path) -> list[str]:
    """Schema `description` strings are served publicly and rendered as prose."""
    findings = []
    rel = path.relative_to(REPO_ROOT)

    def walk(node, trail):
        if isinstance(node, dict):
            for key, value in node.items():
                if key == "description" and isinstance(value, str):
                    if ALLOW.search(value):
                        continue
                    for pattern, why in PATTERNS:
                        if pattern.search(value):
                            findings.append(
                                f"  {rel}  {'.'.join(trail) or '<root>'}  {why}\n"
                                f"      {value.strip()[:100]}"
                            )
                            break
                else:
                    walk(value, trail + [str(key)])
        elif isinstance(node, list):
            for item in node:
                walk(item, trail)

    walk(json.loads(path.read_text(encoding="utf-8")), [])
    return findings


def main() -> int:
    findings: list[str] = []
    checked = 0

    for path in sources():
        findings += check_text_file(path)
        checked += 1

    for name in ("ir.schema.json", "results.schema.json"):
        path = REPO_ROOT / "docs" / "schemas" / name
        if path.exists():
            findings += check_schema(path)
            checked += 1

    if findings:
        print("check-site-voice: internal narrative would be published.\n", file=sys.stderr)
        print("\n".join(findings), file=sys.stderr)
        print(
            "\nThese files feed the documentation site, so what is written here is\n"
            "what a reader sees. Rationale belongs in the repository — the design\n"
            "notes and the backlog are not published and are not checked.\n"
            "\n"
            "State the conclusion instead of its history, or append\n"
            "`site-allow: <reason>` to the line.",
            file=sys.stderr,
        )
        return 1

    print(f"check-site-voice: OK ({checked} site-facing sources carry no internal narrative)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
