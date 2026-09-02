---
id: benchmark-credit-americredit-2017-1
title: "Credit: auto ABS with a step-down and a turbo"
slug: "/docs/examples/credit-americredit-2017-1"
description: "The note classes of a sub-prime auto ABS that builds its own overcollateralization: a 22-step waterfall where excess cash accelerates principal toward a target and principal beyond it is retained rather than paid."
source: benchmarks/credit/americredit_2017_1
---

# Credit: auto ABS with a step-down and a turbo

The note classes of a sub-prime auto ABS that builds its own overcollateralization: a 22-step waterfall where excess cash accelerates principal toward a target and principal beyond it is retained rather than paid.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A sub-prime auto lender sold $1.01bn of car loans into a trust and issued $930m
of notes against them in six public classes. The gap between the two is the
noteholders' protection, and on day one it is only 5.75%. The deal's job over
its first half-year is to widen it to 14.75%, out of the difference between what
the borrowers pay — around 12.6% a year — and what the notes cost, which is
0.95% to 3.23%.

That makes the pay-down of any class a question about cash rather than about
collateral. Principal arriving from the loans is not the only thing repaying the
notes, and once the target is reached it stops being enough of an answer either:
principal that would carry enhancement *past* target is held back and released
to the certificateholder, so the notes amortize alongside the pool instead of
ahead of it.

## The reference

The Rule 424(b)(5) prospectus dated 21 February 2017 states the priority of
payments in twenty-two numbered clauses and publishes, on pp. 59-62, the percent
of each class outstanding at all 62 distribution dates under four prepayment
speeds — 0.50%, 1.00%, 1.50% and 2.00% ABS — with a weighted average life to
call and to maturity beneath each table. 1,512 cells, of which **195 are
informative**: the rest are exactly 0 or 100 and assert only "retired by then"
and "not started yet".

An independent implementation of the deal, written from the fourteen
assumptions the tables state, shares no code with the model. The class sizes and coupons those
assumptions carry are **not the deal as priced** — the tables were prepared
before pricing — and reproducing them means using what they assume. See
`SOURCE.md`.

## What it exercises

`auto_abs_tranches` takes the sequential-pay axis of a deal that assumes no
losses and therefore never has to build anything. This case is the other half of
the same pack's job, and the mechanism is the **Step-Down Amount**: the
Noteholders' Principal Distributable Amount is principal collected *less*
whatever would take the notes below the Required Pro Forma Note Balance.

Written the way the prospectus writes it, that is a step-down subtracted from a
distributable amount and then an accelerated principal amount capped by both
available cash and the distance to target. It collapses to one statement about
where the notes end the period:

```cfdl
min(pool - floor, max(required, notes - principal - max(excess, 0.0)))
```

The notes finish at the required balance; cash may stop them getting there; and
overcollateralization may not fall below 0.50% of the initial pool. Clause 18 is
then `min(total, principal)` and clause 20 is `max(total - principal, 0)`, which
is what those clauses mean. The two formulations agree to $0.0000 at all four
speeds.

The **reserve account** is the other thing the target depends on. Clause 19
funds it at closing to 2.0% of the initial pool, and the Required Pro Forma Note
Balance is 14.75% of the pool *less the amount on deposit in it* — so the
reserve sets how far the turbo runs, and through that every class's retirement
date. It is a declared account here, funded by its own inflow at closing rather
than out of the waterfall, which allocates collections and never touched it.
Clause 19 is then the top-up it is: whatever the balance is short of the
required amount. On this deal, with no losses assumed, that is zero at every
period — because the balance is at target, which is what the clause says.

The **clean-up call** is what ends the deal, and it is an occurrence rather than
a condition a pool sits in. Once the pool falls to 10% of its original balance
the servicer may purchase the receivables; the published tables assume it does,
which is why they stop. Each of the twelve assumed pools is a loan pool carrying
the credit pack's own machine, and the election drives their
`amortizing -> retired` edge. Once-ness is the topology — nothing leaves
`retired` — rather than any latch on the event.

The two halves land a period apart, and deliberately. A pool is carried into a
period, collects, and the period ends. The redeeming distribution is the last one
made while the trust still owns the receivables, so the redemption price joins
that pot. When the *next* period opens, the pool carried in has fallen to the
threshold — the purchase has settled — and that is where the transition belongs
and where the published tables put their first zero.

The cash then stops because the pool is gone, not because anything switches it
off. A level-pay pool contract carries a surviving fraction, and every stream it
produces is a balance times an amortization factor times that fraction; the
election writes it to zero, and a pool with no surviving balance pays nothing.
The state and the cash are the same fact rather than two that must be kept in
step.

The **trust is a container**, and it winds itself up. It holds the twelve pools;
the pools hold the receivables. What it still owns is its parts' surviving
fractions summed, and when that reaches zero the trust moves to `wound_up` — one
period after the pools retire, because a container can only act on a settled
part, and winding up any earlier would end the trust before its pools' last
period of activity had been counted. Nothing asserts the trust's state; it
follows from what the trust contains.

All twenty-two clauses are written out. Ten of them — the parity steps and the
final-maturity steps — pay nothing at every period and every speed, because the
pool always covers the notes and every class retires years early. They are there
because the deal has them.

## The result

The model agrees with the reference to **5 cents**, across all seven classes,
every clause of the waterfall and all 63 periods — 32 asserted series, not the
balances alone, and the clauses are asserted past the call as well as up to it:
zero, in every period after the trust is retired. Against the published grid, the reference reproduces:

| | |
|---|---:|
| Informative cells inside the whole-percent floor | **192 of 195** |
| Mean error inside it (0.25 predicted for a correct model) | **0.2470** |
| Maximum error inside it (0.4974 predicted) | **0.4990** |
| Published weighted average lives reproduced exactly | **48 of 48** |

The published grid rounds to whole percentages, so a model that is exactly right
has errors uniform on [0, 0.5]. The mean and the maximum both sit where that
distribution puts them, which is stronger evidence than either cell count: a
model that is subtly wrong shows a biased distribution even when every
individual cell passes.

Five conventions had to be recovered, none of them stated in the document: a
January-cutoff pool pays twice before the first distribution, and pays two
months of servicing fee with it; ABS runs from origination, which retires four
seasoned pools outright at 2.00%; the step-down floor is 0.50% of the initial
pool; and weighted average life runs 30E/360 from closing to the 18th, with a
25-day stub. `NOTES.md` records each, with the readings tested and rejected
against it.

## The delta

**Three cells sit outside the floor**, by 0.60 to 0.68 of a point: Class A-1 at
its second or third distribution date, at three of the four speeds. That is
about $1.1m of principal in one month on a $182m class, and it is left open
rather than fitted. The stub first interest period — 25 days on a 30/360 basis
or 23 actual days from closing to the first payment — was tested and is worse,
which is the arithmetic consequence of the assumption that every month has 30
days.

Every published weighted average life is reproduced, to call and to maturity.

## What the case does not assert

- **One speed.** The model runs at 1.50% ABS. The other three published speeds
  are the same model with `abs_speed` changed, and `docs/20` §2.3 is the reason
  they are not four directories.
