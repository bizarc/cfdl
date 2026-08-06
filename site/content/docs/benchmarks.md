---
id: benchmarks
title: "Validation"
slug: "/docs/benchmarks"
generated: regions
---

# Validation

CFDL's numbers are checked against references it did not produce. Every pack is
gated by a parity suite: each model is diffed against an **independent
reference**, period by period and on summary metrics, inside a tolerance the
case declares. A drift outside that tolerance fails the build.

## Two kinds of reference, and the difference matters

Most cases carry a `reference_gen.py` — a second implementation written against
the same specification. That catches arithmetic and lowering errors.

Some are reconciled instead against an **external** model or published
schedule, and carry no generator. Those are the ones that count. Two of your
own implementations agreeing is not evidence when both came from the same
assumption, and every convention defect found so far has come from the external
kind.

The clearest example is HUD's HOME Multifamily template — a US federal work in
the public domain, so the workbook itself is committed to the repository. A
reader can open the source and check the reconciliation rather than take it on
trust.

## What a case contains

Each `benchmarks/<pack>/<case>/` directory holds:

- `model.cfdl` — the model
- `run.json` — the run configuration
- `case.toml` — the pack, a one-line summary, and the per-period tolerance
- `expected.csv` — period-level expectations from the reference: the model
  total, or each stream in its own column
- `expected_metrics.json` — summary metrics, each with its own tolerance
- either `reference_gen.py`, the independent implementation that produces those
  files, or `NOTES.md`, recording the external reconciliation — what was
  compared, what diverged, and how to repeat it

`tools/benchmark-runner.py` compiles and runs each case with the CLI and fails
if any period or metric drifts outside tolerance. Schedule arithmetic is held
decimal-exact; IRR-class iteratives carry a basis-point tolerance.

## Tolerance is a claim, not a cushion

A tolerance says how close the two implementations are expected to be, and why.
A workbook that prints whole dollars cannot be matched closer than half a
dollar, so those cases carry `0.5` and say so. A ratio quoted to sixteen
significant figures carries `1e-4`. Where a case needs both, tolerances are set
per column rather than one loose number covering everything — a single value
that satisfied the money lines would assert nothing about the ratio.

## The cases

<!-- cfdl:generated benchmark-cases -->
| Pack | Case |
|---|---|
| cre | `hud_home_multifamily` |
| cre | `mit_rentleg_plaza` |
| cre | `office_two_tenant` |
| cre | `one_lincoln_street` |
| cre | `retail_strip` |
| credit | `auto_abs_speed_050` |
| credit | `auto_abs_speed_150` |
| credit | `auto_abs_wal` |
| credit | `float_bridge_pool` |
| credit | `io_bullet_loan` |
| credit | `level_pay_pool` |
| credit | `mbs_pool_conventions` |
| credit | `mbs_pool_ramped` |
| energy | `crest_solar_cost_based` |
| energy | `merchant_capacity` |
| energy | `solar_ppa_microgrid` |
| energy | `utility_pv_singleowner` |
| energy | `wind_ptc_macrs` |
| opco | `banker_dcf_conventions` |
| opco | `damodaran_fcff` |
| opco | `gordon_growth_coned` |
| opco | `lbo_buyout` |

*22 cases.*
<!-- /cfdl:generated benchmark-cases -->

Each case declares in its `case.toml` where its figures came from, and which are
still awaiting practitioner verification.

## Beyond the suite

The benchmarks check CFDL against other implementations. Two other gates check
it against mathematics:

- **Analytic identities** — a par bond discounted at its coupon is worth par;
  an annuity due is worth exactly `(1+i)` times the ordinary annuity. These hold
  for any correct implementation and cannot be satisfied by copying what the
  engine currently does.
- **Cadence parity** — one deal modelled on every calendar must produce the same
  annual economics.
