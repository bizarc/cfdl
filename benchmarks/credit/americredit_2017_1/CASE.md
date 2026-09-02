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
