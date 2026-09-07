## The case

A 29-year affordable multifamily underwriting. Rents are restricted under a
federal HOME subsidy and revert to market in year 15; four operating expense
lines each escalate on their own schedule; a replacement reserve accrues; and a
permanent mortgage, stated from its sizing terms, carries mortgage insurance as
a separate agreement rather than as part of one payment.

## The reference

A federal agency's HOME multifamily underwriting template, published as a
spreadsheet together with a populated example. It publishes a full annual cash
flow. The model runs monthly, as the mortgage pays, and its annual rollup
reproduces the published rows year by year.

**Freely downloadable**, and a populated example ships with it.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | five states, two contracts, eight native streams |
| Language features | declared state for each escalating expense line and the reserve; a monthly calendar reconciled to an annual pro forma through the results' annual rollup |
| Contracts | `cre.permanent_debt`, `cre.mortgage_insurance` |
| Conventions | restricted rents reverting to market mid-hold, per-line escalation, a replacement reserve, mortgage insurance as an agreement of its own |

The five states carry the four operating expense lines and the reserve, each
compounding at its own rate once a year. The two contracts state the first
mortgage as the template's sizing tab states it: $150,000 at 4.00% over 180
months, and mortgage insurance at 0.450% of the original principal. Together
they reproduce the tab's monthly payment of 1,165.7819 to the fourth decimal.

## The result

Net operating income and debt service reproduce the template's own figures:
`domain.cre.noi` = **1,886,475** and `domain.cre.debt_service` = **195,846**.

Asserted: nine stream and subtotal columns at eleven anchor months across 29
years, the coverage ratio at six of them, plus the two lifetime aggregates.

## The delta

The per-month tolerance is 0.05 — a twelfth of half a dollar — because the
template publishes money to whole dollars a year while compounding on unrounded
balances. Its debt service coverage ratio, which the template quotes to sixteen
figures, agrees to five decimal places and is asserted far more tightly than
the money lines.

The template's mortgage payment is principal, interest **and** mortgage
insurance, rounded to whole dollars a year. The contracts pay the unrounded
payment, 0.38 a year more, and the lifetime debt service is asserted at the
template's figure within that rounding.
