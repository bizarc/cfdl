---
id: benchmarks
title: "Benchmark methodology"
slug: "/docs/benchmarks"
---

Every pack is gated by a parity suite: each CFDL model is diffed against an
**independent reference**, period-by-period and on summary metrics, inside a
tolerance the case declares.

Two kinds of reference, and the difference matters. Most cases carry a
`reference_gen.py` — a second implementation written against the same
specification, which catches arithmetic and lowering errors. Some are
reconciled instead against an **external** model or published schedule, and
those carry a `NOTES.md` and no generator: two of your own implementations
agreeing is not evidence when both came from one assumption, and every
convention defect found so far has come from the external kind.

## How a case is built

Each `benchmarks/<pack>/<case>/` directory contains:

- `model.cfdl` — the CFDL model;
- `run.json` — the run configuration;
- `case.toml` — the pack name and per-period tolerance;
- `expected.csv` — period-level expectations from the reference: the model
  total, or each stream in its own column;
- `expected_metrics.json` — summary metrics, each with its own tolerance;
- either `reference_gen.py`, the independent implementation that produces the
  expected files, or `NOTES.md`, recording the external reconciliation — what
  was compared, what diverged, and how to repeat it.

`tools/benchmark-runner.py` compiles and runs each case with the `cfdl` CLI
and fails if any period or metric drifts outside tolerance. Schedule math is
held decimal-exact; IRR-class iteratives use a bps tolerance.

## Cases

| Pack | Case |
|---|---|
| cre | `mit_rentleg_plaza` |
| cre | `office_two_tenant` |
| cre | `retail_strip` |
| credit | `auto_abs_wal` |
| credit | `float_bridge_pool` |
| credit | `io_bullet_loan` |
| credit | `level_pay_pool` |
| credit | `mbs_pool_conventions` |
| energy | `solar_ppa_microgrid` |
| energy | `utility_pv_singleowner` |
| energy | `wind_ptc_macrs` |
| opco | `banker_dcf_conventions` |
| opco | `lbo_buyout` |

> Each case says in its `case.toml` where its figures came from, and which
> are still awaiting practitioner verification.
