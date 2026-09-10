#!/usr/bin/env python3
"""Hold the published EBNF to the language the parser actually accepts.

`docs/02` calls `docs/schemas/CFDL_v0_1_Grammar.ebnf` normative, says
implementations MUST support it, and the site offers it for download for
parser generators. Nobody performed that adaptation for eight versions, so
nobody discovered the grammar did not survive one: `map_entry` omitted the `=`
between a key and its value, which made every `terms` block in the repository
underivable, and `contract_category` was defined and referenced by nothing.
Both are `docs/13` §7.61's predicted recurrence, and both were invisible to
review because a grammar is not run.

This gate runs it. It builds a recogniser FROM the EBNF and requires it to
accept every `.cfdl` file the rest of CI already proves parses. That is the
too-narrow direction — a grammar that rejects valid CFDL — which is the
failure this project keeps having. The too-broad direction (a grammar that
admits what the parser refuses) needs sentence generation and is not attempted
here; `map_inline` is a known instance of it.

WHAT THIS IS NOT. It does not generate the product parser and must never be
allowed to: the hand-written recursive-descent parser exists because the
diagnostics are a feature. This is a second, throwaway reader whose only job
is to disagree.

DELIBERATE PERMISSIVENESS. `IDENT` here matches any identifier-shaped token,
including reserved words. The real lexer distinguishes them, so this gate
would not notice a grammar that lets a keyword stand where a name belongs.
That direction is `check-keyword-register.py`'s and, properly, sentence
generation's. Being permissive here keeps this gate's failures meaningful:
every one is a shipped file the published grammar cannot derive.

No third-party dependency, like every other gate in `tools/`.
"""

from __future__ import annotations

import pathlib
import re
import sys

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

ROOT = pathlib.Path(__file__).resolve().parents[1]
EBNF = ROOT / "docs" / "schemas" / "CFDL_v0_1_Grammar.ebnf"

# Every tree CI proves parses. `fixtures/invalid` is excluded on purpose: those
# files are supposed to be rejected, and a recogniser that accepted them would
# be reporting the wrong thing.
CORPUS = ["fixtures/valid", "fixtures/repairs", "benchmarks", "examples", "training"]


# --------------------------------------------------------------------------
# 1. Tokens, per docs/02 §1.
# --------------------------------------------------------------------------

# DATE before INT, or `2026-01` lexes as 2026 minus 1. Longest-match order.
TOKEN_SPEC = [
    ("WS",      r"[ \t\r\n]+"),
    ("COMMENT", r"//[^\n]*|/\*.*?\*/"),
    ("DATE",    r"\d{4}-\d{2}-\d{2}|\d{4}-\d{2}(?![-\d])"),
    ("DECIMAL", r"\d[\d_]*\.\d[\d_]*"),
    ("INT",     r"\d[\d_]*"),
    ("STRING",  r'"(?:[^"\\]|\\.)*"'),
    ("IDENT",   r"[A-Za-z_][A-Za-z0-9_]*"),
    ("OP",      r"->|\.\.|==|!=|<=|>=|[-+*/%^<>=(){}\[\],.:;~]"),
]
TOKEN_RE = re.compile("|".join(f"(?P<{n}>{p})" for n, p in TOKEN_SPEC), re.S)


class Token:
    __slots__ = ("kind", "text", "line")

    def __init__(self, kind, text, line):
        self.kind, self.text, self.line = kind, text, line

    def __repr__(self):
        return f"{self.kind}:{self.text!r}"


def tokenize(src: str):
    out, pos, line = [], 0, 1
    while pos < len(src):
        m = TOKEN_RE.match(src, pos)
        if not m:
            raise ValueError(f"line {line}: cannot tokenize {src[pos:pos+20]!r}")
        kind, text = m.lastgroup, m.group()
        if kind not in ("WS", "COMMENT"):
            out.append(Token(kind, text, line))
        line += text.count("\n")
        pos = m.end()
    return out


# --------------------------------------------------------------------------
# 2. The EBNF, read as a grammar rather than as prose.
# --------------------------------------------------------------------------

def read_ebnf(path: pathlib.Path):
    src = path.read_text(encoding="utf-8")

    # A production the grammar itself marks NOT IMPLEMENTED describes syntax
    # the parser deliberately refuses. Excluding them is not a fudge: it is
    # reading what the grammar says about itself.
    unimplemented = set()
    for m in re.finditer(r"\(\*((?:[^*]|\*(?!\)))*)\*\)\s*\n([a-z_][a-z_0-9]*)\s*=", src):
        if "NOT IMPLEMENTED" in m.group(1):
            unimplemented.add(m.group(2))

    body = re.sub(r"\(\*.*?\*\)", " ", src, flags=re.S)
    rules = re.findall(r"([a-z_][a-z_0-9]*)\s*=\s*(.*?);", body, flags=re.S)
    if not rules:
        raise SystemExit("check-grammar-recogniser: no productions found — has the EBNF moved?")
    return dict(rules), unimplemented


