---
id: benchmark-credit-level-pay-pool
title: "Credit: level-pay auto pool"
slug: "/docs/examples/credit-level-pay-pool"
source: benchmarks/credit/level_pay_pool
---

# Credit: level-pay auto pool

A level-payment amortizing loan pool — the constant instalment that splits into shrinking interest and growing principal.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A $25m auto loan pool at 6.5% over 120 months, bought at a one-point discount.
Every borrower pays the same instalment each month, which splits into shrinking
interest and growing principal. Layered on top: an 8% constant prepayment rate, a
2% default rate, 35% loss severity, a six-month recovery lag, a 50 basis point
servicing strip and a 1% prepayment penalty.

## The reference

Level-payment pool conventions as defined by the standard market formulas for
amortizing collateral — the same definitional source the mortgage cases use.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared month by month.

The pack lowers this contract to closed-form pool-factor expressions, and the
comparison is against a month-by-month recursion of the same convention.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay`, `credit.purchase` |
| Language features | a pack contract paired with a purchase price |
| Conventions | level-pay amortization, CPR, CDR, loss severity, recovery lag, a servicing strip, a prepayment penalty, purchase at a discount |

## The result

Present value **−295,975.22**, multiple on invested capital **1.225381** and
weighted average life **3.84394 years**.

Asserted: net cash flow per period across 120 months, plus the three summary
figures.

## The delta

None: every period agrees inside a one-cent tolerance. The weighted average life
and multiple carry a basis-point tolerance, since both are computed from an
iterative root rather than a closed form.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.06}}
version 0.1
model "level-pay-pool"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 126

entity asset buyer : Credit.Asset.LoanPool

// $25m homogeneous level-pay pool, 6.5% note rate, 10-year amortization,
// 8 CPR, 2 CDR, 35% severity, 6-month recovery lag, 50bp servicing strip,
// 1% prepayment penalty. The contract term spans term_months +
// recovery_lag_months so recoveries have periods to land in.
contract credit.pool_level_pay.auto_a on entity asset.buyer {
  term 2026-01..2036-06
  terms {
    balance = 25000000
    rate = 0.065
    term_months = 120
    cpr = 0.08
    cdr = 0.02
    severity = 0.35
    recovery_lag_months = 6
    servicing_fee = 0.005
    prepay_penalty_rate = 0.01
  }
}

// Purchased at a 1-point discount (99.0) at close.
contract credit.purchase.auto_a on entity asset.buyer {
  term 2026-01..2026-01
  terms {
    price = 24750000
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.06
  }
}
```

## Verified results

Checked period by period: **1 series** across **126 periods** — **126 values** in all, each within ±0.01 of the reference.

- `net_cash_flow`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | -295,975.22 | ±1 |
| `model.moic` | 1.225381 | ±0.0001 |
| `model.wal_years` | 3.84394 | ±0.0001 |
| `domain.credit.interest` | 6,499,894.55 | ±1 |
| `domain.credit.principal` | 22,978,060.49 | ±1 |
| `domain.credit.recoveries` | 1,167,319.89 | ±1 |
| `domain.credit.servicing` | 499,991.89 | ±1 |
| `domain.credit.penalties` | 82,207.68 | ±1 |
| `domain.credit.wal_years` | 4.056967 | ±0.0001 |
| `domain.credit.collections` | 30,727,482.61 | ±1 |
| `domain.credit.purchase` | 24,750,000 | ±1 |
| `domain.credit.collections_multiple` | 1.241514 | ±0.0001 |
