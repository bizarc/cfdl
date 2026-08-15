#!/usr/bin/env python3
"""Prove that documentation examples do what their prose claims.

The benchmark models are held to an independent reference on every commit.
The examples in the pack guides — which is what a reader actually meets first —
were held to nothing, and it showed: the CRE quick start advertised
"lease-by-lease with recoveries and rollover", contained one tenant, and
computed recoveries of exactly zero against a property that declared no
expenses. It compiled and ran, so nothing objected.

This compiles and runs every complete model in the documentation and fails on:

  * a model that does not compile or run;
  * a stream whose total is zero — a feature shown but not exercised, which
    reads to a newcomer as a broken feature;
  * a stream whose total contradicts its declared direction — an `outflow`
    that nets positive is the pmt sign error that once reported debt service
    as income and overstated an example's NPV tenfold;
  * a stream that pays in more periods than its schedule declares — a
    quarterly schedule that paid every month, twelve times a year instead of
    four, went unnoticed because nothing counted. An upper bound rather than
    equality, since an amount expression may legitimately be zero in some
    periods and a conditional can only reduce the count.

Both stream checks can be waived per-stream where the zero or the sign is the
point being made, using the repo's existing escape-hatch convention:

    // examples-allow: <stream substring> — <reason>

Usage:
    python3 tools/check-doc-examples.py            # check every source
    python3 tools/check-doc-examples.py --verbose  # list every stream total
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import re
import subprocess
import sys
import datetime
import tempfile

# These tools print prose. A Windows console defaults to cp1252, which
# cannot encode every character the check names use, so pin stdout to UTF-8.
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
# Windows names the binary cfdl.exe; everywhere else it is bare `cfdl`.
CLI = REPO_ROOT / "target" / "debug" / ("cfdl.exe" if os.name == "nt" else "cfdl")
PACKS = REPO_ROOT / "packs"

# Documentation whose fenced cfdl blocks are meant to be complete models.
# The pack guides, plus every AUTHORED page on the documentation site.
#
# Generated pages are excluded deliberately. Their code comes from real models
# under `examples/` and `benchmarks/`, which the golden and benchmark suites
# already run — checking it again here would test the same bytes twice and
# report a failure against a page rather than against the model that owns it.
#
# What this reaches is prose an author typed. Two snippets in the language guide
# were wrong on the day it was written — `active_when =` instead of `active
# when`, and a curve fence missing its interpolation mode — and nothing but a
# person running them would have said so.
def _authored_site_pages() -> list[pathlib.Path]:
    docs = REPO_ROOT / "site" / "content" / "docs"
    if not docs.exists():
        return []
    pages = []
    for page in sorted(docs.rglob("*.md")):
        head = page.read_text(encoding="utf-8")[:400]
        if "generated: none" in head or "generated: regions" in head:
            pages.append(page)
    return pages


SOURCES = (
    sorted(REPO_ROOT.glob("packs/*/README.md"))
    + _authored_site_pages()
    # The training site's chapters: authored prose whose fenced models a
    # learner is invited to run, so they are held to the same standard as the
    # pack guides — complete models compile and run, fragments parse.
    + sorted(REPO_ROOT.glob("learn/content/chapters/*.mdx"))
    # The landing page's model, which is not markdown and so was reached by
    # nothing. It sat on the front page with `every monthly` and an untyped
    # entity — neither valid — for as long as it took someone to try it.
    + [p for p in [REPO_ROOT / "site/components/landing/hero-demo-data.ts"] if p.exists()]
)

FENCE = re.compile(r"```cfdl\n(.*?)```", re.S)
# A model held in a TypeScript template literal, e.g. `export const heroModel =
# \`version 0.1 … \`;`. Same contract as a fence: it must start with `version`.
TEMPLATE_MODEL = re.compile(r"=\s*`(version\s.*?)`;", re.S)
ALLOW = re.compile(r"//\s*examples-allow:\s*(\S+)")
USE_PACK = re.compile(r'^\s*use\s+pack\s+"([^"]+)"', re.M)


def complete_models(path: pathlib.Path) -> list[tuple[int, str]]:
    """Fenced cfdl blocks that are whole models, with their line numbers.

    A fragment — the single contract a "Recipes" section shows — has no
    `version` line and cannot be compiled on its own.
    """
    text = path.read_text(encoding="utf-8")
    found = []
    pattern = TEMPLATE_MODEL if path.suffix == ".ts" else FENCE
    for match in pattern.finditer(text):
        body = match.group(1)
        if not re.match(r"\s*version\s", body):
            continue
        # A block that imports other files is one FILE of a multi-file model,
        # not a model. It cannot compile alone by construction, and the pages
        # showing one are teaching exactly that. `examples/cre_multi_file` is
        # the runnable version, and the golden suite covers it.
        if re.search(r"^\s*import\s", body, re.M):
            continue
        line = text[: match.start()].count("\n") + 1
        found.append((line, body))
    return found


def fragments(path: pathlib.Path) -> list[tuple[int, str]]:
    """Fenced cfdl blocks that are NOT whole models.

    Most snippets on the site are fragments — one stream, one contract, one
    schedule line. They cannot be compiled, because they reference entities and
    packs they do not declare, so nothing was checking them at all: `every
    monthly` sat in four guides, and three pack guides declared entities in a
    vocabulary the language had moved off.

    Parsing is the part that does not need the rest of the model. It catches
    every syntax error and says nothing about resolution, which is the correct
    division for a snippet.
    """
    if path.suffix == ".ts":
        return []
    text = path.read_text(encoding="utf-8")
    found = []
    for match in FENCE.finditer(text):
        body = match.group(1)
        if re.match(r"\s*version\s", body):
            continue
        found.append((text[: match.start()].count("\n") + 1, body))
    return found


# The four families an entity can be declared in. A snippet naming anything
# else still parses — the untyped form is open — but it teaches a vocabulary
# the language does not use, which is how `entity real_estate tower` outlived
# the ontology it predates.
ENTITY_DECL = re.compile(r"^\s*entity\s+(\w+)\s+\w+", re.M)
FAMILIES = {"asset", "party", "contract", "reference"}


def hostable(source: str) -> list[str]:
    """One parseable unit per clause the snippet shows.

    Some snippets are smaller than a declaration — a bare `schedule …` line, a
    bare `amount = …`. Those are the clauses a reader is most likely to copy,
    and `every monthly` lived in exactly that kind of snippet, so they are put
    in the smallest host that makes them a declaration rather than skipped.

    A snippet that already starts with a top-level keyword is passed through as
    written.
    """
    body = source.strip()
    if not body:
        return []
    if re.match(r"(entity|stream|contract|option|event|state|curve|assume|use|time|statement|scenario)\b", body):
        return [body]

    def host(schedule: str, amount: str) -> str:
        return (
            "stream doc.snippet on entity asset.subject inflow currency USD {\n"
            f"  {schedule}\n  {amount}\n}}"
        )

    units = []
    for clause in body.splitlines():
        clause = clause.strip()
        if not clause or clause.startswith("//"):
            continue
        if clause.startswith("schedule "):
            units.append(host(clause, "amount = 1"))
        elif clause.startswith("amount ="):
            units.append(host("schedule every month from 2026-01 to 2026-12", clause))
    return units


def check_fragment(path: pathlib.Path, line: int, source: str) -> list[str]:
    rel = f"{path.relative_to(REPO_ROOT)}:{line}"
    problems = []
    for unit in hostable(source):
        with tempfile.TemporaryDirectory() as tmp:
            workdir = pathlib.Path(tmp)
            (workdir / "model.cfdl").write_text(unit, encoding="utf-8")
            done = subprocess.run(
                [str(CLI), "parse", str(workdir)],
                capture_output=True, text=True, encoding="utf-8",
            )
            if done.returncode != 0:
                problems.append(
                    f"{rel} does not parse:\n{indent(done.stdout or done.stderr)}"
                )
    for match in ENTITY_DECL.finditer(source):
        family = match.group(1)
        if family not in FAMILIES:
            problems.append(
                f"{rel} declares `entity {family} …`; the families are "
                + ", ".join(sorted(FAMILIES))
            )
    return problems


def run_model(source: str, workdir: pathlib.Path) -> tuple[dict, dict]:
    """Compile and run one model; returns (ir, results)."""
    (workdir / "model.cfdl").write_text(source, encoding="utf-8")
    (workdir / "run.json").write_text(
        json.dumps({"deterministic": {"annual_discount_rate": 0.08}}),
        encoding="utf-8",
    )
    ir_path = workdir / "ir.json"
    res_path = workdir / "results.json"

    compile_cmd = [str(CLI), "compile", str(workdir), "--packs", str(PACKS),
                   "--out", str(ir_path)]
    done = subprocess.run(compile_cmd, capture_output=True, text=True, encoding="utf-8")
    if done.returncode != 0:
        raise RuntimeError(f"compile failed:\n{indent(done.stderr or done.stdout)}")

    run_cmd = [str(CLI), "run", str(ir_path), "--config", str(workdir / "run.json"),
               "--packs", str(PACKS), "--out", str(res_path)]
    pack = USE_PACK.search(source)
    if pack:
        run_cmd += ["--pack", pack.group(1)]
    done = subprocess.run(run_cmd, capture_output=True, text=True, encoding="utf-8")
    if done.returncode != 0:
        raise RuntimeError(f"run failed:\n{indent(done.stderr or done.stdout)}")

    return json.loads(ir_path.read_text(encoding="utf-8")), json.loads(res_path.read_text(encoding="utf-8"))


def indent(text: str) -> str:
    return "\n".join(f"      {line}" for line in text.strip().splitlines()[:6])


def stream_totals(results: dict) -> dict[str, float]:
    """Total per CASH series.

    A series entry is either a Money object or a bare number. Only the first is
    cash: a FIELD publishes a dimensionless balance, index or factor, which has
    no currency and must not be summed as though it did. This check exists to
    prove every documented stream carries flow, so a field has nothing to say
    to it.

    A field is recognized by its shape rather than a prefix: it publishes under
    the entity that owns it — `asset.trust.available_funds` — because `state.`
    named a model-level state and a field is not one.
    """
    series = results["deterministic"]["series"]
    return {
        name: sum(
            point["amount"] for point in block["values"] if isinstance(point, dict)
        )
        for name, block in series.items()
        # `domain.` is excluded for a different reason than `state.`, and the
        # difference is the point of this gate. A `stream.` appears because the
        # example's author wrote it, so one that totals zero really is a feature
        # the example claims and never exercises. A `domain.` subtotal is
        # emitted by the PACK for every model that uses it, whether or not that
        # model has anything to put in it — a debt-service fold is zero in a
        # debt-free example, which is the model being described accurately
        # rather than the example overclaiming.
        if name != "model.net_cash_flow"
        and not name.startswith("state.")
        and not name.startswith("domain.")
        # A field: `<family>.<entity>.<field>`, three segments and no prefix
        # this gate knows. A stream is `stream.<name>`, so the two never
        # collide.
        and not (name.count(".") == 2 and not name.startswith("stream."))
    }


def _parse_date(value: str) -> datetime.date | None:
    for fmt in ("%Y-%m-%d", "%Y-%m"):
        try:
            return datetime.datetime.strptime(value, fmt).date()
        except ValueError:
            continue
    return None


def _add_months(d: datetime.date, months: int) -> datetime.date:
    total = d.year * 12 + (d.month - 1) + months
    year, month = divmod(total, 12)
    month += 1
    day = min(d.day, [31, 29 if year % 4 == 0 and (year % 100 or year % 400 == 0) else 28,
                      31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1])
    return datetime.date(year, month, day)


def scheduled_occurrences(schedule: dict) -> int | None:
    """How many payments the schedule declares, or None if not countable.

    Only recurring schedules with a resolvable range are counted; `OnDate` and
    phase-bounded schedules are left alone.
    """
    if schedule.get("kind") != "Every":
        return None
    start, end = _parse_date(schedule.get("from") or ""), _parse_date(schedule.get("to") or "")
    if not start or not end or end < start:
        return None

    every = schedule.get("every") or "monthly"
    step_months = {"monthly": 1, "quarterly": 3, "annual": 12}.get(every)
    step_days = {"daily": 1, "weekly": 7}.get(every)
    if step_months is None and step_days is None:
        return None

    count, cursor = 0, start
    while cursor <= end and count < 100_000:
        count += 1
        cursor = (_add_months(cursor, step_months) if step_months
                  else cursor + datetime.timedelta(days=step_days))
    return count


def schedules(ir: dict) -> dict[str, dict]:
    return {s["name"]: s.get("schedule", {}) for s in ir.get("streams", [])}


def nonzero_period_counts(results: dict) -> dict[str, int]:
    """Periods carrying cash, per CASH series. See `stream_totals` on why
    `state.` series are excluded rather than counted as zero-flow streams."""
    series = results["deterministic"]["series"]
    return {
        name: sum(
            1
            for point in block["values"]
            if isinstance(point, dict) and abs(point["amount"]) > 0.005
        )
        for name, block in series.items()
        # `domain.` is excluded for a different reason than `state.`, and the
        # difference is the point of this gate. A `stream.` appears because the
        # example's author wrote it, so one that totals zero really is a feature
        # the example claims and never exercises. A `domain.` subtotal is
        # emitted by the PACK for every model that uses it, whether or not that
        # model has anything to put in it — a debt-service fold is zero in a
        # debt-free example, which is the model being described accurately
        # rather than the example overclaiming.
        if name != "model.net_cash_flow"
        and not name.startswith("state.")
        and not name.startswith("domain.")
        # A field: `<family>.<entity>.<field>`, three segments and no prefix
        # this gate knows. A stream is `stream.<name>`, so the two never
        # collide.
        and not (name.count(".") == 2 and not name.startswith("stream."))
    }


def directions(ir: dict) -> dict[str, str]:
    """Declared direction per stream name, from the IR."""
    return {s["name"]: s.get("direction", "") for s in ir.get("streams", [])}


def check(path: pathlib.Path, line: int, source: str, verbose: bool) -> list[str]:
    waived = ALLOW.findall(source)
    problems: list[str] = []

    with tempfile.TemporaryDirectory() as tmp:
        try:
            ir, results = run_model(source, pathlib.Path(tmp))
        except RuntimeError as err:
            return [f"{path.relative_to(REPO_ROOT)}:{line}: {err}"]

    totals = stream_totals(results)
    if not totals:
        return [f"{path.relative_to(REPO_ROOT)}:{line}: model emits no streams"]

    declared = directions(ir)
    declared_schedules = schedules(ir)
    nonzero = nonzero_period_counts(results)
    for name, total in sorted(totals.items()):
        if verbose:
            print(f"      {name:52} {total:16,.2f}")
        if any(token in name for token in waived):
            continue

        if abs(total) < 0.005:
            problems.append(
                f"{path.relative_to(REPO_ROOT)}:{line}: stream `{name}` totals zero — "
                "the example shows a feature it never exercises"
            )
            continue

        # `stream.<name>` in results maps to `<name>` in the IR.
        bare = name[len("stream.") :] if name.startswith("stream.") else name
        direction = declared.get(bare, "")
        if direction == "outflow" and total > 0:
            problems.append(
                f"{path.relative_to(REPO_ROOT)}:{line}: stream `{name}` is declared "
                f"`outflow` but totals +{total:,.2f} — check the sign of its amount"
            )
        elif direction == "inflow" and total < 0:
            problems.append(
                f"{path.relative_to(REPO_ROOT)}:{line}: stream `{name}` is declared "
                f"`inflow` but totals {total:,.2f} — check the sign of its amount"
            )

        # A stream must not pay more often than its schedule declares. This is
        # the check that would have caught the interval being discarded: a
        # quarterly schedule paid in every month, twelve times a year instead
        # of four, and nothing objected. An upper bound rather than equality,
        # because an amount expression may legitimately evaluate to zero in
        # some periods — a conditional can only reduce the count.
        expected = scheduled_occurrences(declared_schedules.get(bare, {}))
        if expected is not None and nonzero.get(name, 0) > expected:
            problems.append(
                f"{path.relative_to(REPO_ROOT)}:{line}: stream `{name}` pays in "
                f"{nonzero[name]} periods but its schedule declares {expected} "
                f"(every {declared_schedules[bare].get('every')}) — the interval is being ignored"
            )

    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verbose", action="store_true",
                        help="print every stream total as it is checked")
    args = parser.parse_args()

    if not CLI.exists():
        print(f"check-doc-examples: {CLI} not found — run `cargo build -p cfdl-cli`",
              file=sys.stderr)
        return 1

    problems: list[str] = []
    checked = 0
    snippets = 0
    for path in SOURCES:
        for line, source in complete_models(path):
            checked += 1
            if args.verbose:
                print(f"  {path.relative_to(REPO_ROOT)}:{line}")
            problems += check(path, line, source, args.verbose)
        for line, source in fragments(path):
            snippets += 1
            if args.verbose:
                print(f"  {path.relative_to(REPO_ROOT)}:{line} (snippet)")
            problems += check_fragment(path, line, source)

    if problems:
        print(f"\ncheck-doc-examples: {len(problems)} problem(s) in {checked} example(s):\n",
              file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        print(
            "\nIf a zero or a sign is deliberate, waive that stream in the model:\n"
            "  // examples-allow: <stream substring> — <reason>",
            file=sys.stderr,
        )
        return 1

    print(f"check-doc-examples: OK ({checked} documentation examples compile, run, "
          f"and exercise every stream they declare; {snippets} snippets parse)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