def drop_unimplemented(rules: dict, unimplemented: set):
    """Make the unimplemented productions matchable by nothing.

    Not deleted, and nothing cascades. A rule that REACHES an unimplemented one
    is not itself unimplemented: `effects_block` is `"effects" "{"
    { effect_stmt } "}"`, and `effects {}` parses today (`docs/01` §8.3 —
    tolerated, not represented in IR) precisely because the repetition can
    match zero of them. Giving the dead rule no alternatives lets Earley work
    that out for itself, which is what it is good at; cascading by hand made
    this gate narrower than the grammar it was meant to police.
    """
    for name in unimplemented:
        rules.pop(name, None)
    return set(unimplemented)


# --------------------------------------------------------------------------
# 3. EBNF -> BNF. Earley wants flat alternatives, so groups, repetition and
#    option become synthetic rules rather than operators.
# --------------------------------------------------------------------------

class Ebnf:
    """A tiny reader for the dialect this file is written in.

    ISO `{ x }` and `[ x ]` plus the five postfix `x*` uses the file also
    carries. Mixing the two is itself worth knowing about; it is why an
    off-the-shelf ISO tool would not read this grammar unchanged.
    """

    def __init__(self, text):
        self.toks = re.findall(r'"[^"]*"|[A-Za-z_][A-Za-z_0-9]*|[|(){}\[\]*+?]', text)
        self.i = 0

    def peek(self):
        return self.toks[self.i] if self.i < len(self.toks) else None

    def take(self):
        t = self.peek()
        self.i += 1
        return t

    def alternation(self):
        alts = [self.sequence()]
        while self.peek() == "|":
            self.take()
            alts.append(self.sequence())
        return ("alt", alts)

    def sequence(self):
        items = []
        while self.peek() not in (None, "|", ")", "}", "]"):
            items.append(self.factor())
        return ("seq", items)

    def factor(self):
        atom = self.primary()
        while self.peek() in ("*", "+", "?"):
            atom = ({"*": "star", "+": "plus", "?": "opt"}[self.take()], atom)
        return atom

    def primary(self):
        t = self.take()
        if t == "(":
            inner = self.alternation(); self.take(); return inner
        if t == "{":
            inner = self.alternation(); self.take(); return ("star", inner)
        if t == "[":
            inner = self.alternation(); self.take(); return ("opt", inner)
        if t.startswith('"'):
            return ("lit", t[1:-1])
        return ("ref", t)


class Bnf:
    """Flattened grammar: name -> list of alternatives, each a list of symbols."""

    def __init__(self):
        self.rules: dict[str, list[list]] = {}
        self.n = 0

    def fresh(self, hint):
        self.n += 1
        return f"__{hint}{self.n}"

    def add(self, name, alts):
        self.rules.setdefault(name, []).extend(alts)

    def flatten(self, node, out: list):
        kind = node[0]
        if kind == "seq":
            for item in node[1]:
                self.flatten(item, out)
        elif kind == "alt":
            if len(node[1]) == 1:
                self.flatten(node[1][0], out)
            else:
                name = self.fresh("alt")
                self.add(name, [self._seq(a) for a in node[1]])
                out.append(("nt", name))
        elif kind == "lit":
            out.append(("lit", node[1]))
        elif kind == "ref":
            out.append(("nt", node[1]))
        elif kind in ("star", "plus", "opt"):
            inner = self._seq(node[1])
            name = self.fresh(kind)
            if kind == "opt":
                self.add(name, [[], inner])
            elif kind == "star":
                self.add(name, [[], inner + [("nt", name)]])
            else:
                self.add(name, [inner, inner + [("nt", name)]])
            out.append(("nt", name))
        else:
            raise AssertionError(kind)

    def _seq(self, node):
        out: list = []
        self.flatten(node, out)
        return out


# Terminals the EBNF names but does not define; docs/02 §1 supplies them.
# QNAME is a production here rather than a token, because the tokenizer emits
# `.` separately and a terminal cannot span tokens.
TERMINAL_KIND = {
    "IDENT": ("IDENT",),
    "INT": ("INT",),
    "DECIMAL": ("DECIMAL",),
    "STRING": ("STRING",),
    "DATE": ("DATE",),
    "NUMBER": ("INT", "DECIMAL"),
}


