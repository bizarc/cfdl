#!/usr/bin/env python3
"""A pack must not read an instanceable stream family by its bare name.

A model suffixes a contract whenever a deal has more than one of something —
two tenants are `cre.lease_unit.tenant_a` and `.tenant_b` — and a lowering rule
says so by templating its stream name with `{{contract.dot_suffix}}`. Reading
such a family needs `.*`, which matches the bare name AND its children. A bare
pattern matches only the bare name, so every instanced sibling is skipped.

Nothing reports that. `W5022_UNKNOWN_SERIES_REFERENCE` fires when a name matches
NOTHING, which is the case where a model happens to declare no unsuffixed
instance. The dangerous case is the other one: the model declares a bare
instance too, the pattern matches it, no warning is emitted, and the suffixed
siblings are silently dropped from the sum.

IT HAS HAPPENED TWICE, in the same expression, both times in forward NOI:

  cre.pct_rent      a bare read paired with a `.*` read, so an unsuffixed
                    percentage-rent contract entered forward NOI TWICE and
                    inflated the exit price.

  cre.property.opex a bare read of an instanceable family, so an instanced
                    expense line was invisible to forward NOI. NOI overstated,
                    and the exit price struck off it too high.

Each was found by hand, months apart, and each is worth real money on the
reversion. The rule is mechanical, so it should not be found by hand.

Three surfaces are checked, because the same mistake was made independently on
two of them (`packs/energy/metrics.toml` records selectors that "were previously
exact, so a suffixed" stream was missed):

  1. `series_sum` / `series_avg` inside lowering-rule expressions
  2. stream selectors in `metrics.toml`
  3. stream selectors on subtotals and statement rows in `statements.toml`

To read only the unsuffixed instance on purpose, append to the line:

    # series-allow: <reason>
"""

import pathlib
import re
import sys
import tomllib

PACKS = pathlib.Path("packs")
SUFFIX = "{{contract.dot_suffix}}"
SERIES_CALL = re.compile(r'series_(?:sum|avg)\(\s*"([^"]+)"')
ALLOW = re.compile(r"#\s*series-allow:")

# Rule fields that hold an expression the engine evaluates.
EXPR_FIELDS = (
    "amount_expr",
    "active_when",
    "field_init",
    "field_next",
)
SELECTOR_FIELDS = ("numerator_streams", "denominator_streams", "streams")


def families(rules):
    """Stream families a pack emits per instance: base name -> rule id."""
    return {
        rule["stream_name"].replace(SUFFIX, ""): rule["id"]
        for rule in rules
        if SUFFIX in rule.get("stream_name", "")
    }


def allowed_lines(path):
    """Line numbers carrying an explicit waiver."""
    return {
        i
        for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1)
        if ALLOW.search(line)
    }


def find_line(path, needle):
    for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if needle in line:
            return i
    return 0


def check_pack(pack):
    rules_path = pack / "lowering" / "rules.toml"
    if not rules_path.exists():
        return [], 0
    rules = tomllib.loads(rules_path.read_text(encoding="utf-8")).get("rules", [])
    fam = families(rules)
    if not fam:
        return [], 0

    problems = []

    def report(path, base, where):
        line = find_line(path, base)
        if line in allowed_lines(path):
            return
        rel = path.relative_to(pathlib.Path("."))
        problems.append(
            f"  {rel}:{line}\n"
            f"      {where} reads '{base}' by bare name.\n"
            f"      '{base}{SUFFIX}' is emitted per instance by rule "
            f"'{fam[base]}', so this skips every suffixed one.\n"
            f"      Write '{base}.*' — it matches the bare name as well."
        )

    for rule in rules:
        for field in EXPR_FIELDS:
            for name in SERIES_CALL.findall(str(rule.get(field, "") or "")):
                if name in fam:
                    report(rules_path, name, f"rule '{rule['id']}' {field}")

    for fname, sections in (
        ("metrics.toml", ("metrics",)),
        ("statements.toml", ("subtotals", "statements")),
    ):
        path = pack / fname
        if not path.exists():
            continue
        doc = tomllib.loads(path.read_text(encoding="utf-8"))
        for section in sections:
            for item in doc.get(section, []) or []:
                ident = item.get("id", "?")
                for key in SELECTOR_FIELDS:
                    for sel in item.get(key, []) or []:
                        if sel in fam:
                            report(path, sel, f"{section} '{ident}' {key}")
                for row in item.get("rows", []) or []:
                    for sel in row.get("streams", []) or []:
                        if sel in fam:
                            report(path, sel, f"row '{row.get('label', '?')}' streams")

    return problems, len(fam)


def main():
    if not PACKS.is_dir():
        print("check-pack-series: no packs/ directory", file=sys.stderr)
        return 1
    problems, checked = [], 0
    packs = 0
    for pack in sorted(PACKS.iterdir()):
        if not pack.is_dir():
            continue
        found, n = check_pack(pack)
        if n:
            packs += 1
        checked += n
        problems.extend(found)
    if problems:
        print(
            f"check-pack-series: {len(problems)} bare read(s) of an instanceable "
            "stream family.\n",
            file=sys.stderr,
        )
        for p in problems:
            print(p, file=sys.stderr)
        print(
            "\nA bare pattern matches only the unsuffixed stream. Every instanced\n"
            "sibling is skipped, and nothing warns, because the pattern did match\n"
            "something. Append `# series-allow: <reason>` if that is deliberate.",
            file=sys.stderr,
        )
        return 1
    print(
        f"check-pack-series: OK ({checked} instanceable stream families across "
        f"{packs} packs, every read globbed)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
