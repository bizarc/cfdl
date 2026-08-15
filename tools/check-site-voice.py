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
are normative documents published as a labeled section, and a spec citing
another spec by filename is legitimate.

ESCAPE HATCH. Append `site-allow: <reason>` on the offending line, following the
convention `tools/check-doc-examples.py` uses. The CFDL-CE rules use
`ste-allow: <rule id> <reason>` — a separate marker so a reviewer can see which
standard is being waived.

CFDL-CE. This gate also enforces the mechanical subset of the writing standard
(docs/22_cfdl_controlled_english.md): retired spellings, retired synonyms,
number formats, contractions. The word lists come from docs/terminology.toml so
the gate and the register cannot drift. What is deliberately NOT here: sentence
length, voice, and imperative form — those are judgment calls (see the tiering
in docs/22), and a gate that flags judgment gets disabled, which is this file's
founding rule.

Usage: python3 tools/check-site-voice.py
"""

from __future__ import annotations

import json
import pathlib
import re
import sys
import tomllib

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
ALLOW = re.compile(r"site-allow:|ste-allow:")

# Each pattern is a thing that reads as development process rather than as
# documentation. Kept narrow on purpose: a gate that cries wolf gets disabled.
PATTERNS = [
    (re.compile(r"\bdocs/\d{2}_[a-z_]+\.md"), "cites an internal document by filename"),
    (re.compile(r"\bbacklog\b", re.I), "cites the feature backlog"),
    (re.compile(r"\bSUPERSEDED\b"), "carries a supersession banner"),
    (re.compile(r"\boriginally (said|gave|stated|claimed)\b", re.I), "narrates a past mistake"),
    (re.compile(r"\bTODO\b|\bFIXME\b"), "carries a TODO"),
    # A reader does not know or care how a reference was produced. Several
    # sources cannot be redistributed, so the reference is recreated from them —
    # naming the file that does the recreating describes our workshop instead of
    # the case, and reads as though the number were invented here.
    (
        re.compile(r"\breference_gen(\.py)?\b"),
        "names the internal reference generator",
    ),
    (
        re.compile(r"\bin-house\b", re.I),
        "describes where the reference was written rather than what it is",
    ),
    (
        re.compile(r"\bpending practitioner (review|Excel review)\b", re.I),
        "carries an internal review status",
    ),
    # Documentation states what the software does. Calling a statement honest
    # implies the others are not, and it is a claim about the authors rather
    # than about the product.
    (re.compile(r"\b(dis)?honest(ly|y)?\b", re.I), "vouches for its own candour"),
    # Development narrative: what was tried, what was rejected, what a page
    # used to say. All of it belongs in the repository.
    (
        re.compile(
            r"\bwe (chose|decided|considered|rejected|opted|originally|used to)\b", re.I
        ),
        "narrates a decision instead of stating the outcome",
    ),
    (
        re.compile(r"\bthis (page|section|file) (used to|previously)\b", re.I),
        "describes an earlier version of itself",
    ),
    # THE SITE DOES NOT POINT INTO THE REPOSITORY. cfdl.dev is the product's
    # documentation and stands alone; the repository is not something a reader
    # has, and will not stay public. A page that says "generated from
    # examples/foo/" or "see crates/cfdl-server/src/limits.rs" offers a
    # destination that does not exist for them.
    #
    # A path the READER creates — `packs/` beside their own model, their own
    # `model.cfdl` — is not this, which is why the pattern is anchored to the
    # repository's own top-level directories.
    (
        re.compile(
            r"\bgenerated from `?(examples|benchmarks|crates|fixtures|tools|docs)/", re.I
        ),
        "publishes a repository path as provenance",
    ),
    (
        re.compile(
            r"`(crates|fixtures|tools|benchmarks)/[A-Za-z0-9_./<>-]*`"
        ),
        "cites a file in the repository, which a reader does not have",
    ),
    (
        re.compile(r"\bin (this|the) repositor(y|ies)\b|\bthe repo's\b|\bfrom a checkout\b", re.I),
        "assumes the reader has the repository",
    ),
    (re.compile(r"https://github\.com/"), "links into the repository"),
    # Ornament. Each of these is a claim the reader should be left to make.
    (
        re.compile(
            r"\b(blazing(ly)?|lightning[- ]fast|world[- ]class|cutting[- ]edge"
            r"|state[- ]of[- ]the[- ]art|revolutionary|seamless(ly)?|effortless(ly)?"
            r"|game[- ]chang(er|ing)|best[- ]in[- ]class|unparalleled|robust and"
            r"|powerful and|simply put|crown jewel)\b",
            re.I,
        ),
        "reads as marketing rather than documentation",
    ),
]


# --- CFDL-CE: the mechanical subset of docs/22 ------------------------------
#
# Word lists load from the terminology register rather than living here, so a
# new retired spelling is one TOML line, not a code change. A missing or
# unparsable register fails the gate loudly — a prose standard whose word list
# silently vanished would report OK forever.
#
# Matching runs on a copy of the line with inline code spans removed: `run
# config` naming a literal file or field is correct (the register says so), and
# a backticked identifier is code, not prose.
INLINE_CODE = re.compile(r"`[^`]*`")


def ce_patterns() -> list[tuple[re.Pattern[str], str]]:
    register = tomllib.loads(
        (REPO_ROOT / "docs" / "terminology.toml").read_text(encoding="utf-8")
    )
    retired = sorted(register["spelling"]["map"], key=len, reverse=True)
    return [
        (
            re.compile(r"\b(" + "|".join(retired) + r")\b", re.I),
            "uses a spelling the register retired (docs/terminology.toml [spelling.map])",
        ),
        # One concept, one term. The approved forms are `run configuration` and
        # `results document`; the patterns are ordered so the longer approved
        # form never triggers its own prefix.
        (
            re.compile(r"\brun config(?!uration)s?\b", re.I),
            "names the run configuration by a retired synonym",
        ),
        (re.compile(r"\brun settings\b", re.I), "names the run configuration by a retired synonym"),
        (re.compile(r"\boutput document\b", re.I), "names the results document by a retired synonym"),
        (re.compile(r"\bresults doc\b", re.I), "names the results document by a retired synonym"),
        # `hit` only as an instruction aimed at a control — the bare verb has
        # honest uses ("collections hit 60,000") that must not fire.
        (
            re.compile(r"\bHit\b(?=\s+(\*\*|`))"),
            "instructs with `hit`; the approved verb is `click`",
        ),
        # Number formats (docs/22 §4). A digit, then U+00D7, then no digit is a
        # valuation multiple; spaced arithmetic (6,000 × 12) and grid
        # dimensions (3×3) are correct and do not match.
        (
            re.compile(r"\d×(?!\d)"),
            "writes a valuation multiple with U+00D7; write 8.0x",
        ),
        (
            re.compile(r"\$\d[\d,]*(?:\.\d+)?mm\b"),
            "writes millions as mm; write $33.6m, not $33.6mm",
        ),
        (re.compile("‑"), "contains a non-breaking hyphen (U+2011); use a plain hyphen"),
        # No contractions (docs/22 V6). The closed pronoun list keeps
        # possessives ("the model's logic") out.
        (
            re.compile(
                r"\b\w+n[’']t\b"
                r"|\b(?:it|that|there|here|what|let|who|they)[’']s\b"
                r"|\b\w+[’'](?:re|ve|ll)\b",
                re.I,
            ),
            "uses a contraction; write the words out",
        ),
    ]


CE_PATTERNS = ce_patterns()


def spec_sources() -> list[pathlib.Path]:
    """Published pages whose bytes come from docs/, checked for CE only.

    The specifications are exempt from the narrative rules — a spec citing
    another spec by filename is legitimate — but their spelling and formats
    reach readers like any other page. Before this list they were the one
    published surface no prose gate read at all.
    """
    # docs/08 is absent here because sources() already reads it in full.
    found = sorted(REPO_ROOT.glob("docs/0[1-7]_*.md"))
    found.append(REPO_ROOT / "docs" / "glossary.md")
    found.append(REPO_ROOT / "distribution" / "install-configure.md")
    return [p for p in found if p.exists()]


def sources() -> list[pathlib.Path]:
    """Every file whose bytes can reach a site page."""
    found: list[pathlib.Path] = []
    found += sorted(REPO_ROOT.glob("packs/*/README.md"))
    found += sorted(REPO_ROOT.glob("benchmarks/*/*/model.cfdl"))
    found += sorted(REPO_ROOT.glob("benchmarks/*/*/case.toml"))
    found += sorted(REPO_ROOT.glob("benchmarks/*/*/CASE.md"))
    found += sorted(REPO_ROOT.glob("examples/*/README.md"))
    found += sorted(REPO_ROOT.glob("examples/language_tutorial/*/README.md"))
    # The diagnostic register feeds a generated table on an authored Reference
    # page, so what is written there reaches a reader who is not looking at a
    # specification. The rest of docs/08 is published as Specification, where an
    # internal cross-reference is legitimate — only the register is read here.
    found.append(REPO_ROOT / "docs" / "08_diagnostics.md")
    # Authored site pages. These are not generated from anywhere, so nothing
    # else was checking them — the gate was reading every source that reaches a
    # page except the pages themselves.
    # A generated page is a copy: fixing it there would be overwritten on the
    # next sync, and its source is already in this list. Only authored pages
    # are read.
    found += [
        p
        for p in sorted((REPO_ROOT / "site" / "content").rglob("*.md"))
        if "generated: full" not in p.read_text(encoding="utf-8")[:400]
    ]
    # The training site's chapters are published pages like the site's docs —
    # authored, never generated — and the exercise models that will sit beside
    # them are reader-facing the same way the examples are.
    found += sorted((REPO_ROOT / "learn" / "content").rglob("*.mdx"))
    found += sorted(REPO_ROOT.glob("training/exercises/*/*/README.md"))
    return [p for p in found if p.exists()]


def check_text_file(path: pathlib.Path, *, narrative: bool = True) -> list[str]:
    """`narrative=False` runs only the CE rules — the specification exemption."""
    findings = []
    rel = path.relative_to(REPO_ROOT)
    # A case.toml's COMMENTS are maintainer's notes and are no longer published;
    # only its declared `summary` reaches a page.
    only_summary = path.name == "case.toml"
    # A fenced block is a command the reader runs, not prose written at them.
    # `git clone …` in an install page is the instruction; flagging it as
    # narrative would mean deleting the only documented way to install.
    in_fence = False
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if path.suffix == ".md" and line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if ALLOW.search(line):
            continue
        if only_summary and not line.lstrip().startswith("summary"):
            continue
        hit = None
        if narrative:
            for pattern, why in PATTERNS:
                if pattern.search(line):
                    hit = why
                    break
        if hit is None:
            # CE rules see the line without its code spans: a backticked
            # identifier is code, not prose, whatever it is spelled like.
            prose = INLINE_CODE.sub("", line)
            for pattern, why in CE_PATTERNS:
                if pattern.search(prose):
                    hit = why
                    break
        if hit is not None:
            findings.append(f"  {rel}:{n}  {hit}\n      {line.strip()[:100]}")
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
                    prose = INLINE_CODE.sub("", value)
                    hit = next(
                        (why for pattern, why in PATTERNS if pattern.search(value)),
                        None,
                    ) or next(
                        (why for pattern, why in CE_PATTERNS if pattern.search(prose)),
                        None,
                    )
                    if hit is not None:
                        findings.append(
                            f"  {rel}  {'.'.join(trail) or '<root>'}  {hit}\n"
                            f"      {value.strip()[:100]}"
                        )
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

    for path in spec_sources():
        findings += check_text_file(path, narrative=False)
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
            "`site-allow: <reason>` to the line. For a CFDL-CE finding\n"
            "(spelling, terminology, formats, contractions — see docs/22),\n"
            "append `ste-allow: <rule id> <reason>` instead.",
            file=sys.stderr,
        )
        return 1

    print(
        f"check-site-voice: OK ({checked} site-facing sources carry no internal "
        "narrative and follow CFDL-CE)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
