---
id: benchmark-credit-io-bullet-loan
title: "Credit: IO/bullet bridge loan"
slug: "/docs/examples/credit-io-bullet-loan"
source: benchmarks/credit/io_bullet_loan
---

# Credit: IO/bullet bridge loan

An interest-only loan repaying its entire principal in a single balloon at maturity.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.07}}
version 0.1
model "io-bullet-loan"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 64

entity asset buyer : Credit.Asset.LoanPool

// $10mm interest-only pool, 7.25%, 60-month bullet, 5 CPR, 1.5 CDR,
// 40% severity, 4-month recovery lag. Contract term spans
// term_months + recovery_lag_months for the recovery tail.
contract credit.pool_io_bullet.bridge_a on entity asset.buyer {
  term 2026-01..2031-04
  terms {
    balance = 10000000
    rate = 0.0725
    term_months = 60
    cpr = 0.05
    cdr = 0.015
    severity = 0.40
    recovery_lag_months = 4
  }
}

contract credit.purchase.bridge_a on entity asset.buyer {
  term 2026-01..2026-01
  terms {
    price = 10000000
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.07
  }
}
```

## Verified results

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | -61,370.42 | ±1 |
| `model.moic` | 1.286054 | ±0.0001 |
| `model.wal_years` | 3.864922 | ±0.0001 |
| `domain.credit.interest` | 3,088,798.79 | ±1 |
| `domain.credit.principal` | 9,355,691.08 | ±1 |
| `domain.credit.recoveries` | 386,585.35 | ±1 |
| `domain.credit.wal_years` | 4.328274 | ±0.0001 |
| `domain.credit.collections` | 12,831,075.22 | ±1 |
| `domain.credit.purchase` | 10,000,000 | ±1 |
| `domain.credit.collections_multiple` | 1.283108 | ±0.0001 |