def build(rules: dict, dropped: set = frozenset()) -> Bnf:
    g = Bnf()
    for name in dropped:
        g.rules.setdefault(name, [])          # defined, and matches nothing
    for name, rhs in rules.items():
        if name in dropped:
            continue
        tree = Ebnf(rhs).alternation()
        g.add(name, [g._seq(a) for a in tree[1]])
    g.add("QNAME", [[("term", "IDENT")], [("term", "IDENT"), ("lit", "."), ("nt", "QNAME")]])
    for t in TERMINAL_KIND:
        if t not in g.rules:
            g.add(t, [[("term", t)]])
    return g


# --------------------------------------------------------------------------
# 4. Earley. Chosen over a backtracking PEG on purpose: EBNF alternation is
#    unordered, and a PEG's ordered choice would reject files the grammar
#    admits — turning this gate into a source of false accusations.
# --------------------------------------------------------------------------

def matches(sym, tok: Token) -> bool:
    kind, val = sym
    if kind == "lit":
        return tok.text == val
    return tok.kind in TERMINAL_KIND.get(val, (val,))


def recognise(g: Bnf, start: str, toks: list):
    """Return None on success, or the index of the furthest token reached."""
    n = len(toks)
    cols: list[set] = [set() for _ in range(n + 1)]
    cols[0].add((start, 0, 0, 0))          # (rule, alternative, dot, origin)
    furthest = 0

    for i in range(n + 1):
        col = cols[i]
        pending = list(col)
        while pending:
            item = pending.pop()
            name, alt_i, dot, origin = item
            alt = g.rules[name][alt_i]
            if dot == len(alt):                                    # complete
                for pname, palt_i, pdot, porigin in list(cols[origin]):
                    palt = g.rules[pname][palt_i]
                    if pdot < len(palt) and palt[pdot] == ("nt", name):
                        nxt = (pname, palt_i, pdot + 1, porigin)
                        if nxt not in col:
                            col.add(nxt); pending.append(nxt)
                continue
            sym = alt[dot]
            if sym[0] == "nt":                                     # predict
                for k in range(len(g.rules[sym[1]])):
                    nxt = (sym[1], k, 0, i)
                    if nxt not in col:
                        col.add(nxt); pending.append(nxt)
            elif i < n and matches(sym, toks[i]):                  # scan
                cols[i + 1].add((name, alt_i, dot + 1, origin))
                furthest = max(furthest, i + 1)

    for name, alt_i, dot, origin in cols[n]:
        if name == start and origin == 0 and dot == len(g.rules[name][alt_i]):
            return None
    return furthest


# --------------------------------------------------------------------------
# 5. The gate.
# --------------------------------------------------------------------------

def corpus_files():
    seen = []
    for root in CORPUS:
        seen.extend(sorted((ROOT / root).rglob("*.cfdl")))
    return seen


def main() -> int:
    rules, unimplemented = read_ebnf(EBNF)
    dropped = drop_unimplemented(rules, unimplemented)
    g = build(rules, dropped)

    files = corpus_files()
    if len(files) < 300:
        print(
            f"check-grammar-recogniser: found only {len(files)} .cfdl files; "
            "the corpus has moved and this gate would pass by running nothing.",
            file=sys.stderr,
        )
        return 1

    failures = []
    for path in files:
        try:
            toks = tokenize(path.read_text(encoding="utf-8"))
        except ValueError as err:
            failures.append((path, f"tokenizer: {err}"))
            continue
        stop = recognise(g, "module", toks)
        if stop is not None:
            where = toks[stop] if stop < len(toks) else toks[-1]
            failures.append((path, f"line {where.line}: cannot derive past {where.text!r}"))

    if failures:
        print(
            f"check-grammar-recogniser: the published grammar cannot derive "
            f"{len(failures)} of {len(files)} shipped models.\n",
            file=sys.stderr,
        )
        for path, why in failures[:25]:
            print(f"  {path.relative_to(ROOT)}\n      {why}", file=sys.stderr)
        if len(failures) > 25:
            print(f"  ... and {len(failures) - 25} more", file=sys.stderr)
        print(
            "\ndocs/02 calls the grammar normative and the site offers it for\n"
            "parser generators. A model CI proves parses, that the grammar cannot\n"
            "derive, means the published grammar is wrong — fix the EBNF, not this\n"
            "gate. If the syntax is genuinely not implemented, say so in an EBNF\n"
            "comment containing NOT IMPLEMENTED and the production is excluded.",
            file=sys.stderr,
        )
        return 1

    note = f"; {len(dropped)} NOT IMPLEMENTED production(s) excluded" if dropped else ""
    print(
        f"check-grammar-recogniser: OK ({len(files)} shipped models derive from "
        f"the published EBNF; {len(g.rules)} rules{note})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
