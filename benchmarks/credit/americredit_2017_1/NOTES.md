# Notes — americredit_2017_1

What the reference implementation had to recover, and what it has not.

Status: the reference reproduces **192 of 195 informative cells** inside the
grid's own whole-percent rounding floor, and **all 48 published weighted
average lives** exactly. The CFDL model agrees with it to **4.4 cents** on a
$305m class, across seven classes and all 63 periods.

## The error distribution

The published grid is whole percentages, so a model that is exactly right has
errors uniform on [0, 0.5]: mean near 0.25 and maximum over *n* informative
cells near `0.5n/(n+1)`. Measured over the 192 cells inside the floor: **mean
0.2470 against 0.25 predicted, maximum 0.4990 against 0.4974 predicted.** That
is the shape of a model whose remaining error is the issuer's rounding and
nothing else.

The three cells outside the floor are not distributed like that. All three are
Class A-1 in its second or third distribution date — 05/18/17 at 0.50% and
1.00% ABS, and 03/18/17 at 2.00% — and all three are the same sign, the model
retiring A-1 by 0.60 to 0.68 of a point less than the prospectus does. That is
about $1.1m of principal in a single month, on a $182m class.

**This is an open item.** It is smaller and better localised than what it
replaced, and one candidate has been tested and rejected rather than fitted: a
stub first interest period, 25 days on a 30/360 basis or 23 actual days from
the 23 February closing to the 18 March payment. Both are worse, which is the
arithmetic consequence of assumption (iii), that every month has 30 days.

**What it replaced, and how it was wrong.** The first version of this file
recorded eleven cells outside the floor and described them as the model
retiring A-1 *slower* than the prospectus. The sign was backwards: the model
was retiring A-1 **faster**, by up to a point, which meant the model had about
$1.5m more cash in the first month than the issuer did. Read that way the
answer was immediate — the servicing fee, which is $1.9m a month on this pool.
Eight of the eleven cells closed, and two published lives that had been off by
0.01 became exact. A wrong sign in a note is worse than no note: it points the
next reader away from the answer.

## Conventions recovered

**The servicing fee accrues monthly, so the January pools pay it twice in the
first collection period.** The fee is 2.25% per annum on the pool balance, and
the first collection period spans two months for a January-cutoff pool — the
same fact that gives it two scheduled payments. Charging one month on the
opening pool balance instead leaves eleven cells outside the floor and misses
two of the published lives; accruing it per payment month leaves three and
misses none. Note where it sits in the table below: every other rejected
reading is off by a hundred cells or more, and this one by eleven. A wrong
convention this small does not announce itself.

Worth recording for a second reason: the model already had this right on the
cash side and wrong on the balance side. Clause 1 pays the pack's own servicing
series, which accrues per contract per period and therefore charged the
January pools twice without being told to; the balance recurrence carried its
own copy of the arithmetic and charged one month. That is backlog 7.37 in
miniature — the two halves of the same deal, drifting because nothing holds
them together.

**A January cutoff makes two payments before the first distribution date.** Six
of the twelve assumed pools have an assumed cutoff of 1 January 2017 and six of
1 February. First due date is the last day of the cutoff month, distributions
begin 18 March, so the January pools contribute their 31 January *and* 28
February payments to the first collection period and the February pools
contribute one. This is the largest single convention in the case: one payment
for every pool misses all 195 informative cells, three for the January pools
misses 195 as well, and the correct reading misses 3.

**ABS runs from origination, and it can exhaust a seasoned pool outright.**
Prepaying contracts each month are a constant percentage of the pool's
*original* contract count, so a pool seasoned 53 months has already lost
53 x ABS of its contracts. At 2.00% ABS that is more than all of them: four
pools, $59.1m between them, prepay in full in the first collection period. That
is not a degenerate case to be guarded against — it is what produces the
published 32% for Class A-1 at 2.00% ABS after one month, which no smoother
reading reaches. Running ABS from the cutoff date instead misses 166 cells.

**The step-down, not the turbo, is what shapes the middle of the deal.** The
first draft paid the full principal collections to the notes every month and
missed 80 cells by up to 100 points. The Principal Distributable Amount is
principal collected *less* the Step-Down Amount, and the step-down is whatever
would take the notes below `0.8525 x Pool + Reserve`. Once the target is met the
notes track that line exactly, month after month, and the retained principal
goes to the certificateholder. Paying the full collections misses 174 of the 195
informative cells, by up to 98 points.

**The step-down has a floor of 0.50% of the initial pool, and it binds.**
Without it the notes follow the required balance all the way down,
overcollateralization drains to 0.7%, and 27 cells miss by up to 6 points in a
contiguous block where Class C is retiring. The floor is stated twice in the
prospectus, and sweeping it as a free parameter puts the minimum at 0.50%
exactly:

| Floor, percent of the initial pool | Misses | Worst |
|---|---:|---:|
| 0.00% | 27 | 6.05pp |
| 0.25% | 27 | 3.31pp |
| **0.50%, as stated** | **3** | **0.68pp** |
| 0.75% | 31 | 3.31pp |
| 1.00% | 37 | 6.13pp |

The document and the arithmetic agree on the same number, which is the point of
running the sweep at all.

**Weighted average life runs 30E/360 from the closing date to the 18th, with a
25-day stub.** Measuring from period zero at a flat 30/360 overstates every
life by 0.014 years, which is invisible on a long class and fatal on a short
one: 20 of the 48 published figures miss. With the stub, all 48 are exact.
This is the same finding `docs/20` §4 records for Ginnie Mae and Fannie Mae —
three issuers now, three different day counts for this purpose, none of them
stated in the document.

