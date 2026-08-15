---
id: benchmark-credit-float-bridge-pool
title: "Credit: floating-rate bridge pool"
slug: "/docs/examples/credit-float-bridge-pool"
source: benchmarks/credit/float_bridge_pool
---

# Credit: floating-rate bridge pool

A floating-rate bridge loan pool priced off a forward curve, where the coupon resets each period rather than being fixed at origination.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A $15m floating-rate bridge pool. The coupon is a benchmark rate plus 275 basis
points, resetting each period off a stepped forward curve, with a 7.00% floor
that binds in the early periods. It runs 36 months to a bullet, bought at par,
against a 10% prepayment rate, a 2.5% default rate, 45% severity and a five-month
recovery lag.

## The reference

Floating-rate pool conventions as defined by the standard market formulas —
coupon reset off an index, with a rate floor applied before the spread.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared month by month.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_float_io_bullet`, `credit.purchase` |
| Language features | a declared `curve` read per period by `curve_value` with step interpolation |
| Conventions | a coupon that resets rather than fixing at origination, a binding rate floor, a bullet maturity |

Curves are exercised end to end here: the curve statement, its representation
in the compiled model, and the per-period lookup.

## The result

Present value **−433,719.03**, multiple on invested capital **1.151953** and
weighted average life **2.367044 years**.

Asserted: net cash flow per period across 36 months, plus the three summary
figures.

## The delta

None: every period agrees inside a one-cent tolerance, including the periods
where the floor binds and the coupon stops tracking the curve.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.075}}
version 0.1
model "float-bridge-pool"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 41

// Forward index curve (flat-forward / step interpolation): the coupon for a
// period uses the last curve point at or before the period date.
curve sofr {
  2026-01: 0.048
  2026-07: 0.045
  2027-01: 0.042
  2027-07: 0.040
  2028-01: 0.0385
}

entity asset buyer : Credit.Asset.LoanPool

// $15m floating IO bridge pool: SOFR + 275, coupon floored at 7.00% (binds
// once SOFR + margin falls below it) and capped at 9.00% (never binds),
// 36-month bullet, 10 CPR, 2.5 CDR, 45% severity, 5-month recovery lag.
// Contract term spans term_months + recovery_lag_months.
contract credit.pool_float_io_bullet.bridge_f on entity asset.buyer {
  term 2026-01..2029-05
  terms {
    balance = 15000000
    index_curve = "sofr"
    margin = 0.0275
    rate_floor = 0.07
    rate_cap = 0.09
    term_months = 36
    cpr = 0.10
    cdr = 0.025
    severity = 0.45
    recovery_lag_months = 5
  }
}

contract credit.purchase.bridge_f on entity asset.buyer {
  term 2026-01..2026-01
  terms {
    price = 15000000
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.075
  }
}
```

## Verified results

Checked period by period: **1 series** across **41 periods** — **41 values** in all, each within ±0.01 of the reference.

- `net_cash_flow`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | -433,719.03 | ±1 |
| `model.moic` | 1.151953 | ±0.0001 |
| `model.wal_years` | 2.367044 | ±0.0001 |
| `domain.credit.interest` | 2,670,923.28 | ±1 |
| `domain.credit.principal` | 14,053,633.59 | ±1 |
| `domain.credit.recoveries` | 520,501.53 | ±1 |
| `domain.credit.wal_years` | 2.54018 | ±0.0001 |
| `domain.credit.collections` | 17,245,058.4 | ±1 |
| `domain.credit.purchase` | 15,000,000 | ±1 |
| `domain.credit.collections_multiple` | 1.149671 | ±0.0001 |
