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
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import ebnf_grammar  # noqa: E402  (path set above so the tools dir is importable)
from ebnf_grammar import Bnf, Token, TERMINAL_KIND, tokenize  # noqa: E402

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

ROOT = ebnf_grammar.ROOT

# Every tree CI proves parses. `fixtures/invalid` is excluded on purpose: those
# files are supposed to be rejected, and a recogniser that accepted them would
# be reporting the wrong thing.
CORPUS = ["fixtures/valid", "fixtures/repairs", "benchmarks", "examples", "training"]



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
    g, dropped = ebnf_grammar.load()

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
