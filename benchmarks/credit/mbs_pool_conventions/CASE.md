## The case

A 30-year agency mortgage pool run at market-standard conventions: a constant
prepayment rate converted to a single monthly mortality, a default rate, a loss
severity on defaulted balances, and a lag before recoveries arrive. These are the
definitional mechanics of mortgage cash flow, and the conversions between them
are where implementations usually diverge.

## The reference

The industry's own standard formulas for analysing mortgage-backed securities —
the document that *defines* CPR, SMM, PSA and SDA. It ships two complete
176-month cash flow schedules, so the comparison is period by period against the
definitions themselves rather than against someone's reading of them.

**Not redistributable.** The publisher forbids reproduction in any form, so the
source is neither vendored nor quoted; its figures are carried as anchor values
and cited as facts.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay` |
| Language features | pack contract lowering to four separate cash flow lines |
| Conventions | CPR-to-SMM conversion, constant default rate, loss severity, recovery lag |

## The result

Interest, scheduled principal, prepayments and recoveries each reproduce as their
own column across the schedule, rather than only in a total. Four separate lines
means a compensating error in one cannot hide in the sum.

Asserted: four stream columns period by period.

## The delta

The tolerance is 0.51 — just over half a dollar — because the published schedule
prints whole dollars while compounding on unrounded balances. Half a dollar is
the closest any implementation can come to a figure rounded to the dollar.

There is no summary metric: a single reduced number would assert less than the
four columns already do.
