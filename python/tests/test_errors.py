"""Error-path coverage: invalid models raise typed errors with diagnostics."""
from __future__ import annotations

import json

import cfdl_sdk
from cfdl_sdk import CompileError, RunError
from conftest import GOLD_DIAG, PACKS_DIR, invalid_fixture_params

import pytest


@pytest.mark.parametrize("fixture_dir", invalid_fixture_params())
def test_invalid_raises_compile_error(fixture_dir):
    gold = json.loads((GOLD_DIAG / f"{fixture_dir.name}.diag.json").read_text(encoding="utf-8"))
    expected_codes = {d["code"] for d in gold}
    with pytest.raises(CompileError) as excinfo:
        cfdl_sdk.compile(fixture_dir, packs_dir=str(PACKS_DIR))
    diagnostics = excinfo.value.diagnostics
    assert diagnostics, "CompileError carried no diagnostics"
    got_codes = {d.code for d in diagnostics}
    # The SDK reports the same diagnostic set the golden diag captures.
    assert got_codes == expected_codes
    # spans are populated for at least the first diagnostic
    assert diagnostics[0].span is not None


def test_run_error_on_bad_config():
    model = cfdl_sdk.compile("fixtures/valid/minimal_model", packs_dir=str(PACKS_DIR))
    with pytest.raises(RunError):
        model.run(config="{ this is not valid json")