**Re-amortizing a pool over its remaining term and carrying it from an original
schedule are the same thing.** Assumption (ix) says each contract's scheduled
payment repays its current balance over its remaining term, which reads like a
different model from carrying a level-pay schedule forward from origination.
The balance paths are identical — `bf(a, T) / bf(s, T) = bf(a - s, T - s)` for
a level-pay contract — and the implementations agree to the last digit. Worth
recording so nobody spends the afternoon twice.

## Rejected readings

Each of these was tested against all four speeds and is worse. Kept because a
plausible wrong reading costs the same to rediscover as it did to reject.

Counts are cells outside the rounding floor, out of 195 informative.

| Reading | Misses |
|---|---:|
| As implemented (**used**) | 3 |
| One scheduled payment in the first collection period, every pool | 195 |
| Three for the January-cutoff pools | 195 |
| No reserve credit — a flat 14.75% target | 176 |
| Reserve credit at 2.0% of the *current* pool | 176 |
| Principal paid in full, no step-down | 174 |
| ABS measured from the cutoff rather than from origination | 166 |
| Servicing charged one month on the opening pool balance | 11 |

## What this case will assert, and what it cannot

The grid is a to-call grid, so the 195 informative cells and the 24 to-call
lives come from one run. The 24 to-maturity lives need a second run with the
redemption option suppressed, and they differ from the to-call figures only on
Class D. Twelve of the 48 published lives are therefore the only evidence the
no-call scenario produces, and `docs/20` §3.1 still applies: a published
weighted average life cannot be asserted, only reconciled in prose.

Mutation testing has not been done yet. `docs/20` §3.3 asks for it, and this
case has an obvious hole to check: a residual assertion would be one-sided
here, because the certificateholder's step-down release absorbs anything the
notes are not paid.


## The model, and the arithmetic it has to state twice

A class balance is a field. A field cannot read what the waterfall paid it —
see `docs/13` §7.37 — so `model.cfdl` states the distribution twice: once
lagged, inside the seven balance fields, and once at the current period, inside
the twenty-two clauses that pay the cash. Nothing enforces that they agree.
That is the case's structural weakness and it is worth knowing before reading
the file.

Three fields carry the collateral as closed forms of the twelve assumed pools —
`pool_bal`, `pool_prior` and `pool_int` — for the same reason: a recurrence
cannot read the pack's own series either. The pack contracts produce the cash
the waterfall allocates, so the pool is stated twice as well, independently,
and both are pinned to the published grid through `expected.csv`.

## The distribution, in four lines instead of forty

The first model wrote the deal the way the prospectus does — a Step-Down Amount
subtracted from the Principal Distributable Amount, then an Accelerated
Principal Amount capped by both available cash and the distance to target — and
the expression ran past forty lines per class, most of it the same
subexpressions repeated at different depths.

The same arithmetic collapses to a statement about where the notes END the
period:

```
ending = min(pool - floor, max(required, notes - principal - max(excess, 0)))
```

The notes finish at the required balance; cash may stop them getting there; and
overcollateralization may not fall below the floor. Clause 18 is then
`min(total, principal)` and clause 20 is `max(total - principal, 0)`, which is
what those clauses mean — collections first, excess cash after.

Checked rather than assumed: the two formulations agree to **$0.0000** at all
four published speeds, on every class and every period. The prospectus's form
is the one that explains *why*; this one is the one that fits on a page.

## What the cash assertions caught

`expected.csv` asserts every clause of the waterfall as well as every balance.
It was not always so, and adding the cash columns found two defects in a model
whose grid had matched all along — which is the argument for asserting them.

**Clause 16 fired every month for the last third of the deal.** The parity
clauses reduce a class's balance "to the Pool Balance", and the model compared
the balance each class carried into the distribution against the pool balance
*after* that period's collections. Those are two different dates. Once monthly
principal exceeded overcollateralization — around $13m against $5m, from period
37 — the difference read as undercollateralization and clause 16 paid it,
taking $4m a period from the certificateholder. The balances never moved,
because the recurrence carries its own copy of the arithmetic and its copy was
right. Both sides are now measured on the same date, which is what makes the
clause mean what the prospectus says it is for: "principal payments made to
cure this undercollateralization, if any then exists".

**The reference lost the distribution that ends the deal.** Its period loop
broke out as soon as every class was retired, and the break came before the
line item was recorded — so the clean-up call's own distribution, the largest
in the deal, was missing from the record and `expected.csv` asserted zero for
it. The model was right and the expectation was wrong, which is the failure
mode worth fearing in a case: the reference is only a check while it is
checking.

**And a disagreement that was a real question.** At the call the reference
booked the whole payoff as accelerated principal; the model booked that month's
collections under clause 18 and the redemption price under clause 20. The
model's reading is right — the receivables still pay that month — so the
reference now does the same.

## What the model does not carry

- **The final scheduled distribution dates are written but inert.** With no
  losses assumed every class retires years early, so clauses 5, 8, 11, 14 and
  17 pay nothing at every period and every speed. They are in the model because
  the deal has them.
- **The parity clauses are inert for the same reason.** The pool always exceeds
  the notes, so clauses 4, 7, 10, 13 and 16 pay nothing.
- **Anything after the clean-up call.** The call retires the notes at period 47
  and the trust is over, so the cash columns stop there. The model's twelve
  contracts keep amortizing for another eleven periods because a contract
  cannot be bought out — backlog 7.39 — and the certificateholder takes what
  they produce. `model.total` includes it.
- **One speed.** The case runs at 1.50% ABS. The other three published speeds
  are this model with `abs_speed` changed, and `docs/20` §2.3 is the reason
  they are not four directories.
