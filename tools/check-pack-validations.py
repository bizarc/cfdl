#!/usr/bin/env python3
"""Pack validation hygiene: unique diagnostic codes.

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

## 2. (Removed.) Every validation used to have to declare `match` explicitly.

A validation used to match a contract by EXACT name unless it declared
`match = "instance"`, and 33 of 48 shipped validations had not — so
`E7001_OPCO_LINE_MISSING_AMOUNT` rejected `opco.revenue_line` with no amount
and accepted `opco.revenue_line.core` with no amount. One character apart.

Requiring the declaration fixed it here and nowhere else: this gate reads
`packs/*/validations.toml` in THIS repository, and packs are a published
extension point, so an outside author still got the silent default.

The field is gone instead. Matching is now unconditional and shared with
lowering, which never offered the choice in the first place. There is nothing
left to state, so there is nothing left to check.

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

## 4. A metric selector must be able to reach the streams it names.

A lowering rule naming its stream `energy.ppa.revenue{{contract.dot_suffix}}`
emits the BARE name for an unsuffixed contract and `energy.ppa.revenue.plant_a`
for a suffixed one. A metric selecting the bare name EXACTLY therefore reaches
only half of what the rule can produce, and the half it misses contributes
nothing — an absent stream sums to 0 rather than raising.

All fourteen of the energy pack's selectors were in that state, so a suffixed
PPA contract dropped out of revenue, EBITDA and DSCR. Three goldens ship
`energy.ppa.revenue.plant_a`, one carrying $29.9m; none runs with `--pack`, so
`domain_metrics` is absent from all three and nothing looked.

The rule is mechanical: if a lowering rule's `stream_name` is templated, a
metric naming it needs `.*`, which reaches the bare form and its children both.

Usage: python3 tools/check-pack-validations.py
"""

from __future__ import annotations

import collections
import pathlib
import re
import sys

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - CI pins 3.11
    print("python >= 3.11 required (tomllib)", file=sys.stderr)
    raise SystemExit(1)

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
STREAM_NAME_RE = re.compile(r'stream_name = "([^"]+)"')
TEMPLATE_RE = re.compile(r"\{\{[^}]+\}\}")


def templated_stream_names(rules_path: pathlib.Path) -> set[str]:
    """Stream names a pack's rules build by template, with the template removed.

    `energy.ppa.revenue{{contract.dot_suffix}}` yields `energy.ppa.revenue` —
    the bare form an unsuffixed contract produces, and the prefix every
    suffixed one extends.
    """
    names = set()
    for match in STREAM_NAME_RE.finditer(rules_path.read_text(encoding="utf-8")):
        raw = match.group(1)
        if "{{" in raw:
            names.add(TEMPLATE_RE.sub("", raw).rstrip("."))
    return names


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

    # --- 3b. codes cited in authored documentation exist --------------------
    #
    # The Reference pages are written by hand and cite codes in prose. A code
    # that was renumbered, or invented while writing, produces a page that reads
    # authoritatively and sends someone looking for an error that cannot occur.
    #
    # The generated catalog on those pages comes from the register and so
    # cannot drift; this covers the sentences AROUND it. Region interiors are
    # skipped for that reason — they are the register, restated.
    site_docs = REPO_ROOT / "site" / "content" / "docs"
    known = {f"{number}_{suffix}" for number, sfx in documented.items() for suffix in sfx}
    invented: list[str] = []
    if site_docs.exists() and known:
        for page in sorted(site_docs.rglob("*.md")):
            head = page.read_text(encoding="utf-8")[:400]
            if "generated: full" in head or "layer: specification" in head:
                continue
            in_region = False
            for n, line in enumerate(page.read_text(encoding="utf-8").splitlines(), 1):
                if line.strip().startswith("<!-- cfdl:generated"):
                    in_region = True
                    continue
                if line.strip().startswith("<!-- /cfdl:generated"):
                    in_region = False
                    continue
                if in_region:
                    continue
                for match in DOC_CODE_RE.finditer(line):
                    code = f"{match.group(1)}_{match.group(2)}"
                    if code not in known:
                        invented.append(f"  {page.relative_to(REPO_ROOT)}:{n}  {code}")
    if invented:
        print(
            "check-pack-validations: authored pages cite codes that do not exist.\n",
            file=sys.stderr,
        )
        print("\n".join(invented), file=sys.stderr)
        print(
            f"\nEvery code must appear in {DIAGNOSTICS_DOC.relative_to(REPO_ROOT)}.\n"
            "A renumbered or invented code reads authoritatively and sends a reader\n"
            "looking for an error that cannot occur.",
            file=sys.stderr,
        )
        return 1

    # --- 4. metric selectors can reach the streams they name ----------------
    unreachable: list[str] = []
    checked_selectors = 0
    for metrics_path in sorted(PACKS.glob("*/metrics.toml")):
        pack = metrics_path.parent.name
        rules_path = metrics_path.parent / "lowering" / "rules.toml"
        if not rules_path.exists():
            continue
        templated = templated_stream_names(rules_path)
        specs = tomllib.loads(metrics_path.read_text(encoding="utf-8")).get("metrics", [])
        for spec in specs:
            for key in ("numerator_streams", "denominator_streams"):
                for selector in spec.get(key, []):
                    checked_selectors += 1
                    if not selector.endswith(".*") and selector in templated:
                        unreachable.append(f"{pack}: {spec.get('id', '<no id>')}  <-  {selector}")
    if unreachable:
        print(
            f"  {len(unreachable)} metric selector(s) name a stream their pack builds by\n"
            "  template, but name it EXACTLY — so they reach the unsuffixed contract and\n"
            "  silently miss every suffixed one (an absent stream contributes 0):",
            file=sys.stderr,
        )
        for u in unreachable:
            print(f"      {u}", file=sys.stderr)
        print(
            "\n  Append `.*`. It reaches the bare name and its children both, so it is\n"
            "  correct whichever form the contract takes.",
            file=sys.stderr,
        )
        return 1

    print(
        f"check-pack-validations: OK ({len(seen)} pack codes across {len(files)} packs, "
        f"{len(documented)} documented codes, "
        f"each naming one check; "
        f"{checked_selectors} metric selectors reach their streams)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
