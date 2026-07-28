---
id: cookbook-credit
title: "Credit pack guide"
slug: "/docs/cookbooks/credit"
source: packs/credit/README.md
---

Credit / lending pack: fixed-rate loan pools with CPR prepayments, CDR
defaults, loss severity and a recovery lag. Benchmarked in
`benchmarks/credit/` against independent month-by-month reference
implementations.

## Contract types

### `credit.pool_level_pay`

Homogeneous level-pay (fully amortizing) pool. The engine's expression
dialect has no loops, so streams use the exact closed form for a homogeneous
pool under constant SMM/MDR (the standard pool-factor decomposition) — see
the convention block at the top of `lowering/rules.toml`.

Terms:

| term | meaning | default |
|---|---|---|
| `balance` | original pool balance | required |
| `rate` | annual note rate (must be > 0) | required |
| `term_months` | amortization term in months | required |
| `cpr` | annual conditional prepayment rate | `0` |
| `cdr` | annual conditional default rate | `0` |
| `severity` | loss severity on defaulted balance | `0` |
| `recovery_lag_months` | months from default to recovery cash | `0` |
| `servicing_fee` | annual servicing strip on performing balance | `0` |
| `prepay_penalty_rate` | flat penalty rate on voluntary prepayments | `0` |

Streams (all suffixed by contract instance): `credit.pool.interest`,
`credit.pool.sched_principal`, `credit.pool.prepay`,
`credit.pool.recoveries`, `credit.pool.servicing` (outflow),
`credit.pool.penalty`.

`servicing_fee` and `prepay_penalty_rate` are available on every pool type.
The penalty is a flat rate on prepaid balance (simplified yield
maintenance); a discounted make-whole needs an engine primitive and stays
on the parity worklist.

The contract `term` must span `term_months + recovery_lag_months` schedule
periods so the recovery tail has periods to land in; the expressions gate
themselves, so a longer term is harmless.

### `credit.pool_io_bullet`

Interest-only pool with a principal bullet at maturity. Balance declines
only through prepayment and default; the final period pays no SMM
prepayment — the whole surviving balance pays as the bullet. Same terms as
`pool_level_pay` (`rate` may be anything, including 0). Adds stream
`credit.pool.bullet`.

### `credit.pool_float_io_bullet`

Floating-rate IO/bullet pool. The coupon indexes off a model-declared
`curve` statement:

```cfdl
curve sofr {
  2026-01: 0.048
  2026-07: 0.045
}
```

`coupon = clamp(curve_value(index_curve, date) + margin, rate_floor, rate_cap)`.
Balance dynamics are identical to `pool_io_bullet` (rate-independent), so
the closed form stays exact. Extra terms on top of the common credit terms:

| term | meaning | default |
|---|---|---|
| `index_curve` | name of a `curve` declared in the model | required |
| `margin` | spread over the index | `0` |
| `rate_floor` | coupon floor | `0` |
| `rate_cap` | coupon cap | `1` |

Floating **level-pay** pools are not supported: the balance path depends on
the rate path, which has no closed form under the pack's loop-free
expressions.

### `credit.purchase`

Acquisition price paid at `term_start` (`price` term), stream
`credit.purchase.price`. Discount/premium purchases are just a `price`
different from face (e.g. 99.0 = `0.99 * balance`); the level_pay_pool
benchmark purchases at a 1-point discount.

## Conventions

- Defaults leave the pool at the start of the period and earn no interest
  that period.
- `cpr`/`cdr` are annualized; converted with `cpr_to_smm(x) = 1 - (1-x)^(1/12)`.
- Prepayment applies SMM to the performing balance net of scheduled
  principal.
- Recoveries return `(1 - severity)` of defaulted face,
  `recovery_lag_months` later. Defaulted-balance write-offs are not cash
  and emit no stream.

## Metrics

`domain.credit.interest`, `.principal` (scheduled + prepay + bullet),
`.recoveries`, `.penalties`, `.servicing` (omitted when zero),
`.collections` (interest + principal + recoveries + penalties),
`.purchase`, `.collections_multiple` (collections / purchase), and
`.wal_years` — principal-weighted average life over principal returned
(scheduled + prepay + bullet + recoveries; interest/penalties excluded).

## Not in v0.1

- **Floating-rate loans** — needs the `curve` input concept and
  mean-reverting rate paths (stochastic roadmap item 4).
- Zero note rate on `pool_level_pay` (closed form divides by `r`).
- Delinquency states, servicer advances, loan-level heterogeneity.

## Quick start

A $25mm level-pay pool with prepayments, defaults, and a servicing strip:

```cfdl
version 0.1
model "my-pool"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 126

entity fund buyer

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
  }
}

contract credit.purchase.auto_a on entity fund.buyer {
  term 2026-01..2026-01
  terms { price = 24750000 }
}
```

`credit.purchase` takes the dollar purchase amount — a 1-point discount
(99.0) on the $25mm balance here.

Let the contract term span `term_months + recovery_lag_months` so lagged
recoveries have periods to land in.

## Run it

```bash
cfdl compile my-pool --packs packs --out my-pool/ir.json
cfdl run my-pool/ir.json --packs packs --pack credit --out my-pool/results.json --rate 0.08
```

Domain metrics include interest/principal/recoveries/collections, the
collections multiple, servicing, penalties, and principal WAL
(`domain.credit.wal_years`).

## Recipes

**Floating-rate IO bullet off a rate curve** (margin/floor/cap against a
model-declared curve):

```cfdl
curve sofr linear {
  2026-01: 0.050
  2027-01: 0.038
}

contract credit.pool_float_io_bullet.bridge on entity fund.buyer {
  term 2026-01..2028-12
  terms {
    balance = 15000000
    index_curve = "sofr"
    margin = 0.0275
    rate_floor = 0.065
    rate_cap = 0.095
    term_months = 36
    cpr = 0.05
    cdr = 0.01
    severity = 0.30
    recovery_lag_months = 3
  }
}
```

**IO pool with bullet maturity**: `credit.pool_io_bullet` — same loss
vocabulary, interest-only until the balloon.

Full worked models: `benchmarks/credit/level_pay_pool/`,
`io_bullet_loan/`, `float_bridge_pool/`, and the loan-pool notebook in
`examples/notebooks/`.

## Worked example models

Benchmark cases are validated period-by-period against an independent
reference implementation.

- [Credit: floating-rate bridge pool](/docs/examples/credit-float-bridge-pool)
- [Credit: IO/bullet bridge loan](/docs/examples/credit-io-bullet-loan)
- [Credit: level-pay auto pool](/docs/examples/credit-level-pay-pool)
