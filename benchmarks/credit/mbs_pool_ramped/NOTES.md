# MBS pool at ramped conventions — 150% PSA, 100% SDA

## The reference, and what may be committed

The published industry reference schedule for MBS cash flows works one pool
twice: at a flat 1% SMM / 1% MDR, and at 150% PSA with 100% SDA. The flat run is
`benchmarks/credit/mbs_pool_conventions`. This is the ramped run.

The source is free to download and states that reproduction in any form is
forbidden. It is not vendored, its tables are not reproduced, and it is cited
once here for provenance. The figures below are anchor values carried for
regression, cited as facts.

## What it asserts

Same pool as the flat case: $100m, 8% WAC, 20% loss severity, 12-month recovery.
25 anchors across 348 months, four streams — interest, scheduled principal,
voluntary prepayments and recoveries — all within `period_tolerance = 0.51`,
which is the whole-dollar rounding floor of the source.

The pack derives its per-period rates from the two speeds stated as multiples.
They agree with the reference's own stated rate columns at month 1: monthly
prepayment 0.000250, monthly default 0.000017.

## Why this run was unreachable

Under a ramp the survival factor is a cumulative product with no elementary
closed form. Every pool factor in the pack was `pow(k, p)`, which is that
product only when `k` is constant. Closed by declared state variables
(`docs/13_feature_backlog.md` 2.1).

## The finding: the lagged pool factor read the wrong point of the curve

Recoveries are taken on the balance that defaulted `lag` months earlier, so the
recoveries rules read a lagged survival state. That state advances on ticks
after the lag, and its step `j` must consume the hazard of age `j-1`, which is
`elapsed - lag - 1`. It was consuming `elapsed - 1`.

Under a constant hazard the two are identical, so nothing saw it: the flat case,
all 111 goldens and the lag identity in `tools/analytic-checks.py` all passed.
Against this ramped reference the error is monotone in months compounded and
reaches **7.6%** on recoveries by month 60 — the curve rises, so reading a later
age overstates the hazard, shrinks the balance that defaults, and understates
what comes back.

Interest, scheduled principal and prepayments were correct throughout. Only the
lagged path was wrong, and only a ramp could show it.

The lag identity now runs at 150% PSA / 100% SDA rather than a flat hazard, and
fails if the lagged age is restored to `elapsed - 1`.

## Anchors, not all 348 months

Same reasoning as the flat case: a convention error appears in every period, so
25 anchors catch it as reliably as 348 with an order of magnitude less
extraction. Full reconciliation was performed; only the anchors are committed.
