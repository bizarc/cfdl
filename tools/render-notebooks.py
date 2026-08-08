#!/usr/bin/env python3
"""Execute the example notebooks and render them as documentation pages.

The notebooks are the only place the Python SDK is shown end to end, but a
reader could not see a single output without cloning the repo and building the
Rust extension. This publishes them.

Rendering happens here rather than at site-build time because neither the site
CI runner nor Vercel has Python or Rust. The output is committed and guarded
against going stale by site/scripts/check-notebooks-fresh.mjs, the same
arrangement the wasm bundle already uses.

The source notebooks stay output-stripped; everything written here is a derived
artifact.

Usage:
    python3 tools/render-notebooks.py            # execute and write
    python3 tools/render-notebooks.py --check    # fail if anything would change
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys

# These tools print prose. A Windows console defaults to cp1252, which
# cannot encode every character the check names use, so pin stdout to UTF-8.
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
NOTEBOOK_DIR = REPO_ROOT / "examples" / "notebooks"
PAGE_DIR = REPO_ROOT / "site" / "content" / "docs" / "notebooks"
IMAGE_DIR = REPO_ROOT / "site" / "public" / "notebooks"
REPO_BLOB = "https://github.com/bizarc/cfdl/blob/main"

# Per-notebook page metadata. Keyed by notebook stem so a renamed notebook
# fails loudly here rather than silently publishing under a changed slug.
PAGES = {
    "01_energy_solar_microgrid": {
        "id": "notebook-energy",
        "slug": "energy-solar-microgrid",
        "pack": "energy",
    },
    "02_cre_office_acquisition": {
        "id": "notebook-cre",
        "slug": "cre-office-acquisition",
        "pack": "cre",
    },
    "03_credit_loan_pool": {
        "id": "notebook-credit",
        "slug": "credit-loan-pool",
        "pack": "credit",
    },
    "04_opco_lbo": {
        "id": "notebook-opco",
        "slug": "opco-lbo",
        "pack": "opco",
    },
}

# ANSI colour codes leak into tracebacks and some reprs; they are noise in a
# static page.
ANSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")


def strip_ansi(text: str) -> str:
    return ANSI.sub("", text)


SETUP_MARKER = "cfdl_render_setup"


def execute(path: pathlib.Path) -> dict:
    """Run a notebook and return the executed document.

    Executed from the repo root so the notebook's own root resolution finds the
    checkout.

    A setup cell enabling the inline matplotlib backend is prepended at
    execution time and skipped when rendering. Without it the figure is never
    emitted as display data and the plot cell publishes as the text
    `<Axes: ...>` instead of a chart. Injecting it here rather than editing the
    notebooks keeps the sources exactly as a reader would run them. Note this
    deliberately does not honour MPLBACKEND=Agg, which CI sets — Agg suppresses
    figure capture, which is harmless when only checking for exceptions but
    would silently drop every chart from these pages.
    """
    import nbformat
    from nbclient import NotebookClient

    notebook = nbformat.read(path, as_version=4)
    os.environ.pop("MPLBACKEND", None)

    setup = nbformat.v4.new_code_cell("%matplotlib inline")
    setup.metadata[SETUP_MARKER] = True
    notebook.cells.insert(0, setup)

    client = NotebookClient(
        notebook,
        timeout=600,
        kernel_name="python3",
        resources={"metadata": {"path": str(REPO_ROOT)}},
    )
    client.execute()
    return notebook


def fence(text: str, lang: str = "") -> list[str]:
    """Wrap text in a fenced block, widening the fence if the text contains one."""
    body = strip_ansi(text).rstrip("\n")
    if not body.strip():
        return []
    ticks = "```"
    while ticks in body:
        ticks += "`"
    return [f"{ticks}{lang}", body, ticks, ""]


def render_outputs(cell: dict, images: dict[str, bytes], slug: str, index: int) -> list[str]:
    """Render a code cell's outputs as plain Markdown.

    Deliberately no raw HTML: the docs route compiles content with
    `format: "md"`, so a pandas `_repr_html_` table would be emitted as
    escaped text. The `text/plain` repr renders faithfully in a code fence and
    additionally picks up the site's copy button.
    """
    lines: list[str] = []
    for position, output in enumerate(cell.get("outputs", [])):
        kind = output.get("output_type")

        if kind == "stream":
            lines += fence("".join(output.get("text", [])))
            continue

        if kind == "error":
            # Execution already failed the build; render it rather than hide it.
            lines += fence("\n".join(output.get("traceback", [])))
            continue

        data = output.get("data", {})
        if "image/png" in data:
            payload = data["image/png"]
            raw = base64.b64decode(payload if isinstance(payload, str) else "".join(payload))
            name = f"cell-{index:02d}-{position}.png"
            images[name] = raw
            alt = "Chart produced by the preceding cell"
            lines += [f"![{alt}](/notebooks/{slug}/{name})", ""]
            continue

        if "text/plain" in data:
            payload = data["text/plain"]
            text = payload if isinstance(payload, str) else "".join(payload)
            lines += fence(text)

    return lines


def render(path: pathlib.Path, notebook: dict) -> tuple[str, dict[str, bytes], str]:
    meta = PAGES[path.stem]
    slug = meta["slug"]
    images: dict[str, bytes] = {}
    body: list[str] = []
    title = None

    # Indices number the author's cells only, so injecting the setup cell
    # cannot renumber every committed image file.
    cells = [c for c in notebook["cells"] if not c.get("metadata", {}).get(SETUP_MARKER)]

    for index, cell in enumerate(cells):
        source = "".join(cell["source"]).rstrip("\n")

        if cell["cell_type"] == "markdown":
            # The leading H1 supplies the frontmatter title and is re-emitted at
            # the top of the page below. sync-content.mjs strips it and stops
            # there, and because the docs route never renders the frontmatter
            # title as a heading, every page it generates ships without an h1.
            if title is None and source.startswith("# "):
                head, _, rest = source.partition("\n")
                title = head[2:].strip()
                source = rest.strip()
            if source:
                body += [source, ""]
            continue

        if cell["cell_type"] == "code":
            if source:
                body += fence(source, "python")
            body += render_outputs(cell, images, slug, index)

    if title is None:
        raise SystemExit(f"{path.name}: no leading H1 to use as the page title")

    page = [
        "---",
        f"id: {meta['id']}",
        f'title: "{title}"',
        f'slug: "/docs/notebooks/{slug}"',
        f"source: examples/notebooks/{path.name}",
        # Declares who owns the bytes, for the site's manifest check. This
        # script does, wholly — the page is re-rendered from the notebook.
        "generated: full",
        "---",
        "",
        f"# {title}",
        "",
        "> Outputs below are real: the notebook runs against the "
        f"`{meta['pack']}` pack's benchmark model, which CFDL validates "
        "against an independent reference. To run it yourself, see "
        "[the Python SDK guide](/docs/python-sdk).",
        "",
        *body,
    ]
    return "\n".join(page).rstrip("\n") + "\n", images, title


def index_page(toc: list[tuple[str, str, str]]) -> str:
    """The landing page behind the sidebar's single "Notebooks" entry."""
    lines = [
        "---",
        "id: notebooks",
        'title: "Notebooks"',
        'slug: "/docs/notebooks"',
        "generated: full",
        "---",
        "",
        "# Notebooks",
        "",
        "One worked notebook per domain pack, each walking a benchmark model "
        "through the Python SDK — compile, run, cash flows, metrics, and a "
        "what-if. The outputs and charts on these pages are real: each page "
        "is produced by executing its notebook against the released packs "
        "and engine.",
        "",
    ]
    for title, slug, _filename in toc:
        lines.append(f"- [{title}](/docs/notebooks/{slug})")
    lines += [
        "",
        "To run them yourself, see [the Python SDK guide](/docs/python-sdk).",
        "",
    ]
    return "\n".join(lines)


