# CFDL Python SDK

`cfdl-sdk` compiles and runs [CFDL](https://cfdl.dev) cash-flow models from
Python, with pandas accessors over the results. The compiler and engine are
embedded in-process (a Rust extension module) — no separate binary or server.

## Install (editable, local)

From the repository root:

```bash
pip install -e "python/[dev,viz]"
```

`pandas>=2.0` is a hard dependency. The optional `viz` extra adds matplotlib.

## Quickstart

```python
import cfdl_sdk

results = cfdl_sdk.run(
    "examples/cre_developer",
    packs_dir="packs",
    config="examples/cre_developer/run.base.json",
)

results.cashflows()      # wide DataFrame: one column per stream, PeriodIndex
results.metrics()        # flat Series of core + domain metrics
results.metrics_frame()  # metric / value / currency / source(core|domain:<pack>)
results.scenarios()      # one row per scenario (when the run declares them)
results.monte_carlo()    # per-metric summary stats (mean/stdev/min/max/p50)
results.annual()         # annual rollup, when present
```

`config` accepts a dict, a path to a run-config JSON file, or a raw JSON
string. `pack` (for domain metrics) is auto-detected from a `pack` file in the
model directory, or pass it explicitly. Split compile/run:

```python
model = cfdl_sdk.compile("examples/cre_developer", packs_dir="packs")
results = model.run(config={"deterministic": {"annual_discount_rate": 0.08}})
```

## Plotting (`[viz]` extra)

```python
results.plot.cashflows()             # step plot of per-period flows
results.plot.cumulative()            # cumulative net cash flow
results.plot.mc_distribution("model.npv")
```

## Errors

`CompileError` carries structured `.diagnostics` (each a `Diagnostic` with
`code`, `message`, `span`, ...); `RunError` carries an engine message. Both
subclass `CfdlError`.

## Low-level passthrough

`compile_model(model_dir, packs_dir=None) -> str` and
`run_ir(ir_json, packs_dir=None, config_json=None, rate=0.0, as_of=None,
pack=None) -> str` return raw JSON strings, mirroring the `cfdl` CLI.

## Tests

```bash
make py-test     # or: python3 -m pytest -q python/tests
```
