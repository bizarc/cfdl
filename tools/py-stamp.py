#!/usr/bin/env python3
"""Source stamp for the locally-installed Python extension.

`cfdl_sdk` is half Python and half a compiled Rust extension (`_native`). The
Python half is installed editable, so it tracks the working tree. The compiled
half does not: it is rebuilt only when someone runs `make py-develop`, and
nothing said when it had gone stale.

So it went stale, and stayed stale across four releases. `make notebooks-render`
failed with

    E4004_MISSING_PACK: unknown variant `terms_mutually_exclusive`

which reads like a broken pack and is nothing of the sort — the pack was fine
and the extension predated the commit that added that validation kind. The
error names the symptom in the wrong crate, which is the worst kind, and there
was no signal at all until something happened to exercise the changed code
path.

This is the same shape as site/scripts/wasm-stamp.mjs, and for the same reason:
a version check cannot see a change that ships without a version bump, and the
commit that broke this one did not bump anything. A hash over the source bytes
needs neither git history nor a version.

`packs/` is in the input set because crates/cfdl-pack `include_str!`s every
pack TOML at compile time — editing a lowering rule changes the extension with
no Rust source change at all. That is precisely the case that broke.

The stamp lives beside the built `.so` and is not committed: it describes one
machine's build, and a fresh clone correctly has none, so the check fails with
the remedy rather than passing vacuously.

Usage:
  python3 tools/py-stamp.py --write    (called by `make py-develop`)
  python3 tools/py-stamp.py --check    (called before rendering notebooks)
"""

from __future__ import annotations

import hashlib
import pathlib
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
STAMP = REPO_ROOT / "python" / "cfdl_sdk" / ".build-stamp"

# Everything whose bytes end up inside the compiled extension.
INPUTS = [
    "crates/cfdl-py",
    "crates/cfdl-compile",
    "crates/cfdl-engine",
    "crates/cfdl-metrics",
    "crates/cfdl-pack",
    "crates/cfdl-calc",
    "crates/cfdl-expr",
    "crates/cfdl-parser",
    "crates/cfdl-lexer",
    "crates/cfdl-resolver",
    "crates/cfdl-validate",
    "packs",
    "Cargo.toml",
]

SKIP_DIRS = {"target", "node_modules", ".git", "__pycache__"}
SKIP_NAMES = {".DS_Store", ".build-stamp"}


def _walk(path: pathlib.Path, out: list[pathlib.Path]) -> None:
    if not path.exists():
        return  # a declared input that does not exist is simply not hashed
    if path.is_file():
        if path.name not in SKIP_NAMES:
            out.append(path)
        return
    if not path.is_dir():
        return
    for entry in sorted(path.iterdir()):
        if entry.name in SKIP_DIRS:
            continue
        _walk(entry, out)


def digest() -> str:
    files: list[pathlib.Path] = []
    for rel in sorted(INPUTS):
        _walk(REPO_ROOT / rel, files)
    h = hashlib.sha256()
    for path in sorted(files):
        # Hash the path too, so a rename is a change.
        h.update(str(path.relative_to(REPO_ROOT)).encode())
        h.update(b"\0")
        h.update(path.read_bytes())
        h.update(b"\0")
    return h.hexdigest()


def main() -> int:
    mode = sys.argv[1] if len(sys.argv) > 1 else "--check"
    current = digest()

    if mode == "--write":
        STAMP.parent.mkdir(parents=True, exist_ok=True)
        STAMP.write_text(current + "\n")
        print(f"py-stamp: wrote {STAMP.relative_to(REPO_ROOT)} ({current[:12]}…)")
        return 0

    if mode != "--check":
        print(f"py-stamp: unknown mode {mode!r} (expected --write or --check)", file=sys.stderr)
        return 2

    if not STAMP.exists():
        print(
            "py-stamp: the compiled cfdl_sdk extension has never been stamped on this\n"
            "          machine, so it cannot be trusted to match the working tree.\n\n"
            "  Build and stamp it:\n"
            "    make py-develop\n",
            file=sys.stderr,
        )
        return 1

    stamped = STAMP.read_text().strip()
    if stamped != current:
        print(
            "py-stamp: engine or pack sources changed since the cfdl_sdk extension\n"
            "          was built, so it is running older code than the working tree.\n\n"
            f"  extension built from : {stamped[:12]}…\n"
            f"  sources now hash     : {current[:12]}…\n\n"
            "  Rebuild it:\n"
            "    make py-develop\n",
            file=sys.stderr,
        )
        return 1

    print(f"py-stamp: OK (extension built from the current sources, {current[:12]}…)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
