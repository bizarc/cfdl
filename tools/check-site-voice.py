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
standard is being waived, and it turns off only the rule it names.

An annotation is a TRAILING COMMENT: `<!-- ... -->` in Markdown, `//` in a model
file, `#` in TOML. A sentence that merely mentions one is prose and waives
nothing, which is the difference between documenting the escape hatch and using
it.

CFDL-CE. This gate also enforces the mechanical subset of the writing standard
(docs/22_cfdl_controlled_english.md): retired spellings, retired synonyms,
number formats, contractions. The word lists come from docs/terminology.toml so
the gate and the register cannot drift. What is deliberately NOT here: sentence
length, voice, and imperative form — those are judgment calls (see the tiering
in docs/22), and a gate that flags judgment gets disabled, which is this file's
founding rule.

THE REGISTRY. Every check is a `Rule` carrying the id a reviewer cites, the
layer it belongs to and the document types it binds. Layer one is universal and
applies to every published word; layer two is by type and is not implemented
yet. An exemption names the rules it exempts, so the specification carve-out can
be read rather than inferred from a boolean at the call site.

Usage:
  python3 tools/check-site-voice.py            gate: fails on any finding
  python3 tools/check-site-voice.py --report   measure: groups by rule, never fails
"""

from __future__ import annotations

import json
import html
import pathlib
import re
import sys
import tomllib
from collections import Counter
from dataclasses import dataclass

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
# An ste-allow waiver names the docs/22 rule it waives. An id docs/22 does not
# declare is a typo that silently waives nothing forever, so it is a failure —
# the annotation contract (docs/22 §5) only works if the ids are real. The
# annotation itself is matched by WAIVER, below, which requires the comment form.
RULE_ID = re.compile(r"^\|\s*([SVWCP]\d+)\s*\|")


def ce_rule_ids() -> set[str]:
    """Rule ids declared in docs/22 §3 (the first column of the rule tables)."""
    ids = set()
    doc = (REPO_ROOT / "docs" / "22_cfdl_controlled_english.md").read_text(encoding="utf-8")
    for line in doc.splitlines():
        match = RULE_ID.match(line.strip())
        if match:
            ids.add(match.group(1))
    return ids


CE_RULE_IDS = None  # populated lazily; docs/22 is read once


# --- The rule registry ------------------------------------------------------
#
# A rule carries the id a reviewer cites, the layer it belongs to, and the types
# it binds. Before this structure the gate held two flat lists of (pattern,
# message) pairs and a boolean at the call site, so nothing could say WHICH rule
# an exemption exempted — the specification exemption turned off all sixteen
# narrative patterns while its docstring claimed it turned off one.
#
# LAYER ONE is universal: it applies to every published word whatever the
# document is. LAYER TWO is by type. Only layer one is implemented today; the
# `layer` field exists so the second can be added a rule at a time rather than
# as one unreviewable change.
#
# The ids beginning `N` and `X` are not yet declared in docs/22 §3. They name
# rules this gate has always enforced without writing them down. docs/22 gains
# them when the standard is restructured; until then the selftest checks the
# implemented set against the ids the document does declare, in both directions.

ALL_TYPES = frozenset({"task", "concept", "reference", "marketing"})

UNIVERSAL = "universal"
BY_TYPE = "by_type"


@dataclass(frozen=True)
class Rule:
    """One mechanical check, and the rule of the standard it enforces."""

    id: str
    layer: str
    why: str
    pattern: re.Pattern[str]
    # "prose" matches the line with inline code spans removed: a backticked
    # identifier is code, not prose, whatever it is spelled like. "raw" matches
    # the line as written.
    on: str = "prose"
    types: frozenset[str] = ALL_TYPES
    # A rule the escape hatch may not waive. Naming a competitor is the first.
    waivable: bool = True


@dataclass(frozen=True)
class Waiver:
    """One annotation, and the rules it turns off on its line."""

    kind: str  # "site" or "ste"
    rule_ids: frozenset[str]  # empty means every rule, which is site-allow
    reason: str

    def waives(self, rule: Rule) -> bool:
        return not self.rule_ids or rule.id in self.rule_ids


@dataclass(frozen=True)
class Finding:
    """One rule firing at one place, rendered the way the gate has always."""

    rel: str
    why: str
    excerpt: str
    rule_id: str
    lineno: int | None = None
    locus: str = ""  # the trail into a JSON document, for a schema finding

    def render(self) -> str:
        where = f"{self.rel}:{self.lineno}" if self.lineno is not None else self.rel
        locus = f"  {self.locus}" if self.locus else ""
        return f"  {where}{locus}  {self.why}\n      {self.excerpt}"


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

# The rule each narrative pattern enforces, in PATTERNS order. Several patterns
# share an id because one rule has several tells: a repository reference is a
# generator path, a crate path, "from a checkout" and a GitHub link alike.
#
#   N1  cites an internal document
#   N2  discloses unfinished work
#   N3  narrates development
#   N4  references the repository
#   N6  claims something about the authors rather than the product
#   W6  marketing ornament (docs/22 §3.3)
NARRATIVE_IDS = (
    "N1",  # docs/NN_name.md
    "N2",  # backlog
    "N3",  # SUPERSEDED
    "N3",  # originally said
    "N2",  # TODO / FIXME
    "N4",  # reference_gen
    "N4",  # in-house
    "N2",  # pending practitioner review
    "N6",  # honest / dishonest
    "N3",  # we chose / decided
    "N3",  # this page used to
    "N4",  # generated from <repo path>
    "N4",  # `crates/...`
    "N4",  # in this repository
    "N4",  # github.com
    "W6",  # ornament
)

# The specification exemption. A normative document published as a labeled
# section may cite another specification by filename, and before the registry
# this exemption silently covered every narrative rule rather than this set.
# Narrowing it is a corpus change and belongs in its own commit, so the set is
# spelled out here rather than quietly reduced.
SPEC_EXEMPT = frozenset(NARRATIVE_IDS)


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

# The rule each CFDL-CE pattern enforces, in ce_patterns() order. W2 has four
# tells because four retired synonyms name two defined things.
#
#   W1  one word, one form          W3  the approved verb for an action
#   W2  one concept, one term       V6  no contractions
#   X1  a multiple is 8.0x, not 8.0×   X2  millions are m, not mm
#   X3  a hyphen is U+002D
CE_IDS = ("W1", "W2", "W2", "W2", "W2", "W3", "X1", "X2", "X3", "V6")

# W3 reads the line as written. Its pattern is anchored to the markup that makes
# `Hit` an instruction aimed at a control — bold, or a backticked control name —
# and the code-stripped copy every other CFDL-CE rule reads has already removed
# the backticks, so that half of the pattern could never match. The selftest
# found it. Rerun over the corpus: no finding changes.
CE_READS_RAW = frozenset({"W3"})


def build_registry() -> tuple[Rule, ...]:
    """Every mechanical rule, in the order the gate has always applied them.

    Order is load-bearing: the first rule to match a line is the one reported,
    and the narrative rules read the raw line while the CFDL-CE rules read it
    with its code spans removed.
    """
    if len(NARRATIVE_IDS) != len(PATTERNS):
        raise SystemExit(
            f"check-site-voice: {len(PATTERNS)} narrative patterns but "
            f"{len(NARRATIVE_IDS)} ids. Every pattern names the rule it enforces."
        )
    if len(CE_IDS) != len(CE_PATTERNS):
        raise SystemExit(
            f"check-site-voice: {len(CE_PATTERNS)} CFDL-CE patterns but "
            f"{len(CE_IDS)} ids. Every pattern names the rule it enforces."
        )
    rules = [
        Rule(id=rid, layer=UNIVERSAL, why=why, pattern=pattern, on="raw")
        for rid, (pattern, why) in zip(NARRATIVE_IDS, PATTERNS)
    ]
    rules += [
        Rule(
            id=rid,
            layer=UNIVERSAL,
            why=why,
            pattern=pattern,
            on="raw" if rid in CE_READS_RAW else "prose",
        )
        for rid, (pattern, why) in zip(CE_IDS, CE_PATTERNS)
    ]
    return tuple(rules)


REGISTRY = build_registry()


def rules_for(exempt: frozenset[str]) -> tuple[Rule, ...]:
    """The rules that bind a source, given the exemptions its group carries."""
    return tuple(rule for rule in REGISTRY if rule.id not in exempt)


# --- The escape hatch -------------------------------------------------------
#
# A waiver is a trailing comment, which is how every one in the repository is
# written. Requiring the comment form is what separates an annotation from a
# sentence that merely mentions one: before this, any line containing the string
# `ste-allow:` was skipped entirely, so the backlog citation in docs/22's own
# status line was invisible to the gate rather than caught by it, and the rule
# id it parsed out of that line was a backtick.
_OPENER = r"(?:<!--|//|\#|/\*|\{/\*)"
_CLOSER = r"(?:-->|\*/\}|\*/)?"
WAIVER = re.compile(
    rf"(?:^|\s){_OPENER}\s*(?P<kind>site|ste)-allow:\s*(?P<body>.*?)\s*{_CLOSER}\s*$"
)


def parse_waiver(prose: str) -> Waiver | None:
    """Read a trailing annotation. `prose` has had its code spans removed."""
    match = WAIVER.search(prose)
    if match is None:
        return None
    body = match.group("body").strip()
    if match.group("kind") == "ste":
        # `ste-allow: <rule id> <reason>` — the id names what is being waived.
        head, _, rest = body.partition(" ")
        return Waiver(kind="ste", rule_ids=frozenset({head}) if head else frozenset(), reason=rest.strip())
    return Waiver(kind="site", rule_ids=frozenset(), reason=body)


def validate_waiver(waiver: Waiver, rel: str, lineno: int, excerpt: str) -> list[Finding]:
    """A waiver that names nothing real, or explains nothing, waives nothing."""
    global CE_RULE_IDS
    if CE_RULE_IDS is None:
        CE_RULE_IDS = ce_rule_ids()
    findings = []
    known = CE_RULE_IDS | {rule.id for rule in REGISTRY}
    for rule_id in sorted(waiver.rule_ids):
        if rule_id not in known:
            findings.append(
                Finding(
                    rel=rel,
                    lineno=lineno,
                    rule_id="WAIVER",
                    why=f"ste-allow names rule '{rule_id}', which docs/22 does not declare",
                    excerpt=excerpt,
                )
            )
            continue
        unwaivable = [r for r in REGISTRY if r.id == rule_id and not r.waivable]
        if unwaivable:
            findings.append(
                Finding(
                    rel=rel,
                    lineno=lineno,
                    rule_id="WAIVER",
                    why=f"{rule_id} may not be waived",
                    excerpt=excerpt,
                )
            )
    return findings


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
    # The private case pages are the most externally visible prose in the
    # repository: they are written to be read by someone outside the project.
    # They live in site/app rather than site/content, so nothing checked them
    # until a reader found idiom and personification on a page in front of a
    # client.
    found += sorted(REPO_ROOT.glob("site/app/private/*/content.html"))
    return [p for p in found if p.exists()]


_HTML_DROP = re.compile(r"<(details|style|script|pre)\b.*?</\1>", re.S)
_HTML_TAG = re.compile(r"<[^>]+>")


def _lines_of(path: pathlib.Path) -> list[str]:
    """Lines of prose. An HTML page is stripped to its sentences first.

    The collapsed model listing, the code blocks and the stylesheet are not
    prose and are dropped whole; everything else keeps its line numbering as
    closely as the markup allows.
    """
    text = path.read_text(encoding="utf-8")
    if path.suffix != ".html":
        return text.splitlines()
    text = _HTML_DROP.sub(lambda m: "\n" * m.group(0).count("\n"), text)
    text = _HTML_TAG.sub(" ", text)
    return [html.unescape(line) for line in text.splitlines()]


def check_text_file(path: pathlib.Path, *, exempt: frozenset[str] = frozenset()) -> list[Finding]:
    """Check one published source against every rule its group does not exempt."""
    return check_lines(
        _lines_of(path),
        exempt=exempt,
        suffix=path.suffix,
        # A case.toml's COMMENTS are maintainer's notes and are no longer
        # published; only its declared `summary` reaches a page.
        only_summary=path.name == "case.toml",
        label=path.relative_to(REPO_ROOT).as_posix(),
    )


def check_lines(
    lines,
    *,
    exempt: frozenset[str] = frozenset(),
    suffix: str = ".md",
    only_summary: bool = False,
    label: str = "<case>",
) -> list[Finding]:
    """The checking, with the reading taken out.

    Taking lines rather than a path is what makes a rule testable: the selftest
    exercises `.mdx` and `.html` handling without a file, and nothing in the gate
    has to be mocked.
    """
    findings: list[Finding] = []
    rel = label
    applicable = rules_for(exempt)
    # A fenced block is a command the reader runs, not prose written at them.
    # `git clone …` in an install page is the instruction; flagging it as
    # narrative would mean deleting the only documented way to install.
    in_fence = False
    for n, line in enumerate(lines, 1):
        if suffix == ".md" and line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if only_summary and not line.lstrip().startswith("summary"):
            continue
        # A rule reads the raw line or the line with its code spans removed.
        prose = INLINE_CODE.sub("", line)
        waiver = parse_waiver(prose)
        if waiver is not None:
            findings += validate_waiver(waiver, rel, n, line.strip()[:100])
        for rule in applicable:
            if waiver is not None and waiver.waives(rule):
                continue
            subject = line if rule.on == "raw" else prose
            if rule.pattern.search(subject):
                findings.append(
                    Finding(
                        rel=rel,
                        lineno=n,
                        rule_id=rule.id,
                        why=rule.why,
                        excerpt=line.strip()[:100],
                    )
                )
                break
    return findings


def check_schema(path: pathlib.Path, *, exempt: frozenset[str] = frozenset()) -> list[Finding]:
    """Schema `description` strings are served publicly and rendered as prose."""
    findings: list[Finding] = []
    rel = path.relative_to(REPO_ROOT).as_posix()
    applicable = rules_for(exempt)

    def walk(node, trail):
        if isinstance(node, dict):
            for key, value in node.items():
                if key == "description" and isinstance(value, str):
                    prose = INLINE_CODE.sub("", value)
                    trail_s = ".".join(trail) or "<root>"
                    # A description carries its annotation the same way a line
                    # does, and suppression is per rule here too.
                    waiver = parse_waiver(prose)
                    if waiver is not None:
                        findings.extend(
                            Finding(rel=rel, locus=trail_s, rule_id=f.rule_id,
                                    why=f.why, excerpt=f.excerpt)
                            for f in validate_waiver(waiver, rel, 0, value.strip()[:100])
                        )
                    for rule in applicable:
                        if waiver is not None and waiver.waives(rule):
                            continue
                        subject = value if rule.on == "raw" else prose
                        if rule.pattern.search(subject):
                            findings.append(
                                Finding(
                                    rel=rel,
                                    locus=trail_s,
                                    rule_id=rule.id,
                                    why=rule.why,
                                    excerpt=value.strip()[:100],
                                )
                            )
                            break
                else:
                    walk(value, trail + [str(key)])
        elif isinstance(node, list):
            for item in node:
                walk(item, trail)

    walk(json.loads(path.read_text(encoding="utf-8")), [])
    return findings


# --- Selftest ---------------------------------------------------------------
#
# The gate parses docs/22, loads word lists from the register, walks JSON
# documents and handles two escape hatches, and until now nothing exercised any
# of it. The only evidence of testing was a sentence in docs/21 recording a
# manual pass that cannot be re-run.
#
# Cases live here rather than under a fixture directory because a positive case
# is literally a string the gate bans: a fixture file would have to be excluded
# from the gate's own reading, and a reader debugging a failure wants the case
# beside the rule.


@dataclass(frozen=True)
class Case:
    """One line, and the rule it must or must not trip."""

    rule: str  # the id expected to fire; "" means nothing may fire
    text: str
    suffix: str = ".md"
    only_summary: bool = False
    exempt: frozenset[str] = frozenset()
    note: str = ""


SELFTEST: tuple[Case, ...] = (
    # --- W1, the retired spellings -----------------------------------------
    Case("W1", "The premium is amortised over the term."),
    Case("", "The premium is amortized over the term."),
    Case("", "The `amortising` flag is a pack identifier.", note="code span, not prose"),
    # --- W2, one concept one term ------------------------------------------
    Case("W2", "Edit the run config before the run."),
    Case("W2", "Check the output document for the totals."),
    Case("", "Edit the run configuration before the run."),
    Case("", "Read the results document.", note="the approved form, not its prefix"),
    # --- W3, the approved verb ---------------------------------------------
    Case("W3", "Hit **Run** to evaluate the model."),
    Case("W3", "Hit `Run` to evaluate the model."),
    Case("", "Collections hit 60,000 in the third year.", note="not aimed at a control"),
    # --- V6, contractions against possessives ------------------------------
    Case("V6", "It's evaluated once per period."),
    Case("V6", "The run doesn't converge."),
    Case("V6", "They're declared in the pack."),
    Case("", "The model's logic is declarative.", note="possessive, not a contraction"),
    Case("", "The quarter's results are published.", note="possessive"),
    # --- X1 to X3, the number formats --------------------------------------
    Case("X1", "The exit is struck at 8.0× EBITDA."),
    Case("", "The schedule holds 6,000 × 12 units.", note="spaced arithmetic"),
    Case("", "The grid is 3×3.", note="dimensions, not a multiple"),
    Case("X2", "The purchase price is $33.6mm."),
    Case("", "The purchase price is $33.6m."),
    Case("X3", "A non‑breaking hyphen is invisible in review."),
    Case("", "A plain-hyphen compound is correct."),
    # --- N1 to N6, the narrative rules -------------------------------------
    Case("N1", "See docs/13_feature_backlog.md for the rest."),
    Case("", "See docs/13_feature_backlog.md for the rest.", exempt=frozenset({"N1"}),
         note="the specification carve-out"),
    Case("N2", "The remaining items are on the backlog."),
    Case("N2", "TODO: finish this section."),
    Case("N3", "We chose the second form for its symmetry."),
    Case("N3", "This page previously gave two reasons."),
    Case("N4", "Run it from a checkout of the repository."),
    Case("N4", "See `crates/cfdl-engine/src/lib.rs` for the loop."),
    Case("N6", "This is an honest account of the arithmetic."),
    Case("W6", "A blazingly fast engine."),
    Case("", "The engine evaluates 40,000 periods in a second.", note="a figure, not a claim"),
    # --- Fenced blocks ------------------------------------------------------
    Case("", "```bash\ngit clone https://github.com/bizarc/cfdl\n```",
         note="a command the reader runs is not prose written at them"),
    Case("N4", "Clone it from https://github.com/bizarc/cfdl.",
         note="the same string outside a fence"),
    # --- The escape hatch ---------------------------------------------------
    Case("", "See `crates/cfdl-cli` for the flag.  <!-- site-allow: explained here -->"),
    Case("WAIVER", "x  <!-- ste-allow: Z9 no such rule -->",
         note="an id docs/22 does not declare waives nothing forever"),
    Case("", "The amortised form.  <!-- ste-allow: W1 the register is quoted -->"),
    Case("N2", "The backlog is cited here.  <!-- ste-allow: W1 wrong rule -->",
         note="a named waiver must not suppress a different rule"),
    Case("N2", "The tier map is unchecked (backlog 7.82). See `ste-allow:` in §5.",
         note="a backticked mention is code, and waives nothing"),
    Case("N2", "The backlog says ste-allow: is the escape hatch.",
         note="a mid-sentence mention is prose, and waives nothing"),
    Case("", "// TODO finish the pool  // site-allow: the note is for a maintainer",
         suffix=".cfdl", note="a line comment is the opener in a model file"),
    Case("N2", "// TODO finish the pool", suffix=".cfdl",
         note="the same note without an annotation"),
    # --- A case.toml publishes only its summary -----------------------------
    Case("", "# TODO a maintainer's note", suffix=".toml", only_summary=True),
    Case("N2", 'summary = "TODO write this"', suffix=".toml", only_summary=True),
)

# Rules docs/22 declares and this gate deliberately does not enforce. They are
# judgment calls, and a gate that flags judgment gets disabled. Listing them
# means adding a rule to docs/22 forces a decision here rather than being
# silently unimplemented.
NOT_MECHANICAL = frozenset(
    {
        "S1", "S2", "S3", "S4", "S5",   # length and list form
        "V1", "V2", "V3", "V4", "V5",   # voice, tense, imperative form
        "W4", "W5", "W7",               # registration, noun clusters, first use
        "C1", "C2", "C3", "C4", "C5",   # clarity
        "P1", "P2", "P3", "P4",         # procedures
    }
)


def selftest() -> int:
    """Prove each rule fires where it must, stays silent where it must not."""
    failed = 0

    for case in SELFTEST:
        got = check_lines(
            case.text.splitlines(),
            exempt=case.exempt,
            suffix=case.suffix,
            only_summary=case.only_summary,
        )
        ids = [f.rule_id for f in got]
        want = [case.rule] if case.rule else []
        if ids != want:
            failed += 1
            trailer = f"  ({case.note})" if case.note else ""
            print(
                f"FAIL {case.text.splitlines()[0][:70]!r}{trailer}\n"
                f"     want {want or 'no finding'}, got {ids or 'no finding'}",
                file=sys.stderr,
            )

    # Structural assertions. The cases prove the rules fire; these prove the
    # registry is coherent, which is what actually drifts.
    declared = ce_rule_ids()
    implemented = {rule.id for rule in REGISTRY}

    unimplemented = declared - implemented - NOT_MECHANICAL
    if unimplemented:
        failed += 1
        print(
            f"FAIL docs/22 declares {sorted(unimplemented)}, which this gate neither "
            f"enforces nor lists in NOT_MECHANICAL",
            file=sys.stderr,
        )

    both = implemented & NOT_MECHANICAL
    if both:
        failed += 1
        print(f"FAIL {sorted(both)} is both enforced and listed as judgment", file=sys.stderr)

    for rule in REGISTRY:
        if rule.layer == UNIVERSAL and rule.types != ALL_TYPES:
            failed += 1
            print(f"FAIL {rule.id} is universal but does not bind every type", file=sys.stderr)

    covered = {case.rule for case in SELFTEST if case.rule}
    uncovered = implemented - covered
    if uncovered:
        failed += 1
        print(f"FAIL no positive case for {sorted(uncovered)}", file=sys.stderr)
    if not any(case.rule == "" for case in SELFTEST):
        failed += 1
        print("FAIL no negative case at all", file=sys.stderr)

    # Every retired spelling must be reachable by W1, or the register holds a
    # word the gate silently ignores.
    register = tomllib.loads((REPO_ROOT / "docs" / "terminology.toml").read_text(encoding="utf-8"))
    for retired in register["spelling"]["map"]:
        if not check_lines([f"The {retired} form is retired."]):
            failed += 1
            print(f"FAIL [spelling.map] holds {retired!r}, which no rule catches", file=sys.stderr)

    if failed:
        print(f"check-site-voice: {failed} selftest case(s) FAILED", file=sys.stderr)
        return 1
    print(
        f"check-site-voice: selftest OK ({len(SELFTEST)} cases, "
        f"{len(implemented)} rules, {len(REGISTRY)} checks)",
        file=sys.stderr,
    )
    return 0


def collect() -> tuple[list[Finding], int]:
    """Every finding across every published source, and how many were read."""
    findings: list[Finding] = []
    checked = 0

    for path in sources():
        findings += check_text_file(path)
        checked += 1

    for path in spec_sources():
        findings += check_text_file(path, exempt=SPEC_EXEMPT)
        checked += 1

    for name in ("ir.schema.json", "results.schema.json"):
        path = REPO_ROOT / "docs" / "schemas" / name
        if path.exists():
            findings += check_schema(path)
            checked += 1

    return findings, checked


def report(findings: list[Finding], checked: int) -> int:
    """Print what the corpus holds, grouped by rule, and pass regardless.

    A measurement, not a gate. Before widening a rule or narrowing an exemption,
    this says how much it will surface — the number that decides whether the
    change is one commit or twelve.
    """
    print(f"check-site-voice --report: {checked} site-facing sources, {len(findings)} findings\n")
    if not findings:
        print("  nothing to report")
        return 0
    by_rule = Counter(f.rule_id for f in findings)
    width = max(len(rid) for rid in by_rule)
    for rid, count in sorted(by_rule.items(), key=lambda kv: (-kv[1], kv[0])):
        files = len({f.rel for f in findings if f.rule_id == rid})
        why = next(f.why for f in findings if f.rule_id == rid)
        plural = "file " if files == 1 else "files"
        print(f"  {rid:<{width}}  {count:>4}  in {files:>3} {plural}   {why}")
    print("\n  worst files:")
    by_file = Counter(f.rel for f in findings)
    for rel, count in by_file.most_common(10):
        print(f"  {count:>4}  {rel}")
    return 0


def main() -> int:
    if "--selftest" in sys.argv[1:]:
        return selftest()
    if "--report" in sys.argv[1:]:
        return report(*collect())

    findings, checked = collect()

    if findings:
        print("check-site-voice: internal narrative would be published.\n", file=sys.stderr)
        print("\n".join(f.render() for f in findings), file=sys.stderr)
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
