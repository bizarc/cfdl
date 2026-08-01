---
id: benchmark-credit-mbs-pool-conventions
title: "credit: mbs pool conventions"
slug: "/docs/examples/credit-mbs-pool-conventions"
source: benchmarks/credit/mbs_pool_conventions
---

# credit: mbs pool conventions

a 30-year agency-MBS pool at market-standard prepayment, default and recovery conventions. Validated for parity against the published industry reference schedule for MBS cash flows. That source is external and not redistributable, so it is not vendored and its tables are not reproduced — the figures below are anchor values carried for regression, cited as facts. Every number asserted here is external, not ours. There is deliberately no reference_gen.py and no expected_metrics.json: a second implementation of our own is exactly what this case exists to replace. See NOTES.md. period_tolerance = 0.51 — the reference figures are rounded to whole dollars, so half a dollar is the tightest bound they can support. Not loosened beyond that: every asserted figure matches within it.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The model

```cfdl
version 0.1
model "mbs-pool-conventions"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 372

entity fund buyer

// A 30-year agency-MBS pool at the market-standard prepayment and default
// conventions, reconciled against the published industry reference schedule.
//
// A new 30-year pool: $100m, 8% WAC, 20% loss severity, 12-month recovery,
// servicer advances. Cash Flow A runs it at a flat 1% SMM and 1% MDR — the
// constant-hazard case, which is why it is reproducible today. Cash Flow B on
// the same pool uses 150% PSA and 100% SDA, whose ramps the pack cannot yet
// express; see docs/13_feature_backlog.md.
//
// The pack takes ANNUAL cpr/cdr and converts with cpr_to_periodic, so a
// monthly 1% SMM is stated here as its annual equivalent:
//   cpr = 1 - (1 - 0.01)^12 = 0.11361512828387077
// which converts back to exactly 0.01. Practitioners quote SMM directly, so
// having to do this by hand is itself a small gap — also in the backlog.
//
// 372 periods = 360 months of pool life plus the 12-month recovery tail.
contract credit.pool_level_pay.a on entity fund.buyer {
  term 2026-01..2056-12
  terms {
    balance = 100000000
    rate = 0.08
    term_months = 360
    cpr = 0.11361512828387077
    cdr = 0.11361512828387077
    severity = 0.20
    recovery_lag_months = 12
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.08
  }
}
```

## Verified results

| Metric | Value | Tolerance |
|---|---:|---:|
