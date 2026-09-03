---
id: benchmark-credit-fnma-remic-2019-2-g3
title: "Credit: Fannie Mae REMIC with a stripped coupon"
slug: "/docs/examples/credit-fnma-remic-2019-2-g3"
description: "Security Group 3 of a Fannie Mae REMIC: a seasoned mortgage pool passing through to a single class, with the coupon stripped between it and an interest-only class that carries no principal."
source: benchmarks/credit/fnma_remic_2019_2_g3
---

# Credit: Fannie Mae REMIC with a stripped coupon

Security Group 3 of a Fannie Mae REMIC: a seasoned mortgage pool passing through to a single class, with the coupon stripped between it and an interest-only class that carries no principal.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

Fannie Mae REMIC Trust 2019-2 divides $307,727,958 across three groups. Group 3
is $148,372,434 of Fannie Mae mortgage-backed securities, and its priority of
payments is a single line:

> The Group 3 Principal Distribution Amount to AB until retired.

Which makes it sound like nothing to model. The interest is where the deal is.

The pool passes through at **5.00%**. Class AB takes **3.25%** of that, and the
remaining **1.75%** is sold separately as Class IO — an interest-only class with
no principal balance at all, entitled instead to 5.00% of a *notional* balance
set at 35.0000000674% of AB's. The two reconstruct the pass-through rate:

```
3.25%  +  0.350000000674 x 5.00%  =  5.00000000337%
```

An investor in IO owns a slice of a coupon and nothing else. Their position
shrinks as AB amortizes, they receive not one dollar of principal, and if the
loans prepay quickly they lose most of what they paid.

## The reference

The Prospectus Supplement dated 24 January 2019, page S-14, which publishes for
Classes AB and IO the percentage of the original balance outstanding after each
January's distribution for thirty years, at seven prepayment speeds, with a
weighted average life for each. See `SOURCE.md`.

