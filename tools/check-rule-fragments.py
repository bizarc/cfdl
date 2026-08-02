#!/usr/bin/env python3
"""Repeated expression fragments in a pack rule file must be byte-identical.

`packs/credit/lowering/rules.toml` carries the same hazard arithmetic in 18
rules. That was tolerable while the arithmetic was one `pow(k, p)` call; it is
not now that each copy contains the PSA, SDA and ABS branches.

The reason this needs a gate rather than care: **every committed golden and
every benchmark runs at a CONSTANT hazard**, taking the `speed == 0` branch of
each `if`. Nothing in the suite evaluates the ramp branches, so a typo in one
is invisible to them.

Measured rather than assumed, by injecting a 10x typo (`0.002` -> `0.02`) into
one PSA branch and re-blessing:

    typo in a shared `state_next`      -> E5021_DUPLICATE_LOWERED_STATE
    typo in one rule's `amount_expr`   -> gold PASSES, benchmarks PASS,
                                          analytic checks PASS, only this gate
                                          fails

So half the surface is already protected and half is not. `state_next` is safe
because five rules per family declare the SAME state, and the compiler requires
their text to agree — a real guarantee, worth knowing about before adding
another. `amount_expr` has no such peer: each rule's is independent, so a typo
there changes one stream's hazard and nothing notices.

That gap is what this gate closes. The invariant is structural: whatever the
fragment says, all copies must say the same thing. A wrong fragment is then
wrong everywhere at once, which the external ramped cases DO catch.

Checked per file:

  * every `state_next` sharing a `state_name` prefix is identical
  * the hazard sub-expressions (`psa_speed`, `sda_speed`, `abs_speed` branches)
    appear with the same text everywhere they appear at all

Usage: python3 tools/check-rule-fragments.py
"""

from __future__ import annotations

import collections
import pathlib
import re
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
RULES = sorted((REPO_ROOT / "packs").glob("*/lowering/rules.toml"))

FIELD_RE = re.compile(r'^(state_next|state_init|state_every)\s*=\s*"(.*)"$', re.M)
STATE_NAME_RE = re.compile(r'^state_name\s*=\s*"([^"{]*)', re.M)

# The hazard branches, keyed by the term that selects them. Each is captured
# from its `if(` through to the matching close, by paren balance.
SPEEDS = ("psa_speed", "sda_speed", "abs_speed")

# The age a hazard is asked about legitimately differs between copies: a stream
# reads the CURRENT payment period, a recurrence reads the one before it, and a
# recoveries rule reads the period the default happened in. Normalising those
# away is what makes the remaining comparison meaningful — the invariant is that
# every copy computes the SAME FUNCTION of age, not that every copy asks about
# the same age.
AGES = (
    "{{time.elapsed_periods}} - {{whole_periods.recovery_lag_months}}",
    "{{time.elapsed_periods}} - 1",
    "{{time.elapsed_periods}}",
)


def normalise_age(expr: str) -> str:
    for age in AGES:  # longest first, so the bare form does not shadow the others
        expr = expr.replace(age, "<AGE>")
    return expr


def branches(text: str, speed: str) -> list[str]:
    """Every `if({{contract.<speed>}} == 0, ...)` expression in `text`."""
    out = []
    needle = f"if({{{{contract.{speed}}}}} == 0,"
    idx = text.find(needle)
    while idx != -1:
        depth, i = 0, idx + 2  # start at the '(' of `if(`
        while i < len(text):
            if text[i] == "(":
                depth += 1
            elif text[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        out.append(text[idx : i + 1])
        idx = text.find(needle, i)
    return out


def main() -> int:
    failures = 0
    checked = 0
    for path in RULES:
        text = path.read_text()
        rel = path.relative_to(REPO_ROOT)

        # 1. one state_name prefix, one recurrence
        by_name: dict[str, set[str]] = collections.defaultdict(set)
        for match in FIELD_RE.finditer(text):
            if match.group(1) != "state_next":
                continue
            before = text[: match.start()]
            name = STATE_NAME_RE.findall(before)
            if name:
                by_name[name[-1]].add(match.group(2))
        by_name = {k: {normalise_age(v) for v in vs} for k, vs in by_name.items()}
        for name, variants in sorted(by_name.items()):
            checked += 1
            if len(variants) > 1:
                failures += 1
                print(
                    f"  {rel}: state '{name}' has {len(variants)} different"
                    " `state_next` expressions:",
                    file=sys.stderr,
                )
                for v in sorted(variants):
                    print(f"      {v[:110]}...", file=sys.stderr)

        # 2. one hazard convention, one spelling
        for speed in SPEEDS:
            found = branches(text, speed)
            if not found:
                continue
            checked += 1
            distinct = sorted({normalise_age(f) for f in found})
            if len(distinct) > 1:
                failures += 1
                print(
                    f"  {rel}: `{speed}` branch is spelled {len(distinct)}"
                    f" different ways across {len(found)} uses:",
                    file=sys.stderr,
                )
                for v in distinct:
                    print(f"      {v[:110]}...", file=sys.stderr)

    if failures:
        print(
            f"\ncheck-rule-fragments: {failures} fragment(s) differ between copies.\n"
            "                      Every golden runs at a constant hazard, so a typo in a\n"
            "                      ramp branch is invisible to the whole suite — that is\n"
            "                      exactly what this gate exists to catch. Make the copies\n"
            "                      identical rather than relaxing the check.",
            file=sys.stderr,
        )
        return 1

    print(f"check-rule-fragments: OK ({checked} repeated fragments identical across {len(RULES)} packs)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
