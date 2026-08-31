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
  The model's contracts keep amortizing past it, because a contract runs for
  its declared term and nothing can end it early; the certificateholder absorbs
  what they produce and `model.total` includes it.
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


entity asset trust : Credit.Asset.LoanPool {
  collateral_type = "auto"

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
                max(prev - (min(max((if((prev.asset.trust.pool_bal) <= 101196992.93, (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e),
         max((prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - (min((prev.asset.trust.pool_bal) - 5059849.65,
          max(((prev.asset.trust.pool_bal) - max(0.1475 * (prev.asset.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - ((prev.asset.trust.pool_prior) - (prev.asset.trust.pool_bal)) - max(((prev.asset.trust.pool_int)
           - (prev.asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.asset.trust.bal_a1 * 0.0007916667
           + prev.asset.trust.bal_a2 * 0.0011331000
           + prev.asset.trust.bal_a3 * 0.0015916667
           + prev.asset.trust.bal_b * 0.0019583333
           + prev.asset.trust.bal_c * 0.0024000000
           + prev.asset.trust.bal_d * 0.0026916667
           + prev.asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (0.0), 0.0), prev.asset.trust.bal_a1)), 0.0))


  // Class A-2, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_a2 init 305000000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.asset.trust.pool_bal) <= 101196992.93, (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e),
         max((prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - (min((prev.asset.trust.pool_bal) - 5059849.65,
          max(((prev.asset.trust.pool_bal) - max(0.1475 * (prev.asset.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - ((prev.asset.trust.pool_prior) - (prev.asset.trust.pool_bal)) - max(((prev.asset.trust.pool_int)
           - (prev.asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.asset.trust.bal_a1 * 0.0007916667
           + prev.asset.trust.bal_a2 * 0.0011331000
           + prev.asset.trust.bal_a3 * 0.0015916667
           + prev.asset.trust.bal_b * 0.0019583333
           + prev.asset.trust.bal_c * 0.0024000000
           + prev.asset.trust.bal_d * 0.0026916667
           + prev.asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.asset.trust.bal_a1), 0.0), prev.asset.trust.bal_a2)), 0.0))


  // Class A-3, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_a3 init 189000000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.asset.trust.pool_bal) <= 101196992.93, (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e),
         max((prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - (min((prev.asset.trust.pool_bal) - 5059849.65,
          max(((prev.asset.trust.pool_bal) - max(0.1475 * (prev.asset.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - ((prev.asset.trust.pool_prior) - (prev.asset.trust.pool_bal)) - max(((prev.asset.trust.pool_int)
           - (prev.asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.asset.trust.bal_a1 * 0.0007916667
           + prev.asset.trust.bal_a2 * 0.0011331000
           + prev.asset.trust.bal_a3 * 0.0015916667
           + prev.asset.trust.bal_b * 0.0019583333
           + prev.asset.trust.bal_c * 0.0024000000
           + prev.asset.trust.bal_d * 0.0026916667
           + prev.asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2), 0.0), prev.asset.trust.bal_a3)), 0.0))


  // Class B, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_b init 73370000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.asset.trust.pool_bal) <= 101196992.93, (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e),
         max((prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - (min((prev.asset.trust.pool_bal) - 5059849.65,
          max(((prev.asset.trust.pool_bal) - max(0.1475 * (prev.asset.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - ((prev.asset.trust.pool_prior) - (prev.asset.trust.pool_bal)) - max(((prev.asset.trust.pool_int)
           - (prev.asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.asset.trust.bal_a1 * 0.0007916667
           + prev.asset.trust.bal_a2 * 0.0011331000
           + prev.asset.trust.bal_a3 * 0.0015916667
           + prev.asset.trust.bal_b * 0.0019583333
           + prev.asset.trust.bal_c * 0.0024000000
           + prev.asset.trust.bal_d * 0.0026916667
           + prev.asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3), 0.0), prev.asset.trust.bal_b)), 0.0))


  // Class C, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_c init 91080000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.asset.trust.pool_bal) <= 101196992.93, (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e),
         max((prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - (min((prev.asset.trust.pool_bal) - 5059849.65,
          max(((prev.asset.trust.pool_bal) - max(0.1475 * (prev.asset.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - ((prev.asset.trust.pool_prior) - (prev.asset.trust.pool_bal)) - max(((prev.asset.trust.pool_int)
           - (prev.asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.asset.trust.bal_a1 * 0.0007916667
           + prev.asset.trust.bal_a2 * 0.0011331000
           + prev.asset.trust.bal_a3 * 0.0015916667
           + prev.asset.trust.bal_b * 0.0019583333
           + prev.asset.trust.bal_c * 0.0024000000
           + prev.asset.trust.bal_d * 0.0026916667
           + prev.asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b), 0.0), prev.asset.trust.bal_c)), 0.0))


  // Class D, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_d init 89550000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.asset.trust.pool_bal) <= 101196992.93, (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e),
         max((prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - (min((prev.asset.trust.pool_bal) - 5059849.65,
          max(((prev.asset.trust.pool_bal) - max(0.1475 * (prev.asset.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - ((prev.asset.trust.pool_prior) - (prev.asset.trust.pool_bal)) - max(((prev.asset.trust.pool_int)
           - (prev.asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.asset.trust.bal_a1 * 0.0007916667
           + prev.asset.trust.bal_a2 * 0.0011331000
           + prev.asset.trust.bal_a3 * 0.0015916667
           + prev.asset.trust.bal_b * 0.0019583333
           + prev.asset.trust.bal_c * 0.0024000000
           + prev.asset.trust.bal_d * 0.0026916667
           + prev.asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c), 0.0), prev.asset.trust.bal_d)), 0.0))


  // Class E, the balance it carries into the distribution date. It is what
  // it carried in last period less what it was paid then, and the payment is
  // computed here because the waterfall cannot tell it.
  bal_e init 23780000.00
       next if(time.t <= 1.0, prev,
                max(prev - (min(max((if((prev.asset.trust.pool_bal) <= 101196992.93, (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e),
         max((prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - (min((prev.asset.trust.pool_bal) - 5059849.65,
          max(((prev.asset.trust.pool_bal) - max(0.1475 * (prev.asset.trust.pool_bal) - prev.reserve, 0.0)),
              (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d + prev.asset.trust.bal_e) - ((prev.asset.trust.pool_prior) - (prev.asset.trust.pool_bal)) - max(((prev.asset.trust.pool_int)
           - (prev.asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (prev.asset.trust.bal_a1 * 0.0007916667
           + prev.asset.trust.bal_a2 * 0.0011331000
           + prev.asset.trust.bal_a3 * 0.0015916667
           + prev.asset.trust.bal_b * 0.0019583333
           + prev.asset.trust.bal_c * 0.0024000000
           + prev.asset.trust.bal_d * 0.0026916667
           + prev.asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (prev.asset.trust.bal_a1 + prev.asset.trust.bal_a2 + prev.asset.trust.bal_a3 + prev.asset.trust.bal_b + prev.asset.trust.bal_c + prev.asset.trust.bal_d), 0.0), prev.asset.trust.bal_e)), 0.0))

}

entity asset p01 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p02 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p03 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p04 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p05 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p06 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p07 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p08 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p09 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p10 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p11 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity asset p12 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
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
// The initial pool balance at the cutoff date, and the reserve stated against
// it. Two other structural amounts are the same shape and still literals —
// 5,059,849.65 is the 0.50% overcollateralization floor and 101,196,992.93 is
// the 10% clean-up call threshold, each written out twenty-eight times. They
// are left alone here: this change is the reserve.
assume initial_pool     = 1011969929.28
assume reserve_required = 0.02 * inputs.initial_pool

// ---------------------------------------------------------------------------
// THE RESERVE ACCOUNT (clause 19). 2.0% of the initial pool, funded at
// closing. It was a literal — 20,239,398.59, written out twenty-eight times,
// once in each balance field's recurrence and once in each waterfall step
// that reads the overcollateralization target. The document does not state a
// number; it states a RULE, and the target is stated against the account
// rather than against a dollar amount: the Required Pro Forma Note Balance is
// 14.75% of the pool "less the amount required on deposit in the reserve
// account" (glossary). So the balance is what the target should read.
//
// FUNDED AT CLOSING, WHICH IS NOT A DISTRIBUTION. The reserve is funded out
// of note proceeds before the first collection period, so it cannot come from
// the waterfall — the waterfall allocates collections, and taking the reserve
// out of them would spend cash the deal never spent. `from` is the account's
// own inflow, and it fires once at period 0.
//
// NOT ROUNDED. The literal was 20,239,398.59; the deal is 2.0% of
// 1,011,969,929.28, which is 20,239,398.5856. The reference computes the
// product (`reference_gen.py`, `RESERVE = 0.02 * POOL0`) and so does this
// now. See NOTES.md for what the 0.0044 moved.
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
waterfall notes.distribution on entity asset.trust {
  schedule every month from 2017-02 to 2022-11

  // Collections for this distribution, and at the first one the January
  // pools' extra month. The redemption price joins the pot on the
  // distribution the clean-up call is first available.
  from series_sum("credit.pool.sched_principal.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + series_sum("credit.pool.prepay.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + series_sum("credit.pool.interest.*", if(time.t == 1.0, 0.0, time.t), time.t)
       + if(asset.trust.pool_bal <= 101196992.93 and asset.trust.pool_prior > 101196992.93, asset.trust.pool_bal, 0.0)

  //  1. the servicer. The pack carries the fee as a negative series on each
  //     pool, so the step is its sum with the sign turned round.
  pay servicing to party.servicer =
        -(series_sum("credit.pool.servicing.*", if(time.t == 1.0, 0.0, time.t), time.t))

  //  2. the trustee, owner trustee, collateral agent and the asset
  //     representations reviewer, inside their annual caps.
  pay trustee_fees to party.trustee = 625.0

  //  3. interest on the Class A-1 Notes (clause 3 pays the Class A classes pari passu)
  pay a1_interest to party.a1_holders = asset.trust.bal_a1 * 0.0007916667

  //  3. interest on the Class A-2 Notes
  pay a2_interest to party.a2_holders = asset.trust.bal_a2 * 0.0011331000

  //  3. interest on the Class A-3 Notes
  pay a3_interest to party.a3_holders = asset.trust.bal_a3 * 0.0015916667

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
  pay a3_parity to party.a3_holders = max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3) - asset.trust.pool_prior, 0.0)

  //  5. the remaining Class A balance on its final scheduled date
  pay a3_final to party.a3_holders = if(time.t >= 12.0, asset.trust.bal_a1, 0.0) + if(time.t >= 39.0, asset.trust.bal_a2, 0.0) + if(time.t >= 54.0, asset.trust.bal_a3, 0.0)

  //  6. interest on the Class B Notes
  pay b_interest to party.b_holders = asset.trust.bal_b * 0.0019583333

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
  pay b_parity to party.b_holders = max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b) - asset.trust.pool_prior, 0.0)

  //  8. the remaining Class B balance on its final scheduled date
  pay b_final to party.b_holders = if(time.t >= 60.0, asset.trust.bal_b, 0.0)

  //  9. interest on the Class C Notes
  pay c_interest to party.c_holders = asset.trust.bal_c * 0.0024000000

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
  pay c_parity to party.c_holders = max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c) - asset.trust.pool_prior, 0.0)

  //  11. the remaining Class C balance on its final scheduled date
  pay c_final to party.c_holders = if(time.t >= 66.0, asset.trust.bal_c, 0.0)

  //  12. interest on the Class D Notes
  pay d_interest to party.d_holders = asset.trust.bal_d * 0.0026916667

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
  pay d_parity to party.d_holders = max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d) - asset.trust.pool_prior, 0.0)

  //  14. the remaining Class D balance on its final scheduled date
  pay d_final to party.d_holders = if(time.t >= 71.0, asset.trust.bal_d, 0.0)

  //  15. interest on the Class E Notes
  pay e_interest to party.e_holders = asset.trust.bal_e * 0.0000000000

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
  pay e_parity to party.e_holders = max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - asset.trust.pool_prior, 0.0)

  //  17. the remaining Class E balance on its final scheduled date
  pay e_final to party.e_holders = if(time.t >= 90.0, asset.trust.bal_e, 0.0)

  // 18. the Noteholders' Principal Distributable Amount — principal
  //     collected LESS the Step-Down Amount, to the most senior class
  //     outstanding and then down.

  pay a1_principal to party.a1_holders =
        min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (0.0), 0.0), asset.trust.bal_a1)

  pay a2_principal to party.a2_holders =
        min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1), 0.0), asset.trust.bal_a2)

  pay a3_principal to party.a3_holders =
        min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2), 0.0), asset.trust.bal_a3)

  pay b_principal to party.b_holders =
        min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3), 0.0), asset.trust.bal_b)

  pay c_principal to party.c_holders =
        min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b), 0.0), asset.trust.bal_c)

  pay d_principal to party.d_holders =
        min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c), 0.0), asset.trust.bal_d)

  pay e_principal to party.e_holders =
        min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d), 0.0), asset.trust.bal_e)

  // 19. the reserve account, funded at closing to 2.0% of the initial
  //     pool. This step is the TOP-UP: whatever the account is short of
  //     its required amount, restored out of collections ahead of
  //     principal. It pays nothing on this deal — no losses are assumed,
  //     so the reserve is never drawn and the shortfall is always zero —
  //     but it pays nothing BECAUSE the balance is at target, which is
  //     what the clause says, rather than because the step is written as
  //     a constant. Previously `pay reserve_topup to party.certificate =
  //     0.0`, which paid the right amount to the wrong payee for no
  //     stated reason.
  pay reserve_topup to account reserve =
        max(0.0, inputs.reserve_required - prev.reserve)

  // 20. the Accelerated Principal Amount: excess cash turboing the
  //     notes toward the target, and at the clean-up call the whole
  //     remaining balance.

  pay a1_accelerated to party.a1_holders =
        max(min(max((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (0.0), 0.0), asset.trust.bal_a1)
            - (min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (0.0), 0.0), asset.trust.bal_a1)), 0.0)

  pay a2_accelerated to party.a2_holders =
        max(min(max((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (asset.trust.bal_a1), 0.0), asset.trust.bal_a2)
            - (min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1), 0.0), asset.trust.bal_a2)), 0.0)

  pay a3_accelerated to party.a3_holders =
        max(min(max((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (asset.trust.bal_a1 + asset.trust.bal_a2), 0.0), asset.trust.bal_a3)
            - (min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2), 0.0), asset.trust.bal_a3)), 0.0)

  pay b_accelerated to party.b_holders =
        max(min(max((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3), 0.0), asset.trust.bal_b)
            - (min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3), 0.0), asset.trust.bal_b)), 0.0)

  pay c_accelerated to party.c_holders =
        max(min(max((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b), 0.0), asset.trust.bal_c)
            - (min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b), 0.0), asset.trust.bal_c)), 0.0)

  pay d_accelerated to party.d_holders =
        max(min(max((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c), 0.0), asset.trust.bal_d)
            - (min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c), 0.0), asset.trust.bal_d)), 0.0)

  pay e_accelerated to party.e_holders =
        max(min(max((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0)))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d), 0.0), asset.trust.bal_e)
            - (min(max((min((if((asset.trust.pool_bal) <= 101196992.93, (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e),
         max((asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - (min((asset.trust.pool_bal) - 5059849.65,
          max(((asset.trust.pool_bal) - max(0.1475 * (asset.trust.pool_bal) - prev.reserve, 0.0)),
              (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d + asset.trust.bal_e) - ((asset.trust.pool_prior) - (asset.trust.pool_bal)) - max(((asset.trust.pool_int)
           - (asset.trust.pool_fee) * 0.0018750000
           - 625.0
           - (asset.trust.bal_a1 * 0.0007916667
           + asset.trust.bal_a2 * 0.0011331000
           + asset.trust.bal_a3 * 0.0015916667
           + asset.trust.bal_b * 0.0019583333
           + asset.trust.bal_c * 0.0024000000
           + asset.trust.bal_d * 0.0026916667
           + asset.trust.bal_e * 0.0000000000)), 0.0)))), 0.0))), ((asset.trust.pool_prior) - (asset.trust.pool_bal))))
           - (asset.trust.bal_a1 + asset.trust.bal_a2 + asset.trust.bal_a3 + asset.trust.bal_b + asset.trust.bal_c + asset.trust.bal_d), 0.0), asset.trust.bal_e)), 0.0)

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

- `asset.trust.bal_a1`
- `asset.trust.bal_a2`
- `asset.trust.bal_a3`
- `asset.trust.bal_b`
- `asset.trust.bal_c`
- `asset.trust.bal_d`
- `asset.trust.bal_e`
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
| `model.total` | 1,215,935,766.43 | ±1 |
