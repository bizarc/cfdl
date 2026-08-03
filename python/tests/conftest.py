"""Shared fixtures and helpers for the CFDL SDK test suite.

Replicates the conventions of ``tools/golden-runner`` so the Python results
match the committed gold outputs exactly:
  - a ``pack`` file in a fixture names the active domain pack;
  - a ``run.json`` provides the run config (which carries the discount rate);
  - when there is no ``run.json``, the runner uses ``--rate 0.10`` (the CLI
    default is 0.0), so we mirror that fallback here.
Comparison is canonical JSON (sorted keys, no whitespace) — an exact match,
no tolerances, the same guarantee the Rust golden job proves on three OSes.
"""
from __future__ import annotations

import json
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
PACKS_DIR = REPO_ROOT / "packs"
VALID_DIR = REPO_ROOT / "fixtures" / "valid"
INVALID_DIR = REPO_ROOT / "fixtures" / "invalid"
GOLD_RESULTS = REPO_ROOT / "gold" / "results"
GOLD_DIAG = REPO_ROOT / "gold" / "diag"


def canon(obj) -> str:
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def fixture_spec(fixture_dir: Path):
    """(pack, config_path_or_None, fallback_rate) for a valid fixture."""
    pack_file = fixture_dir / "pack"
    pack = pack_file.read_text(encoding="utf-8").strip() if pack_file.is_file() else None
    run_json = fixture_dir / "run.json"
    if run_json.is_file():
        return pack, str(run_json), 0.0
    return pack, None, 0.10


def valid_fixture_params():
    for d in sorted(p for p in VALID_DIR.iterdir() if (p / "model.cfdl").is_file()):
        yield pytest.param(d, id=d.name)


def invalid_fixture_params():
    for d in sorted(p for p in INVALID_DIR.iterdir() if (p / "model.cfdl").is_file()):
        gold = GOLD_DIAG / f"{d.name}.diag.json"
        if gold.is_file():
            yield pytest.param(d, id=d.name)
