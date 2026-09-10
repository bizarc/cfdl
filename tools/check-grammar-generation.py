#!/usr/bin/env python3
"""Require the parser to accept what the published grammar admits.

The sibling gate, `check-grammar-recogniser.py`, runs the grammar over shipped
models and catches it being too NARROW — a grammar that cannot derive real
CFDL. This one runs it the other way and catches it being too BROAD: a
production describing syntax the parser refuses, which is just as wrong in a
document `docs/02` calls normative and the site offers for parser generators.
Someone generating a parser from it gets one that accepts what ours rejects,
and their models stop working at our door with no explanation.

Too-broad is the rarer failure and the harder one to see, because no shipped
file exercises it — that is precisely why it survives review. `map_inline` is
the instance that motivated this gate: it describes an inline map literal the
parser accepts in NO form, with or without `=`, and nothing noticed for eight
versions.

METHOD. Each-alternative coverage, not random sentences. For every alternative
of every reachable production, the shortest sentence that forces that
alternative is generated and handed to `cfdl parse`, which reads syntax and
ignores meaning — an unresolved entity reference parses, so a generated
sentence does not have to make sense to prove the point. Coverage is finite
and the same every run; a random walk over a recursive grammar is neither.

A failure means the grammar and the parser disagree, and the grammar is
usually the one to fix — but not always. A production may describe syntax that
was removed by decision, in which case the honest repair is to remove it or to
mark it `NOT IMPLEMENTED`, which both gates then respect.

No third-party dependency, like every other gate in `tools/`.
"""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import ebnf_grammar  # noqa: E402  (path set above so the tools dir is importable)

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

ROOT = ebnf_grammar.ROOT
CLI = ROOT / "target" / "debug" / ("cfdl.exe" if sys.platform == "win32" else "cfdl")

# A placeholder for each terminal the EBNF names but does not define. They only
# have to lex as the right kind; nothing here has to mean anything.
PLACEHOLDER = {
    "IDENT": "a",
    "INT": "1",
    "DECIMAL": "1.0",
    "NUMBER": "1",
    "STRING": '"s"',
    "DATE": "2026-01",
}

# Diagnostics that mean "this is not CFDL". Everything else — an unresolved
# reference, a missing term, a pack that does not exist — is the parser doing
# its job on a sentence that was never meant to be meaningful.
def is_syntax_error(code: str) -> bool:
    return code.startswith("E0")


def reachable(g, start="module"):
    seen, stack = {start}, [start]
    while stack:
        for alt in g.rules.get(stack.pop(), []):
            for kind, val in alt:
                if kind == "nt" and val not in seen:
                    seen.add(val)
                    stack.append(val)
    return seen


def shortest(g):
    """symbol -> shortest terminal string it derives, or None if it derives none."""
    best: dict[str, list | None] = {n: None for n in g.rules}
    changed = True
    while changed:
        changed = False
        for name, alts in g.rules.items():
            for alt in alts:
                out = expand(alt, best)
                if out is None:
                    continue
                if best[name] is None or len(out) < len(best[name]):
                    best[name] = out
                    changed = True
    return best


def expand(alt, best):
    out: list[str] = []
    for kind, val in alt:
        if kind == "lit":
            out.append(val)
        elif kind == "term":
            out.append(PLACEHOLDER.get(val, "a"))
        else:
            sub = best.get(val)
            if sub is None:
                return None
            out.extend(sub)
    return out


def can_reach(g, targets):
    """rule -> True when some derivation from it reaches a target rule."""
    reaches = {t: True for t in targets}
    changed = True
    while changed:
        changed = False
        for name, alts in g.rules.items():
            if reaches.get(name):
                continue
            for alt in alts:
                if any(k == "nt" and reaches.get(v) for k, v in alt):
                    reaches[name] = True
                    changed = True
                    break
    return reaches


