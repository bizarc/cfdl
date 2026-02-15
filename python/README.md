# CFDL Python SDK

Minimal Python bindings for compiling and running CFDL models via the Rust SDK.

## Install (editable, local)

From repository root:

```bash
python -m pip install -e python/
```

## Public API

```python
from cfdl_sdk import compile_model, run_ir
```

- `compile_model(model_dir: str, packs_dir: str | None = None) -> str`
  - Compiles a CFDL model directory and returns IR JSON as a string.
- `run_ir(ir_json: str, packs_dir: str | None = None, config_json: str | None = None) -> str`
  - Runs the engine on IR JSON and returns Results JSON as a string.
  - `config_json` accepts either a JSON string or a filesystem path to a run-config JSON file.

## Smoke test

```bash
python -m pytest -q python/tests
```
