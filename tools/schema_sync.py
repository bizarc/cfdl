"""Propagate a JSON schema to its two copies, in one canonical form.

Three files hold one contract: the source under `docs/schemas/`, a byte-identical
mirror under `site/public/schemas/` that the site serves, and a fenced block
embedded in the docs page a human actually opens. `check-ir-schema.py` and
`check-results-schema.py` both assert the three agree — but neither could make
them agree, so keeping them in step meant pasting the same text into three
places by hand.

That is not a theoretical risk. The results schema drifted for four releases
and every one of the 67 committed goldens violated it; `docs/06` was the copy
that fell furthest behind, because it is the one a paste is most likely to miss.
A gate that can only say "these differ" leaves the expensive half of the job to
whoever is least equipped to do it reliably.

The canonical form is `json.dumps(schema, indent=2, ensure_ascii=False)` plus a
trailing newline. `ensure_ascii` matters: the descriptions contain typographic
punctuation, and the default would escape it into `\\u2019`, which reproduces
neither committed file.

Used by the `--write` mode of both gates. Never called during a check run.
"""

from __future__ import annotations

import json
import pathlib

FENCE_OPEN = "```json"
FENCE_CLOSE = "```"


def canonical(schema: object) -> str:
    """The one serialisation both committed copies are written in."""
    return json.dumps(schema, indent=2, ensure_ascii=False) + "\n"


def embed_in_doc(doc_text: str, schema: object) -> str:
    """Replace the doc's single fenced JSON block, preserving all prose.

    Raises rather than guessing if the page does not hold exactly one block:
    rewriting the wrong fence, or appending to a page that has none, would
    corrupt a hand-written document to fix a generated one.
    """
    if doc_text.count(FENCE_OPEN) != 1:
        raise ValueError(
            f"expected exactly one {FENCE_OPEN} block, found {doc_text.count(FENCE_OPEN)}"
        )
    prefix, rest = doc_text.split(FENCE_OPEN, 1)
    if FENCE_CLOSE not in rest:
        raise ValueError(f"{FENCE_OPEN} block is never closed")
    _, suffix = rest.rsplit(FENCE_CLOSE, 1)
    return f"{prefix}{FENCE_OPEN}\n{canonical(schema)}{FENCE_CLOSE}{suffix}"


def sync(
    schema_path: pathlib.Path,
    mirror_path: pathlib.Path,
    doc_path: pathlib.Path,
    repo_root: pathlib.Path,
) -> list[str]:
    """Rewrite the mirror and the doc from the source schema.

    Returns the repo-relative paths actually changed, so a caller can report
    "already in sync" rather than claiming a write it did not make. The source
    is read and never written — it is the thing being propagated.
    """
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    changed: list[str] = []

    source_text = canonical(schema)
    if schema_path.read_text(encoding="utf-8") != source_text:
        schema_path.write_text(source_text, encoding="utf-8")
        changed.append(str(schema_path.relative_to(repo_root)))

    if mirror_path.exists() and mirror_path.read_text(encoding="utf-8") != source_text:
        mirror_path.write_text(source_text, encoding="utf-8")
        changed.append(str(mirror_path.relative_to(repo_root)))

    if doc_path.exists():
        doc_text = doc_path.read_text(encoding="utf-8")
        rewritten = embed_in_doc(doc_text, schema)
        if rewritten != doc_text:
            doc_path.write_text(rewritten, encoding="utf-8")
            changed.append(str(doc_path.relative_to(repo_root)))

    return changed
