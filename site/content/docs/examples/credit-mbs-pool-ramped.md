---
id: benchmark-credit-mbs-pool-ramped
title: "Credit: mortgage pool on a prepayment ramp"
slug: "/docs/examples/credit-mbs-pool-ramped"
source: benchmarks/credit/mbs_pool_ramped
---

# Credit: mortgage pool on a prepayment ramp

A mortgage pool on a ramping prepayment curve, where speeds build over the first thirty months before levelling off.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

The same 30-year agency mortgage pool, but on a **ramping** prepayment curve:
speeds build month by month over the first thirty months, then level off. A ramp
is the standard market assumption for a seasoning pool, and it is the case a
constant-hazard shortcut gets wrong.

## The reference

The industry's own standard formulas for analysing mortgage-backed securities,
which define the ramp and publish a complete cash flow schedule computed on it.

**Not redistributable.** The publisher forbids reproduction in any form, so the
source is neither vendored nor quoted; its figures are carried as anchor values
and cited as facts.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay` |
| Language features | pack contract lowering to four cash flow lines; a per-period pool factor carried as state |
| Conventions | a prepayment ramp over thirty months, CPR-to-SMM conversion, default, severity, recovery lag |

The ramp is why this case exists alongside the constant-speed one. Under a
changing hazard the surviving balance is a running product, and a closed-form
`pow(k, p)` is exact only while the rate holds still.

## The result

Interest, scheduled principal, prepayments and recoveries each reproduce as their
own column across the schedule.

Asserted: four stream columns period by period.

## The delta

The tolerance is 0.51 — just over half a dollar — set by the published
schedule's whole-dollar rounding rather than by anything about this pool.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "mbs-pool-ramped"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 372

entity asset buyer : Credit.Asset.LoanPool

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
contract credit.pool_level_pay.a on entity asset.buyer {
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

Checked period by period: **4 series** across **25 periods**, each within ±0.51 of the reference.

- `credit.pool.interest.a`
- `credit.pool.sched_principal.a`
- `credit.pool.prepay.a`
- `credit.pool.recoveries.a`

