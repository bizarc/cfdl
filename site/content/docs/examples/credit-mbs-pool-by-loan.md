---
id: benchmark-credit-mbs-pool-by-loan
title: "Credit: a mortgage pool modeled loan by loan"
slug: "/docs/examples/credit-mbs-pool-by-loan"
source: benchmarks/credit/mbs_pool_by_loan
---

# Credit: a mortgage pool modeled loan by loan

The same mortgage pool declared loan by loan, with the published pool schedule asserted against the aggregate the engine rolls up from its children.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A $100mm agency mortgage pool — 8% weighted average coupon, 360-month term, 20%
loss severity, twelve-month recovery lag, prepaying at a flat 1% single monthly
mortality against a 1% monthly default rate.

It is the same pool as the mortgage pool conventions case, at a different
grain. There it is one pool. Here it is **four loans of $40mm, $30mm, $20mm and
$10mm that belong to a pool**, and the pool itself holds no contract. Every
figure asserted against the pool is an aggregate.

## The reference

The industry's own standard formulas for analysing mortgage-backed securities —
the document that defines CPR, SMM, PSA and SDA — and the complete 176-month
cash flow schedule it publishes for this pool.

**Not redistributable.** The publisher forbids reproduction in any form, so the
source is neither vendored nor quoted; its figures are carried as anchor values
and cited as facts.

The reference publishes four columns: interest, scheduled amortization,
voluntary prepayments and principal recoveries. The pool's cash in a period is
their sum, so the anchors here are the published figures added together. Addition
is the only step taken.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Declared | five typed assets, one of them a parent; four contract instances |
| Language features | **`part of` hierarchy**, typed entity fields, per-instance contract suffixes |
| Conventions | level-pay amortization, SMM on the gross balance, MDR, a lagged recovery |

Two aggregates are asserted, computed by unrelated code:

- `entity.asset.pool.net_cash_flow` — the **hierarchy rollup**, aggregating the
  children a `part of` relation names rather than a matching name prefix.
- `domain.credit.gross_collections` — the **category subtotal**, the pack folding
  four contract instances into one domain line.

Both must reproduce the same published schedule. A defect in either shows as a
divergence between them.

## The result

**25 anchor months on both columns, across a 372-period grid.** Every one agrees
with the published schedule within the tolerance the source's rounding allows.

The rollup is also exact against the single-pool model: over all 372 periods,
`entity.asset.pool.net_cash_flow` here and `model.net_cash_flow` there agree to
**zero** — not within a tolerance, exactly. Splitting $100mm into four unequal
loans changes nothing about the pool's cash.

## The delta

Largest residual anywhere: **1.76 dollars**, against a tolerance of 2.01.

It is the source's rounding, not arithmetic. Each published figure is given to
the whole dollar and up to four are added, so two dollars bounds the difference
before any model is run.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "mbs-pool-by-loan"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 372

// THE SAME POOL AS `mbs_pool_conventions`, MODELED AT A DIFFERENT GRAIN.
//
// That case declares one $100m pool and asserts the published schedule against
// it. This one declares the SAME $100m as four loans that belong to a pool, and
// asserts the SAME published figures against the pool.
//
// The published numbers are therefore doing two jobs. They still check the
// conventions — level-pay amortization, SMM on the gross balance, MDR, a
// twelve-month recovery lag. And because the pool holds no contract of its own,
// every figure asserted at the pool level is an aggregate the engine computed
// by walking `part of`. A rollup that summed the wrong children, or that
// aggregated by name prefix rather than by the relation, cannot reproduce a
// schedule it did not produce.
//
// The four balances are uneven so that the aggregation is tested: four equal
// loans would agree with the pool under any rule that divided by four.

entity asset pool : Credit.Asset.LoanPool {
  collateral_type            = "residential_mortgage"
  original_balance           = 100000000
  weighted_average_coupon    = 0.08
  weighted_average_maturity  = 360
}

entity asset loan_a : Credit.Asset.Loan {
  original_balance = 40000000
  coupon           = 0.08
  part of asset.pool
}

entity asset loan_b : Credit.Asset.Loan {
  original_balance = 30000000
  coupon           = 0.08
  part of asset.pool
}

entity asset loan_c : Credit.Asset.Loan {
  original_balance = 20000000
  coupon           = 0.08
  part of asset.pool
}

entity asset loan_d : Credit.Asset.Loan {
  original_balance = 10000000
  coupon           = 0.08
  part of asset.pool
}

// Every loan carries the pool's conventions, because the reference's pool is
// homogeneous: one 8% WAC, one term, one hazard pair. The pack takes ANNUAL
// cpr/cdr and converts with `cpr_to_periodic`, so a monthly 1% SMM is stated
// as its annual equivalent, 1 - (1 - 0.01)^12, which converts back to exactly
// 0.01. The same restatement `mbs_pool_conventions` makes.

contract credit.pool_level_pay.a on entity asset.loan_a {
  term 2026-01..2056-12
  terms {
    balance = 40000000
    rate = 0.08
    term_months = 360
    cpr = 0.11361512828387077
    cdr = 0.11361512828387077
    severity = 0.20
    recovery_lag_months = 12
  }
}

contract credit.pool_level_pay.b on entity asset.loan_b {
  term 2026-01..2056-12
  terms {
    balance = 30000000
    rate = 0.08
    term_months = 360
    cpr = 0.11361512828387077
    cdr = 0.11361512828387077
    severity = 0.20
    recovery_lag_months = 12
  }
}

contract credit.pool_level_pay.c on entity asset.loan_c {
  term 2026-01..2056-12
  terms {
    balance = 20000000
    rate = 0.08
    term_months = 360
    cpr = 0.11361512828387077
    cdr = 0.11361512828387077
    severity = 0.20
    recovery_lag_months = 12
  }
}

contract credit.pool_level_pay.d on entity asset.loan_d {
  term 2026-01..2056-12
  terms {
    balance = 10000000
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

Checked period by period: **2 series** across **25 periods** — **50 values** in all, each within ±2.01 of the reference.

- `entity.asset.pool.net_cash_flow`
- `domain.credit.gross_collections`

