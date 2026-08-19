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

The model agrees with the reference to **4.4 cents** on a $305m class, across
all seven classes and all 63 periods. Against the published grid, the reference
reproduces:

| | |
|---|---:|
| Informative cells inside the whole-percent floor | **184 of 195** |
| Mean error inside it (0.25 predicted for a correct model) | **0.2479** |
| Maximum error inside it (0.4973 predicted) | **0.4990** |
| Published weighted average lives reproduced exactly | **46 of 48** |

The published grid rounds to whole percentages, so a model that is exactly right
has errors uniform on [0, 0.5]. The mean and the maximum both sit where that
distribution puts them, which is stronger evidence than either cell count: a
model that is subtly wrong shows a biased distribution even when every
individual cell passes.

Four conventions had to be recovered, none of them stated in the document: a
January-cutoff pool pays twice before the first distribution; ABS runs from
origination, which retires four seasoned pools outright at 2.00%; the step-down
floor is 0.50% of the initial pool; and weighted average life runs 30E/360 from
closing to the 18th, with a 25-day stub. `NOTES.md` records each, with the
readings tested and rejected against it.

## The delta

**Eleven cells sit outside the floor**, by 0.50 to 0.98 of a point. Every one is
Class A-1 or A-2, in the first six distribution dates, and every one is in the
same direction: the model retires A-1 slightly slower than the prospectus does.
The single missed life is A-1 at 1.00% ABS — 0.224 years against a published
0.23, a rounding-boundary miss rather than a mechanism.

It is a first-period convention worth about $1.5m of principal in month one, and
it is left open rather than fitted. Three candidate readings were tested and are
all worse: a stub first interest period on either day count (13 and 16 misses
against 11 — the arithmetic consequence of the assumption that every month has
30 days), the servicing fee on the closing rather than the opening pool balance
(worth $60,000), and a third scheduled payment in the first collection period
(195 misses).

Mutation testing has not been run. This case has the hole `docs/20` §3.2 warns
about: the certificateholder's step-down release absorbs whatever the notes are
not paid, so a residual assertion here would be one-sided by construction.

`model.total` is a regression anchor from this model, not an external figure.
Every external assertion is the independent reference against the published
grid; `expected.csv` holds that reference's per-period class balances.
