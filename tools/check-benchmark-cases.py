#!/usr/bin/env python3
"""Every benchmark case describes itself, to the same outline.

WHY THIS EXISTS. A benchmark page used to carry a one-sentence summary, the
model source and a metrics table. A reader could see WHAT was asserted and not
what the case was, what the reference was, whether the reference could be
redistributed, or what a residual meant — and for several cases the residual is
the most interesting thing about them.

The outline is fixed so the twenty-five read as one set rather than
twenty-five essays. Prose that is NOT for readers — which figure was wrong and
for how long, why a tolerance moved — stays in NOTES.md and in the comments at
the top of case.toml, neither of which is published.

Usage: python3 tools/check-benchmark-cases.py
"""
from __future__ import annotations

import pathlib
import re
import sys

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

ROOT = pathlib.Path(__file__).resolve().parents[1]

# The outline, in order. Each answers one of the questions a reader arrives with.
REQUIRED = [
    "## The case",       # what deal is this
    "## The reference",  # what is it checked against, and can that ship
    "## What it exercises",  # which pack, which contracts, which language features
    "## The result",     # how well did it match, and what was asserted
    "## The delta",      # what any residual means, or that it is exact
]


def main() -> int:
    cases = sorted(p.parent for p in (ROOT / "benchmarks").glob("*/*/model.cfdl"))
    missing: list[str] = []
    problems: list[str] = []

    for case_dir in cases:
        rel = case_dir.relative_to(ROOT).as_posix()
        described = case_dir / "CASE.md"
        if not described.exists():
            missing.append(rel)
            continue

        text = described.read_text(encoding="utf-8")
        # The site page already carries a title; `sync-content.mjs` embeds this
        # file verbatim under it, so an H1 here renders as a second title
        # stacked on the first. Three cases shipped that way before this check.
        if re.search(r"^# ", text, flags=re.M):
            problems.append(
                f"{rel}/CASE.md: carries an H1 ('# ...'). The published page "
                f"supplies the title; start at '## The case'."
            )
            continue
        headings = re.findall(r"^## .+$", text, flags=re.M)
        if headings[: len(REQUIRED)] != REQUIRED:
            problems.append(
                f"{rel}/CASE.md: headings are {headings or '[none]'};\n"
                f"    expected, in order: {REQUIRED}"
            )
            continue

        # A heading with nothing under it is worse than an absent file: the page
        # renders an empty section and reads as an oversight rather than a gap.
        for index, heading in enumerate(REQUIRED):
            start = text.index(heading) + len(heading)
            end = (
                text.index(REQUIRED[index + 1])
                if index + 1 < len(REQUIRED)
                else len(text)
            )
            if not text[start:end].strip():
                problems.append(f"{rel}/CASE.md: '{heading}' is empty.")

    if missing:
        print(f"check-benchmark-cases: {len(missing)} of {len(cases)} cases have no CASE.md:")
        for rel in missing:
            print(f"  {rel}")
    for problem in problems:
        print(f"check-benchmark-cases: {problem}")

    if problems:
        return 1
    if missing:
        # A hard failure now that every case is described: a new benchmark
        # arrives with its description or it does not arrive.
        print("  Every case describes itself. Add CASE.md with the headings above.")
        return 1

    print(f"check-benchmark-cases: OK ({len(cases)} cases, each to the same outline)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
