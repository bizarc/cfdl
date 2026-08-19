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
