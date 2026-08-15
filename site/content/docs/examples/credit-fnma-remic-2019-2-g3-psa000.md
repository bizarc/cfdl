---
id: benchmark-credit-fnma-remic-2019-2-g3-psa000
title: "Credit: Fannie Mae REMIC at 0% PSA"
slug: "/docs/examples/credit-fnma-remic-2019-2-g3-psa000"
description: "Group 3 of a Fannie Mae REMIC with the mortgage loans never prepaying — the supplement's own alternative collateral of new 7.50% thirty-year loans, amortizing on schedule for thirty years."
source: benchmarks/credit/fnma_remic_2019_2_g3_psa000
---

# Credit: Fannie Mae REMIC at 0% PSA

Group 3 of a Fannie Mae REMIC with the mortgage loans never prepaying — the supplement's own alternative collateral of new 7.50% thirty-year loans, amortizing on schedule for thirty years.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

No prepayments at all — and not the deal's own loans. For this column the supplement swaps the collateral for its stated alternative: new loans with 360-month original and remaining terms at 7.50%, so the pool amortizes on pure schedule for thirty years and the strip to the 5.00% pass-through is 2.50% rather than 0.451%. The class takes until January 2049 to retire, and the table publishes a value in every one of the thirty years.

The deal is Security Group 3 of Fannie Mae REMIC Trust 2019-2: a $148,372,434
pass-through with the coupon stripped between a principal class and a notional
interest-only class. `fnma_remic_2019_2_g3` ships the 198% pricing speed and
carries the deal's full description; this case moves the prepayment assumption
to 0% PSA and asserts the decrement column the supplement publishes for it.

## The reference

The same table as the base case: the Prospectus Supplement dated
24 January 2019, page S-14, which publishes for Classes AB and IO the percent
of original balance outstanding after each January's distribution at seven
prepayment speeds, with a weighted average life for each. This case takes the
0% PSA column. See the base case's `SOURCE.md`.

## What it exercises

The same model as the base case with one term changed, and for 0% PSA the collateral itself: the supplement prepares this column on new 7.50% loans with 360-month original and remaining terms, so the case also exercises a 2.50% servicing and guaranty strip against a thirty-year schedule. What the seven
cases prove together is stronger than any one alone: a convention error in the
prepayment curve, the seasoning ramp or the payment timing that hides under
one column's whole-percent rounding has to hide under all seven columns and
seven published weighted average lives simultaneously.

## The result

**180 asserted values**, every one within the half-percent floor the
table's whole-percent rounding sets. Worst balance disagreement
**0.489 percentage points** against the 0.5 floor.

| | |
|---|---|
| Weighted average life | **20.2290**, published **20.2** |
| Residual to Classes R and RL | **0.0000000000**, every period |
| Principal returned to AB | 148,372,434.00 against an original of 148,372,434 |

The weighted average life is asserted at ±0.07: 0.05 is the print floor of a
figure published to one decimal, and ~0.015 is the axis — the engine measures
on its month-end axis while the deal distributes on the 25th measured from
late-January settlement, a bias uniform across all seven published speeds.

## The delta

The strip identity — 3.25% to AB plus 5.00% of the notional balance
reconstructing the 5.00% pass-through — holds to ten decimal places at this
speed as at every other, which is what makes the residual assertion exact
while the balances carry the table's rounding.

Everything structural — the no-losses guarantee, the compositional boundary
that keeps Groups 1 and 2 out, the one-line waterfall — is as the base case
states it.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.045}}
version 0.1
model "fnma-remic-2019-2-g3-psa000"
use pack "credit" version "0.1.0"
time calendar monthly from 2019-02 for 361