- **The weighted average lives.** All 48 are reproduced by the reference and
  none is asserted by the case: `docs/20` §3.1 — a published life still has no
  series or metric to check it against.
- **The other three speeds' call dates.** The call date moves with the speed,
  and the reference reproduces all four; the case asserts the one it runs.

## What the expectations are, and are not

`expected.csv` is the reference implementation's, not the prospectus's. It
holds every class balance and every clause of the distribution, so the
waterfall is pinned as well as the grid — without the cash columns the model's
twenty-two steps would be unchecked, since the balances come from the
recurrence rather than from the waterfall.

The two implementations are independent in their arithmetic and **not** in
their inputs: the model was generated from the same twelve-pool table and the
same class terms the reference carries, so a transcription error would appear
in both. What guards the inputs is the published grid itself — 195 informative
cells at four speeds is not something a mistyped balance or coupon survives.

`model.total` is the reference's too. The trust's net cash is every collection
on the receivables less the servicing fee taken out of them, over the collection
periods the trust owns and no others, and the reference computes it that way
without reference to the model — so the figure carries the clean-up call in it:
collections stop when the servicer buys the loans. The redemption price is not
part of it, since the trust passes that straight out to the noteholders rather
than earning it.

Every external assertion is the independent reference against the published
grid, and `expected.csv` holds that reference's own output: every class balance
and every clause of the distribution, in every period of the book.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.03}}
version 0.1
model "americredit-2017-1"
use pack "credit" version "0.1.0"
time calendar monthly from 2017-01 for 71

// A 22-STEP AUTO ABS WATERFALL, against the issuer's published grid.
//
// AmeriCredit Automobile Receivables Trust 2017-1 publishes, for six note
// classes at four ABS speeds, the percent of each class outstanding at every
// distribution date. `auto_abs_tranches` took the sequential-pay axis of a
// deal that could not lose money and whose notes therefore never had to build
// enhancement. This deal turbos: excess cash accelerates principal until
// overcollateralization reaches 14.75% of the pool net of the reserve, and
// principal that would take the notes below that target is RETAINED — the
// Step-Down Amount — rather than paid.
//
// THE CALENDAR STARTS IN JANUARY AND THE FIRST DISTRIBUTION IS PERIOD 1.
// Six of the twelve assumed pools have a 1 January cutoff and six a
// 1 February cutoff, and the first due date is the last day of the cutoff
// month. So the January pools pay twice before the 18 March distribution and
// the February pools once. Starting the book in January and each contract on
// its own cutoff says that structurally, rather than by special-casing the
// first period.
//
// A CLASS BALANCE IS A FIELD, and the field cannot see what the waterfall
// paid. `docs/14` §3.1 puts stream series up to t-1 in a recurrence's
// environment; `compute_states` supplies `prev_states` and `prev_self` and
// nothing else, so a field reads no series at all — not this waterfall's, not
// the pack's. The consequence is visible below: the distribution arithmetic
// appears twice, once lagged inside the balance fields and once at the current
// period inside the waterfall. They are the same arithmetic and they must stay
// that way. Recorded as a capability gap; see NOTES.md.
//
// The three fields that carry the collateral — `pool_bal`, `pool_prior` and
// `pool_int` — are closed forms of the twelve assumed pools rather than
// recurrences, for the same reason: a field cannot read the pack's own
// collateral series. The pack contracts below produce the cash the waterfall
// allocates, so the two are independent statements of the same pool and
// `expected.csv` pins both.
//
// Reference: EXTERNAL. Rule 424(b)(5) prospectus dated 21 February 2017,
// pp. 57-62 — see SOURCE.md. Run at 1.50% ABS; the other three published
// speeds are this model with one term changed.


