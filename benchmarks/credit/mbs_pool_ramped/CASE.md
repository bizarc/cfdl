## The case

The same 30-year agency mortgage pool, but on a **ramping** prepayment curve:
speeds build month by month over the first thirty months, then level off. A ramp
is the standard market assumption for a seasoning pool, and it is the case a
constant-hazard shortcut gets wrong.

## The reference

The industry's own standard formulas for analysing mortgage-backed securities,
which define the ramp and publish a complete cash flow schedule computed on it.

**Not redistributable.** The publisher forbids reproduction in any form, so the
source is neither vendored nor quoted; its figures are carried as anchor values
and cited as facts.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay` |
| Language features | pack contract lowering to four cash flow lines; a per-period pool factor carried as state |
| Conventions | a prepayment ramp over thirty months, CPR-to-SMM conversion, default, severity, recovery lag |

The ramp is why this case exists alongside the constant-speed one. Under a
changing hazard the surviving balance is a running product, and a closed-form
`pow(k, p)` is exact only while the rate holds still.

## The result

Interest, scheduled principal, prepayments and recoveries each reproduce as their
own column across the schedule.

Asserted: four stream columns period by period.

## The delta

The tolerance is 0.51 — just over half a dollar — set by the published
schedule's whole-dollar rounding rather than by anything about this pool.
