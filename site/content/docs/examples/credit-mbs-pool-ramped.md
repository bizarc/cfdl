---
id: benchmark-credit-mbs-pool-ramped
title: "credit: mbs pool ramped"
slug: "/docs/examples/credit-mbs-pool-ramped"
source: benchmarks/credit/mbs_pool_ramped
---

# credit: mbs pool ramped

A mortgage pool on a ramping prepayment curve, where speeds build over the first thirty months before levelling off.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "mbs-pool-ramped"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 372

entity fund buyer

// The SAME 30-year agency-MBS pool as benchmarks/credit/mbs_pool_conventions —
// $100m, 8% WAC, 20% loss severity, 12-month recovery — run at the published
// reference's RAMPED conventions rather than its flat ones.
//
// That reference works the pool twice: once at a flat 1% SMM / 1% MDR, which
// mbs_pool_conventions takes, and once at 150% PSA with 100% SDA, which is this
// case. The ramped run was unreachable until the pool factor became a
// per-period state: under a ramp the survival factor is a cumulative product
// with no elementary closed form, and every pool factor in the pack was
// pow(k, p), valid only for a constant hazard.
//
// The two speeds are stated as MULTIPLES of the published curves. The pack
// derives the per-period rates from them, and they agree with the reference's
// own stated rate columns at month 1: monthly prepayment 0.000250 and monthly
// default 0.000017.
//
// age_months is 0 — a new pool, so the curves start at their first month.
//
// 372 periods = 360 months of pool life plus the 12-month recovery tail.
contract credit.pool_level_pay.a on entity fund.buyer {
  term 2026-01..2056-12
  terms {
    balance = 100000000
    rate = 0.08
    term_months = 360
    psa_speed = 1.5
    sda_speed = 1.0
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