entity container trust : Container.SPV {
  lifecycle trust_life

  // WHAT THE TRUST STILL OWNS, summed from the pools themselves rather than
  // restated. Each pool's contract carries a surviving fraction — the share of
  // it still performing — and this is their total: twelve at closing, zero once
  // the servicer has bought them all. It reads `prev` because a field names a
  // period's value at close, which does not exist yet inside a rule; the parts'
  // close is last period's, which is exactly the settled quantity wanted.
  surviving init 12.0
       next prev.asset.p01.credit_level_pay_survival_p01
           + prev.asset.p02.credit_level_pay_survival_p02
           + prev.asset.p03.credit_level_pay_survival_p03
           + prev.asset.p04.credit_level_pay_survival_p04
           + prev.asset.p05.credit_level_pay_survival_p05
           + prev.asset.p06.credit_level_pay_survival_p06
           + prev.asset.p07.credit_level_pay_survival_p07
           + prev.asset.p08.credit_level_pay_survival_p08
           + prev.asset.p09.credit_level_pay_survival_p09
           + prev.asset.p10.credit_level_pay_survival_p10
           + prev.asset.p11.credit_level_pay_survival_p11
           + prev.asset.p12.credit_level_pay_survival_p12

  // The pool after this period's collections. Period 0 is the
  // January collection period, before any distribution.
  pool_bal init 1011969929.28
           next if((time.t) <= 0.0, 1011969929.28, 999357.60 * (max(1.0 - 0.015 * (53.0 + (time.t)), 0.0) / 0.2050000000) * (pv(0.0131575000, max(61.0 - (53.0 + (time.t)), 0.0), -1.0) / pv(0.0131575000, 8.0, -1.0))
           + 18401017.06 * (max(1.0 - 0.015 * (53.0 + (time.t)), 0.0) / 0.2050000000) * (pv(0.0111400000, max(72.0 - (53.0 + (time.t)), 0.0), -1.0) / pv(0.0111400000, 19.0, -1.0))
           + 3063342.92 * (max(1.0 - 0.015 * (38.0 + (time.t)), 0.0) / 0.4300000000) * (pv(0.0115650000, max(65.0 - (38.0 + (time.t)), 0.0), -1.0) / pv(0.0115650000, 27.0, -1.0))
           + 2629247.98 * (max(1.0 - 0.015 * (2.0 + (time.t)), 0.0) / 0.9700000000) * (pv(0.0102091667, max(48.0 - (2.0 + (time.t)), 0.0), -1.0) / pv(0.0102091667, 46.0, -1.0))
           + 21301021.93 * (max(1.0 - 0.015 * (3.0 + (time.t)), 0.0) / 0.9550000000) * (pv(0.0107941667, max(61.0 - (3.0 + (time.t)), 0.0), -1.0) / pv(0.0107941667, 58.0, -1.0))
           + 285432214.08 * (max(1.0 - 0.015 * (2.0 + (time.t)), 0.0) / 0.9700000000) * (pv(0.0105358333, max(72.0 - (2.0 + (time.t)), 0.0), -1.0) / pv(0.0105358333, 70.0, -1.0))
           + 2076350.36 * (max(1.0 - 0.015 * (54.0 + (time.t)), 0.0) / 0.2050000000) * (pv(0.0129500000, max(61.0 - (54.0 + (time.t)), 0.0), -1.0) / pv(0.0129500000, 8.0, -1.0))
           + 37654758.33 * (max(1.0 - 0.015 * (54.0 + (time.t)), 0.0) / 0.2050000000) * (pv(0.0114466667, max(72.0 - (54.0 + (time.t)), 0.0), -1.0) / pv(0.0114466667, 19.0, -1.0))
           + 9848795.79 * (max(1.0 - 0.015 * (41.0 + (time.t)), 0.0) / 0.4000000000) * (pv(0.0115608333, max(67.0 - (41.0 + (time.t)), 0.0), -1.0) / pv(0.0115608333, 27.0, -1.0))
           + 4675311.33 * (max(1.0 - 0.015 * (4.0 + (time.t)), 0.0) / 0.9550000000) * (pv(0.0106325000, max(49.0 - (4.0 + (time.t)), 0.0), -1.0) / pv(0.0106325000, 46.0, -1.0))
           + 43859779.20 * (max(1.0 - 0.015 * (4.0 + (time.t)), 0.0) / 0.9550000000) * (pv(0.0109775000, max(61.0 - (4.0 + (time.t)), 0.0), -1.0) / pv(0.0109775000, 58.0, -1.0))
           + 582028732.70 * (max(1.0 - 0.015 * (3.0 + (time.t)), 0.0) / 0.9700000000) * (pv(0.0104741667, max(72.0 - (3.0 + (time.t)), 0.0), -1.0) / pv(0.0104741667, 70.0, -1.0)))


  // The same, one period back: what the pool carried into the period.
  pool_prior init 1011969929.28
             next if((time.t - 1.0) <= 0.0, 1011969929.28, 999357.60 * (max(1.0 - 0.015 * (53.0 + (time.t - 1.0)), 0.0) / 0.2050000000) * (pv(0.0131575000, max(61.0 - (53.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0131575000, 8.0, -1.0))
           + 18401017.06 * (max(1.0 - 0.015 * (53.0 + (time.t - 1.0)), 0.0) / 0.2050000000) * (pv(0.0111400000, max(72.0 - (53.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0111400000, 19.0, -1.0))
           + 3063342.92 * (max(1.0 - 0.015 * (38.0 + (time.t - 1.0)), 0.0) / 0.4300000000) * (pv(0.0115650000, max(65.0 - (38.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0115650000, 27.0, -1.0))
           + 2629247.98 * (max(1.0 - 0.015 * (2.0 + (time.t - 1.0)), 0.0) / 0.9700000000) * (pv(0.0102091667, max(48.0 - (2.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0102091667, 46.0, -1.0))
           + 21301021.93 * (max(1.0 - 0.015 * (3.0 + (time.t - 1.0)), 0.0) / 0.9550000000) * (pv(0.0107941667, max(61.0 - (3.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0107941667, 58.0, -1.0))
           + 285432214.08 * (max(1.0 - 0.015 * (2.0 + (time.t - 1.0)), 0.0) / 0.9700000000) * (pv(0.0105358333, max(72.0 - (2.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0105358333, 70.0, -1.0))
           + 2076350.36 * (max(1.0 - 0.015 * (54.0 + (time.t - 1.0)), 0.0) / 0.2050000000) * (pv(0.0129500000, max(61.0 - (54.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0129500000, 8.0, -1.0))
           + 37654758.33 * (max(1.0 - 0.015 * (54.0 + (time.t - 1.0)), 0.0) / 0.2050000000) * (pv(0.0114466667, max(72.0 - (54.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0114466667, 19.0, -1.0))
           + 9848795.79 * (max(1.0 - 0.015 * (41.0 + (time.t - 1.0)), 0.0) / 0.4000000000) * (pv(0.0115608333, max(67.0 - (41.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0115608333, 27.0, -1.0))
           + 4675311.33 * (max(1.0 - 0.015 * (4.0 + (time.t - 1.0)), 0.0) / 0.9550000000) * (pv(0.0106325000, max(49.0 - (4.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0106325000, 46.0, -1.0))
           + 43859779.20 * (max(1.0 - 0.015 * (4.0 + (time.t - 1.0)), 0.0) / 0.9550000000) * (pv(0.0109775000, max(61.0 - (4.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0109775000, 58.0, -1.0))
           + 582028732.70 * (max(1.0 - 0.015 * (3.0 + (time.t - 1.0)), 0.0) / 0.9700000000) * (pv(0.0104741667, max(72.0 - (3.0 + (time.t - 1.0)), 0.0), -1.0) / pv(0.0104741667, 70.0, -1.0)))


  // Interest collected for this distribution. The January pools
  // contribute two months of it at the first one.
  pool_int init 0.0
          next 999357.60 * (max(1.0 - 0.015 * (53.0 + ((time.t) - 1.0)), 0.0) / 0.2050000000) * (pv(0.0131575000, max(61.0 - (53.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0131575000, 8.0, -1.0)) * 0.0131575000
           + 18401017.06 * (max(1.0 - 0.015 * (53.0 + ((time.t) - 1.0)), 0.0) / 0.2050000000) * (pv(0.0111400000, max(72.0 - (53.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0111400000, 19.0, -1.0)) * 0.0111400000
           + 3063342.92 * (max(1.0 - 0.015 * (38.0 + ((time.t) - 1.0)), 0.0) / 0.4300000000) * (pv(0.0115650000, max(65.0 - (38.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0115650000, 27.0, -1.0)) * 0.0115650000
           + 2629247.98 * (max(1.0 - 0.015 * (2.0 + ((time.t) - 1.0)), 0.0) / 0.9700000000) * (pv(0.0102091667, max(48.0 - (2.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0102091667, 46.0, -1.0)) * 0.0102091667
           + 21301021.93 * (max(1.0 - 0.015 * (3.0 + ((time.t) - 1.0)), 0.0) / 0.9550000000) * (pv(0.0107941667, max(61.0 - (3.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0107941667, 58.0, -1.0)) * 0.0107941667
           + 285432214.08 * (max(1.0 - 0.015 * (2.0 + ((time.t) - 1.0)), 0.0) / 0.9700000000) * (pv(0.0105358333, max(72.0 - (2.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0105358333, 70.0, -1.0)) * 0.0105358333
           + (2076350.36 * (max(1.0 - 0.015 * (54.0 + ((time.t) - 1.0)), 0.0) / 0.2050000000) * (pv(0.0129500000, max(61.0 - (54.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0129500000, 8.0, -1.0)) * 0.0129500000 + if((time.t) == 1.0, 2076350.36 * (max(1.0 - 0.015 * (53.0), 0.0) / 0.2050000000) * (pv(0.0129500000, max(61.0 - (53.0), 0.0), -1.0) / pv(0.0129500000, 8.0, -1.0)) * 0.0129500000, 0.0))
           + (37654758.33 * (max(1.0 - 0.015 * (54.0 + ((time.t) - 1.0)), 0.0) / 0.2050000000) * (pv(0.0114466667, max(72.0 - (54.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0114466667, 19.0, -1.0)) * 0.0114466667 + if((time.t) == 1.0, 37654758.33 * (max(1.0 - 0.015 * (53.0), 0.0) / 0.2050000000) * (pv(0.0114466667, max(72.0 - (53.0), 0.0), -1.0) / pv(0.0114466667, 19.0, -1.0)) * 0.0114466667, 0.0))
           + (9848795.79 * (max(1.0 - 0.015 * (41.0 + ((time.t) - 1.0)), 0.0) / 0.4000000000) * (pv(0.0115608333, max(67.0 - (41.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0115608333, 27.0, -1.0)) * 0.0115608333 + if((time.t) == 1.0, 9848795.79 * (max(1.0 - 0.015 * (40.0), 0.0) / 0.4000000000) * (pv(0.0115608333, max(67.0 - (40.0), 0.0), -1.0) / pv(0.0115608333, 27.0, -1.0)) * 0.0115608333, 0.0))
           + (4675311.33 * (max(1.0 - 0.015 * (4.0 + ((time.t) - 1.0)), 0.0) / 0.9550000000) * (pv(0.0106325000, max(49.0 - (4.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0106325000, 46.0, -1.0)) * 0.0106325000 + if((time.t) == 1.0, 4675311.33 * (max(1.0 - 0.015 * (3.0), 0.0) / 0.9550000000) * (pv(0.0106325000, max(49.0 - (3.0), 0.0), -1.0) / pv(0.0106325000, 46.0, -1.0)) * 0.0106325000, 0.0))
           + (43859779.20 * (max(1.0 - 0.015 * (4.0 + ((time.t) - 1.0)), 0.0) / 0.9550000000) * (pv(0.0109775000, max(61.0 - (4.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0109775000, 58.0, -1.0)) * 0.0109775000 + if((time.t) == 1.0, 43859779.20 * (max(1.0 - 0.015 * (3.0), 0.0) / 0.9550000000) * (pv(0.0109775000, max(61.0 - (3.0), 0.0), -1.0) / pv(0.0109775000, 58.0, -1.0)) * 0.0109775000, 0.0))
           + (582028732.70 * (max(1.0 - 0.015 * (3.0 + ((time.t) - 1.0)), 0.0) / 0.9700000000) * (pv(0.0104741667, max(72.0 - (3.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0104741667, 70.0, -1.0)) * 0.0104741667 + if((time.t) == 1.0, 582028732.70 * (max(1.0 - 0.015 * (2.0), 0.0) / 0.9700000000) * (pv(0.0104741667, max(72.0 - (2.0), 0.0), -1.0) / pv(0.0104741667, 70.0, -1.0)) * 0.0104741667, 0.0))


  // What the servicing fee accrues on. One accrual a month, so a
  // January pool carries two of them into the first distribution —
  // the same fact that gives it two payments. This is NOT the
  // opening pool balance, and reading it as one costs eleven
  // published cells and two of the published lives.
  pool_fee init 0.0
          next 999357.60 * (max(1.0 - 0.015 * (53.0 + ((time.t) - 1.0)), 0.0) / 0.2050000000) * (pv(0.0131575000, max(61.0 - (53.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0131575000, 8.0, -1.0))
           + 18401017.06 * (max(1.0 - 0.015 * (53.0 + ((time.t) - 1.0)), 0.0) / 0.2050000000) * (pv(0.0111400000, max(72.0 - (53.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0111400000, 19.0, -1.0))
           + 3063342.92 * (max(1.0 - 0.015 * (38.0 + ((time.t) - 1.0)), 0.0) / 0.4300000000) * (pv(0.0115650000, max(65.0 - (38.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0115650000, 27.0, -1.0))
           + 2629247.98 * (max(1.0 - 0.015 * (2.0 + ((time.t) - 1.0)), 0.0) / 0.9700000000) * (pv(0.0102091667, max(48.0 - (2.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0102091667, 46.0, -1.0))
           + 21301021.93 * (max(1.0 - 0.015 * (3.0 + ((time.t) - 1.0)), 0.0) / 0.9550000000) * (pv(0.0107941667, max(61.0 - (3.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0107941667, 58.0, -1.0))
           + 285432214.08 * (max(1.0 - 0.015 * (2.0 + ((time.t) - 1.0)), 0.0) / 0.9700000000) * (pv(0.0105358333, max(72.0 - (2.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0105358333, 70.0, -1.0))
           + (2076350.36 * (max(1.0 - 0.015 * (54.0 + ((time.t) - 1.0)), 0.0) / 0.2050000000) * (pv(0.0129500000, max(61.0 - (54.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0129500000, 8.0, -1.0)) + if((time.t) == 1.0, 2076350.36 * (max(1.0 - 0.015 * (53.0), 0.0) / 0.2050000000) * (pv(0.0129500000, max(61.0 - (53.0), 0.0), -1.0) / pv(0.0129500000, 8.0, -1.0)), 0.0))
           + (37654758.33 * (max(1.0 - 0.015 * (54.0 + ((time.t) - 1.0)), 0.0) / 0.2050000000) * (pv(0.0114466667, max(72.0 - (54.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0114466667, 19.0, -1.0)) + if((time.t) == 1.0, 37654758.33 * (max(1.0 - 0.015 * (53.0), 0.0) / 0.2050000000) * (pv(0.0114466667, max(72.0 - (53.0), 0.0), -1.0) / pv(0.0114466667, 19.0, -1.0)), 0.0))
           + (9848795.79 * (max(1.0 - 0.015 * (41.0 + ((time.t) - 1.0)), 0.0) / 0.4000000000) * (pv(0.0115608333, max(67.0 - (41.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0115608333, 27.0, -1.0)) + if((time.t) == 1.0, 9848795.79 * (max(1.0 - 0.015 * (40.0), 0.0) / 0.4000000000) * (pv(0.0115608333, max(67.0 - (40.0), 0.0), -1.0) / pv(0.0115608333, 27.0, -1.0)), 0.0))
           + (4675311.33 * (max(1.0 - 0.015 * (4.0 + ((time.t) - 1.0)), 0.0) / 0.9550000000) * (pv(0.0106325000, max(49.0 - (4.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0106325000, 46.0, -1.0)) + if((time.t) == 1.0, 4675311.33 * (max(1.0 - 0.015 * (3.0), 0.0) / 0.9550000000) * (pv(0.0106325000, max(49.0 - (3.0), 0.0), -1.0) / pv(0.0106325000, 46.0, -1.0)), 0.0))
           + (43859779.20 * (max(1.0 - 0.015 * (4.0 + ((time.t) - 1.0)), 0.0) / 0.9550000000) * (pv(0.0109775000, max(61.0 - (4.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0109775000, 58.0, -1.0)) + if((time.t) == 1.0, 43859779.20 * (max(1.0 - 0.015 * (3.0), 0.0) / 0.9550000000) * (pv(0.0109775000, max(61.0 - (3.0), 0.0), -1.0) / pv(0.0109775000, 58.0, -1.0)), 0.0))
           + (582028732.70 * (max(1.0 - 0.015 * (3.0 + ((time.t) - 1.0)), 0.0) / 0.9700000000) * (pv(0.0104741667, max(72.0 - (3.0 + ((time.t) - 1.0)), 0.0), -1.0) / pv(0.0104741667, 70.0, -1.0)) + if((time.t) == 1.0, 582028732.70 * (max(1.0 - 0.015 * (2.0), 0.0) / 0.9700000000) * (pv(0.0104741667, max(72.0 - (2.0), 0.0), -1.0) / pv(0.0104741667, 70.0, -1.0)), 0.0))


  // Class A-1, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_a1 init 182000000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.container.trust.pool_bal) <= inputs.call_threshold, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.pool_bal) - 5059849.65,
          max(((prev.container.trust.pool_bal) - max(0.1475 * (prev.container.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.pool_bal)) - max(((prev.container.trust.pool_int)
           - (prev.container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.container.trust.bal_a1 * 0.0007916667
           + prev.container.trust.bal_a2 * 0.0011331000
           + prev.container.trust.bal_a3 * 0.0015916667
           + prev.container.trust.bal_b * 0.0019583333
           + prev.container.trust.bal_c * 0.0024000000
           + prev.container.trust.bal_d * 0.0026916667
           + prev.container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (0.0), 0.0), prev.container.trust.bal_a1)), 0.0))


  // Class A-2, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_a2 init 305000000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.container.trust.pool_bal) <= inputs.call_threshold, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.pool_bal) - 5059849.65,
          max(((prev.container.trust.pool_bal) - max(0.1475 * (prev.container.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.pool_bal)) - max(((prev.container.trust.pool_int)
           - (prev.container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.container.trust.bal_a1 * 0.0007916667
           + prev.container.trust.bal_a2 * 0.0011331000
           + prev.container.trust.bal_a3 * 0.0015916667
           + prev.container.trust.bal_b * 0.0019583333
           + prev.container.trust.bal_c * 0.0024000000
           + prev.container.trust.bal_d * 0.0026916667
           + prev.container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.container.trust.bal_a1), 0.0), prev.container.trust.bal_a2)), 0.0))


  // Class A-3, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_a3 init 189000000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.container.trust.pool_bal) <= inputs.call_threshold, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.pool_bal) - 5059849.65,
          max(((prev.container.trust.pool_bal) - max(0.1475 * (prev.container.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.pool_bal)) - max(((prev.container.trust.pool_int)
           - (prev.container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.container.trust.bal_a1 * 0.0007916667
           + prev.container.trust.bal_a2 * 0.0011331000
           + prev.container.trust.bal_a3 * 0.0015916667
           + prev.container.trust.bal_b * 0.0019583333
           + prev.container.trust.bal_c * 0.0024000000
           + prev.container.trust.bal_d * 0.0026916667
           + prev.container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.container.trust.bal_a1 + prev.container.trust.bal_a2), 0.0), prev.container.trust.bal_a3)), 0.0))


  // Class B, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_b init 73370000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.container.trust.pool_bal) <= inputs.call_threshold, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.pool_bal) - 5059849.65,
          max(((prev.container.trust.pool_bal) - max(0.1475 * (prev.container.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.pool_bal)) - max(((prev.container.trust.pool_int)
           - (prev.container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.container.trust.bal_a1 * 0.0007916667
           + prev.container.trust.bal_a2 * 0.0011331000
           + prev.container.trust.bal_a3 * 0.0015916667
           + prev.container.trust.bal_b * 0.0019583333
           + prev.container.trust.bal_c * 0.0024000000
           + prev.container.trust.bal_d * 0.0026916667
           + prev.container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3), 0.0), prev.container.trust.bal_b)), 0.0))


  // Class C, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_c init 91080000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.container.trust.pool_bal) <= inputs.call_threshold, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.pool_bal) - 5059849.65,
          max(((prev.container.trust.pool_bal) - max(0.1475 * (prev.container.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.pool_bal)) - max(((prev.container.trust.pool_int)
           - (prev.container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.container.trust.bal_a1 * 0.0007916667
           + prev.container.trust.bal_a2 * 0.0011331000
           + prev.container.trust.bal_a3 * 0.0015916667
           + prev.container.trust.bal_b * 0.0019583333
           + prev.container.trust.bal_c * 0.0024000000
           + prev.container.trust.bal_d * 0.0026916667
           + prev.container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b), 0.0), prev.container.trust.bal_c)), 0.0))


  // Class D, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_d init 89550000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.container.trust.pool_bal) <= inputs.call_threshold, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.pool_bal) - 5059849.65,
          max(((prev.container.trust.pool_bal) - max(0.1475 * (prev.container.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.pool_bal)) - max(((prev.container.trust.pool_int)
           - (prev.container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.container.trust.bal_a1 * 0.0007916667
           + prev.container.trust.bal_a2 * 0.0011331000
           + prev.container.trust.bal_a3 * 0.0015916667
           + prev.container.trust.bal_b * 0.0019583333
           + prev.container.trust.bal_c * 0.0024000000
           + prev.container.trust.bal_d * 0.0026916667
           + prev.container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c), 0.0), prev.container.trust.bal_d)), 0.0))


  // Class E, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_e init 23780000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.container.trust.pool_bal) <= inputs.call_threshold, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.pool_bal) - 5059849.65,
          max(((prev.container.trust.pool_bal) - max(0.1475 * (prev.container.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.pool_bal)) - max(((prev.container.trust.pool_int)
           - (prev.container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.container.trust.bal_a1 * 0.0007916667
           + prev.container.trust.bal_a2 * 0.0011331000
           + prev.container.trust.bal_a3 * 0.0015916667
           + prev.container.trust.bal_b * 0.0019583333
           + prev.container.trust.bal_c * 0.0024000000
           + prev.container.trust.bal_d * 0.0026916667
           + prev.container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d), 0.0), prev.container.trust.bal_e)), 0.0))

}

entity asset p01 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p02 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p03 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p04 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p05 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p06 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p07 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p08 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p09 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p10 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p11 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity asset p12 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}



contract credit.pool_level_pay.p01 on entity asset.p01 {
  term 2017-02..2017-09
  terms {
    balance = 999357.6
    rate = 0.15789
    term_months = 8
    age_months = 53
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p02 on entity asset.p02 {
  term 2017-02..2018-08
  terms {
    balance = 18401017.06
    rate = 0.13368
    term_months = 19
    age_months = 53
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p03 on entity asset.p03 {
  term 2017-02..2019-04
  terms {
    balance = 3063342.92
    rate = 0.13878
    term_months = 27
    age_months = 38
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p04 on entity asset.p04 {
  term 2017-02..2020-11
  terms {
    balance = 2629247.98
    rate = 0.12251
    term_months = 46
    age_months = 2
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p05 on entity asset.p05 {
  term 2017-02..2021-11
  terms {
    balance = 21301021.93
    rate = 0.12953
    term_months = 58
    age_months = 3
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p06 on entity asset.p06 {
  term 2017-02..2022-11
  terms {
    balance = 285432214.08
    rate = 0.12643
    term_months = 70
    age_months = 2
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p07 on entity asset.p07 {
  term 2017-01..2017-08
  terms {
    balance = 2076350.36
    rate = 0.1554
    term_months = 8
    age_months = 53
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p08 on entity asset.p08 {
  term 2017-01..2018-07
  terms {
    balance = 37654758.33
    rate = 0.13736
    term_months = 19
    age_months = 53
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p09 on entity asset.p09 {
  term 2017-01..2019-03
  terms {
    balance = 9848795.79
    rate = 0.13873
    term_months = 27
    age_months = 40
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p10 on entity asset.p10 {
  term 2017-01..2020-10
  terms {
    balance = 4675311.33
    rate = 0.12759
    term_months = 46
    age_months = 3
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p11 on entity asset.p11 {
  term 2017-01..2021-10
  terms {
    balance = 43859779.2
    rate = 0.13173
    term_months = 58
    age_months = 3
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.pool_level_pay.p12 on entity asset.p12 {
  term 2017-01..2022-10
  terms {
    balance = 582028732.7
    rate = 0.12569
    term_months = 70
    age_months = 2
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

entity party servicer : Credit.Party.Servicer { name = "Servicer" }
entity party trustee : Credit.Party.Issuer { name = "Trustee, owner trustee, collateral agent and asset representations reviewer" }
entity party certificate : Credit.Party.Investor { name = "Certificateholder" }
entity party a1_holders : Credit.Party.Investor { name = "Class A-1 noteholders" }
entity party a2_holders : Credit.Party.Investor { name = "Class A-2 noteholders" }
entity party a3_holders : Credit.Party.Investor { name = "Class A-3 noteholders" }
entity party b_holders : Credit.Party.Investor { name = "Class B noteholders" }
entity party c_holders : Credit.Party.Investor { name = "Class C noteholders" }
entity party d_holders : Credit.Party.Investor { name = "Class D noteholders" }
entity party e_holders : Credit.Party.Investor { name = "Class E noteholders" }

// ---------------------------------------------------------------------------
// The initial pool balance at the cutoff date, and the reserve required
// against it. The prospectus states the reserve as a RULE — 2.0% of the
// initial pool — so the model states it as one: 20,239,398.5856, the product
// rather than a rounded dollar amount.
assume initial_pool     = 1011969929.28
assume reserve_required = 0.02 * inputs.initial_pool

// The clean-up call threshold, stated the way the prospectus states it (p. 83)
// — 10% of the initial pool, the product rather than a rounded dollar amount.
// It was written out as 101196992.93 in thirty places, which is the shape
// clause 19's reserve had before it became an account.
assume call_threshold   = 0.10 * inputs.initial_pool

// ---------------------------------------------------------------------------
// THE RESERVE ACCOUNT (clause 19), funded at closing.
//
// The reserve is not inert here even though it is never drawn. The Required
// Pro Forma Note Balance — the target the turbo runs toward — is 14.75% of the
// pool "less the amount required on deposit in the reserve account"
// (glossary), so the reserve is what the target is stated against, and every
// class's retirement date depends on it. That is why it is an account whose
// balance the target reads, rather than a number repeated wherever the target
// appears.
//
// FUNDED AT CLOSING, WHICH IS NOT A DISTRIBUTION. The reserve comes out of
// note proceeds before the first collection period, so it cannot be funded
// from the waterfall: the waterfall allocates collections, and taking the
// reserve out of them would spend cash the deal never spent. `from` is the
// account's own inflow — the one that is not an allocation — and it fires
// once, at period 0. Clause 19 below is then the top-up it is.
account reserve {
  from if(time.t == 0.0, inputs.reserve_required, 0.0)
}


// ---------------------------------------------------------------------------
// THE TRUST IS AN SPV, AND IT WINDS UP WHEN IT IS EMPTY.
//
// Not when someone says so. The condition is the trust's own holding — the
// pools it contains, summed — reaching zero, so the state is DERIVED from what
// the container holds rather than asserted alongside it.
//
// It lands one period after the pools retire, and that is correct rather than a
// lag to engineer away: state is evaluated as a period opens, so the trust can
// only see a settled pool. Terminating it any earlier would end the trust
// before the pools' last period of activity had been counted.
lifecycle trust_life {
  initial amortizing
  state amortizing, wound_up
  amortizing -> wound_up when container.trust.surviving == 0.0
}

// ---------------------------------------------------------------------------
// THE CLEAN-UP CALL, EXERCISED.
//
// Once the pool falls to 10% or less of its original balance the servicer may
// purchase the receivables (SOURCE.md; prospectus p. 83), and the published
// decrement tables assume it does — which is why they stop. Every class is at
// zero from the distribution the purchase pays for, and the four speeds each
// end on their own date.
//
// IT IS AN OCCURRENCE, NOT A CONDITION THE POOL SITS IN. `docs/36` §2.2
// retired `called` as a state for exactly this reason, and the credit pack's
// pool machine carries the edge it left in its place — `amortizing ->
// retired`, with no way back. Each of the twelve assumed pools carries that
// machine, because each is a `Credit.Asset.LoanPool`, and it is THEIR
// receivables the servicer buys. The trust is the container they sit in; it
// retires because they did, further down.
//
// THE STATE IS EVALUATED AS THE PERIOD OPENS. A pool is carried into a period,
// collects, and the period ends. When the pool the trust carries IN has fallen
// to the threshold, the purchase has already settled on the distribution just
// made. So the guard reads `pool_prior` — the balance as the period opened —
// and the transition lands the period after the redeeming distribution, which
// is where the published tables put the first zero.
//
// ONCE-NESS IS TOPOLOGY, NOT A LATCH (`docs/34` D1). The condition holds for
// the rest of the book; the event fires on its rising edge, and could not fire
// twice in any case, because nothing leaves `retired`.
//
// AND THE CASH STOPS BECAUSE THE POOL IS GONE, not because anything switches
// it off. `credit.pool_level_pay` carries one piece of state — a surviving
// fraction, `field_init 1` and `field_next prev * (...)` — and every one of the
// six streams it lowers is the pool's balance times an amortization factor
// times that fraction. Setting it to zero settles it there, the recurrence
// resumes from zero, and all six go to zero for good. A purchased pool has no
// surviving balance, and its state says so; nothing is being suppressed.
//
// The alternative was `deactivate stream` on all seventy-two lowered streams.
// It produced identical cash and left the model asserting that 48.78% of pool
// p01 was still performing while its streams were silent — a declared state
// and a behavior that disagree. This spelling cannot drift that way, because
// the same fact drives both.
//
// The pack should arguably do this itself: it declares the `retired` state and
// it declares the survival recurrence, and connects them nowhere, so `retired`
// is a state a pool can enter with no consequence to its cash. See NOTES.md.
event clean_up_call when container.trust.pool_prior <= inputs.call_threshold {
  set entity asset.p01.status = "retired"
  set entity asset.p01.credit_level_pay_survival_p01 = 0
  set entity asset.p01.credit_level_pay_survival_lag_p01 = 0
  set entity asset.p02.status = "retired"
  set entity asset.p02.credit_level_pay_survival_p02 = 0
  set entity asset.p02.credit_level_pay_survival_lag_p02 = 0
  set entity asset.p03.status = "retired"
  set entity asset.p03.credit_level_pay_survival_p03 = 0
  set entity asset.p03.credit_level_pay_survival_lag_p03 = 0
  set entity asset.p04.status = "retired"
  set entity asset.p04.credit_level_pay_survival_p04 = 0
  set entity asset.p04.credit_level_pay_survival_lag_p04 = 0
  set entity asset.p05.status = "retired"
  set entity asset.p05.credit_level_pay_survival_p05 = 0
  set entity asset.p05.credit_level_pay_survival_lag_p05 = 0
  set entity asset.p06.status = "retired"
  set entity asset.p06.credit_level_pay_survival_p06 = 0
  set entity asset.p06.credit_level_pay_survival_lag_p06 = 0
  set entity asset.p07.status = "retired"
  set entity asset.p07.credit_level_pay_survival_p07 = 0
  set entity asset.p07.credit_level_pay_survival_lag_p07 = 0
  set entity asset.p08.status = "retired"
  set entity asset.p08.credit_level_pay_survival_p08 = 0
  set entity asset.p08.credit_level_pay_survival_lag_p08 = 0
  set entity asset.p09.status = "retired"
  set entity asset.p09.credit_level_pay_survival_p09 = 0
  set entity asset.p09.credit_level_pay_survival_lag_p09 = 0
  set entity asset.p10.status = "retired"
  set entity asset.p10.credit_level_pay_survival_p10 = 0
  set entity asset.p10.credit_level_pay_survival_lag_p10 = 0
  set entity asset.p11.status = "retired"
  set entity asset.p11.credit_level_pay_survival_p11 = 0
  set entity asset.p11.credit_level_pay_survival_lag_p11 = 0
  set entity asset.p12.status = "retired"
  set entity asset.p12.credit_level_pay_survival_p12 = 0
  set entity asset.p12.credit_level_pay_survival_lag_p12 = 0
}

// ---------------------------------------------------------------------------
// The priority of payments — all twenty-two clauses, in the prospectus's order
// (pp. 77-78). Clauses 4, 5, 7, 8, 10, 11, 13, 14, 16 and 17 are the parity
// and final-maturity steps: with no defaults, losses or repurchases assumed,
// the pool always covers the notes and every class retires long before its
// final scheduled date, so each of them pays nothing. They are written out
// because the deal has them.
// ---------------------------------------------------------------------------
// NOT `from available`, for two reasons the binding cannot yet carry: the
// first distribution draws TWO periods of collections (the January pools pay
// in period 0 and distribute in period 1, and `available` is one period), and
// the clean-up call adds a redemption price that is not stream cash. Both are
// stated in the windowed pot below.
waterfall notes.distribution on entity container.trust {
  schedule every month from 2017-02 to 2022-11

  // Collections for this distribution, and at the first one the January
  // pools' extra month. The redemption price joins the pot on the distribution
  // the clean-up call is first available — the period the pool crosses the
  // threshold, which is two reads of a declared field rather than a pattern.
  // A misspelled field is refused; a selector matching nothing folds to zero
  // in silence, and that silence would stop the largest distribution here.
  from series_sum("credit.pool.sched_principal.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + series_sum("credit.pool.prepay.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + series_sum("credit.pool.interest.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + if(container.trust.pool_prior > inputs.call_threshold
            and container.trust.pool_bal <= inputs.call_threshold, container.trust.pool_bal, 0.0)

  //  1. the servicer. The pack carries the fee as a negative series on each
  //     pool, so the step is its sum with the sign turned round.
  pay servicing to party.servicer =
        -(series_sum("credit.pool.servicing.*", if(time.t == 1.0, 0.0, time.t), time.t))

  //  2. the trustee, owner trustee, collateral agent and the asset
  //     representations reviewer, inside their annual caps.
  pay trustee_fees to party.trustee = 625.0

  //  3. interest on the Class A-1 Notes (clause 3 pays the Class A classes pari passu)
  pay a1_interest to party.a1_holders = container.trust.bal_a1 * 0.0007916667

  //  3. interest on the Class A-2 Notes
  pay a2_interest to party.a2_holders = container.trust.bal_a2 * 0.0011331000

  //  3. interest on the Class A-3 Notes
  pay a3_interest to party.a3_holders = container.trust.bal_a3 * 0.0015916667

  //  4. principal to reduce the Class A balance to the pool
  //     balance. THE TWO SIDES ARE MEASURED ON THE SAME DATE: the
  //     classes carry these balances into the distribution, so the
  //     pool they are compared against is the one it carried in too.
  //     Against the pool after this period's collections the step
  //     fires every month once principal collected exceeds
  //     overcollateralization, and what it would be curing is not
  //     undercollateralization but the pay-down about to happen at
  //     clause 18. The prospectus says what these clauses are for —
  //     'principal payments made to cure this undercollateralization,
  //     if any then exists' — and with no losses assumed, none does.
  pay a3_parity to party.a3_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3) - container.trust.pool_prior, 0.0)

  //  5. the remaining Class A balance on its final scheduled date
  pay a3_final to party.a3_holders = if(time.t >= 12.0, container.trust.bal_a1, 0.0) + if(time.t >= 39.0, container.trust.bal_a2, 0.0) + if(time.t >= 54.0, container.trust.bal_a3, 0.0)

  //  6. interest on the Class B Notes
  pay b_interest to party.b_holders = container.trust.bal_b * 0.0019583333

  //  7. principal to reduce the Class B balance and everything senior to the pool
  //     balance. THE TWO SIDES ARE MEASURED ON THE SAME DATE: the
  //     classes carry these balances into the distribution, so the
  //     pool they are compared against is the one it carried in too.
  //     Against the pool after this period's collections the step
  //     fires every month once principal collected exceeds
  //     overcollateralization, and what it would be curing is not
  //     undercollateralization but the pay-down about to happen at
  //     clause 18. The prospectus says what these clauses are for —
  //     'principal payments made to cure this undercollateralization,
  //     if any then exists' — and with no losses assumed, none does.
  pay b_parity to party.b_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b) - container.trust.pool_prior, 0.0)

  //  8. the remaining Class B balance on its final scheduled date
  pay b_final to party.b_holders = if(time.t >= 60.0, container.trust.bal_b, 0.0)

  //  9. interest on the Class C Notes
  pay c_interest to party.c_holders = container.trust.bal_c * 0.0024000000

  //  10. principal to reduce the Class C balance and everything senior to the pool
  //     balance. THE TWO SIDES ARE MEASURED ON THE SAME DATE: the
  //     classes carry these balances into the distribution, so the
  //     pool they are compared against is the one it carried in too.
  //     Against the pool after this period's collections the step
  //     fires every month once principal collected exceeds
  //     overcollateralization, and what it would be curing is not
  //     undercollateralization but the pay-down about to happen at
  //     clause 18. The prospectus says what these clauses are for —
  //     'principal payments made to cure this undercollateralization,
  //     if any then exists' — and with no losses assumed, none does.
  pay c_parity to party.c_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c) - container.trust.pool_prior, 0.0)

  //  11. the remaining Class C balance on its final scheduled date
  pay c_final to party.c_holders = if(time.t >= 66.0, container.trust.bal_c, 0.0)

  //  12. interest on the Class D Notes
  pay d_interest to party.d_holders = container.trust.bal_d * 0.0026916667

  //  13. principal to reduce the Class D balance and everything senior to the pool
  //     balance. THE TWO SIDES ARE MEASURED ON THE SAME DATE: the
  //     classes carry these balances into the distribution, so the
  //     pool they are compared against is the one it carried in too.
  //     Against the pool after this period's collections the step
  //     fires every month once principal collected exceeds
  //     overcollateralization, and what it would be curing is not
  //     undercollateralization but the pay-down about to happen at
  //     clause 18. The prospectus says what these clauses are for —
  //     'principal payments made to cure this undercollateralization,
  //     if any then exists' — and with no losses assumed, none does.
  pay d_parity to party.d_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d) - container.trust.pool_prior, 0.0)

  //  14. the remaining Class D balance on its final scheduled date
  pay d_final to party.d_holders = if(time.t >= 71.0, container.trust.bal_d, 0.0)

  //  15. interest on the Class E Notes
  pay e_interest to party.e_holders = container.trust.bal_e * 0.0000000000

  //  16. principal to reduce the Class E balance and everything senior to the pool
  //     balance. THE TWO SIDES ARE MEASURED ON THE SAME DATE: the
  //     classes carry these balances into the distribution, so the
  //     pool they are compared against is the one it carried in too.
  //     Against the pool after this period's collections the step
  //     fires every month once principal collected exceeds
  //     overcollateralization, and what it would be curing is not
  //     undercollateralization but the pay-down about to happen at
  //     clause 18. The prospectus says what these clauses are for —
  //     'principal payments made to cure this undercollateralization,
  //     if any then exists' — and with no losses assumed, none does.
  pay e_parity to party.e_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - container.trust.pool_prior, 0.0)

  //  17. the remaining Class E balance on its final scheduled date
  pay e_final to party.e_holders = if(time.t >= 90.0, container.trust.bal_e, 0.0)

  // 18. the Noteholders' Principal Distributable Amount — principal
  //     collected LESS the Step-Down Amount, to the most senior class
  //     outstanding and then down.

  pay a1_principal to party.a1_holders =
        min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (0.0), 0.0), container.trust.bal_a1)

  pay a2_principal to party.a2_holders =
        min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1), 0.0), container.trust.bal_a2)

  pay a3_principal to party.a3_holders =
        min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2), 0.0), container.trust.bal_a3)

  pay b_principal to party.b_holders =
        min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3), 0.0), container.trust.bal_b)

  pay c_principal to party.c_holders =
        min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b), 0.0), container.trust.bal_c)

  pay d_principal to party.d_holders =
        min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c), 0.0), container.trust.bal_d)

  pay e_principal to party.e_holders =
        min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d), 0.0), container.trust.bal_e)

  // 19. the reserve account. This step is the TOP-UP: whatever the
  //     account is short of its required amount, restored out of
  //     collections ahead of principal. It pays nothing at any period on
  //     this deal — no losses are assumed, so the reserve is never drawn
  //     and the shortfall is always zero — and it pays nothing BECAUSE
  //     the balance is at target, which is what the clause says. Were the
  //     reserve ever drawn, this step would restore it.
  pay reserve_topup to account reserve =
        max(0.0, inputs.reserve_required - prev.reserve)

  // 20. the Accelerated Principal Amount: excess cash turboing the
  //     notes toward the target, and at the clean-up call the whole
  //     remaining balance.

  pay a1_accelerated to party.a1_holders =
        max(min(max((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (0.0), 0.0), container.trust.bal_a1)
            - (min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (0.0), 0.0), container.trust.bal_a1)), 0.0)

  pay a2_accelerated to party.a2_holders =
        max(min(max((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (container.trust.bal_a1), 0.0), container.trust.bal_a2)
            - (min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1), 0.0), container.trust.bal_a2)), 0.0)

  pay a3_accelerated to party.a3_holders =
        max(min(max((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (container.trust.bal_a1 + container.trust.bal_a2), 0.0), container.trust.bal_a3)
            - (min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2), 0.0), container.trust.bal_a3)), 0.0)

  pay b_accelerated to party.b_holders =
        max(min(max((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3), 0.0), container.trust.bal_b)
            - (min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3), 0.0), container.trust.bal_b)), 0.0)

  pay c_accelerated to party.c_holders =
        max(min(max((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b), 0.0), container.trust.bal_c)
            - (min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b), 0.0), container.trust.bal_c)), 0.0)

  pay d_accelerated to party.d_holders =
        max(min(max((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c), 0.0), container.trust.bal_d)
            - (min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c), 0.0), container.trust.bal_d)), 0.0)

  pay e_accelerated to party.e_holders =
        max(min(max((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d), 0.0), container.trust.bal_e)
            - (min(max((min((if((container.trust.pool_bal) <= inputs.call_threshold, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((container.trust.pool_bal) - 5059849.65,
          max(((container.trust.pool_bal) - max(0.1475 * (container.trust.pool_bal) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((container.trust.pool_prior) - (container.trust.pool_bal)) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((container.trust.pool_prior) - (container.trust.pool_bal))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d), 0.0), container.trust.bal_e)), 0.0)

  // 21. trustee amounts above the cap that held them at clause 2
  pay trustee_excess to party.trustee = owed.trustee_fees - paid.trustee_fees

  // 22. everything that survives — the step-down release and the
  //     excess cash the turbo did not need.
  pay residual to party.certificate = remaining
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.03}}
```

## Verified results

Checked period by period: **31 series** across **64 periods** — **1960 values** in all, each within ±1.00 of the reference.

- `container.trust.bal_a1`
- `container.trust.bal_a2`
- `container.trust.bal_a3`
- `container.trust.bal_b`
- `container.trust.bal_c`
- `container.trust.bal_d`
- `container.trust.bal_e`
- `notes.distribution.servicing`
- `notes.distribution.trustee_fees`
- `notes.distribution.a1_interest`
- `notes.distribution.a2_interest`
- `notes.distribution.a3_interest`
- `notes.distribution.b_interest`
- `notes.distribution.c_interest`
- `notes.distribution.d_interest`
- `notes.distribution.e_interest`
- `notes.distribution.a1_principal`
- `notes.distribution.a2_principal`
- `notes.distribution.a3_principal`
- `notes.distribution.b_principal`
- `notes.distribution.c_principal`
- `notes.distribution.d_principal`
- `notes.distribution.e_principal`
- `notes.distribution.a1_accelerated`
- `notes.distribution.a2_accelerated`
- `notes.distribution.a3_accelerated`
- `notes.distribution.b_accelerated`
- `notes.distribution.c_accelerated`
- `notes.distribution.d_accelerated`
- `notes.distribution.e_accelerated`
- `notes.distribution.residual`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.total` | 1,115,050,449.22 | ±1 |
