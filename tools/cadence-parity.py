#!/usr/bin/env python3
"""Assert that a pack lowers the same deal to the same economics on any calendar.

The golden runner diffs each fixture against its own blessed output, so it
cannot compare two fixtures to each other — and "cadence-neutral" is exactly a
statement about two fixtures. Nothing else in the suite can express it.

A parity group is a set of fixtures describing ONE deal, written identically
except for `time calendar`. The invariant (I1) is that their ANNUAL figures
agree:

    sum over the periods of a year of (X_year / periods_per_year) == X_year

which holds for any per-period amount derived from an annual one, on any grid,
and fails the moment a rule divides by a literal 12 again. `annual_rollup` is
already in the results on a calendar-annual index, so this needs no engine
support.

What I1 deliberately does NOT cover:

  * NPV. A monthly stream's cash arrives on average ~5.5 months earlier within
    the year than an annual stream's single year-end payment, so at 12% the
    two differ by ~5-6%. That is correct economics, not drift — and it is why
    benchmarks/cre/mit_rentleg_plaza must stay annual to reproduce the MIT
    year-end pro forma's published $2,292,810.
  * Nominally-accruing debt. `rate / ppy` is a genuinely different accrual per
    cadence: a 6% loan is 0.5%/month and 1.5%/quarter, whose effective annual
    rates differ, and every loan document agrees. Those rules are checked
    against a ppy-parameterized reference generator instead, not against each
    other.

  * Leap years on a DAILY grid. An annual quantity spread as X_year / 365 is
    the Act/365-Fixed convention, so a 366-day year pays 366/365 of it — about
    0.27% more. The convention is right; the annual identity simply does not
    hold across a leap year at daily resolution. Daily fixtures therefore use a
    non-leap window rather than a loosened tolerance, which would also hide
    real errors of that size.

Add a group by dropping fixtures named `<prefix>_<calendar>` into
fixtures/valid/ and listing the prefix in GROUPS.

Usage: python3 tools/cadence-parity.py
"""

from __future__ import annotations

import json
import os
import pathlib
import subprocess
import sys
import tempfile

# These tools print prose. A Windows console defaults to cp1252, which
# cannot encode every character the check names use, so pin stdout to UTF-8.
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
# Windows names the binary cfdl.exe; everywhere else it is bare `cfdl`.
# The `.exists()` guard below is what makes this matter: subprocess would
# find cfdl.exe on its own, but the preflight check would not.
CLI = REPO_ROOT / "target" / "debug" / ("cfdl.exe" if os.name == "nt" else "cfdl")
FIXTURES = REPO_ROOT / "fixtures" / "valid"
PACKS = REPO_ROOT / "packs"

# Relative tolerance. Generous enough for decimal/float round-tripping through
# JSON, tight enough that a one-period misalignment or a stray /12 cannot hide.
REL_TOL = 1e-9

# Parity groups: (fixture prefix, pack, calendars, per-stream waivers).
# A waiver needs a reason; see the module docstring for the two legitimate
# kinds. Mirrors the escape-hatch convention in tools/check-doc-examples.py.
GROUPS: list[tuple[str, str, list[str], dict[str, str]]] = [
    (
        "pack_cadence_probe",
        "testpack",
        ["monthly", "quarterly", "annual"],
        {},
    ),
    (
        "pack_cadence_energy",
        "energy",
        ["monthly", "quarterly", "annual", "daily"],
        {},
    ),
    (
        "pack_cadence_opco",
        "opco",
        ["monthly", "quarterly", "annual"],
        {},
    ),
    # Credit is NOT an I1 group across quarterly/annual: `rate / ppy` is a
    # nominal accrual, so a 6% loan genuinely differs at 0.5%/month against
    # 1.5%/quarter. Those cadences are covered by the benchmark reference
    # generators instead (I2). What IS an exact identity is the daily-book
    # case: a daily grid paying monthly is the same 36 payments as a monthly
    # grid, so its annual totals must match to the last cent.
    (
        "pack_cadence_credit",
        "credit",
        ["monthly", "daily_monthly_pay"],
        {},
    ),
    (
        "pack_cadence_cre",
        "cre",
        ["monthly", "quarterly", "annual"],
        {},
    ),
]


class FixtureError(RuntimeError):
    """A fixture failed to compile or run; carries the CLI's own diagnostics."""


