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

> **Supported calendars: all of them.** Two distinct daily shapes both work,
> and the difference matters:
>
> 1. **A daily book that pays monthly** — the ordinary mortgage or ABS pool.
>    Declare `payment_frequency = "month"` and periods-per-year comes from the
>    payment rhythm (12), not the calendar (365). A 30-year mortgage on a daily
>    book is still 360 payments, not 10,950. Verified exactly: the same pool on
>    a 39-period monthly grid and an 1186-period daily book agrees to the cent
>    on every stream.
> 2. **Genuinely daily accrual** — warehouse lines, repo, revolvers,
>    daily-reset floaters. Leave `payment_frequency` unset and ppy is 365.
>
> `term_months` must divide into whole payment periods. For daily accrual that
> means a tenor that is a multiple of 12 months (360 months → 10,950 days);
> otherwise `E5015_TERM_MONTHS_NOT_DIVISIBLE` names the nearest legal values.
>
> Note the two rate conventions, which must not be confused. The note rate,
> servicing strip and float margin are **nominal** and divide by ppy. CPR and
> CDR are **effective annual** and take a root — `cpr_to_periodic(x, ppy)`,
> which is exactly `cpr_to_smm(x)` at ppy = 12.
>
> **Day count is selectable** via a `day_count` term: `30/360` (the default,
> and what every existing model gets), `30e/360`, `act/360` or `act/365`. It
> applies to the note rate, the servicing strip and the floating
> index-plus-margin — every nominal rate in the pack.
>
> Under `act/360`, 6% on 1,200,000 accrues 6,200 in a 31-day January and 5,600
> in February, against a flat 6,000 under 30/360, and 73,000 over a 365-day
> year rather than 72,000. That 365/360 uplift is the convention's whole point,
> and it is the USD credit default. A misspelling is `E5019_UNKNOWN_DAY_COUNT`,
> not a silent fallback.
>
> **Amortisation has its own day count.** A level-pay pool strikes its payment
> once and then accrues interest period by period, so `amortization_day_count`
> selects the basis the payment is struck from and defaults to `day_count`.
> Setting `day_count = "act/360"` with `amortization_day_count = "30/360"` — the
> common US commercial case — holds the payment constant while interest varies
> with month length and scheduled principal absorbs the difference. Setting only
> `day_count` leaves every existing model unchanged. Applies to
> `credit.pool_level_pay`; IO/bullet contracts have no amortisation to strike.
>
> Annual totals do **not** match across monthly / quarterly / annual, and
> should not: nominal accrual means a 6% loan is 0.5%/month and 1.5%/quarter,
> which are different instruments. Those cadences are checked against the
> benchmark reference generators rather than against each other.

## Conventions, and where they come from

Prepayment and default follow the market-standard MBS conventions for CPR,
SMM and the standard prepayment and default curves; the pack is checked for
parity against the published industry reference schedule.

- **SMM applies to the balance at the BEGINNING of the period, net of
  scheduled amortisation only.** Defaults are not removed from the base.
- Because both attritions are drawn from that same base, survival is
  **additive**: `k = (1 - mdr) - smm`, not `(1 - mdr)(1 - smm)`.
- CPR and CDR are effective annual rates and convert by a root
  (`cpr_to_periodic`); note rates are nominal and convert by division.

`benchmarks/credit/mbs_pool_conventions` asserts anchor figures across the life
of a 30-year pool and passes, including recoveries — a level-pay pool's
defaulted balance keeps amortising in foreclosure, so what is liquidated is the
amortised balance rather than face. One known gap remains, in
`docs/13_feature_backlog.md`: age-varying prepayment and default curves are not
expressible, so only constant-hazard pools can be modelled.

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

> **The WAL time axis.** A collection at the close of period `t` sits at
> `(t + 1)/ppy` years, not `t/ppy` — so a monthly pool's first collection is at
> one twelfth of a year. That matches the market definition, the one a
> prospectus states as "the number of years from the closing date to the
> related distribution date", and it is the same axis the pack's cash is
> discounted on. Reconstructed from an issuer-published schedule, the
> difference is not academic: on a short auto class with a published WAL of
> 0.37 years, measuring from period zero gives 0.286.
>
> Two limits worth knowing. The origin is the **model start**, not a separate
> settlement date — identical whenever the deal closes at t = 0, which every
> credit model here does. And precision is period fractions rather than actual
> days, the same convention as discounting, so this will not tie to a published
> Act/360 figure in the fourth decimal.

## Not in v0.1

- **Floating-rate loans** — needs the `curve` input concept and
  mean-reverting rate paths (stochastic roadmap item 4).
- Zero note rate on `pool_level_pay` (closed form divides by `r`).
- Delinquency states, servicer advances, loan-level heterogeneity.

## Quick start

A $25mm level-pay pool with prepayments, defaults, a servicing strip, and a prepayment penalty:

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
    prepay_penalty_rate = 0.01
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

## Metrics reference

Computed automatically whenever a model runs with the `credit` pack, alongside the core metrics (NPV, IRR, MOIC, payback, WAL). Enumerated from the pack definition, so this list is always complete.

| Metric | Type | Built from |
|---|---|---|
| `domain.credit.interest` | money | derived |
| `domain.credit.principal` | money | derived |
| `domain.credit.recoveries` | money | derived |
| `domain.credit.penalties` | money | derived |
| `domain.credit.servicing` | money | derived |
| `domain.credit.wal_years` | number | derived |
| `domain.credit.collections` | money | derived |
| `domain.credit.purchase` | money | derived |
| `domain.credit.collections_multiple` | number | `domain.credit.collections` ÷ `domain.credit.purchase` |

## Validations reference

Checked at compile time. Each is a stable diagnostic code that is never renamed or reused; see [diagnostics](/docs/language-reference/diagnostics).

| Code | Rejects |
|---|---|
| `E9001_CREDIT_INVALID_BALANCE` | Credit pool 'balance' must be greater than 0. |
| `E9002_CREDIT_INVALID_RATE` | Credit pool 'rate' must be a non-negative annual rate. |
| `E9003_CREDIT_INVALID_TERM_MONTHS` | Credit pool 'term_months' must be a whole number greater than 0. |
| `E9010_CREDIT_INVALID_CPR` | Credit 'cpr' must be an annual rate between 0 and 1. |
| `E9011_CREDIT_INVALID_CDR` | Credit 'cdr' must be an annual rate between 0 and 1. |
| `E9012_CREDIT_INVALID_SEVERITY` | Credit 'severity' must be a fraction between 0 and 1. |
| `E9013_CREDIT_INVALID_RECOVERY_LAG` | Credit 'recovery_lag_months' must be a whole number of months, 0 or more. |
| `E9014_CREDIT_INVALID_SERVICING_FEE` | Credit 'servicing_fee' must be an annual rate between 0 and 1. |
| `E9015_CREDIT_INVALID_PREPAY_PENALTY` | Credit 'prepay_penalty_rate' must be a rate between 0 and 1. |
| `E9020_CREDIT_RATE_FLOOR_ABOVE_CAP` | Credit 'rate_floor' must be less than or equal to 'rate_cap'. |

## Worked example models

Benchmark cases are validated period-by-period against an independent
reference implementation.

- [Credit: floating-rate bridge pool](/docs/examples/credit-float-bridge-pool)
- [Credit: IO/bullet bridge loan](/docs/examples/credit-io-bullet-loan)
- [Credit: level-pay auto pool](/docs/examples/credit-level-pay-pool)
- [credit: mbs pool conventions](/docs/examples/credit-mbs-pool-conventions)
