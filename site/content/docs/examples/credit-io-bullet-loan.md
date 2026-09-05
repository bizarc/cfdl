---
id: benchmark-credit-io-bullet-loan
title: "Credit: IO/bullet bridge loan"
slug: "/docs/examples/credit-io-bullet-loan"
description: "An interest-only loan repaying its entire principal in a single balloon at maturity."
source: benchmarks/credit/io_bullet_loan
---

# Credit: IO/bullet bridge loan

An interest-only loan repaying its entire principal in a single balloon at maturity.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A $10m interest-only bridge pool at 7.25% for 60 months, bought at par. Nothing
amortizes: the borrower pays interest monthly and the entire principal returns in
a single balloon at maturity. Against that sit a 5% constant prepayment rate, a
1.5% default rate, 40% loss severity and a four-month recovery lag.

## The reference

Interest-only and bullet-maturity conventions as defined by the standard market
formulas for non-amortizing collateral.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared month by month.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.loan`, `credit.purchase` |
| Language features | a pack contract paired with a purchase price |
| Conventions | interest-only accrual, a bullet maturity, CPR, CDR, severity, recovery lag |

With no scheduled amortization, weighted average life is driven entirely by
prepayment and default.

## The result

Present value **−61,370.42**, multiple on invested capital **1.286054** and
weighted average life **3.864922 years**.

Asserted: net cash flow per period across 60 months, plus the three summary
figures.

## The delta

None: every period agrees inside a one-cent tolerance.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.07}}
version 0.1
model "io-bullet-loan"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 64

entity asset buyer : Credit.Asset.Loan

// $10m interest-only pool, 7.25%, 60-month bullet, 5 CPR, 1.5 CDR,
// 40% severity, 4-month recovery lag. Contract term spans
// term_months + recovery_lag_months for the recovery tail.
contract credit.loan.bridge_a on entity asset.buyer {
  term 2026-01..2031-04
  terms {
    amortization = "interest_only"
    principal = 10000000
    interest_rate = 0.0725
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

Checked period by period: **1 series** across **64 periods** — **64 values** in all, each within ±0.01 of the reference.

- `net_cash_flow`

Summary metrics for the base run:

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
