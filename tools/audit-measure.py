#!/usr/bin/env python3
"""Measurements behind docs/21_documentation_standards_audit.md.

Usage: python3 tools/audit-measure.py [repo root]

Every figure quoted in the audit is printed by this script. Nothing in the
report is estimated by eye, and re-running this is how a reader checks that.

NOT A GATE. It exits 0 whatever it finds and is deliberately absent from the
makefile. The numbers here are a snapshot of a corpus that is supposed to
change; a target that failed when prose moved would be measuring the wrong
thing. The gate that will enforce the writing standard belongs in
tools/check-site-voice.py, per docs/22_cfdl_controlled_english.md.

Re-run it when the audit is revisited, and update the figures in docs/21 and
the `occurrences` fields in docs/terminology.toml from its output.

METHOD. "Prose" means what a reader reads as sentences. The following are
removed before any sentence is counted, because none of them are prose and
each one would distort sentence-length statistics in a different direction:

  - YAML frontmatter (metadata, not prose)
  - fenced code blocks (code, and long lines that are not sentences)
  - inline code spans, replaced by the single token CODE (a backticked
    identifier is one lexical item to a reader, not `model.payback_years`
    counted as three words)
  - JSX/HTML tags (markup)
  - table rows (cells are fragments; they would halve the measured mean)
  - ATX headings (titles, not sentences)
  - MDX import/export lines

A "sentence" is a run terminated by . ! or ? and containing more than two
words. The >2 filter drops list bullets that are bare labels and the residue
of stripping, both of which are fragments rather than short sentences.
"""

from __future__ import annotations

import pathlib
import re
import statistics
import sys

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

# Default to the repository this file lives in, so the script works from any
# working directory. The same convention as tools/check-site-voice.py.
ROOT = (
    pathlib.Path(sys.argv[1]).resolve()
    if len(sys.argv) > 1
    else pathlib.Path(__file__).resolve().parents[1]
)

CORPORA = {
    "site/content/docs": ["site/content/docs/**/*.md"],
    "learn/content/chapters": ["learn/content/chapters/*.mdx"],
    "training exercise prompts": ["training/exercises/*/*/README.md"],
}

FENCE = re.compile(r"^\s*```")
FRONTMATTER = re.compile(r"\A---\n.*?\n---\n", re.S)
INLINE_CODE = re.compile(r"`[^`]*`")
TAG = re.compile(r"<[^>]+>")
HEADING = re.compile(r"^\s{0,3}#{1,6}\s")
TABLE = re.compile(r"^\s*\|")
MDX_IO = re.compile(r"^\s*(import|export)\s")
SENT_SPLIT = re.compile(r"(?<=[.!?])\s+")

# A form of "to be" followed by a past participle. The irregular list is
# explicit because -ed alone misses "written", "built", "shown", which are the
# participles this corpus actually uses.
PASSIVE = re.compile(
    r"\b(is|are|was|were|be|been|being)\b\s+(\w+ly\s+)?"
    r"(\w+ed|written|built|given|taken|made|shown|held|kept|set|put|read|run|"
    r"drawn|known|seen|done|sent|paid|left|found|lost|meant|thought|brought)\b",
    re.I,
)
# STE bans the -ing form as a modifier. Detected in the two positions where it
# is a modifier rather than a gerund subject: opening the sentence, or opening
# a clause after a comma or semicolon.
ING_MODIFIER = re.compile(r"(\A|[,;]\s)(\w+ing)\b")
# Genuine contractions only. A bare 's is far more often a possessive
# ("the model's logic") than a contraction, so 's counts only in the closed set
# of pronoun/adverb forms where it cannot be possessive.
CONTRACTION = re.compile(
    r"\b(\w+n't|\w+'(re|ve|ll|d|m)|(it|that|there|here|what|let|who|he|she)'s)\b",
    re.I,
)

FUNCTION_WORDS = set(
    """the a an of to in on for and or but is are was were be been being with by
    as at from that this these those it its if then than when while not no all
    any each per via into over under about which who whose there here so such
    both same other more most less least only just also very can could would
    should may might must will shall do does did has have had one two three""".split()
)


