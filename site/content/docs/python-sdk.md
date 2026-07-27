---
id: python-sdk
title: Python SDK
slug: /docs/python-sdk
---

# Python SDK

`cfdl_sdk` compiles and runs CFDL models from Python with pandas accessors
over the results. The compiler and engine are embedded in-process as a Rust
extension module — no separate binary or server, and results are
byte-identical to the CLI's.

## Install

See [Install for Python](/docs/install/python). Short version, from a checkout:
`pip install -e "python/[dev,viz]"`.

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
results.monte_carlo()    # per-metric summary stats (mean/stdev/percentiles)
results.annual()         # annual rollup, when present
```

`config` accepts a dict, a path to a run-config JSON file, or a raw JSON
string. `pack` (for domain metrics) is auto-detected from a `pack` file in
the model directory, or can be passed explicitly. `compile()` and
`Model.run()` are available separately when you want to reuse compiled IR.

Compile problems raise `CompileError` with structured `.diagnostics`
(code, message, span); runtime problems raise `RunError`.

## Notebooks

Four industry notebooks are built on the benchmark models — the same models
CFDL validates against an independent reference. Each is published here with
the outputs and chart it actually produced, so you can read the SDK's DataFrame
surface on a realistic model before installing anything:

- [Solar PPA microgrid](/docs/notebooks/energy-solar-microgrid)
- [CRE office acquisition](/docs/notebooks/cre-office-acquisition)
- [Credit loan pool](/docs/notebooks/credit-loan-pool)
- [Operating company LBO](/docs/notebooks/opco-lbo)

To run them yourself, install the `notebooks` extra and open the folder from a
checkout — they read their models from `benchmarks/`:

```bash
pip install -e "python/[notebooks]"
jupyter lab examples/notebooks
```
