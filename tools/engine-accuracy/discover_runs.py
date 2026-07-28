#!/usr/bin/env python3
"""
Discover all example model runs for engine accuracy validation.
Outputs example_runs.json: list of { model_root, run_label, config_path, packs_dir }.
Run from repo root.
"""
import json
import os
from pathlib import Path
from typing import List, Optional, Tuple

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
EXAMPLES = REPO_ROOT / "examples"

# Model roots (relative to repo) that require --packs packs. All others are standalone.
PACKS_REQUIRED = frozenset({
    "examples/cre_developer",
    "examples/cre_lease_up",
    "examples/cre_phased",
    "examples/cre_multi_file",
    "examples/cre_development_with_financing",
    "examples/opco_basic",
    "examples/opco_with_growth",
    "examples/opco_multi_file",
    "examples/language_tutorial/simple_contract",
    "examples/language_tutorial/with_pack",
})


def model_roots():
    """Yield (model_root, slug) for every directory under examples that has model.cfdl."""
    for root, _dirs, files in os.walk(EXAMPLES):
        if "model.cfdl" not in files:
            continue
        root_path = Path(root)
        try:
            rel = root_path.relative_to(REPO_ROOT)
        except ValueError:
            continue
        model_root = str(rel)
        # Slug for output filenames: cre_developer, language_tutorial_minimal_model
        slug = model_root.replace("examples/", "", 1).replace("/", "_")
        yield model_root, slug


def run_configs(model_root: str) -> List[Tuple[str, Optional[str]]]:
    """
    Return list of (run_label, config_path or None) for this model_root.
    config_path is relative to repo root (e.g. examples/cre_developer/run.base.json).
    """
    dir_path = REPO_ROOT / model_root
    if not dir_path.is_dir():
        return []

    base = dir_path / "run.base.json"
    stress = dir_path / "run.stress.json"
    single = dir_path / "run.json"

    if base.exists() and stress.exists():
        out = [
            ("base", str(base.relative_to(REPO_ROOT))),
            ("stress", str(stress.relative_to(REPO_ROOT))),
        ]
        if single.exists():
            out.append(("default", str(single.relative_to(REPO_ROOT))))
        return out
    if single.exists():
        return [("default", str(single.relative_to(REPO_ROOT)))]
    return [("default", None)]


def main():
    runs = []
    for model_root, slug in sorted(model_roots()):
        needs_packs = model_root in PACKS_REQUIRED
        for run_label, config_path in run_configs(model_root):
            runs.append({
                "model_root": model_root,
                "run_label": run_label,
                "config_path": config_path,
                "packs_dir": "packs" if needs_packs else None,
                "slug": slug,
            })
    out_path = Path(__file__).parent / "example_runs.json"
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(runs, f, indent=2)
    print(f"Wrote {len(runs)} runs to {out_path}")


if __name__ == "__main__":
    main()