# Paths whose contents can change what a rendered notebook page says. Kept in
# step with SOURCE_PATHS in site/scripts/check-notebooks-fresh.mjs.
#
# `benchmarks` is NOT here as a whole directory. The notebooks read four
# specific benchmark models, and a case in another pack cannot change what they
# print — but a directory-wide input made every new benchmark demand a full
# notebook re-render, which is a false alarm on the action this repository takes
# most often. A gate that cries wolf gets disabled, so it is narrowed to the
# models actually read, discovered from the notebooks rather than listed here
# where the list would go stale.
STAMP_INPUTS = [
    "examples/notebooks", "packs", "python/cfdl_sdk",
    "crates/cfdl-py", "crates/cfdl-compile", "crates/cfdl-engine",
    "crates/cfdl-metrics", "crates/cfdl-pack", "crates/cfdl-calc",
    "crates/cfdl-parser", "crates/cfdl-lexer", "crates/cfdl-resolver",
    "crates/cfdl-validate", "tools/render-notebooks.py",
]

BENCHMARK_REF = re.compile(r"benchmarks/([a-z_]+)/([a-z_0-9]+)")


def benchmark_inputs(repo_root: pathlib.Path) -> list[str]:
    """The benchmark directories the notebooks actually read."""
    found = set()
    for notebook in sorted((repo_root / "examples" / "notebooks").glob("*.ipynb")):
        text = notebook.read_text(encoding="utf-8")
        for pack, case in BENCHMARK_REF.findall(text):
            if (repo_root / "benchmarks" / pack / case).is_dir():
                found.add(f"benchmarks/{pack}/{case}")
    return sorted(found)


