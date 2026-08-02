# Auto ABS collateral — matching an issuer's published amortisation

## The source

A weighted-average-life exhibit filed with the securities regulator by an
auto-receivables trust. Public record, freely readable, and unusually complete:
it disaggregates the pool into 50 hypothetical sub-pools, states each one's
balance, APR and remaining term, and then publishes — for six note classes, at
seven prepayment speeds, for every monthly distribution date — the percent of
that class still outstanding, plus its weighted average life both to maturity
and to the clean-up call.

It also states the WAL definition outright, which is why the previous commit
exists: *the amount of each principal payment multiplied by the number of years
from the closing date to the related distribution date, summed, divided by the
sum of the payments.* That sentence is what showed CFDL was measuring from
period zero and reporting every weighted average life one period short.

## The result

The pool's scheduled principal, run through the credit pack at zero prepayment
speed, against the issuer's published Class A-2 column:

| distribution date | CFDL | published | diff |
|---|---|---|---|
| 10/15/18 | 87.367 | 87.37 | −0.003 |
| 11/15/18 | 74.682 | 74.68 | +0.002 |
| 12/15/18 | 61.944 | 61.94 | +0.004 |
| 01/15/19 | 49.154 | 49.15 | +0.004 |
| 02/15/19 | 36.312 | 36.31 | +0.002 |
| 03/15/19 | 23.416 | 23.42 | −0.004 |
| 04/15/19 | 10.466 | 10.47 | −0.004 |
| 05/15/19 | 0.000 | 0.00 | 0.000 |

**Worst disagreement 0.004 percentage points.** The exhibit rounds to 0.01, so
0.005 is the floor a reader can check against — this sits on it.

Class A-2 weighted average life to maturity: **0.3695 years against a published
0.37**.

And the aggregate: `domain.credit.principal` returns **537,640,787.96**, the
exhibit's own stated pool balance, to the cent. That is 43 level-pay sub-pools
at 43 different rates and terms each returning exactly the balance the issuer
stated for it.

## Why a fitted constant, and why that is still evidence

The exhibit publishes percentages of each note class, and this pack models
collateral rather than a liability stack, so the two are related by the class's
initial balance — which this exhibit does not state. It is fitted: 112,026,000.

That sounds circular and is not, because **one constant has to fit all eight
points at once**. A scale factor can move the whole curve up or down; it cannot
change its shape. If the amortisation were wrong — wrong day count, wrong
annuity factor, wrong handling of the 0% sub-pools, a single mis-stated term —
no constant would reconcile eight successive points to four decimal places. The
fit is the test, and the residual is the source's rounding rather than ours.

The relationship holds at all because Class A-1 had already retired, so A-2 was
receiving 100% of pool principal for the whole of its life. Its pay-down *is*
the pool's principal collections, scaled.

## What is not reachable, and why

The exhibit is a 7-speed × 6-class grid. Two of those axes are out of reach and
this case deliberately takes only the third.

**The prepayment speeds — NO LONGER OUT OF REACH.** They use the Absolute
Prepayment Model: a constant number of *original* units prepaying each month, so
the implied SMM rises over the life and `k` is not constant. Every pool factor
in this pack was `pow(k, p)`, valid only when it is.

Declared state variables closed that (`docs/13_feature_backlog.md` 2.1). Two of
the six non-zero columns are now their own cases,
`benchmarks/credit/auto_abs_speed_050` and `_150`, reconciling to 0.0048 and
0.0036 percentage points. Building them found that ABS is indexed from loan
ORIGINATION rather than from closing — worth 20 percentage points on this
seasoned pool at 1.50% ABS — which is recorded in those cases' NOTES.

**The note classes.** Percent-outstanding per class needs a sequential-pay
waterfall with overcollateralisation and a reserve account. This pack models the
collateral side only. That is the waterfall roadmap item, not a gap this case
can close.

Also unmodelled: the 10% clean-up call, which is why the exhibit's "to call" and
"to maturity" WAL rows are identical for A-2 — it retires long before the call
could be exercised.

## The 0% APR sub-pools

Four of the 50 sub-pools carry a **0.000% APR** — promotional financing, 2.80%
of the pool by balance. The pack could not model them: its level-pay closed form
uses

    S(p) = ((1+r)^n − (1+r)^p) / ((1+r)^n − 1)

which is 0/0 at `r = 0`. The `rate` validation only ever required *non-negative*,
so a zero rate was accepted and produced NaN — a wrong answer rather than a
refusal, which is the worse of the two failures.

Fixed without adding anything to the language. `S(p)` has an exact identity as a
ratio of annuity present values,

    S(p) = pv(r, n − p, −1) / pv(r, n, −1)

and the level payment factor is `−pmt(r, n − p, 1)`. Both `pv` and `pmt` already
carried the `r = 0` limit, so the zero-rate case simply works: straight-line
principal, no interest. The rewritten expressions are also shorter than the ones
they replaced, and produce **byte-identical results on all 107 goldens** — the
identity is exact, not an approximation.

One inconsistency surfaced on the way. The prepayment rule computed "balance net
of scheduled principal" with a *different* expression than the scheduled
principal rule itself. The two agree whenever `amortization_day_count` equals
`day_count`, which every shipped model satisfies, so nothing moved — but under
an Actual accrual with a 30/360 amortisation they would have disagreed. Both now
use the one formula.
