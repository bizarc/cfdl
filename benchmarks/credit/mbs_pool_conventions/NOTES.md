# MBS pool conventions — what checking against an external reference found

## The reference, and what may be committed

The industry's published reference schedule for MBS cash flows (SIFMA's
*Standard Formulas for the Analysis of Mortgage-Backed Securities*, cited here
once for provenance and not elsewhere in the repo). Free to download;
**reproduction in any form is forbidden**.

So this case is validation-only, and the posture matters: the document is not
vendored, its tables are not reproduced, and `expected.csv` carries a sparse set
of anchor months on the four columns the pack emits. Numbers and formulae are
facts and may be asserted against. Shipped documentation claims *parity* with
the market convention and does not cite the document — this file is the one
place the provenance is recorded. Contrast `benchmarks/cre/mit_rentleg_plaza`,
whose CC BY-NC-SA source could have shipped.

## What it asserts

A new 30-year pool: $100m, 8% WAC, 20% loss severity, 12-month recovery,
servicer advances, at a flat **1% SMM and 1% MDR**. Constant hazards, which is
why it is reproducible today — the ramped variant on the same pool runs 150% PSA
and 100% SDA, and those curves are not expressible (see below).

**25 anchor months on four streams**: Actual Interest, Actual Amort, Voluntary
Prepayments and Principal Recovery, spread over the life of the pool. There is
deliberately no `reference_gen.py` and no `expected_metrics.json`. Every number
here is external; a second implementation of our own is precisely what it exists
to replace, and an aggregate we computed ourselves would dilute that.

Anchors rather than all 348 months because a convention error appears in *every*
period — it is systematic, not sporadic — so 25 catch it as reliably as 348
while keeping the extraction an order of magnitude smaller. Reconciliation was
performed against the full schedule; only the anchors are committed.

## Three defects it found

### 1. The prepayment base — fixed

The pack applied SMM to the balance **net of defaults**. SMM is defined on the
loans outstanding at the *beginning* of the month, via
`SMM = (Fsched − F2) / Fsched` where `Fsched = F1 × BAL2/BAL1` — the beginning
balance after *scheduled amortisation only*. Defaults are not removed.

Period 1: reference 999,329, the pack 989,336 — short by exactly the 1% MDR. The
pack's rules header described this behaviour but cited nothing; it was a
misreading of the quantity it calls SMM, not a competing convention.

**And the survival factor changes with it**, which is not obvious. Prepayments
taken from the survivors compose multiplicatively, `k = (1−mdr)(1−smm)`. Taken
from the same beginning balance, as the convention specifies, they are additive:
`k = (1−mdr) − smm`. The difference is `mdr·smm`, 9,993 in period 1, compounding
over 360. The reference ending balance settles it: 97,934,244 additive against
97,944,237 multiplicative. Fixing the base and leaving `k` alone would have been
internally inconsistent — the benchmark caught that too, as a residual that grew
period on period.

The three existing credit benchmarks moved. Their `reference_gen.py` scripts
carried the same misreading as the engine, which is why they had always agreed
— the same failure `tools/analytic-checks.py` was written for.

### 2. The recovery basis — fixed

The pack recovered `(1 − severity)` of **face** after the lag. A defaulted loan
keeps amortising while it sits in foreclosure, so what is liquidated is the
**amortised** balance.

The reference's amortised-default-balance column gives the mechanism exactly, in
three relationships that hold on every row:

```
recovery + loss = amortised default balance
loss            = severity x ORIGINAL defaulted face
amortised bal   = face x S(p)/S(p-lag)     S = the scheduled balance factor
                                           already inside the pack's closed form
```

So `recovery(p) = face(p−lag) × [ S(p)/S(p−lag) − severity ]`, floored at zero.
**336 of 336 asserted recoveries now agree within 0.51** (whole-dollar rounding
in the source is the binding constraint, not the engine). Level-pay only: an
IO/bullet loan's defaulted balance does not amortise, so face is already right
there, and the asymmetry in `lowering/rules.toml` is deliberate.

The zero floor does not bind anywhere on a 30-year pool, which is why the
reference could not have revealed it. It binds on a short amortising term, where
the balance is nearly gone by the time foreclosure completes — `credit_pool_smoke`
(12 months) fell 9,573.67 → 2,700.77 on this change, far more than the 1–8% the
30-year case suggested.

### 3. A payment struck from a varying divisor — fixed

Generalising the accrual divisor into the level-pay closed form had made
`day_count = "act/360"` recompute *both* interest and scheduled principal from a
divisor that varies with month length, so the implied payment swung 697k–754k
where the market holds it fixed. `amortization_day_count` now strikes the
payment (defaulting to `day_count`, so every existing model is unchanged),
interest accrues on `day_count`, and principal is the plug. Verified against a
hand-built table: constant payment to 1e-6, interest varying by month length.

## Why months 1–348 and not 360

Every column agrees within half a dollar through month 348. The misses fall in
months 349–360, where the reference figures are tens of dollars against a $100m
pool and whole-dollar rounding dominates — the extraction's own consistency
checks flag the same window, where the implied default rate stops matching a flat
1% MDR. That is also the final recovery-lag window, so servicer-advance treatment
of the last payoff is the likely cause. Not worth chasing tens of dollars; worth
recording that it was noticed.

## What the ramped variant would need

150% PSA and 100% SDA. The hazard itself is closed-form —
`CPR = min(speed × 0.2 × max(1, min(MONTH, 30)), 100)`, expressible with
`min`/`clamp` today. The blocker is the balance: every pool factor here is
`pow(k, p)`, valid only for constant `k`. Under a ramp the survival factor is a
cumulative product with no elementary closed form, and the expression language
has no `exp`/`ln` to sum logs instead.

The natural fix is a calc builtin holding the schedule, the same pattern as
`macrs_rate` — not per-period state, which would over-scope it. Backlog.
