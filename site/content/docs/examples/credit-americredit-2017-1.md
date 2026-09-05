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

All twenty-two clauses are written out. Ten of them — the parity steps and the
final-maturity steps — pay nothing at every period and every speed, because the
pool always covers the notes and every class retires years early. They are there
because the deal has them.

## The result

The model agrees with the reference to **5 cents**, across all seven classes,
every clause of the waterfall and all 63 periods — 32 asserted series, not the
balances alone. Against the published grid, the reference reproduces:

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
- **Anything after the clean-up call.** The call retires the notes at period 47
  and there is no trust left to distribute from, so the cash columns end there.
  The loans are repurchased at the next period — one event at the trust,
  reading the balance it holds — and produce nothing more inside the model.
- **Mutation testing.** `docs/20` §3.3 asks for it and it has not been run. The
  hole `docs/20` §3.2 warns about is present by construction here: the
  certificateholder's step-down release absorbs whatever the notes are not
  paid, so a residual assertion is one-sided.

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

`model.total` is a regression anchor from this model, not an external figure.
Every external assertion is the independent reference against the published
grid; `expected.csv` holds that reference's per-period class balances.

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
// THE POOL BALANCE IS NOT STATED HERE AT ALL. Each loan's balance is an
// account its contract opens and its own collections move (docs/42), and the
// trust's balance is the fold of its twelve loans' through `part of` — read
// as `prev.container.trust.balance`, the balance the pool carried into the
// period. The balance after this period's collections is that opening less
// the principal the loans paid this period, which the waterfall computes from
// the loans' own series where the prospectus measures against it. The
// interest and servicing bases stay as closed forms (`pool_int`, `pool_fee`):
// they need the balance at each loan's own accrual dates, which is the
// pack's arithmetic and not a sum of accounts.
//
// Reference: EXTERNAL. Rule 424(b)(5) prospectus dated 21 February 2017,
// pp. 57-62 — see SOURCE.md. Run at 1.50% ABS; the other three published
// speeds are this model with one term changed.


