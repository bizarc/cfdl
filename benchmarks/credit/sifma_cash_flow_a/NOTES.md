# SIFMA Cash Flow A — what checking against the definitional source found

## The source, and what may be committed

SIFMA, *Standard Formulas for the Analysis of Mortgage-Backed Securities*
(Uniform Practices Manual, Chapter SF), "Sample Cash Flows", Cash Flow A. Free
direct download from sifma.org; **"reproduction in any form is strictly
forbidden"**.

So this case is validation-only, and the posture matters: the PDF is not
vendored, its tables are not reproduced wholesale, and `expected.csv` carries
only the three columns the pack emits, for the months it can be checked
against. Numbers and formulae are facts and may be asserted against and cited.
This is the opposite of `benchmarks/cre/mit_rentleg_plaza`, whose CC BY-NC-SA
source could have shipped.

## What it asserts

A new 30-year pool: $100m, 8% WAC, 20% loss severity, 12-month recovery,
servicer advances, at a flat **1% SMM and 1% MDR**. Constant hazards, which is
why it is reproducible today — Cash Flow B on the same pool runs 150% PSA and
100% SDA, and those ramps are not expressible (see below).

**1044 published figures across 348 months**, on three streams: Actual
Interest, Actual Amort and Voluntary Prepayments. There is deliberately no
`reference_gen.py` and no `expected_metrics.json`. Every number in this case
came from SIFMA; a second implementation of our own is precisely what it exists
to replace, and an aggregate we computed ourselves would dilute that.

## Two defects it found

### 1. The prepayment base — fixed

The pack applied SMM to the balance **net of defaults**. SIFMA §2a defines SMM
as "the percentage of the mortgage loans outstanding **at the beginning of a
month** assumed to terminate during the month", with
`SMM = (Fsched − F2) / Fsched` where `Fsched = F1 × BAL2/BAL1` — the beginning
balance after *scheduled amortisation only*. Defaults are not removed.

Period 1: SIFMA 999,329, the pack 989,336 — short by exactly the 1% MDR. The
pack's rules header described this behaviour but cited no source; it was a
misreading of the quantity it calls SMM, not a competing convention.

**And the survival factor changes with it**, which is not obvious. Prepayments
taken from the survivors compose multiplicatively, `k = (1−mdr)(1−smm)`. Taken
from the same beginning balance, as SIFMA specifies, they are additive:
`k = (1−mdr) − smm`. The difference is `mdr·smm`, 9,993 in period 1, compounding
over 360. The published ending balance settles it: 97,934,244 additive against
97,944,237 multiplicative. Fixing the base and leaving `k` alone would have
been internally inconsistent — the benchmark caught that too, as a residual
that grew period on period.

The three existing credit benchmarks moved. Their `reference_gen.py` scripts
carried the same misreading as the engine, which is why they had always agreed
— the same failure `tools/analytic-checks.py` was written for.

### 2. The recovery basis — open, deliberately not guessed at

The pack recovers `(1 − severity)` of **face** after the lag. SIFMA continues
to amortise a defaulted loan while it sits in foreclosure — Chapter SF: *"the
amortization schedule continues to be computed even while it is in
foreclosure"* — and recovers `(1 − severity)` of the **amortised** balance.

Measured against Cash Flow A the pack over-recovers by ~1.1% at month 13,
rising to ~7.9% by month 240. Two candidate formulations were tested against
the published Principal Recovery column and neither reproduced it: the
pool-level scheduled-factor ratio `S(m)/S(m−12)` lands within 0.2% early and
drifts to 12% at the tail, and the per-loan remaining-balance ratio is a flat
~7.4% out.

The mechanism is confirmed but the exact formula is not, so **Principal
Recovery is not asserted here** and the pack is unchanged. Guessing at a
finance convention is the failure this whole exercise exists to prevent.
Tracked in `docs/13_feature_backlog.md`.

## Why months 1–348 and not 360

Every column agrees within half a dollar through month 348. All twelve misses
fall in months 349–360, where the published figures are tens of dollars against
a $100m pool and whole-dollar rounding dominates — the extraction's own
internal consistency checks flag the same window, where the implied default
rate stops matching a flat 1% MDR. That is also the final recovery-lag window,
so SIFMA's servicer-advance treatment of the last payoff is the likely cause.
Not worth chasing tens of dollars; worth recording that it was noticed.

## What Cash Flow B would need

150% PSA and 100% SDA. The hazard itself is closed-form —
`CPR = min(speed × 0.2 × max(1, min(MONTH, 30)), 100)`, expressible with
`min`/`clamp` today. The blocker is the balance: every pool factor here is
`pow(k, p)`, valid only for constant `k`. Under a ramp the survival factor is a
cumulative product with no elementary closed form, and the expression language
has no `exp`/`ln` to sum logs instead.

The natural fix is a calc builtin holding the schedule, the same pattern as
`macrs_rate` — not per-period state, which would over-scope it. Backlog.
