---
id: benchmark-credit-level-pay-pool
title: "Credit: level-pay auto pool"
slug: "/docs/examples/credit-level-pay-pool"
source: benchmarks/credit/level_pay_pool
---

# Credit: level-pay auto pool

$25mm level-pay auto pool, 6.5% / 120mo, 8 CPR, 2 CDR, 35% severity, 6mo recovery lag, 50bp servicing strip, 1% prepay penalty, purchased at a 1-point discount (99.0). The pack lowers to the closed-form pool-factor expressions; the reference is an independent month-by-month recursion of the same convention.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The model

```cfdl
version 0.1
model "level-pay-pool"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 126

entity fund buyer

// $25mm homogeneous level-pay pool, 6.5% note rate, 10-year amortization,
// 8 CPR, 2 CDR, 35% severity, 6-month recovery lag, 50bp servicing strip,
// 1% prepayment penalty. The contract term spans term_months +
// recovery_lag_months so recoveries have periods to land in.
contract credit.pool_level_pay.auto_a on entity fund.buyer {
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
contract credit.purchase.auto_a on entity fund.buyer {
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

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | -295,975.22 | ±1 |
| `model.moic` | 1.225381 | ±0.0001 |
| `model.wal_years` | 3.817027 | ±0.0001 |
| `domain.credit.interest` | 6,499,894.55 | ±1 |
| `domain.credit.principal` | 22,978,060.49 | ±1 |
| `domain.credit.recoveries` | 1,167,319.89 | ±1 |
| `domain.credit.servicing` | 499,991.89 | ±1 |
| `domain.credit.penalties` | 82,207.68 | ±1 |
| `domain.credit.wal_years` | 3.973633 | ±0.0001 |
| `domain.credit.collections` | 30,727,482.61 | ±1 |
| `domain.credit.purchase` | 24,750,000 | ±1 |
| `domain.credit.collections_multiple` | 1.241514 | ±0.0001 |
