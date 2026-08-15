---
id: benchmark-credit-mbs-pool-conventions
title: "Credit: mortgage pool conventions"
slug: "/docs/examples/credit-mbs-pool-conventions"
description: "A mortgage pool priced under standard market conventions, reconciling published factors, CPR and SMM against a fixed prepayment vector."
source: benchmarks/credit/mbs_pool_conventions
---

# Credit: mortgage pool conventions

A mortgage pool priced under standard market conventions, reconciling published factors, CPR and SMM against a fixed prepayment vector.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A 30-year agency mortgage pool run at market-standard conventions: a constant
prepayment rate converted to a single monthly mortality, a default rate, a loss
severity on defaulted balances, and a lag before recoveries arrive. These are
the definitional mechanics of mortgage cash flow.

## The reference

The industry's own standard formulas for analysing mortgage-backed securities —
the document that *defines* CPR, SMM, PSA and SDA. It ships two complete
176-month cash flow schedules, so the comparison is period by period against
the definitions themselves.

**Not redistributable.** The publisher forbids reproduction in any form, so the
source is neither vendored nor quoted; its figures are carried as anchor values
and cited as facts.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay` |
| Language features | pack contract lowering to four separate cash flow lines |
| Conventions | CPR-to-SMM conversion, constant default rate, loss severity, recovery lag |

## The result

Interest, scheduled principal, prepayments and recoveries each reproduce as
their own column across the schedule, rather than only in a total.

Asserted: four stream columns period by period.

## The delta

The tolerance is 0.51 — just over half a dollar — because the published schedule
prints whole dollars while compounding on unrounded balances. Half a dollar is
the closest any implementation can come to a figure rounded to the dollar.

There is no summary metric; the four columns are asserted directly.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "mbs-pool-conventions"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 372

entity asset buyer : Credit.Asset.LoanPool

// A 30-year agency-MBS pool at the market-standard prepayment and default
// conventions, reconciled against the published industry reference schedule.
//
// A new 30-year pool: $100m, 8% WAC, 20% loss severity, 12-month recovery,
// servicer advances. Cash Flow A runs it at a flat 1% SMM and 1% MDR — the
// constant-hazard case, which is why it is reproducible today. Cash Flow B on
// the same pool uses 150% PSA and 100% SDA, whose ramps the pack cannot yet
// express directly today.
//
// The pack takes ANNUAL cpr/cdr and converts with cpr_to_periodic, so a
// monthly 1% SMM is stated here as its annual equivalent:
//   cpr = 1 - (1 - 0.01)^12 = 0.11361512828387077
// which converts back to exactly 0.01. Practitioners quote SMM directly, so
// having to do this by hand is itself a small gap.
//
// 372 periods = 360 months of pool life plus the 12-month recovery tail.
contract credit.pool_level_pay.a on entity asset.buyer {
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

Checked period by period: **4 series** across **25 periods** — **95 values** in all, each within ±0.51 of the reference.

- `credit.pool.interest.a`
- `credit.pool.sched_principal.a`
- `credit.pool.prepay.a`
- `credit.pool.recoveries.a`

