# Notes — americredit_2017_1

What the reference implementation had to recover, and what it has not.

Status: the reference reproduces **184 of 195 informative cells** inside the
grid's own whole-percent rounding floor, and **46 of the 48 published weighted
average lives** exactly. The CFDL model is not written yet; this file records
the conventions it will have to carry.

## The error distribution

The published grid is whole percentages, so a model that is exactly right has
errors uniform on [0, 0.5]: mean near 0.25 and maximum over *n* informative
cells near `0.5n/(n+1)`. Measured over the 184 cells inside the floor: **mean
0.2479 against 0.25 predicted, maximum 0.4990 against 0.4973 predicted.** That
is the shape of a model whose remaining error is the issuer's rounding and
nothing else.

The 11 cells outside the floor are not distributed like that. Every one of them
is Class A-1 or A-2, in the first six distribution dates, and every one is in
the same direction — the model retires A-1 slightly slower than the prospectus
does, by 0.50 to 0.98 of a point. The single missed life is A-1 at 1.00% ABS:
0.224 years against a published 0.23, which is a rounding-boundary miss rather
than a mechanism.

**This is an open item, deliberately left open.** It is a first-period
convention worth about $1.5m of principal in month one, and three candidate
readings were tested and rejected rather than fitted:

- a stub first interest period, 25 days on a 30/360 basis or 23 actual days
  from the 23 February closing to the 18 March payment. Both are worse — 13 and
  16 misses against 11 — which is the arithmetic consequence of assumption
  (iii), that every month has 30 days;
- the servicing fee on the closing rather than the opening pool balance, worth
  only $60,000; and
- a third scheduled payment in the first collection period, which misses 195
  cells.

## Conventions recovered

**A January cutoff makes two payments before the first distribution date.** Six
of the twelve assumed pools have an assumed cutoff of 1 January 2017 and six of
1 February. First due date is the last day of the cutoff month, distributions
begin 18 March, so the January pools contribute their 31 January *and* 28
February payments to the first collection period and the February pools
contribute one. This is the largest single convention in the case: one payment
for every pool misses 195 of the 195 informative cells, three for the January
pools misses 195 as well, and the correct reading misses 11.

**ABS runs from origination, and it can exhaust a seasoned pool outright.**
Prepaying contracts each month are a constant percentage of the pool's
*original* contract count, so a pool seasoned 53 months has already lost
53 x ABS of its contracts. At 2.00% ABS that is more than all of them: four
pools, $59.1m between them, prepay in full in the first collection period. That
is not a degenerate case to be guarded against — it is what produces the
published 32% for Class A-1 at 2.00% ABS after one month, which no smoother
reading reaches. Running ABS from the cutoff date instead misses 162 cells.

**The step-down, not the turbo, is what shapes the middle of the deal.** The
first draft paid the full principal collections to the notes every month and
missed 80 cells by up to 100 points. The Principal Distributable Amount is
principal collected *less* the Step-Down Amount, and the step-down is whatever
would take the notes below `0.8525 x Pool + Reserve`. Once the target is met the
notes track that line exactly, month after month, and the retained principal
goes to the certificateholder. Paying the full collections misses 182 of the 195
informative cells, by up to 100 points.

**The step-down has a floor of 0.50% of the initial pool, and it binds.**
Without it the notes follow the required balance all the way down,
overcollateralization drains to 0.7%, and 35 cells miss by up to 6 points in a
contiguous block where Class C is retiring. The floor is stated twice in the
prospectus, and sweeping it as a free parameter puts the minimum at 0.50%
exactly:

| Floor, percent of the initial pool | Misses | Worst |
|---|---:|---:|
| 0.00% | 35 | 6.05pp |
| 0.25% | 35 | 3.31pp |
| **0.50%, as stated** | **11** | **0.98pp** |
| 0.75% | 39 | 3.31pp |
| 1.00% | 45 | 6.13pp |

The document and the arithmetic agree on the same number, which is the point of
running the sweep at all.

**Weighted average life runs 30E/360 from the closing date to the 18th, with a
25-day stub.** Measuring from period zero at a flat 30/360 overstates every
life by 0.014 years, which is invisible on a long class and fatal on a short
one: 20 of the 48 published figures miss. With the stub, 46 of 48 are exact.
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
| Reserve credited in the required balance, floor at 0.50% (**used**) | 11 |
| No reserve credit — a flat 14.75% target | 184 |
| Reserve credit at 2.0% of the *current* pool | 184 |
| Principal paid in full, no step-down | 182 |
| One scheduled payment in the first collection period, every pool | 195 |
| Three for the January-cutoff pools | 195 |
| ABS measured from the cutoff rather than from origination | 162 |

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
