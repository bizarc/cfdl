#!/usr/bin/env python3
"""Validate every results golden against the published results schema.

The sibling of check-ir-schema.py, and it was overdue. The IR schema became a
gate after it drifted; the results schema was left as documentation, and drifted
further and for longer — every one of the 67 committed goldens violated it:

  - `results_version` declared `const "0.1"` while the engine has emitted "0.2"
    since 0.3.0, so the single field whose whole job is to say which shape a
    document has was itself wrong in every document;
  - `deterministic.annual_rollup` was emitted by 62 goldens and undeclared;
  - the root-level `domain_metrics` was emitted by 8 and undeclared.

Four releases, a published contract served at cfdl.dev/schemas, and nothing
noticed. That is the argument for this file existing rather than for a one-off
correction.

Note the asymmetry with the IR gate: `additionalProperties: false` means a NEW
emitted field fails loudly, which is what caught `Series.offset` here. A field
that stops being emitted only fails if it is `required`, so keep the required
lists honest.

Usage: python3 tools/check-results-schema.py
"""

from __future__ import annotations

import json
import os
import pathlib
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA_PATH = REPO_ROOT / "docs" / "schemas" / "results.schema.json"
MIRROR_PATH = REPO_ROOT / "site" / "public" / "schemas" / "CFDL_v0_1_Results.schema.json"
DOC_PATH = REPO_ROOT / "docs" / "06_results_schema.md"
GOLD_RESULTS = REPO_ROOT / "gold" / "results"


def main() -> int:
    try:
        from jsonschema import Draft202012Validator
    except ImportError:
        # Skipping keeps a local run working without the dependency, but a gate
        # that can pass without running is not a gate — which is the whole
        # reason this file exists. CI sets CFDL_REQUIRE_SCHEMA_GATE=1.
        if os.environ.get("CFDL_REQUIRE_SCHEMA_GATE"):
            print(
                "check-results-schema: jsonschema is not installed and CFDL_REQUIRE_SCHEMA_GATE\n"
                "                      is set, so this gate cannot be skipped. `pip install jsonschema`.",
                file=sys.stderr,
            )
            return 1
        print(
            "check-results-schema: jsonschema not installed; skipping.\n"
            "                      install it with `pip install jsonschema` to run this gate.",
            file=sys.stderr,
        )
        return 0

    schema = json.loads(SCHEMA_PATH.read_text())
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema)

    # Three copies of one contract, and only ever one of them gets read. The
    # site serves the mirror; docs/06 is what a human opens. Both drifted —
    # docs/06 was four releases behind — so both are checked against the
    # source of truth rather than trusted.
    if MIRROR_PATH.exists() and json.loads(MIRROR_PATH.read_text()) != schema:
        print(
            f"check-results-schema: {MIRROR_PATH.relative_to(REPO_ROOT)} differs from\n"
            f"                      {SCHEMA_PATH.relative_to(REPO_ROOT)}. The site serves\n"
            "                      the mirror, so they must agree.",
            file=sys.stderr,
        )
        return 1

    if DOC_PATH.exists():
        text = DOC_PATH.read_text()
        fenced = text.split("```json", 1)
        embedded = None
        if len(fenced) == 2:
            try:
                embedded = json.loads(fenced[1].rsplit("```", 1)[0])
            except json.JSONDecodeError:
                embedded = None
        if embedded != schema:
            print(
                f"check-results-schema: {DOC_PATH.relative_to(REPO_ROOT)} does not embed the\n"
                f"                      current {SCHEMA_PATH.relative_to(REPO_ROOT)}. That page is\n"
                "                      generated; regenerate it rather than editing it.",
                file=sys.stderr,
            )
            return 1

    goldens = sorted(GOLD_RESULTS.glob("*.json"))
    if not goldens:
        print(f"check-results-schema: no results goldens under {GOLD_RESULTS}", file=sys.stderr)
        return 1

    failures = 0
    for path in goldens:
        errors = sorted(
            validator.iter_errors(json.loads(path.read_text())),
            key=lambda e: list(e.absolute_path),
        )
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
            f"\ncheck-results-schema: {failures} of {len(goldens)} results goldens violate the schema.\n"
            "                      Either the emitter is wrong or the schema is — decide which,\n"
            "                      rather than loosening the schema to match a defect. A new\n"
            "                      emitted field needs declaring AND a results_version bump.",
            file=sys.stderr,
        )
        return 1

    print(
        f"check-results-schema: OK ({len(goldens)} results goldens satisfy the published schema)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
