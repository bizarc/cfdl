#!/usr/bin/env python3
"""Every notebook cell carries the `id` its own format version requires.

The four notebooks under `examples/notebooks/` declare `nbformat 4.5`, in which
a cell `id` is REQUIRED, and all 64 of their cells were missing one. They were
invalid against the version they claimed, and nothing said so: nbformat repairs
the omission in memory on every read, and prints a warning that the repair is
going away.

    MissingIDFieldWarning: Cell is missing an id field, this will become a
    hard error in future nbformat versions.

That is a build failure scheduled for a day when nobody changed anything. The
trigger is a dependency upgrade rather than a commit, which is the expensive
kind to diagnose — `make verify` and CI go red together, and the diff that
explains them is in someone else's repository.

IDS ARE ASSIGNED DETERMINISTICALLY, from the notebook's name and the cell's
position. `nbformat.validator.normalize()` would do this job with RANDOM ids,
which works once and then regenerates a different answer every time anyone runs
it — 64 changed lines for no change in meaning. A derived id makes a
regeneration a no-op, which is the same argument `round_amount` settles for
published numbers and the workflow runtime settles by refusing `Math.random()`.

AN ID ALREADY PRESENT IS NEVER REWRITTEN. An id is a cell's identity, so the
committed value wins over anything this would compute: inserting a cell
assigns an id to the new cell alone, rather than renumbering every cell below
it. The derivation is therefore only ever used to FILL A GAP.

The renderer does not round-trip the notebooks — `tools/render-notebooks.py`
reads them, inserts its setup cell in memory, executes, and writes only the
pages, the images and the stamp. So a committed id is durable, and ids never
reach the rendered output, which names images by index. Fixing this changes the
notebooks and `.render-stamp`, and no published page.

Usage: python3 tools/check-notebook-ids.py            # check
       python3 tools/check-notebook-ids.py --write    # assign the missing ids
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
NOTEBOOKS = REPO_ROOT / "examples" / "notebooks"

# nbformat's own constraint on a cell id: 1 to 64 characters of this alphabet.
VALID_ID = re.compile(r"^[a-zA-Z0-9\-_]{1,64}$")

# `id` became required in nbformat 4.5. A notebook declaring less than that is
# not in scope: its cells are correct without one.
ID_REQUIRED_FROM = (4, 5)


def derived_id(stem: str, index: int, taken: set[str]) -> str:
    """A stable id for one cell, long enough to be unique in its notebook."""
    digest = hashlib.sha1(f"{stem}:{index}".encode()).hexdigest()
    # Eight hex characters is 32 bits against a few dozen cells. Widen rather
    # than collide, so the function is total instead of nearly total.
    for length in range(8, len(digest) + 1):
        candidate = digest[:length]
        if candidate not in taken:
            return candidate
    raise SystemExit(f"could not derive a unique id for {stem} cell {index}")


def main() -> int:
    write = "--write" in sys.argv[1:]
    notebooks = sorted(NOTEBOOKS.glob("*.ipynb"))
    if not notebooks:
        print(f"check-notebook-ids: no notebooks under {NOTEBOOKS.relative_to(REPO_ROOT)}")
        return 0

    missing: list[str] = []
    invalid: list[str] = []
    duplicated: list[str] = []
    written = 0
    checked = 0

    for path in notebooks:
        rel = path.relative_to(REPO_ROOT)
        document = json.loads(path.read_text(encoding="utf-8"))
        version = (document.get("nbformat", 0), document.get("nbformat_minor", 0))
        if version < ID_REQUIRED_FROM:
            continue

        cells = document.get("cells", [])
        checked += len(cells)
        taken = {c["id"] for c in cells if isinstance(c.get("id"), str)}
        changed = False

        for index, cell in enumerate(cells):
            current = cell.get("id")
            if isinstance(current, str) and VALID_ID.match(current):
                continue
            if isinstance(current, str):
                invalid.append(f"{rel} cell {index}: id '{current}' is not a valid id")
                continue
            if not write:
                missing.append(f"{rel} cell {index}")
                continue
            assigned = derived_id(path.stem, index, taken)
            taken.add(assigned)
            # `id` sorts first in the serialised cell, which is where nbformat
            # writes it and where a reviewer expects to find it.
            cells[index] = {"id": assigned, **cell}
            changed = True
            written += 1

        ids = [c.get("id") for c in cells if isinstance(c.get("id"), str)]
        for duplicate in {i for i in ids if ids.count(i) > 1}:
            duplicated.append(f"{rel}: id '{duplicate}' is used by more than one cell")

        if changed:
            document["cells"] = cells
            # Trailing newline and 1-space indent are what nbformat writes, so
            # a later `nbformat.write` produces no incidental diff.
            path.write_text(
                json.dumps(document, indent=1, ensure_ascii=False) + "\n",
                encoding="utf-8",
            )

    problems = missing + invalid + duplicated
    if write:
        print(
            f"check-notebook-ids: assigned {written} id(s) across "
            f"{len(notebooks)} notebook(s)"
        )
        if invalid or duplicated:
            for problem in invalid + duplicated:
                print(f"  {problem}")
            return 1
        return 0

    if problems:
        print("check-notebook-ids: FAIL")
        for problem in problems:
            print(f"  {problem}")
        print()
        print("  A cell id is required from nbformat 4.5, and nbformat will stop")
        print("  repairing the omission. Assign them:")
        print("    python3 tools/check-notebook-ids.py --write")
        return 1

    print(
        f"check-notebook-ids: OK ({checked} cells across {len(notebooks)} "
        "notebooks carry a unique id)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