def tracked_files(repo_root: pathlib.Path, rel: str) -> list[pathlib.Path]:
    """The GIT-TRACKED files under `rel`, sorted by path.

    Walking the working directory instead made the digest machine-dependent:
    a local `make py-develop` leaves a compiled extension and a build stamp
    inside `python/cfdl_sdk/` that no CI runner has, so the same commit hashed
    to two different values and the freshness gate failed on a current render.
    Tracked files are the same everywhere, which is what a committed stamp has
    to be comparable against.
    """
    out = subprocess.run(
        ["git", "-C", str(repo_root), "ls-files", "-z", "--", rel],
        capture_output=True,
        check=True,
    )
    names = [n for n in out.stdout.decode().split("\0") if n]
    return [repo_root / n for n in sorted(names)]


def write_render_stamp(repo_root: pathlib.Path) -> None:
    """Record what the pages were rendered against.

    The freshness guard used to require that a render produced a diff whenever
    an input changed. A compiler change that does not alter notebook output —
    a new diagnostic, say — then made the gate unsatisfiable: re-rendering
    yielded nothing to commit, so the gate stayed red. A stamp records that the
    render *ran* against these inputs, which is the thing actually being
    asserted.
    """
    digest = hashlib.sha256()
    for rel in sorted(STAMP_INPUTS + benchmark_inputs(repo_root)):
        for f in tracked_files(repo_root, rel):
            if not f.is_file():
                continue
            digest.update(str(f.relative_to(repo_root)).encode())
            digest.update(f.read_bytes())
    stamp = repo_root / "site" / "content" / "docs" / "notebooks" / ".render-stamp"
    stamp.write_text(digest.hexdigest() + "\n", encoding="utf-8")
    print(f"wrote {stamp.relative_to(repo_root)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail instead of writing when the rendered output would change",
    )
    args = parser.parse_args()

    notebooks = sorted(NOTEBOOK_DIR.glob("*.ipynb"))
    unknown = [p.stem for p in notebooks if p.stem not in PAGES]
    if unknown:
        raise SystemExit(
            f"No page metadata for {', '.join(unknown)} — add an entry to PAGES "
            "in tools/render-notebooks.py."
        )

    stale: list[str] = []

    def emit(target: pathlib.Path, payload: bytes) -> None:
        rel = target.relative_to(REPO_ROOT)
        if args.check:
            if not target.exists():
                stale.append(f"missing: {rel}")
            elif _digest(target.read_bytes()) != _digest(payload):
                stale.append(f"stale:   {rel}")
            return
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(payload)
        print(f"  wrote {rel}")

    toc: list[tuple[str, str, str]] = []  # (title, slug, notebook filename)
    for path in notebooks:
        print(f"executing {path.relative_to(REPO_ROOT)}", flush=True)
        page, images, title = render(path, execute(path))
        slug = PAGES[path.stem]["slug"]
        toc.append((title, slug, path.name))

        emit(PAGE_DIR / f"{slug}.md", page.encode())
        for name, payload in images.items():
            emit(IMAGE_DIR / slug / name, payload)

    emit(PAGE_DIR / "index.md", index_page(toc).encode())

    if stale:
        print("\nRendered notebook pages are out of date:", file=sys.stderr)
        for entry in stale:
            print(f"  {entry}", file=sys.stderr)
        print("\nRegenerate them:\n  make notebooks-render", file=sys.stderr)
        return 1

    write_render_stamp(REPO_ROOT)
    return 0


def _digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


if __name__ == "__main__":
    raise SystemExit(main())
