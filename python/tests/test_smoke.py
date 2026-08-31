import json
from pathlib import Path

import pytest

from cfdl_sdk import compile_model, run_ir


REPO_ROOT = Path(__file__).resolve().parents[2]
PACKS_DIR = REPO_ROOT / "packs"


@pytest.mark.parametrize(
    "example_name,config_name",
    [
        ("cre_developer", "run.base.json"),
        ("opco_basic", "run.base.json"),
    ],
)
def test_compile_and_run_examples(example_name: str, config_name: str) -> None:
    example_dir = REPO_ROOT / "examples" / example_name
    config_path = example_dir / config_name

    ir_json = compile_model(str(example_dir), packs_dir=str(PACKS_DIR))
    ir = json.loads(ir_json)
    assert ir["ir_version"] == "0.1"
    assert isinstance(ir.get("streams", []), list)

    results_json = run_ir(
        ir_json,
        packs_dir=str(PACKS_DIR),
        config_json=str(config_path),
    )
    results = json.loads(results_json)
    assert results["results_version"] == "0.7"
    assert "deterministic" in results