def prose_lines(path: pathlib.Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    text = FRONTMATTER.sub("", text)
    out: list[str] = []
    in_fence = False
    for line in text.splitlines():
        if FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if not line.strip() or HEADING.match(line) or TABLE.match(line) or MDX_IO.match(line):
            continue
        line = INLINE_CODE.sub(" CODE ", line)
        line = TAG.sub(" ", line)
        # Leading list markers and blockquote markers are structure, not words.
        line = re.sub(r"^\s*([-*+]|\d+\.|>)\s+", "", line)
        if line.strip():
            out.append(line.strip())
    return out


def sentences(lines: list[str]) -> list[str]:
    out = []
    for line in lines:
        for s in SENT_SPLIT.split(line):
            if len(s.split()) > 2:
                out.append(s.strip())
    return out


def files_for(globs: list[str]) -> list[pathlib.Path]:
    found: list[pathlib.Path] = []
    for g in globs:
        found += sorted(ROOT.glob(g))
    return found


def corpus_report() -> None:
    print("=" * 78)
    print("1. CORPUS AND SENTENCE LENGTH  (STE: 20 words procedural, 25 descriptive)")
    print("=" * 78)
    header = f"{'corpus':28} {'files':>5} {'words':>7} {'sents':>6} {'mean':>5} {'med':>4} {'p90':>4} {'max':>4} {'>20':>6} {'>25':>6}"
    print(header)
    print("-" * len(header))
    for name, globs in CORPORA.items():
        paths = files_for(globs)
        sents: list[str] = []
        for p in paths:
            sents += sentences(prose_lines(p))
        lens = sorted(len(s.split()) for s in sents)
        words = sum(lens)
        if not lens:
            print(f"{name:28} {len(paths):5}  (no prose found)")
            continue
        o20 = sum(1 for x in lens if x > 20)
        o25 = sum(1 for x in lens if x > 25)
        print(
            f"{name:28} {len(paths):5} {words:7,} {len(lens):6,} "
            f"{words / len(lens):5.1f} {statistics.median(lens):4.0f} "
            f"{lens[int(len(lens) * 0.9)]:4} {lens[-1]:4} "
            f"{100 * o20 / len(lens):5.0f}% {100 * o25 / len(lens):5.0f}%"
        )


def construction_report() -> None:
    print()
    print("=" * 78)
    print("2. SENTENCE CONSTRUCTION")
    print("=" * 78)
    header = f"{'corpus':28} {'sents':>6} {'passive':>14} {'-ing modifier':>16} {'contractions':>13}"
    print(header)
    print("-" * len(header))
    for name, globs in CORPORA.items():
        sents: list[str] = []
        for p in files_for(globs):
            sents += sentences(prose_lines(p))
        if not sents:
            continue
        pas = sum(1 for s in sents if PASSIVE.search(s))
        ing = sum(1 for s in sents if ING_MODIFIER.search(s))
        con = sum(len(CONTRACTION.findall(s)) for s in sents)
        print(
            f"{name:28} {len(sents):6,} {pas:6,} ({100 * pas / len(sents):3.0f}%) "
            f"{ing:8,} ({100 * ing / len(sents):3.0f}%) {con:13,}"
        )


def paragraph_report() -> None:
    """STE caps a descriptive paragraph at 6 sentences."""
    print()
    print("=" * 78)
    print("2b. PARAGRAPH LENGTH  (STE: maximum 6 sentences)")
    print("=" * 78)
    header = f"{'corpus':28} {'paras':>6} {'mean':>6} {'max':>5} {'>6 sentences':>14}"
    print(header)
    print("-" * len(header))
    for name, globs in CORPORA.items():
        counts: list[int] = []
        for p in files_for(globs):
            # A paragraph is a run of prose lines with no blank line between
            # them; prose_lines() has already dropped headings, tables and code,
            # so a break in the original file is a paragraph break here.
            text = FRONTMATTER.sub("", p.read_text(encoding="utf-8"))
            in_fence = False
            block: list[str] = []
            for line in text.splitlines():
                if FENCE.match(line):
                    in_fence = not in_fence
                    continue
                if in_fence:
                    continue
                if not line.strip() or HEADING.match(line) or TABLE.match(line):
                    if block:
                        n = len(sentences(prose_lines_from(block)))
                        if n:
                            counts.append(n)
                        block = []
                    continue
                block.append(line)
            if block:
                n = len(sentences(prose_lines_from(block)))
                if n:
                    counts.append(n)
        if not counts:
            continue
        over = sum(1 for c in counts if c > 6)
        print(
            f"{name:28} {len(counts):6,} {sum(counts) / len(counts):6.1f} "
            f"{max(counts):5} {over:6,} ({100 * over / len(counts):3.0f}%)"
        )


def prose_lines_from(lines: list[str]) -> list[str]:
    out = []
    for line in lines:
        line = INLINE_CODE.sub(" CODE ", line)
        line = TAG.sub(" ", line)
        line = re.sub(r"^\s*([-*+]|\d+\.|>)\s+", "", line)
        if line.strip():
            out.append(line.strip())
    return out


def count_word(word: str, globs: list[str]) -> int:
    pat = re.compile(rf"\b{re.escape(word)}\b", re.I)
    n = 0
    for g in globs:
        for p in sorted(ROOT.glob(g)):
            n += len(pat.findall(p.read_text(encoding="utf-8")))
    return n


ALL_PROSE = [g for globs in CORPORA.values() for g in globs]

VARIANTS = [
    ("modeling", "modelling"),
    ("amortization", "amortisation"),
    ("amortizing", "amortising"),
    ("amortize", "amortise"),
    ("behavior", "behaviour"),
    ("organized", "organised"),
    ("license", "licence"),
    ("catalog", "catalogue"),
    ("analyze", "analyse"),
    ("normalize", "normalise"),
]

SYNONYMS = [
    ("name for the run settings object", ["run config", "run configuration", "run settings"]),
    ("verb for activating a control", ["click", "hit", "press", "tap"]),
    ("name for the output artefact", ["results document", "results doc", "output document"]),
]


def vocabulary_report() -> None:
    print()
    print("=" * 78)
    print("3. VOCABULARY CONSISTENCY  (STE: one word, one form, one meaning)")
    print("=" * 78)
    print("Spelling variants of the same word, both present in published prose:")
    for us, uk in VARIANTS:
        a, b = count_word(us, ALL_PROSE), count_word(uk, ALL_PROSE)
        if a and b:
            print(f"    CONFLICT  {us:14} {a:4}   vs  {uk:14} {b:4}")
        elif a or b:
            print(f"    single    {us if a else uk:14} {a or b:4}")
    print()
    print("Competing names for one concept:")
    for concept, terms in SYNONYMS:
        counts = []
        for t in terms:
            pat = re.compile(rf"\b{re.escape(t)}\b", re.I)
            n = sum(
                len(pat.findall(p.read_text(encoding="utf-8")))
                for g in ALL_PROSE
                for p in sorted(ROOT.glob(g))
            )
            counts.append((t, n))
        used = [c for c in counts if c[1]]
        flag = "CONFLICT" if len(used) > 1 else "ok      "
        print(f"    {flag}  {concept}")
        for t, n in counts:
            print(f"                  {t:24} {n:5}")


def noun_cluster_report() -> None:
    print()
    print("=" * 78)
    print("4. NOUN CLUSTERS  (STE: maximum 3 words)")
    print("=" * 78)
    from collections import Counter

    c: Counter[str] = Counter()
    for g in ALL_PROSE:
        for p in sorted(ROOT.glob(g)):
            for line in prose_lines(p):
                for m in re.finditer(r"\b([a-z]+(?:\s+[a-z]+){3,})\b", line):
                    ws = m.group(1).split()
                    if all(w.lower() not in FUNCTION_WORDS for w in ws):
                        c[" ".join(ws[:5])] += 1
    print(f"{sum(c.values())} candidate clusters of 4+ content words; top 12 by frequency:")
    for k, v in c.most_common(12):
        print(f"    {v:3}  {k}")


NORMATIVE_SPECS = [
    "docs/01_language_spec.md",
    "docs/04_compiler_spec.md",
    "docs/07_pack_interface.md",
    "docs/03_expression_environment.md",
]


def normative_report() -> None:
    print()
    print("=" * 78)
    print("5. NORMATIVE KEYWORDS  (RFC 2119 / BCP 14)")
    print("=" * 78)
    kw = re.compile(r"\b(MUST NOT|SHALL NOT|SHOULD NOT|MUST|SHALL|SHOULD|MAY|REQUIRED|OPTIONAL|RECOMMENDED)\b")
    total = 0
    for rel in NORMATIVE_SPECS:
        p = ROOT / rel
        if not p.exists():
            continue
        n = len(kw.findall(p.read_text(encoding="utf-8")))
        total += n
        print(f"    {n:4}  {rel}")
    print(f"    {total:4}  TOTAL")
    cite = re.compile(r"RFC\s*2119|BCP\s*14")
    hits = [
        p.relative_to(ROOT)
        for p in ROOT.rglob("*.md")
        if "node_modules" not in str(p) and "target" not in str(p) and cite.search(p.read_text(encoding="utf-8", errors="ignore"))
    ]
    print(f"\n    Files citing RFC 2119 or BCP 14 anywhere in the repository: {len(hits)}")
    for h in hits:
        print(f"        {h}")


def frontmatter_report() -> None:
    print()
    print("=" * 78)
    print("6. FRONTMATTER  (ISO/IEC/IEEE 26514 findability)")
    print("=" * 78)
    for label, glob in [
        ("site/content/docs", "site/content/docs/**/*.md"),
        ("learn/content/chapters", "learn/content/chapters/*.mdx"),
    ]:
        paths = sorted(ROOT.glob(glob))
        keys: dict[str, int] = {}
        for p in paths:
            m = FRONTMATTER.match(p.read_text(encoding="utf-8"))
            if not m:
                continue
            for line in m.group(0).splitlines()[1:-1]:
                if ":" in line:
                    keys[line.split(":", 1)[0].strip()] = keys.get(line.split(":", 1)[0].strip(), 0) + 1
        print(f"    {label}  ({len(paths)} files)")
        for k, v in sorted(keys.items(), key=lambda kv: -kv[1]):
            print(f"        {k:14} {v:4}/{len(paths)}")


def generated_report() -> None:
    print()
    print("=" * 78)
    print("7. EDITABILITY OF site/content/docs")
    print("=" * 78)
    from collections import Counter

    c: Counter[str] = Counter()
    words: Counter[str] = Counter()
    # A page carrying no `generated:` marker is not unowned: sync-content.mjs
    # holds it in its own manifest and would fail the build otherwise (see the
    # `unowned` check at the end of that file). Those pages are written from
    # examples/*/README.md, so they are generated too — just not self-labelled.
    where = {
        "none": "AUTHORED — edit the page itself",
        "regions": "edit outside <!-- cfdl:generated --> fences",
        "full": "edit docs/*.md, distribution/install-configure.md",
        "source": "edit benchmarks/, or the template in sync-content.mjs",
        "script manifest": "edit examples/*/README.md",
    }
    for p in sorted(ROOT.glob("site/content/docs/**/*.md")):
        text = p.read_text(encoding="utf-8")
        m = re.search(r"^generated:\s*(\S+)", text, re.M)
        kind = m.group(1) if m else ("source" if re.search(r"^source:", text, re.M) else "script manifest")
        c[kind] += 1
        words[kind] += sum(len(s.split()) for s in sentences(prose_lines(p)))
    for k in sorted(c, key=lambda x: -c[x]):
        print(f"    {k:16} {c[k]:4} pages  {words[k]:7,} prose words   {where.get(k, '?')}")
    print(f"    {'TOTAL':16} {sum(c.values()):4} pages  {sum(words.values()):7,} prose words")


def main() -> None:
    print(f"CFDL documentation measurements — repository root {ROOT}\n")
    corpus_report()
    construction_report()
    paragraph_report()
    vocabulary_report()
    noun_cluster_report()
    normative_report()
    frontmatter_report()
    generated_report()


if __name__ == "__main__":
    main()
