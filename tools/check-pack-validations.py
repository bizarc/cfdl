#!/usr/bin/env python3
"""Pack validation hygiene: unique codes, and a deliberate match mode.

## 1. Every diagnostic code must name exactly one check.

The numeric prefix is the stable identifier — it is what a user greps for, what
a support conversation quotes, and what a downstream tool matches on. If two
unrelated failures both answer to `E7010`, all of them become unsearchable and
the documentation for one is wrong for the other.

They did. `E7010` named both OPCO_LINE_AMBIGUOUS_AMOUNT and
OPCO_WC_MISSING_AMOUNT_OR_RULE; `E7011` and `E6030` were each doubled the same
way. Nobody noticed because nothing looked.

Worth recording how this gate came to exist: while renumbering those three, the
replacement code chosen for one of them was ALSO already taken, creating a
fourth duplicate in the act of fixing the first three. Picking a free code by
reading the file is not reliable — that is what a gate is for.

Scope note: the crates contain deliberately corrupted codes inside parser tests
(`E7001_WRONG_PACK` is a valid code with its prefix swapped, used to assert that
a mismatched pack is rejected). Those are fixtures, not emissions, so this gate
reads `packs/*/validations.toml` only. Engine-emitted codes live in Rust string
literals scattered across crates and would need a different, noisier extraction;
the pack files are where authors add codes by hand and where all four
collisions occurred.

## 2. Every validation must declare `match` explicitly.

A validation matches a contract by EXACT name unless it declares
`match = "instance"`. Contracts are routinely written suffixed
(`opco.revenue_line.core`), so an unflagged validation is silently skipped on
the form models actually use — it never fires, and nothing says so.

33 of 48 shipped validations were in that state. Credit had all ten flagged;
cre, energy and opco had almost none, so `E7001_OPCO_LINE_MISSING_AMOUNT`
rejected `opco.revenue_line` with no amount and accepted
`opco.revenue_line.core` with no amount. One character of difference.

Defaulting is the trap, so this requires the choice to be WRITTEN — either
`instance` or `exact`. An author who wants exact matching may still have it;
they just have to say so.

Usage: python3 tools/check-pack-validations.py
"""

from __future__ import annotations

import collections
import pathlib
import re
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
PACKS = REPO_ROOT / "packs"

CODE_RE = re.compile(r'code = "(E\d+)_([A-Z0-9_]+)"')
MATCH_RE = re.compile(r'^match = ', re.M)


def main() -> int:
    seen: dict[str, list[tuple[str, str]]] = collections.defaultdict(list)
    files = sorted(PACKS.glob("*/validations.toml"))
    if not files:
        print(f"check-pack-validations: no validations under {PACKS}", file=sys.stderr)
        return 1

    for path in files:
        pack = path.parent.name
        for match in CODE_RE.finditer(path.read_text()):
            seen[match.group(1)].append((pack, match.group(2)))

    # --- 2. every validation states its match mode -------------------------
    unstated = []
    for path in files:
        pack = path.parent.name
        for block in path.read_text().split("[[validations]]")[1:]:
            if not MATCH_RE.search(block):
                code = CODE_RE.search(block)
                unstated.append(f"{pack}: {code.group(0).split(chr(34))[1] if code else '<no code>'}")
    if unstated:
        print(
            f"  {len(unstated)} validation(s) do not state `match`, so they"
            " default to EXACT\n  and are silently skipped on suffixed contracts:",
            file=sys.stderr,
        )
        for u in unstated:
            print(f"      {u}", file=sys.stderr)
        print(
            '\n  Add `match = "instance"` (or `match = "exact"` if that is really meant).\n'
            "  It must follow the CLOSE of a multi-line `contracts` array.",
            file=sys.stderr,
        )
        return 1

    # --- 1. codes are unique ----------------------------------------------
    failures = 0
    for number, uses in sorted(seen.items()):
        distinct = sorted(set(uses))
        if len(distinct) > 1:
            failures += 1
            print(f"  {number} names {len(distinct)} different checks:", file=sys.stderr)
            for pack, suffix in distinct:
                print(f"      {pack}: {number}_{suffix}", file=sys.stderr)

    if failures:
        print(
            f"\ncheck-pack-validations: {failures} numeric code(s) name more than one check.\n"
            "                        A code is an identifier, so pick a free number rather\n"
            "                        than reusing one. Renumbering a SHIPPED code is breaking\n"
            "                        for anyone matching on it — say so in the changelog.",
            file=sys.stderr,
        )
        return 1

    print(
        f"check-pack-validations: OK ({len(seen)} codes across {len(files)} packs, "
        f"each naming one check; every validation states its match mode)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
