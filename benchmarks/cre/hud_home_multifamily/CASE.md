## The case

A 29-year affordable multifamily underwriting. Rents are restricted under a
federal HOME subsidy and revert to market in year 15; four operating expense
lines each escalate on their own schedule; a replacement reserve accrues; and a
permanent mortgage carries both debt service and mortgage insurance, which are
separate obligations rather than one payment.

## The reference

A federal agency's HOME multifamily underwriting template, published as a
spreadsheet together with a populated example. It publishes a full annual cash
flow, so every line is checkable year by year.

**Freely downloadable**, and a populated example ships with it.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | five states, ten native streams |
| Language features | declared state for each escalating expense line and the reserve |
| Conventions | restricted rents reverting to market mid-hold, per-line escalation, a replacement reserve, mortgage insurance separated from debt service |

The five states carry the four operating expense lines and the reserve, each
compounding at its own rate — the shape that `pow(1 + r, t)` gets wrong the
moment a rate moves.

## The result

Net operating income and debt service reproduce the template's own figures:
`domain.cre.noi` = **1,886,475** and `domain.cre.debt_service` = **195,846**.

Asserted: ten stream columns across 29 years, plus the two aggregates.

## The delta

The per-period tolerance is 0.5 — half a dollar — because the template publishes
money to whole dollars while compounding on unrounded balances. Its debt service
coverage ratio, which the template quotes to sixteen figures, agrees to five
decimal places and is asserted far more tightly than the money lines.

One convention this case settled: the template's mortgage payment is principal,
interest **and** mortgage insurance, but insurance is not debt service. Treating
it as such understated the coverage ratio.