// GROUP 3 OF FANNIE MAE REMIC TRUST 2019-2 AT 0% PSA, against the same
// decrement table as `fnma_remic_2019_2_g3`, which ships the 198% pricing
// speed and carries the deal's full description. This case is one of the six
// other published columns: same collateral, same strip, same one-line
// waterfall — only the prepayment assumption moves — and for this column the collateral itself: the supplement prepares 0% PSA on its own alternative assumption of 360-month original and remaining terms at 7.50%.
//
// See `fnma_remic_2019_2_g3/model.cfdl` for why the class balances are fields
// and why the balances land one period behind the distributions (the deal's
// own convention: interest is struck on the balance immediately prior to the
// distribution date).

entity asset trust : Credit.Asset.LoanPool {
  collateral_type = "residential"
}

entity asset pool : Credit.Asset.LoanPool {
  collateral_type = "residential"
  part of asset.trust

  balance init 148372434.0
               * (1.0 - ((-pmt(0.00625, 360.0, 1.0)) - 0.00625))
               * (1.0 - cpr_to_periodic(min(0.0 * 0.002 * max(1.0, min(1.0, 30.0)), 1.0), 12.0))
          next if(time.t < 360.0,
                  prev * (1.0 - ((-pmt(0.00625, 360.0 - time.t, 1.0)) - 0.00625))
                       * (1.0 - cpr_to_periodic(min(0.0 * 0.002 * max(1.0, min(time.t + 1.0, 30.0)), 1.0), 12.0)),
                  0.0)
}

entity asset ab : Credit.Asset.Tranche {
  seniority = 1
  original_balance = 148372434.0
  balance init 148372434.0
          next prev.asset.pool.balance
}

entity asset io : Credit.Asset.Tranche {
  seniority = 1
  original_balance = 51930352.0
  balance init 51930352.0
          next prev.asset.pool.balance * 0.350000000674
}

entity party ab_holders : Credit.Party.Investor { name = "Class AB holders" }
entity party io_holders : Credit.Party.Investor { name = "Class IO holders" }
entity party residual : Credit.Party.Investor { name = "Classes R and RL" }

// The 0% PSA column's own collateral: 360/360 months at 7.50%, per the supplement's stated alternative assumption. The 2.50% strip nets the pass-through to 5.00% exactly, as 0.451% does for the pricing assumptions.
contract credit.pool_level_pay.g3 on entity asset.pool {
  term 2019-02..2049-01
  terms {
    balance = 148372434
    rate = 0.075
    term_months = 360
    age_months = 0
    psa_speed = 0.0
    servicing_fee = 0.025
  }
}

waterfall g3.principal on entity asset.trust {
  schedule every month from 2019-02 to 2049-01

  from series_sum("credit.pool.sched_principal.*", time.t, time.t)
       + series_sum("credit.pool.prepay.*", time.t, time.t)

  pay ab_principal to party.ab_holders = remaining
}

// One month's interest on the balance each certificate carried into the
// distribution date: AB at 3.25%, IO at 5.00% of its notional. The pool
// passes through at 5.00% under every column's assumptions, so the residual
// step should take nothing, and expected.csv asserts that it takes nothing.
waterfall g3.interest on entity asset.trust {
  schedule every month from 2019-02 to 2049-01

  from series_sum("credit.pool.interest.*", time.t, time.t)
       + series_sum("credit.pool.servicing.*", time.t, time.t)

  pay ab_interest to party.ab_holders = asset.ab.balance * (0.0325 / 12.0)
  pay io_interest to party.io_holders = asset.io.balance * (0.05 / 12.0)
  pay residual    to party.residual   = remaining
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.045}}
```

## Verified results

Checked period by period: **6 series** across **60 periods** — **180 values** in all, each within the tolerance shown.

- `domain.credit.principal_paid_to_date` — within ±741862.17
- `asset.ab.balance` — within ±741862.17
- `asset.io.balance` — within ±259651.76
- `g3.interest.ab_interest` — within ±2009.21
- `g3.interest.io_interest` — within ±1081.88
- `g3.interest.residual` — within ±0.01

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `domain.credit.principal` | 148,372,434 | ±0.01 |
| `domain.credit.wal_years` | 20.2 | ±0.07 |
| `model.total` | 298,443,458.45 | ±1 |
