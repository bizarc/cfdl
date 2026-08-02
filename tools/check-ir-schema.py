#!/usr/bin/env python3
"""Validate every IR golden against the published IR schema.

The schema at docs/schemas/ir.schema.json is a public contract — it is served
at cfdl.dev/schemas and consumers write against it — but nothing checked that
the compiler's output actually satisfies it. It drifted: `metrics` was listed
as required while no compiler has ever emitted it, `stub` and weekday rules
were declared and never produced, and an `EndOfMonth` rule wrote `day: 0`
against a stated 1..31 bound. Every one of those went unnoticed because the
schema was documentation rather than a gate.

This makes it a gate. Any golden that violates the schema fails the build, so
the schema and the emitter cannot diverge without someone deciding which of
them is wrong.

Usage: python3 tools/check-ir-schema.py
"""

from __future__ import annotations

import json
import os
import pathlib
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA_PATH = REPO_ROOT / "docs" / "schemas" / "ir.schema.json"
GOLD_IR = REPO_ROOT / "gold" / "ir"


def main() -> int:
    try:
        from jsonschema import Draft202012Validator
    except ImportError:
        # Skipping keeps a local run working without the dependency, but a gate
        # that can pass without running is not a gate. CI sets
        # CFDL_REQUIRE_SCHEMA_GATE=1 so the skip is a failure there.
        if os.environ.get("CFDL_REQUIRE_SCHEMA_GATE"):
            print(
                "check-ir-schema: jsonschema is not installed and CFDL_REQUIRE_SCHEMA_GATE\n"
                "                 is set, so this gate cannot be skipped. `pip install jsonschema`.",
                file=sys.stderr,
            )
            return 1
        print(
            "check-ir-schema: jsonschema not installed; skipping.\n"
            "                 install it with `pip install jsonschema` to run this gate.",
            file=sys.stderr,
        )
        return 0

    schema = json.loads(SCHEMA_PATH.read_text())
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema)

    goldens = sorted(GOLD_IR.glob("*.json"))
    if not goldens:
        print(f"check-ir-schema: no IR goldens under {GOLD_IR}", file=sys.stderr)
        return 1

    failures = 0
    for path in goldens:
        errors = sorted(validator.iter_errors(json.loads(path.read_text())),
                        key=lambda e: list(e.absolute_path))
        if not errors:
            continue
        failures += 1
        print(f"  {path.relative_to(REPO_ROOT)}", file=sys.stderr)
        for err in errors[:5]:
            where = "/".join(str(p) for p in err.absolute_path) or "<root>"
            print(f"      {where}: {err.message}", file=sys.stderr)
        if len(errors) > 5:
            print(f"      … and {len(errors) - 5} more", file=sys.stderr)

    if failures:
        print(
            f"\ncheck-ir-schema: {failures} of {len(goldens)} IR goldens violate the schema.\n"
            "                 Either the emitter is wrong or the schema is — decide which,\n"
            "                 rather than loosening the schema to match a defect.",
            file=sys.stderr,
        )
        return 1

    print(f"check-ir-schema: OK ({len(goldens)} IR goldens satisfy the published schema)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
