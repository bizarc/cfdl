#!/usr/bin/env python3
"""Validate every run configuration in the repository against the run schema.

`docs/schemas/run.schema.json` describes what a `run.json` may contain, and
until this gate existed nothing read it. Not the engine, which parses run
configs with serde and applies its own rules; not the CLI; not a test.

It drifted, in the direction that silence always allows. `valuation_grain` has
been accepted by the engine and documented in the user guide for as long as it
has existed, and the schema never listed it. Because `DeterministicCase` sets
`additionalProperties: false`, a run stating its own grain would have been
rejected by the schema it is supposed to conform to. Nobody noticed, because
nothing checked.

This makes it a gate. A run config that violates the schema fails the build, so
the schema and the parser cannot diverge again without someone deciding which
of them is wrong.

Usage: python3 tools/check-run-schema.py
"""

from __future__ import annotations

import json
import os
import pathlib
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
SCHEMA_PATH = REPO_ROOT / "docs" / "schemas" / "run.schema.json"

# Every directory a run config can live in. Kept explicit rather than globbing
# the repository, so a stray fixture somewhere unexpected is a decision rather
# than an accident.
SEARCH_ROOTS = ("benchmarks", "fixtures", "training", "examples")


def main() -> int:
    try:
        from jsonschema import Draft202012Validator
    except ImportError:
        # Skipping keeps a local run working without the dependency, but a gate
        # that can pass without running is not a gate. CI sets
        # CFDL_REQUIRE_SCHEMA_GATE=1 so the skip is a failure there.
        if os.environ.get("CFDL_REQUIRE_SCHEMA_GATE"):
            print(
                "check-run-schema: jsonschema is not installed and CFDL_REQUIRE_SCHEMA_GATE\n"
                "                  is set, so this gate cannot be skipped. `pip install jsonschema`.",
                file=sys.stderr,
            )
            return 1
        print(
            "check-run-schema: jsonschema not installed; skipping.\n"
            "                  install it with `pip install jsonschema` to run this gate.",
            file=sys.stderr,
        )
        return 0

    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema)

    configs = sorted(
        path
        for root in SEARCH_ROOTS
        for path in (REPO_ROOT / root).rglob("run.json")
    )
    if not configs:
        print("check-run-schema: no run configs found; nothing to check.", file=sys.stderr)
        return 1

    failures = 0
    for path in configs:
        rel = path.relative_to(REPO_ROOT)
        try:
            document = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as err:
            print(f"check-run-schema: {rel} is not valid JSON: {err}", file=sys.stderr)
            failures += 1
            continue
        # Report every violation in a file rather than the first, so one run
        # names all the work instead of one error at a time.
        for error in sorted(validator.iter_errors(document), key=lambda e: list(e.path)):
            where = "/".join(str(p) for p in error.path) or "(root)"
            print(f"check-run-schema: {rel}: {where}: {error.message}", file=sys.stderr)
            failures += 1

    if failures:
        print(
            f"\ncheck-run-schema: {failures} violation(s) across {len(configs)} run configs.\n"
            "                  Either the config is wrong, or the schema does not describe\n"
            "                  what the engine accepts. Decide which before silencing this.",
            file=sys.stderr,
        )
        return 1

    print(f"check-run-schema: OK ({len(configs)} run configs match the published schema)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