def sentence_for(g, best, target_rule, target_alt):
    """Shortest sentence from `module` in which target_rule takes target_alt."""
    reaches = can_reach(g, {target_rule})
    hit = [False]

    def emit(name, forced=False, path=frozenset()):
        # A rule that reaches the target may also reach itself. Once it is on
        # the path, take its shortest expansion instead of descending again —
        # the target is reached once, which is all coverage asks.
        if name in path:
            return best[name]
        path = path | {name}
        alts = g.rules[name]
        if name == target_rule and forced and not hit[0]:
            alt = alts[target_alt]
            hit[0] = True
        else:
            viable = [a for a in alts if any(k == "nt" and reaches.get(v) for k, v in a)]
            alt = viable[0] if not hit[0] and viable else None
            if alt is None:
                return best[name]
        out: list[str] = []
        for kind, val in alt:
            if kind == "lit":
                out.append(val)
            elif kind == "term":
                out.append(PLACEHOLDER.get(val, "a"))
            elif val == target_rule and not hit[0]:
                sub = emit(val, forced=True, path=path)
                if sub is None:
                    return None
                out.extend(sub)
            elif reaches.get(val) and not hit[0]:
                sub = emit(val, path=path)
                if sub is None:
                    return None
                out.extend(sub)
            else:
                sub = best.get(val)
                if sub is None:
                    return None
                out.extend(sub)
        return out

    if not reaches.get("module"):
        return None
    result = emit("module")
    return result if hit[0] else None


def render(toks: list) -> str:
    """Join generated tokens the way CFDL is written.

    A dotted name is written closed up — `cre.lease.rent`, never `cre . lease`
    — and the parser reads it that way. Joining every token with a space
    produced sentences no modeller would write and blamed the grammar for
    rejecting them, which would have made this gate a liar.
    """
    out = ""
    for tok in toks:
        if not out:
            out = tok
        elif tok == "." or out.endswith(".") or tok in (",",):
            out += tok
        else:
            out += " " + tok
    return out + "\n"


def parse(text: str):
    """Run `cfdl parse` over one generated sentence. Returns a syntax-error code, or None."""
    with tempfile.TemporaryDirectory() as tmp:
        d = pathlib.Path(tmp)
        (d / "model.cfdl").write_text(text, encoding="utf-8")
        done = subprocess.run(
            [str(CLI), "--json", "parse", str(d)],
            capture_output=True, text=True, encoding="utf-8",
        )
        if done.returncode == 0:
            return None
        try:
            diags = json.loads(done.stdout or "[]")
        except json.JSONDecodeError:
            return "UNPARSEABLE_OUTPUT"
        for d_ in diags if isinstance(diags, list) else []:
            code = d_.get("code", "")
            if is_syntax_error(code):
                return code
        return None


def main() -> int:
    if not CLI.exists():
        print(f"check-grammar-generation: {CLI} not found — run `cargo build -p cfdl-cli`")
        return 1

    g, dropped = ebnf_grammar.load()
    live = reachable(g)
    best = shortest(g)

    # Synthetic rules (groups, repetition, option) carry no meaning of their
    # own; covering the named productions covers them by construction.
    targets = [
        (name, i)
        for name in sorted(live)
        if not name.startswith("__") and name in g.rules
        for i in range(len(g.rules[name]))
    ]

    failures = []
    generated = 0
    for name, alt_i in targets:
        toks = sentence_for(g, best, name, alt_i)
        if toks is None:
            continue
        generated += 1
        text = render(toks)
        code = parse(text)
        if code:
            failures.append((name, alt_i, code, text.strip()))

    if failures:
        print(
            f"check-grammar-generation: the parser refuses {len(failures)} of "
            f"{generated} sentences the published grammar admits.\n",
            file=sys.stderr,
        )
        for name, alt_i, code, text in failures[:25]:
            snippet = text if len(text) < 160 else text[:157] + "..."
            print(f"  {name} (alternative {alt_i + 1}) -> {code}\n      {snippet}", file=sys.stderr)
        if len(failures) > 25:
            print(f"  ... and {len(failures) - 25} more", file=sys.stderr)
        print(
            "\ndocs/02 calls the grammar normative and the site offers it for parser\n"
            "generators, so a production the parser refuses is a promise this project\n"
            "does not keep. Fix the EBNF. If the syntax was removed by decision, say\n"
            "so in a comment containing NOT IMPLEMENTED, or delete the production —\n"
            "both gates respect the annotation.",
            file=sys.stderr,
        )
        return 1

    note = f"; {len(dropped)} NOT IMPLEMENTED production(s) excluded" if dropped else ""
    print(
        f"check-grammar-generation: OK ({generated} sentences generated from the "
        f"published EBNF, every one accepted by the parser{note})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
