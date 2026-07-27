---
id: benchmarks
title: "Benchmark methodology"
slug: "/docs/benchmarks"
---

Every pack is gated by a parity suite: each CFDL model is diffed against an
**independent reference** implementation, period-by-period and on summary
metrics, inside a tolerance the case declares.

## How a case is built

Each `benchmarks/<pack>/<case>/` directory contains:

- `model.cfdl` — the CFDL model;
- `run.json` — the run configuration;
- `case.toml` — the pack name and per-period tolerance;
- `expected.csv` — period-level net cash flow from the reference;
- `expected_metrics.json` — summary metrics, each with its own tolerance;
- `reference_gen.py` — the independent reference that produces the expected
  files (a month-by-month recursion, distinct from the engine's evaluation).

`tools/benchmark-runner.py` compiles and runs each case with the `cfdl` CLI
and fails if any period or metric drifts outside tolerance. Schedule math is
held decimal-exact; IRR-class iteratives use a bps tolerance.

## Cases

| Pack | Case |
|---|---|
| cre | `office_two_tenant` |
| cre | `retail_strip` |
| credit | `float_bridge_pool` |
| credit | `io_bullet_loan` |
| credit | `level_pay_pool` |
| energy | `solar_ppa_microgrid` |
| energy | `wind_ptc_macrs` |
| opco | `lbo_buyout` |

> Reference models are independent implementations; those still awaiting
> practitioner verification say so in their `case.toml`.
