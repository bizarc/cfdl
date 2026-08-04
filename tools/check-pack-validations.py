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

Codes are identified by LETTER plus number (`E5010`, `W3001`), not by the number
alone, so `E3500` and `W3500` are distinct and may coexist. Both checks below
originally matched an `E` followed by digits, and so were blind to warnings
entirely.

## 3. Documented diagnostic codes must be unique too.

Check 1 reads `packs/*/validations.toml`, which is where authors add codes by
hand — and where all four original collisions were. It does not see codes the
ENGINE emits, which live in Rust string literals.

That gap bit: adding two lowering diagnostics, `E5010` and `E5011` were picked
by eye and both were already taken (`E5010_TERM_UNKNOWN_INPUT`,
`E5011_TERM_CLIP_OUT_OF_BOUNDS`). Same failure as before, one layer over.

Extracting from Rust would need to exclude deliberately corrupted codes inside
parser tests. `docs/08_diagnostics.md` is the published register and every
engine code is listed there, so checking uniqueness across that page catches the
collision at the point an author is most likely to make it, without the false
positives.

Usage: python3 tools/check-pack-validations.py
"""

from __future__ import annotations

import collections
import pathlib
import re
import sys

# These tools print prose. A Windows console defaults to cp1252, which
# cannot encode every character the check names use, so pin stdout to UTF-8.
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
PACKS = REPO_ROOT / "packs"
DIAGNOSTICS_DOC = REPO_ROOT / "docs" / "08_diagnostics.md"
# The identifier is the LETTER plus the number, not the number alone: `E3500`
# and `W3500` are different codes and may coexist. Before this was widened both
# regexes matched only `E`, so warning codes were invisible to both uniqueness
# checks — a `W3500` could be added twice, or added without ever being
# documented, and nothing looked. That is the same blind spot check 3 exists to
# close, one letter over.
DOC_CODE_RE = re.compile(r"`([EWI]\d+)_([A-Z0-9_]+)`")

CODE_RE = re.compile(r'code = "([EWI]\d+)_([A-Z0-9_]+)"')
MATCH_RE = re.compile(r'^match = ', re.M)


def main() -> int:
    seen: dict[str, list[tuple[str, str]]] = collections.defaultdict(list)
    files = sorted(PACKS.glob("*/validations.toml"))
    if not files:
        print(f"check-pack-validations: no validations under {PACKS}", file=sys.stderr)
        return 1

    for path in files:
        pack = path.parent.name
        for match in CODE_RE.finditer(path.read_text(encoding="utf-8")):
            seen[match.group(1)].append((pack, match.group(2)))

    # --- 2. every validation states its match mode -------------------------
    unstated = []
    for path in files:
        pack = path.parent.name
        for block in path.read_text(encoding="utf-8").split("[[validations]]")[1:]:
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

    # --- 3. documented codes are unique ------------------------------------
    documented: dict[str, list[str]] = collections.defaultdict(list)
    if DIAGNOSTICS_DOC.exists():
        for match in DOC_CODE_RE.finditer(DIAGNOSTICS_DOC.read_text(encoding="utf-8")):
            if match.group(2) not in documented[match.group(1)]:
                documented[match.group(1)].append(match.group(2))
    doc_failures = 0
    for number, suffixes in sorted(documented.items()):
        if len(suffixes) > 1:
            doc_failures += 1
            print(f"  {number} names {len(suffixes)} different checks in", file=sys.stderr)
            print(f"      {DIAGNOSTICS_DOC.relative_to(REPO_ROOT)}:", file=sys.stderr)
            for suffix in suffixes:
                print(f"      {number}_{suffix}", file=sys.stderr)
    if doc_failures:
        print(
            f"\ncheck-pack-validations: {doc_failures} documented code(s) name more than one\n"
            "                        check. Pick a free number — reading the file to find one\n"
            "                        is exactly what produced the last two collisions.",
            file=sys.stderr,
        )
        return 1

    print(
        f"check-pack-validations: OK ({len(seen)} pack codes across {len(files)} packs, "
        f"{len(documented)} documented codes, "
        f"each naming one check; every validation states its match mode)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
