"""The published EBNF, read as a grammar rather than as prose.

Shared by the two gates that hold `docs/schemas/CFDL_v0_1_Grammar.ebnf` to the
language the parser actually accepts — `check-grammar-recogniser.py` in the
too-narrow direction and `check-grammar-generation.py` in the too-broad one.
One reader, so the two can never disagree about what the grammar says; that
disagreement is the whole class of bug these gates exist to catch.

Not a gate itself, and imported the way `schema_sync` is.
"""

from __future__ import annotations

import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
EBNF = ROOT / "docs" / "schemas" / "CFDL_v0_1_Grammar.ebnf"


def load():
    """The grammar, flattened to BNF, with unimplemented productions dead."""
    rules, unimplemented = read_ebnf(EBNF)
    dropped = drop_unimplemented(rules, unimplemented)
    return build(rules, dropped), dropped


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

