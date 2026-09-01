#!/usr/bin/env python3
"""The diagnostic register in `docs/08` must be the codes the tools emit.

Nothing compared them, and the drift ran both ways. Measured 2026-08-29, before
this gate existed: 198 codes documented against 192 emitted, with 18 documented
by nothing that could produce them — `E1002_DUPLICATE_CONTRACT` and
`E1007_DUPLICATE_EVENT` among them, so two contracts or two events under one
name both reached the IR and the spec promised a check that did not exist.
Three codes the packs emit were absent from the page entirely, and two pack
READMEs named a code number that the validation under it does not use.

A register nobody checks is worse than no register: an agent reads `docs/08` as
the repair catalog, and a promised code that never fires teaches it to expect
a diagnostic that will not come.

Extended 2026-09-01 to warnings (W-codes) after the same drift was caught in
review: a W-code's emission was deleted in a refactor and no gate noticed,
while five documents kept describing it.

Four things must agree:

  1. what the CRATES emit         a code literal in crates/**/*.rs
  2. what the PACKS emit          `code = "..."` in packs/*/validations.toml
  3. what `docs/08` documents     the register
  4. what pack READMEs cite       the same numbers, under the same names

A number may name exactly one thing. Codes used only as test data are listed
below with the reason, because a test fixture's code is not a language surface.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "08_diagnostics.md"
CODE = re.compile(r"[EW]\d{4}_[A-Z0-9_]+")

# Codes that exist only as test data, with why. Not language surface.
TEST_ONLY = {
    "E0001_TEST": "lexer/parser diagnostic plumbing tests",
    "E0002_TEST": "lexer/parser diagnostic plumbing tests",
    "E0003_A": "diagnostic ordering test",
    "E0003_B": "diagnostic ordering test",
    "E6099_X": "pack validation loader test",
    "E7001_WRONG_PACK": "pack mismatch test in the validation loader",
}

# Documented codes nothing has ever emitted (git -S finds no history). Their
# fate — build the check or retire the entry — is a register decision to take
# deliberately, not a gate side effect. Every addition here needs that decision
# scheduled; do not let this list grow as a way past the gate.
DOCUMENTED_ONLY = {
    "W3001_EXPR_TYPE_UNKNOWN": "register entry predates any implementation",
    "W3002_OBS_REF_EXTRACTION_FAILED": "register entry predates any implementation",
}


def codes_in(path: pathlib.Path) -> set[str]:
    return set(CODE.findall(path.read_text(encoding="utf-8")))


def main() -> int:
    emitted: dict[str, pathlib.Path] = {}
    for crate in sorted((ROOT / "crates").rglob("*.rs")):
        if "target" in crate.parts:
            continue
        for code in codes_in(crate):
            emitted.setdefault(code, crate)
    for validations in sorted((ROOT / "packs").glob("*/validations.toml")):
        for code in codes_in(validations):
            emitted.setdefault(code, validations)

    documented = codes_in(DOC)
    readme_codes: dict[str, pathlib.Path] = {}
    for readme in sorted((ROOT / "packs").glob("*/README.md")):
        for code in codes_in(readme):
            readme_codes.setdefault(code, readme)

    problems: list[str] = []

    for code in sorted(set(emitted) - documented - set(TEST_ONLY)):
        problems.append(
            f"  {code}\n"
            f"      emitted by {emitted[code].relative_to(ROOT)}, documented nowhere.\n"
            f"      Add it to docs/08, or stop emitting it."
        )

    for code in sorted(documented - set(emitted) - set(DOCUMENTED_ONLY)):
        problems.append(
            f"  {code}\n"
            f"      documented in docs/08, emitted by nothing.\n"
            f"      Build the check, or delete the entry — a promised diagnostic\n"
            f"      that never fires is a lie the repair catalog repeats."
        )

    for code, readme in sorted(readme_codes.items()):
        if code in emitted or code in TEST_ONLY:
            continue
        problems.append(
            f"  {code}\n"
            f"      cited by {readme.relative_to(ROOT)}, emitted by nothing.\n"
            f"      Pack READMEs must name the code its validation actually uses."
        )

    by_number: dict[str, set[str]] = {}
    # A test fixture may reuse a number deliberately; only real surface collides.
    for code in (set(emitted) | documented | set(readme_codes)) - set(TEST_ONLY):
        by_number.setdefault(code[:5], set()).add(code)
    for number, names in sorted(by_number.items()):
        if len(names) > 1:
            problems.append(
                f"  {number}\n"
                f"      names {len(names)} different things: {', '.join(sorted(names))}.\n"
                f"      A number identifies one condition."
            )

    if problems:
        print("check-diagnostic-parity: the register and the code disagree.\n")
        print("\n".join(problems))
        print(
            f"\n  {len(documented)} documented, "
            f"{len(set(emitted) - set(TEST_ONLY))} emitted (excluding test-only codes)."
        )
        return 1

    print(
        f"check-diagnostic-parity: OK ({len(documented)} codes; docs/08, the crates,"
        " the pack validations and the pack READMEs agree)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