This case takes the 198% PSA column, the pricing speed.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay` |
| Language features | two waterfalls over one collateral, one for principal and one for interest; entity fields carrying class balances |
| Conventions | PSA on a pool seasoned past the ramp, a servicing and guaranty strip, a stripped coupon, a notional interest-only class |

**This is the first case in the repo where a coupon is stripped.** Every other
credit case pays interest at the rate the asset earns. Here three rates are in
play at once and none of them is the mortgage rate:

- the loans carry a **5.451%** weighted average coupon;
- **0.451%** is the servicing and guaranty strip, carried as `servicing_fee`, so
  what reaches the trust is 5.00% exactly;
- that 5.00% is then split 3.25% / 1.75% between a principal class and a
  notional one.

The interest waterfall is the test. It declares a residual step, and if the
strip is right that step takes nothing:

```cfdl
pay ab_interest to party.ab_holders = asset.ab.balance * (0.0325 / 12.0)
pay io_interest to party.io_holders = asset.io.balance * (0.05 / 12.0)
pay residual    to party.residual   = remaining
```

**The residual is zero in all 361 periods, to ten decimal places.**

A note on why the class balances are fields at all. AB is a pass-through, so its
balance is the pool's, and IO's is a fixed fraction of AB's — neither carries
state of its own, and neither is copied. The field says what the class *is*
(`next prev.asset.pool.balance`) and derives the number. That the balances land
one period behind the distributions is not a workaround here but the deal's own
convention: the supplement strikes interest on "the outstanding balance of that
Certificate immediately prior to that Distribution Date", which is precisely the
number these fields hold.

## The result

**Worst disagreement 0.3640 percentage points** across 30 published cells,
against a floor of 0.5 set by the table's whole-percent rounding.

| | |
|---|---|
| Decrement, worst / mean | 0.3640 pp / 0.0841 pp |
| Informative cells (published value neither 0 nor 100) | 14 of 30 |
| Weighted average life | **4.696 → 4.7**, published **4.7** |
| Residual to Classes R and RL | **0.0000000000**, every period |
| Principal returned to AB | 148,372,434.00 against an original of 148,372,434 |

Over the 14 informative cells the mean error is 0.1803 pp — the signature of the
issuer's rounding and nothing else.

The interest legs are asserted too, and they are external rather than model
output: a published balance multiplied by a coupon stated on the cover. Their
tolerance is the balance band carried through the coupon.

The weighted average life is asserted at ±0.07: 0.05 is the print floor of a
figure published to one decimal, and ~0.015 is the axis — the engine measures
on its month-end axis while the deal distributes on the 25th measured from
late-January settlement, a uniform bias across all seven published speeds.

The other six published speeds — 0%, 100%, 300%, 400%, 700% and 1000% — each
ship as their own case (`fnma_remic_2019_2_g3_psa000` through `_psa1000`),
asserting their own decrement columns and weighted average lives, including
0% PSA, which the supplement prepares on its own alternative assumption of a
360-month original and remaining term at 7.50%.

## The delta

**Group 3 of three.** Groups 1 and 2 are Structured Collateral: their assets are
seventeen tranches of other Fannie Mae REMICs issued between 2002 and 2006. The
instrument is fully specified for those groups too — one line each, the same as
this one — but the cash arriving at them is another instrument's output, and
reproducing the published tables would need those seventeen deals' own
collateral. That is a compositional boundary, not a gap in this document, and it
is why only Group 3 is here.

**One speed here, seven in all.** This case ships the pricing speed; the other
six columns are sibling cases, so a convention error that hides under the
rounding floor at one speed has to hide at all seven simultaneously.

**No losses.** Fannie Mae guarantees timely payment of principal and interest,
so the collateral cannot default in a way the classes would see.

**Seasoned past the ramp.** Weighted average loan age is 175 months, so 198% PSA
is a flat 11.88% CPR in every period. The ramp is written out in full anyway —
the model should say the pool prepays at 198% PSA, not at 11.88% CPR, because
the second is a consequence of the first and stops being true if the seasoning
changes.

`model.total` is a regression anchor from this model. Every other assertion is a
published figure or derived from one.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.03}}
version 0.1
model "fnma-remic-2019-2-g3"
use pack "credit" version "0.1.0"
time calendar monthly from 2019-02 for 361

// GROUP 3 OF A FANNIE MAE REMIC, against the issuer's own decrement table.
//
// Fannie Mae REMIC Trust 2019-2 has three groups. Groups 1 and 2 are STRUCTURED
// COLLATERAL — their assets are seventeen tranches of other REMICs issued
// between 2002 and 2006, so the cash arriving at those groups is another
// instrument's output and has to be supplied rather than derived. Group 3 is
// the one backed directly by mortgage-backed securities, so it is complete in
// this document and is the group modeled here.
//
// A REMIC IS A FUNCTION FROM DOLLARS RECEIVED TO DOLLARS ALLOCATED, and this
// group's function is one line: everything to AB. What makes it worth a case is
// not the waterfall but the STRIP. The pool passes through at 5.00% against a
// 5.451% weighted average coupon; AB takes 3.25% of that, and the remaining
// 1.75% is sold separately as IO — a notional class with no principal, whose
// balance is 35.0000000674% of AB's and which therefore shrinks exactly as AB
// does.
//
// The two coupons reconstruct the pass-through rate to nine decimal places:
//
//     3.25%  +  0.350000000674 x 5.00%  =  5.00000000337%
//
// so the interest waterfall below should exhaust the pool's interest and leave
// the residual class nothing. That identity is asserted, and it is the reason
// this deal is more than a single pass-through pool.
//
// A FIELD CARRIES THE OPENING BALANCE. A recurrence may read the previous
// period's fields and no stream at all (docs/14), so a class balance at t is
// the balance FOLLOWING the distribution at t-1. That is not a workaround here,
// it is the deal's own convention: the supplement says interest on each
// certificate is "one month's interest on the outstanding balance of that
// Certificate immediately prior to that Distribution Date", which is exactly
// the number these fields hold.
//
// The published decrement table states balances outstanding after each January's
// distribution, so `expected.csv` asserts them one row later, and the timeline
// carries 361 periods so the January 2049 row has somewhere to land.
//
// Reference: Prospectus Supplement dated 24 January 2019 to the REMIC
// Prospectus dated 1 November 2018. See SOURCE.md.

entity asset trust : Credit.Asset.LoanPool {
  collateral_type = "residential"
}

// The Group 3 MBS. `balance` restates the level-pay amortization the pack
// already applies — one step per period, at the mortgage rate, against the PSA
// curve — because a field cannot read a stream. It is not an independent
// number: `expected.csv` asserts the pack's own cumulative principal against the
// same published column, so both are pinned to the issuer's figures rather than
// to each other.
//
// The loans are SEASONED PAST THE RAMP. Weighted average loan age is 175
// months, so `min(age, 30)` is 30 in every period and 198% PSA is a flat
// 11.88% CPR throughout. The ramp is written out in full anyway, because what
// the model should say is "this pool prepays at 198% PSA", not "this pool
// prepays at 11.88% CPR" — the second is a consequence, and it stops being true
// the moment the seasoning changes.
entity asset pool : Credit.Asset.LoanPool {
  collateral_type = "residential"
  part of asset.trust

  balance init 148372434.0
               * (1.0 - ((-pmt(0.0045425, 173.0, 1.0)) - 0.0045425))
               * (1.0 - cpr_to_periodic(min(1.98 * 0.002 * max(1.0, min(176.0, 30.0)), 1.0), 12.0))
          next if(time.t < 173.0,
                  prev * (1.0 - ((-pmt(0.0045425, 173.0 - time.t, 1.0)) - 0.0045425))
                       * (1.0 - cpr_to_periodic(min(1.98 * 0.002 * max(1.0, min(time.t + 176.0, 30.0)), 1.0), 12.0)),
                  0.0)
}

// AB — the pass-through class. It takes every dollar of principal, so its
// balance IS the pool's balance one period back. Nothing is copied: the field
// says "AB is a pass-through" and derives the number rather than tracking it.
entity asset ab : Credit.Asset.Tranche {
  seniority = 1
  original_balance = 148372434.0
  balance init 148372434.0
          next prev.asset.pool.balance
}

// IO — a notional class. It has no principal and receives none; the balance
// exists only to strike its interest, and it is a fixed fraction of AB's.
entity asset io : Credit.Asset.Tranche {
  seniority = 1
  original_balance = 51930352.0
  balance init 51930352.0
          next prev.asset.pool.balance * 0.350000000674
}

entity party ab_holders : Credit.Party.Investor { name = "Class AB holders" }
entity party io_holders : Credit.Party.Investor { name = "Class IO holders" }
entity party residual : Credit.Party.Investor { name = "Classes R and RL" }

// THE GROUP 3 MBS. $148,372,434 of Fannie Mae certificates passing through at
// 5.00%, against mortgage loans the supplement assumes at a 5.451% weighted
// average coupon, 173 months remaining and 175 months of seasoning as of
// 1 January 2019. 198% PSA is the pricing speed of the seven the decrement
// table publishes.
//
// `rate` is the MORTGAGE rate, because that is what amortizes the loans and so
// sets the principal the trust passes through. The 0.451% between it and the
// 5.00% pass-through rate is the servicing and guaranty strip, and it is
// carried as `servicing_fee` so that what reaches the trust is 5.00% exactly —
// which is what makes the AB and IO coupons add up below.
contract credit.pool_level_pay.g3 on entity asset.pool {
  term 2019-02..2033-06
  terms {
    principal = 148372434
    interest_rate = 0.05451
    term_months = 173
    age_months = 175
    psa_speed = 1.98
    servicing_fee = 0.00451
  }
}

// ---------------------------------------------------------------------------
// Distributions of principal
//
//   "The Group 3 Principal Distribution Amount to AB until retired."
//
// That is the entire priority of payments for this group. IO receives no
// principal, and the residual classes are entitled to nothing until AB is gone.
// ---------------------------------------------------------------------------
// NARROWER THAN `available`, deliberately: the supplement distributes
// principal as its own amount, so this waterfall draws that slice rather than
// the group's whole cash. `docs/03` §3.2 keeps the `from` expression free
// for exactly this.
waterfall g3.principal on entity asset.trust {
  schedule every month from 2019-02 to 2033-06

  from series_sum("credit.pool.sched_principal.*", time.t, time.t)
       + series_sum("credit.pool.prepay.*", time.t, time.t)

  pay ab_principal to party.ab_holders = remaining
}

// ---------------------------------------------------------------------------
// Distributions of interest
//
// One month's interest on the balance each certificate carried into the
// distribution date. AB at 3.25% and IO at 5.00% of a notional that is
// 35.0000000674% of AB's balance — together, the 5.00% the pool passes through.
//
// The residual step is the test. If the two coupons are right it takes nothing,
// and `expected.csv` asserts that it takes nothing.
// ---------------------------------------------------------------------------
// NARROWER THAN `available`, deliberately: the supplement distributes
// interest as its own amount, so this waterfall draws that slice rather than
// the group's whole cash. `docs/03` §3.2 keeps the `from` expression free
// for exactly this.
waterfall g3.interest on entity asset.trust {
  schedule every month from 2019-02 to 2033-06

  // What the TRUST receives, which is not what the loans pay. The pack's
  // interest line is gross, at the 5.451% mortgage coupon; the servicing and
  // guaranty strip is a separate outflow and is stored negative, so adding it
  // nets it off and leaves the 5.00% pass-through rate exactly.
  from series_sum("credit.pool.interest.*", time.t, time.t)
       + series_sum("credit.pool.servicing.*", time.t, time.t)

  pay ab_interest to party.ab_holders = asset.ab.balance * (0.0325 / 12.0)
  pay io_interest to party.io_holders = asset.io.balance * (0.05 / 12.0)
  pay residual    to party.residual   = remaining
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.03}}
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
| `model.total` | 183,313,276.29 | ±1 |
| `domain.credit.wal_years` | 4.7 | ±0.07 |
