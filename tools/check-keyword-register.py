#!/usr/bin/env python3
"""The reserved-word list in `docs/01` §18 must be the lexer's list, exactly.

Three lists have to agree and nothing was comparing them:

  1. what the LEXER reserves          `keyword_from` in crates/cfdl-lexer
  2. what the SPEC documents          docs/01 §18
  3. what the PARSER actually reads   any `Keyword::X` outside `keyword_text`

The drift was one-directional and quiet. The lexer reserves 95 words; §18 listed
57, so 38 words were unusable as identifiers with nothing telling a reader why.
And 14 reserved words are read by no production at all — they render in error
messages and nowhere else — which is legitimate (reserving early keeps a later
feature from breaking existing models) but only if it is stated.

`Mon` through `Sun` are the case that shows why this matters. §18 documented
them as weekday anchors for a weekly schedule. No production reads them, `on`
accepts only `day <n>` or `eom`, and `weekly` is not even a calendar frequency —
so the specification described syntax that has never existed.

So §18 is split, and this gate holds the split honest:

  §18.1  in use                    must be exactly the words the parser reads
  §18.2  reserved, no production   must be exactly the words it does not

Adding a keyword to the lexer now fails here until §18 says which it is.
"""

import pathlib
import re
import sys

LEXER = pathlib.Path("crates/cfdl-lexer/src/lib.rs")
PARSER = pathlib.Path("crates/cfdl-parser/src/lib.rs")
SPEC = pathlib.Path("docs/01_language_spec.md")


def lexer_keywords():
    """word -> Keyword variant, from the `keyword_from` match arms."""
    text = LEXER.read_text(encoding="utf-8")
    body = text[text.index("fn keyword_from") :]
    body = body[: body.index("\n}")]
    return dict(re.findall(r'"([A-Za-z_]+)"\s*=>\s*Keyword::(\w+)', body))


def parser_consumed(keywords):
    """Words some production reads, i.e. mentioned outside `keyword_text`."""
    text = PARSER.read_text(encoding="utf-8")
    start = text.index("fn keyword_text")
    end = text.index("\n}", start)
    return {
        word
        for word, variant in keywords.items()
        if any(
            not (start <= m.start() < end)
            for m in re.finditer(rf"Keyword::{variant}\b", text)
        )
    }


def spec_sections():
    """The word list under each heading — the paragraph after the colon.

    Only that paragraph. The prose below each list quotes keywords too, and
    naming `Mon` while explaining why it is unimplemented must not read as
    declaring it."""
    text = SPEC.read_text(encoding="utf-8")
    out = {}
    for key, heading in (("used", "### 18.1"), ("unused", "### 18.2")):
        start = text.index(heading)
        colon = text.index(":\n\n", start) + 3
        end = text.index("\n\n", colon)
        out[key] = set(re.findall(r"`([A-Za-z_]+)`", text[colon:end]))
    return out


def main():
    keywords = lexer_keywords()
    consumed = parser_consumed(keywords)
    documented = spec_sections()

    want_used = {w for w in keywords if w in consumed}
    want_unused = {w for w in keywords if w not in consumed}

    problems = []

    def diff(label, want, have, section):
        for w in sorted(want - have):
            problems.append(f"  `{w}` is {label} but §{section} does not list it")
        for w in sorted(have - want):
            if w not in keywords:
                problems.append(
                    f"  §{section} lists `{w}`, which the lexer does not reserve"
                )
            else:
                other = "18.2" if section == "18.1" else "18.1"
                problems.append(
                    f"  §{section} lists `{w}`, but it belongs in §{other}"
                )

    diff("read by a production", want_used, documented["used"], "18.1")
    diff("reserved and read by none", want_unused, documented["unused"], "18.2")

    if problems:
        print(
            f"check-keyword-register: {len(problems)} disagreement(s) between the "
            "lexer, the parser and docs/01 §18.\n",
            file=sys.stderr,
        )
        for p in problems:
            print(p, file=sys.stderr)
        print(
            "\n§18 is the published list of what a modeller may not name a thing.\n"
            "A word the lexer reserves and the spec omits is an identifier that\n"
            "stops working with nothing to explain it.",
            file=sys.stderr,
        )
        return 1

    print(
        f"check-keyword-register: OK ({len(keywords)} reserved words; "
        f"{len(want_used)} read by a production, {len(want_unused)} reserved "
        "for a feature not yet built; §18 matches the lexer exactly)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