entity container trust : Container.SPV {
  // What the pool carried into the period: the trust's balance is the fold of
  // its twelve loans' balances (docs/42 §3.4), read as the prior close. The
  // field exists so the class balances, which read one period back, can see
  // what the pool carried into the period before.
  // At the first distribution the pool "carried in" is the cutoff balance:
  // the January loans have already paid once in period 0, and that payment
  // belongs to the first distribution rather than to the balance the classes
  // are measured against.
  pool_prior init 1011969929.28
             next if(time.t <= 1.0, prev, prev.container.trust.balance)


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
                max(prev - (min(max((if((prev.container.trust.balance) <= 101196992.93, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.balance) - 5059849.65,
          max(((prev.container.trust.balance) - max(0.1475 * (prev.container.trust.balance) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.balance)) - max(((prev.container.trust.pool_int)
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
                max(prev - (min(max((if((prev.container.trust.balance) <= 101196992.93, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.balance) - 5059849.65,
          max(((prev.container.trust.balance) - max(0.1475 * (prev.container.trust.balance) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.balance)) - max(((prev.container.trust.pool_int)
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
                max(prev - (min(max((if((prev.container.trust.balance) <= 101196992.93, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.balance) - 5059849.65,
          max(((prev.container.trust.balance) - max(0.1475 * (prev.container.trust.balance) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.balance)) - max(((prev.container.trust.pool_int)
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
                max(prev - (min(max((if((prev.container.trust.balance) <= 101196992.93, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.balance) - 5059849.65,
          max(((prev.container.trust.balance) - max(0.1475 * (prev.container.trust.balance) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.balance)) - max(((prev.container.trust.pool_int)
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
                max(prev - (min(max((if((prev.container.trust.balance) <= 101196992.93, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.balance) - 5059849.65,
          max(((prev.container.trust.balance) - max(0.1475 * (prev.container.trust.balance) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.balance)) - max(((prev.container.trust.pool_int)
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
                max(prev - (min(max((if((prev.container.trust.balance) <= 101196992.93, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.balance) - 5059849.65,
          max(((prev.container.trust.balance) - max(0.1475 * (prev.container.trust.balance) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.balance)) - max(((prev.container.trust.pool_int)
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
                max(prev - (min(max((if((prev.container.trust.balance) <= 101196992.93, (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e),
         max((prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - (min((prev.container.trust.balance) - 5059849.65,
          max(((prev.container.trust.balance) - max(0.1475 * (prev.container.trust.balance) - prev.reserve, 0.0)),
              (prev.container.trust.bal_a1 + prev.container.trust.bal_a2 + prev.container.trust.bal_a3 + prev.container.trust.bal_b + prev.container.trust.bal_c + prev.container.trust.bal_d + prev.container.trust.bal_e) - ((prev.container.trust.pool_prior) - (prev.container.trust.balance)) - max(((prev.container.trust.pool_int)
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

entity asset p01 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p02 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p03 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p04 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p05 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p06 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p07 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p08 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p09 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p10 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p11 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}

entity asset p12 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of container.trust
}



contract credit.loan.p01 on entity asset.p01 {
  term 2017-02..2017-09
  terms {
    principal = 999357.6
    interest_rate = 0.15789
    term_months = 8
    age_months = 53
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p02 on entity asset.p02 {
  term 2017-02..2018-08
  terms {
    principal = 18401017.06
    interest_rate = 0.13368
    term_months = 19
    age_months = 53
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p03 on entity asset.p03 {
  term 2017-02..2019-04
  terms {
    principal = 3063342.92
    interest_rate = 0.13878
    term_months = 27
    age_months = 38
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p04 on entity asset.p04 {
  term 2017-02..2020-11
  terms {
    principal = 2629247.98
    interest_rate = 0.12251
    term_months = 46
    age_months = 2
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p05 on entity asset.p05 {
  term 2017-02..2021-11
  terms {
    principal = 21301021.93
    interest_rate = 0.12953
    term_months = 58
    age_months = 3
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p06 on entity asset.p06 {
  term 2017-02..2022-11
  terms {
    principal = 285432214.08
    interest_rate = 0.12643
    term_months = 70
    age_months = 2
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p07 on entity asset.p07 {
  term 2017-01..2017-08
  terms {
    principal = 2076350.36
    interest_rate = 0.1554
    term_months = 8
    age_months = 53
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p08 on entity asset.p08 {
  term 2017-01..2018-07
  terms {
    principal = 37654758.33
    interest_rate = 0.13736
    term_months = 19
    age_months = 53
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p09 on entity asset.p09 {
  term 2017-01..2019-03
  terms {
    principal = 9848795.79
    interest_rate = 0.13873
    term_months = 27
    age_months = 40
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p10 on entity asset.p10 {
  term 2017-01..2020-10
  terms {
    principal = 4675311.33
    interest_rate = 0.12759
    term_months = 46
    age_months = 3
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p11 on entity asset.p11 {
  term 2017-01..2021-10
  terms {
    principal = 43859779.2
    interest_rate = 0.13173
    term_months = 58
    age_months = 3
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

contract credit.loan.p12 on entity asset.p12 {
  term 2017-01..2022-10
  terms {
    principal = 582028732.7
    interest_rate = 0.12569
    term_months = 70
    age_months = 2
    abs_speed = 0.015
    servicing_fee = 0.0225
  }
}

// THE CLEAN-UP CALL REPURCHASES THE COLLATERAL. The redemption price joins
// the pot at the distribution where the pool first falls to 10% (the
// waterfall's own test, below); from the next period the loans belong to the
// servicer, so the trust collects nothing more. One event, at the trust,
// reading the fold; each loan's machine writes the balance off on
// `repurchased` (docs/42 §3.5).
event clean_up_call when prev.container.trust.balance <= 101196992.93 {
  set entity asset.p01.status = "repurchased"
  set entity asset.p02.status = "repurchased"
  set entity asset.p03.status = "repurchased"
  set entity asset.p04.status = "repurchased"
  set entity asset.p05.status = "repurchased"
  set entity asset.p06.status = "repurchased"
  set entity asset.p07.status = "repurchased"
  set entity asset.p08.status = "repurchased"
  set entity asset.p09.status = "repurchased"
  set entity asset.p10.status = "repurchased"
  set entity asset.p11.status = "repurchased"
  set entity asset.p12.status = "repurchased"
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
  // pools' extra month. The redemption price joins the pot on the
  // distribution the clean-up call is first available.
  from series_sum("credit.loan.sched_principal.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + series_sum("credit.loan.prepay.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + series_sum("credit.loan.interest.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93 and if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance) > 101196992.93, (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)), 0.0)

  //  1. the servicer. The pack carries the fee as a negative series on each
  //     pool, so the step is its sum with the sign turned round.
  pay servicing to party.servicer =
        -(series_sum("credit.loan.servicing.*", if(time.t == 1.0, 0.0, time.t), time.t))

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
  pay a3_parity to party.a3_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3) - if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance), 0.0)

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
  pay b_parity to party.b_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b) - if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance), 0.0)

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
  pay c_parity to party.c_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c) - if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance), 0.0)

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
  pay d_parity to party.d_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d) - if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance), 0.0)

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
  pay e_parity to party.e_holders = max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance), 0.0)

  //  17. the remaining Class E balance on its final scheduled date
  pay e_final to party.e_holders = if(time.t >= 90.0, container.trust.bal_e, 0.0)

  // 18. the Noteholders' Principal Distributable Amount — principal
  //     collected LESS the Step-Down Amount, to the most senior class
  //     outstanding and then down.

  pay a1_principal to party.a1_holders =
        min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (0.0), 0.0), container.trust.bal_a1)

  pay a2_principal to party.a2_holders =
        min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1), 0.0), container.trust.bal_a2)

  pay a3_principal to party.a3_holders =
        min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1 + container.trust.bal_a2), 0.0), container.trust.bal_a3)

  pay b_principal to party.b_holders =
        min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3), 0.0), container.trust.bal_b)

  pay c_principal to party.c_holders =
        min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b), 0.0), container.trust.bal_c)

  pay d_principal to party.d_holders =
        min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c), 0.0), container.trust.bal_d)

  pay e_principal to party.e_holders =
        min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
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
        max(min(max((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
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
            - (min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (0.0), 0.0), container.trust.bal_a1)), 0.0)

  pay a2_accelerated to party.a2_holders =
        max(min(max((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
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
            - (min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1), 0.0), container.trust.bal_a2)), 0.0)

  pay a3_accelerated to party.a3_holders =
        max(min(max((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
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
            - (min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1 + container.trust.bal_a2), 0.0), container.trust.bal_a3)), 0.0)

  pay b_accelerated to party.b_holders =
        max(min(max((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
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
            - (min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3), 0.0), container.trust.bal_b)), 0.0)

  pay c_accelerated to party.c_holders =
        max(min(max((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
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
            - (min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b), 0.0), container.trust.bal_c)), 0.0)

  pay d_accelerated to party.d_holders =
        max(min(max((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
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
            - (min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
           - (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c), 0.0), container.trust.bal_d)), 0.0)

  pay e_accelerated to party.e_holders =
        max(min(max((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
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
            - (min(max((min((if((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) <= 101196992.93, (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e),
         max((container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - (min((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - 5059849.65,
          max(((prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - max(0.1475 * (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)) - prev.reserve, 0.0)),
              (container.trust.bal_a1 + container.trust.bal_a2 + container.trust.bal_a3 + container.trust.bal_b + container.trust.bal_c + container.trust.bal_d + container.trust.bal_e) - ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t))) - max(((container.trust.pool_int)
           - (container.trust.pool_fee) * 0.0018750000
           - 625.0
           - (container.trust.bal_a1 * 0.0007916667
           + container.trust.bal_a2 * 0.0011331000
           + container.trust.bal_a3 * 0.0015916667
           + container.trust.bal_b * 0.0019583333
           + container.trust.bal_c * 0.0024000000
           + container.trust.bal_d * 0.0026916667
           + container.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((if(time.t <= 1.0, 1011969929.28, prev.container.trust.balance)) - (prev.container.trust.balance - series_sum("credit.loan.sched_principal.*", time.t, time.t) - series_sum("credit.loan.prepay.*", time.t, time.t) + series_sum("credit.loan.defaults.*", time.t, time.t)))))
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

Checked period by period: **31 series** across **64 periods** — **1576 values** in all, each within ±1.00 of the reference.

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
