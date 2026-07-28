"""API-surface behavior: config forms, packs_dir inheritance, viz import guard."""
from __future__ import annotations

import json

import cfdl_sdk
from conftest import PACKS_DIR, canon

import pytest


def test_config_dict_path_string_equivalent():
    model_dir = "examples/cre_developer"
    config_path = "examples/cre_developer/run.base.json"
    config_dict = json.loads(open(config_path).read())
    config_str = json.dumps(config_dict)

    r_path = cfdl_sdk.run(model_dir, packs_dir=str(PACKS_DIR), config=config_path)
    r_dict = cfdl_sdk.run(model_dir, packs_dir=str(PACKS_DIR), config=config_dict)
    r_str = cfdl_sdk.run(model_dir, packs_dir=str(PACKS_DIR), config=config_str)

    assert canon(r_path.raw) == canon(r_dict.raw) == canon(r_str.raw)


def test_model_run_inherits_packs_dir():
    model = cfdl_sdk.compile("fixtures/valid/credit_pool_smoke", packs_dir=str(PACKS_DIR))
    # pack auto-detected from the fixture `pack` file; packs_dir inherited
    results = model.run(rate=0.10)
    assert results.raw.get("domain_metrics") is not None
    assert results.raw["domain_metrics"]["pack"] == "credit"


def test_split_compile_then_run_matches_convenience():
    model = cfdl_sdk.compile("examples/cre_developer", packs_dir=str(PACKS_DIR))
    r_split = model.run(config="examples/cre_developer/run.base.json")
    r_conv = cfdl_sdk.run(
        "examples/cre_developer",
        packs_dir=str(PACKS_DIR),
        config="examples/cre_developer/run.base.json",
    )
    assert canon(r_split.raw) == canon(r_conv.raw)


def test_viz_import_error_message(monkeypatch):
    import builtins
    import sys

    real_import = builtins.__import__

    def blocked(name, *args, **kwargs):
        if name.startswith("matplotlib"):
            raise ImportError("blocked for test")
        return real_import(name, *args, **kwargs)

    for mod in [m for m in sys.modules if m.startswith("matplotlib")]:
        monkeypatch.delitem(sys.modules, mod, raising=False)
    monkeypatch.setattr(builtins, "__import__", blocked)

    from cfdl_sdk import viz

    results = cfdl_sdk.run("fixtures/valid/minimal_model", packs_dir=str(PACKS_DIR))
    with pytest.raises(ImportError, match=r"cfdl-sdk\[viz\]"):
        viz.plot_cashflows(results)
