---
id: python-sdk
title: Python SDK
slug: /python-sdk
---

# Python SDK

`cfdl_sdk` compiles and runs CFDL models from Python with pandas accessors
over the results. The compiler and engine are embedded in-process as a Rust
extension module — no separate binary or server, and results are
byte-identical to the CLI's.

## Install

From a repository checkout:

```bash
pip install -e "python/[dev,viz]"
```

`pandas>=2.0` is a hard dependency; the optional `viz` extra adds matplotlib
plotting. Wheels are built for CPython ≥ 3.10 (abi3) on Linux
(x86_64/aarch64), macOS (universal2), and Windows.

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

`examples/notebooks/` contains executed industry notebooks built on the
benchmark models: a solar microgrid, a CRE office property, a loan pool, and
an LBO. They are the fastest way to see the SDK's DataFrame surface on
realistic models.
