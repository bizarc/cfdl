"""The SDK must reproduce the committed gold results byte-for-byte.

This is the Python mirror of ``tools/golden-runner``: for every valid fixture,
compile + run through the SDK and canonical-compare against
``gold/results/<name>.results.json``.
"""
from __future__ import annotations

import json

import cfdl_sdk
from conftest import GOLD_RESULTS, PACKS_DIR, canon, fixture_spec, valid_fixture_params

import pytest


@pytest.mark.parametrize("fixture_dir", valid_fixture_params())
def test_matches_gold(fixture_dir):
    pack, config, rate = fixture_spec(fixture_dir)
    results = cfdl_sdk.run(
        fixture_dir,
        packs_dir=str(PACKS_DIR),
        config=config,
        rate=rate,
        pack=pack,
    )
    gold_path = GOLD_RESULTS / f"{fixture_dir.name}.results.json"
    assert gold_path.is_file(), f"missing gold for {fixture_dir.name}"
    gold = json.loads(gold_path.read_text(encoding="utf-8"))

    assert results.raw["results_version"] == "0.9"
    assert results.model_hash == gold["model_hash"]
    assert canon(results.raw) == canon(gold)
