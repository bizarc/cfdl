#!/usr/bin/env python3
"""Generate docs/glossary.md from docs/terminology.toml.

Usage:
    python3 tools/gen-glossary.py            # write docs/glossary.md
    python3 tools/gen-glossary.py --check    # fail if it is out of date

WHY GENERATED. ISO/IEC/IEEE 26514 and IEC/IEEE 82079-1 both require defined
terms for a product with specialist vocabulary, and this one has two that
overlap: finance and compiler construction. A hand-written glossary would be a
second list of definitions beside terminology.toml, and the two would disagree
within a release — which is worse than no glossary, because a reader cannot tell
which one is current.

So the register is the source and the page is output. A term is defined once, in
the file the writing standard already points at.

WHAT IS PUBLISHED. Technical Names and Technical Verbs only. The register's
other sections — preferred spellings, not-approved words, the pedagogical
metaphors — are editorial instructions to writers, not information a reader
needs, and publishing them would put the house style guide on the product's
documentation site.

The page is written to docs/ rather than straight into site/content because
site/scripts/sync-content.mjs already has a byte-copy path for docs/*.md, and
reusing it means the glossary is owned and freshness-checked the same way every
specification page is. It also keeps this script free of any knowledge of the
site's layout.
"""

from __future__ import annotations

import pathlib
import sys
import tomllib

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

ROOT = pathlib.Path(__file__).resolve().parents[1]
REGISTER = ROOT / "docs" / "terminology.toml"
OUTPUT = ROOT / "docs" / "glossary.md"

# Printed above each group. The order is the order a reader meets the ideas:
# what the language is made of, what the compiler does with it, what the domain
# calls things.
CATEGORIES = [
    ("language", "Language", "The constructs a model is written from."),
    ("compiler", "Compiler", "What the toolchain does with a model, and what it reports."),
    ("finance", "Finance", "Domain terms the packs and the documentation use in their standard sense."),
]


def entries(register: dict) -> list[dict]:
    # A term may be registered before it is built, to reserve the word and to
    # record why a name was chosen over its rejected alternatives. The register
    # wants that entry; the published page must not carry it, because the page
    # promises "the one meaning it carries" for terms the product actually uses.
    # `status = "proposed"` keeps the two apart. Absent means shipped.
    return [
        e
        for e in register.get("technical_name", [])
        if e.get("status", "shipped") != "proposed"
    ]


def render(register: dict) -> str:
    names = entries(register)
    verbs = register.get("technical_verb", [])

    out = [
        "# Glossary",
        "",
        "Every term CFDL uses in a specific sense, with the one meaning it carries.",
        "",
        "Terms are listed with one definition each. Where a term is an abbreviation,",
        "the expansion given is the form to use on first mention.",
        "",
    ]

    for key, heading, blurb in CATEGORIES:
        group = sorted(
            (e for e in names if e.get("category") == key),
            key=lambda e: e["term"].lower(),
        )
        if not group:
            continue
        out += [f"## {heading}", "", blurb, ""]
        for e in group:
            term = e["term"]
            expanded = e.get("expand_on_first_use")
            head = f"**{term}**"
            if expanded and expanded.lower() != term.lower():
                head += f" — {expanded}"
            out += [f"{head}", "", f"{e['definition']}", ""]

    if verbs:
        out += [
            "## Verbs",
            "",
            "Each of these describes one action and is used for no other.",
            "",
        ]
        for e in sorted(verbs, key=lambda e: e["term"].lower()):
            out += [f"**{e['term']}**", "", f"{e['definition']}", ""]

    return "\n".join(out).rstrip() + "\n"


def main() -> int:
    register = tomllib.loads(REGISTER.read_text(encoding="utf-8"))
    page = render(register)
    check = "--check" in sys.argv

    if check:
        current = OUTPUT.read_text(encoding="utf-8") if OUTPUT.exists() else ""
        if current != page:
            print(
                "gen-glossary: docs/glossary.md is out of date with "
                "docs/terminology.toml.\n\n  Regenerate it:  make glossary",
                file=sys.stderr,
            )
            return 1
        n = len(entries(register)) + len(register.get("technical_verb", []))
        print(f"gen-glossary: OK ({n} terms, page matches the register)")
        return 0

    OUTPUT.write_text(page, encoding="utf-8")
    n = len(entries(register)) + len(register.get("technical_verb", []))
    print(f"gen-glossary: wrote {OUTPUT.relative_to(ROOT)} ({n} terms)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