def _cfdl(args: list[str], what: str) -> None:
    done = subprocess.run(args, capture_output=True, text=True, encoding="utf-8")
    if done.returncode != 0:
        # Surface the compiler's diagnostics. A traceback here says only that
        # a subprocess exited non-zero, which sends the reader to the wrong
        # place entirely.
        detail = (done.stdout + done.stderr).strip() or f"exit {done.returncode}"
        raise FixtureError(f"{what} failed:\n" + "\n".join(
            "      " + line for line in detail.splitlines()[:8]))


def run_fixture(directory: pathlib.Path, pack: str) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        ir = pathlib.Path(tmp) / "ir.json"
        results = pathlib.Path(tmp) / "results.json"
        _cfdl(
            [str(CLI), "compile", str(directory), "--packs", str(PACKS), "--out", str(ir)],
            f"compiling {directory.name}",
        )
        _cfdl(
            [str(CLI), "run", str(ir), "--packs", str(PACKS), "--pack", pack,
             "--out", str(results)],
            f"running {directory.name}",
        )
        return json.loads(results.read_text(encoding="utf-8"))["deterministic"]


def annual_series(block: dict) -> dict[str, list[float]]:
    """Per-stream annual figures, whatever the model's own calendar.

    An annual model has no `annual_rollup` — its periods already are years — so
    fall back to the raw series rather than skipping the comparison, which
    would quietly exempt the case most likely to be wrong.
    """
    source = block.get("annual_rollup", {}).get("series") or block.get("series", {})
    out: dict[str, list[float]] = {}
    for name, series in source.items():
        if not name.startswith("stream."):
            continue
        out[name] = [point["amount"] for point in series["values"]]
    return out


def compare(baseline_name: str, baseline: dict, other_name: str, other: dict,
            waivers: dict[str, str]) -> list[str]:
    failures: list[str] = []
    for stream in sorted(set(baseline) | set(other)):
        if stream in waivers:
            continue
        if stream not in baseline or stream not in other:
            failures.append(f"    {stream}: present in only one of {baseline_name}, {other_name}")
            continue
        a, b = baseline[stream], other[stream]
        if len(a) != len(b):
            failures.append(
                f"    {stream}: {len(a)} annual periods in {baseline_name}, {len(b)} in {other_name}"
            )
            continue
        for year, (x, y) in enumerate(zip(a, b)):
            scale = max(abs(x), abs(y), 1.0)
            if abs(x - y) / scale > REL_TOL:
                failures.append(
                    f"    {stream}: year {year + 1} is {x:,.6f} on {baseline_name} "
                    f"but {y:,.6f} on {other_name}"
                )
    return failures


def main() -> int:
    if not CLI.exists():
        print(f"cadence-parity: {CLI} not found; run `cargo build -p cfdl-cli` first.")
        return 1

    failed = 0
    checked = 0
    for prefix, pack, calendars, waivers in GROUPS:
        runs: dict[str, dict[str, list[float]]] = {}
        for calendar in calendars:
            directory = FIXTURES / f"{prefix}_{calendar}"
            if not directory.is_dir():
                print(f"  FAIL  {prefix}: missing fixture {directory.relative_to(REPO_ROOT)}")
                failed += 1
                break
            try:
                runs[calendar] = annual_series(run_fixture(directory, pack))
            except FixtureError as error:
                print(f"  FAIL  {prefix}: {error}")
                failed += 1
                break
        else:
            baseline_calendar = calendars[0]
            for calendar in calendars[1:]:
                checked += 1
                problems = compare(
                    baseline_calendar, runs[baseline_calendar],
                    calendar, runs[calendar],
                    waivers,
                )
                if problems:
                    failed += 1
                    print(f"  FAIL  {prefix}: {baseline_calendar} vs {calendar}")
                    for problem in problems:
                        print(problem)
                else:
                    print(f"  ok    {prefix}: {baseline_calendar} vs {calendar}")
            for stream, reason in sorted(waivers.items()):
                print(f"  waived {prefix}: {stream} — {reason}")

    if failed:
        print(f"\ncadence-parity: {failed} comparison(s) failed.")
        print("A pack rule is producing different annual economics on different")
        print("calendars, which means a period-length assumption is still baked in.")
        return 1

    print(f"\ncadence-parity: OK ({checked} cross-calendar comparisons)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
