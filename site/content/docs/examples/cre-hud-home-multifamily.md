---
id: benchmark-cre-hud-home-multifamily
title: "CRE: HOME-funded affordable multifamily"
slug: "/docs/examples/cre-hud-home-multifamily"
description: "A 29-year affordable multifamily underwriting from HUD's HOME Multifamily template, with restricted rents reverting to market at year 15 and a first mortgage, stated from its sizing terms, that stops paying before the hold ends."
source: benchmarks/cre/hud_home_multifamily
---

# CRE: HOME-funded affordable multifamily

A 29-year affordable multifamily underwriting from HUD's HOME Multifamily template, with restricted rents reverting to market at year 15 and a first mortgage, stated from its sizing terms, that stops paying before the hold ends.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

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

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
// HUD HOME Multifamily Underwriting Template — populated Sample workbook.
//
// A 20-unit HOME-assisted rental development, 29-year operating pro forma.
//
// THIS IS THE ONE SOURCE WE CAN SHIP. It is a US federal work dedicated to the
// public domain, so unlike every other external case in this repo the reference
// workbook itself sits beside this model, in reference/, and the source can be
// named rather than described. See NOTES.md.
//
// THE AFFORDABILITY CLIFF is the interesting mechanic. HOME-assisted units are
// rent-restricted for the affordability period and revert to market rents after
// it. The workbook carries both tracks side by side and switches between them,
// so gross rent steps 199,062 -> 290,708 between years 14 and 15 — a 46% jump
// that dwarfs the 2% trend either side of it. A model that got the trend right
// and the switch wrong would look correct for thirteen years.
//
// THE CALENDAR IS MONTHLY BECAUSE THE INSTRUMENTS ARE. The mortgage pays
// monthly, the rent roll is monthly, and the vacancy reads the rent. The
// workbook publishes an ANNUAL pro forma, which is a view of the results —
// the annual rollup — and never a reason for an annual calendar. Every
// published line is level within a year, so each month carries a twelfth of
// the year's figure and the monthly coverage ratio is the annual one.
//
// WHY THE OPERATING LINES ARE NATIVE STREAMS. `cre.vacancy_loss` takes a
// CONSTANT `potential_gross_year`, so vacancy cannot track a rent roll that
// steps 46% at the cliff; and the four expense lines escalate by a ROUNDED
// RECURRENCE the pack's closed form cannot state. The streams are named into
// the pack's taxonomy so `--pack cre` domain metrics still aggregate them,
// which is the same posture benchmarks/cre/mit_rentleg_plaza takes. The
// mortgage and its insurance are the pack's: `cre.permanent_debt` and
// `cre.mortgage_insurance`, stated as the sizing tab states them.
//
// Rounding: the workbook rounds every pro forma line to whole dollars, and
// computes rent loss from the ROUNDED gross rent. We carry full precision, so
// agreement is to the dollar a year rather than to the cent. That is the
// source's floor, not ours.

version 0.1
model "hud-home-multifamily"
use pack "cre" version "0.1.0"
time calendar monthly from 2024-01 for 348

entity asset home_project : CRE.Asset.RealProperty {
  // The affordability regime, as a fact about the building. An event clears it
  // once and permanently; every line keyed to rent reads it rather than
  // restating the switch.
  restricted init 1.0

  // THE OPERATING LINES ARE THE PROPERTY'S, each an ANNUAL figure escalating
  // on the trend once a year. The trend is the shared assumption; the amounts
  // are facts about this building, so they belong to it. Each steps in
  // January — the workbook's year boundary — and holds through the year; the
  // streams below pay a twelfth of it each month.
  // management, escalating on the trend.
  opex_management init inputs.opex_management
       next if(time.t - 12 * round_down(time.t / 12, 0) == 0,
               round_to(prev * (1 + inputs.opex_trend), 1), prev)
  // maintenance, escalating on the trend.
  opex_maintenance init inputs.opex_maintenance
       next if(time.t - 12 * round_down(time.t / 12, 0) == 0,
               round_to(prev * (1 + inputs.opex_trend), 1), prev)
  // utilities, escalating on the trend.
  opex_utilities init inputs.opex_utilities
       next if(time.t - 12 * round_down(time.t / 12, 0) == 0,
               round_to(prev * (1 + inputs.opex_trend), 1), prev)
  // taxes and ins, escalating on the trend.
  opex_taxes_ins init inputs.opex_taxes_ins
       next if(time.t - 12 * round_down(time.t / 12, 0) == 0,
               round_to(prev * (1 + inputs.opex_trend), 1), prev)

  // The replacement reserve, on the same trend.
  reserve init inputs.reserve_y1
          next if(time.t - 12 * round_down(time.t / 12, 0) == 0,
                  round_to(prev * (1 + inputs.opex_trend), 1), prev)
}

// ---------------------------------------------------------------------------
// Stated in the Sample workbook's Pro Forma Assumptions tab.
// ---------------------------------------------------------------------------

assume rent_restricted_y1 = 153881.28   // HOME-restricted gross rent, year 1
assume rent_market_y1     = 220320.00   // the market track the same units revert to
assume rent_trend         = 0.02
assume other_income_y1    = 2448.00
assume other_trend        = 0.02
assume vacancy_rate       = 0.07
// The four published expense sub-lines, not their total. The workbook
// escalates and ROUNDS each one independently and then sums, so rounding the
// total is not the same arithmetic — 102,501 is the sum, never an input.
assume opex_management    = 37413.00
assume opex_maintenance   = 37925.00
assume opex_utilities     = 12300.00
assume opex_taxes_ins     = 14863.00
assume opex_trend         = 0.025
assume reserve_y1         = 21013.00    // replacement reserve deposit
// The first mortgage, from the workbook's First Mortgage Sizing tab: the
// loan's TERMS, not its payment. The tab states $150,000 at 4.00% over a
// 15-year term, self-amortizing, paid monthly, with mortgage insurance at
// 0.450% of the original principal. Its "Calculated Monthly P+I+MIP Payment"
// of 1,165.7819 is what those terms produce, and the contracts below
// reproduce it to the fourth decimal.
assume first_mortgage     = 150000.00   // sizing tab, lender's proposed loan amount
assume mortgage_rate      = 0.04        // sizing tab, interest rate
// The amortization term is a literal: a months term converts to periods at
// compile time (E5017), so it is stated on the contract — 180, the sizing
// tab's 15-year term.
assume mip_rate           = 0.0045      // sizing tab, 0.450% of original principal

// Restriction runs through year 14; year 15 is the first at market rents. The
// assumptions tab states a 15-year affordability period, and the workbook's own
// switch fires one year earlier than that label reads — see NOTES.md.
//
// THE EVENT BELOW IS WHY THAT DISCREPANCY IS NOW AUDITABLE rather than only
// explained in a comment. The restriction expires once and does not come back,
// and the topology says so: nothing returns the entity to the restricted
// state, which is how once-ness is declared.
// The run publishes the transition, so the period and date the reversion took
// effect are in the results and can be checked against the source workbook
// instead of re-derived from a `<` in an expression.
assume restricted_years   = 14

event affordability_expires when time.t >= inputs.restricted_years * 12 {
  set entity asset.home_project.restricted = 0.0
}

// ---------------------------------------------------------------------------
// Revenue
// ---------------------------------------------------------------------------

stream cre.unit.base_rent.home on entity asset.home_project inflow currency USD {
  schedule every month from 2024-01 to 2052-12
  category operating.revenue.base_rent
  // An annual figure trending once a year, paid by the month.
  amount = if(asset.home_project.restricted == 1.0,
            inputs.rent_restricted_y1,
            inputs.rent_market_y1)
           * pow(1 + inputs.rent_trend, round_down(time.t / 12, 0)) / 12
}

stream cre.ops.revenue on entity asset.home_project inflow currency USD {
  schedule every month from 2024-01 to 2052-12
  category operating.revenue.other
  amount = inputs.other_income_y1 * pow(1 + inputs.other_trend, round_down(time.t / 12, 0)) / 12
}

// Vacancy tracks the active rent track, so it steps at the cliff too — by
// READING the rent it is a percentage of, rather than restating how rent is
// computed. The switch is stated once, in the event above.
stream cre.vacancy.loss on entity asset.home_project outflow currency USD {
  schedule every month from 2024-01 to 2052-12
  category operating.deduction.vacancy
  amount = inputs.vacancy_rate
           * series_sum("cre.unit.base_rent.*", time.t, time.t)
}

// ---------------------------------------------------------------------------
// Expenses — total operating expense and the replacement reserve are separate
// published lines, and both feed the NOI metric's denominator.
// ---------------------------------------------------------------------------

// THE WORKBOOK ESCALATES BY A RECURRENCE, not by a closed form. Year n is last
// year's ALREADY-ROUNDED figure times the trend, rounded again to whole
// dollars — verified directly against two of its four expense sub-lines, which
// reproduce exactly under the recurrence and under no closed form.
//
// `pow(1 + trend, t)` cannot express that: it compounds exact decimals from the
// base, and rounding does not commute with exponentiation, so the two paths
// separate a little more every year. That left a 12.26 residual at year 29 and
// was the sole reason this case carried period_tolerance = 13.
//
// A declared state expresses that recurrence directly. One state per
// sub-line, because each is rounded on its own before the sum; each holds an
// ANNUAL figure and steps in January, and the stream pays a twelfth of it.


// One stream per PUBLISHED sub-line. The workbook reports these four
// separately — it escalates and rounds each on its own before summing — and
// until `cre.opex_line` took a suffix a model could declare exactly one
// expense line, so they had to be added together here and the four published
// figures could not be checked against anything.
//
// The states were already per-sub-line for the rounding reason, so this is a
// decomposition and not a change: the four sum to what the single stream
// carried, to the cent.
stream cre.opex.line.management on entity asset.home_project outflow currency USD {
  schedule every month from 2024-01 to 2052-12
  category operating.expense.opex
  amount = asset.home_project.opex_management / 12
}

stream cre.opex.line.maintenance on entity asset.home_project outflow currency USD {
  schedule every month from 2024-01 to 2052-12
  category operating.expense.opex
  amount = asset.home_project.opex_maintenance / 12
}

stream cre.opex.line.utilities on entity asset.home_project outflow currency USD {
  schedule every month from 2024-01 to 2052-12
  category operating.expense.opex
  amount = asset.home_project.opex_utilities / 12
}

stream cre.opex.line.taxes_insurance on entity asset.home_project outflow currency USD {
  schedule every month from 2024-01 to 2052-12
  category operating.expense.opex
  amount = asset.home_project.opex_taxes_ins / 12
}

// The replacement reserve is its own published line and is semantically not an
// operating expense — HUD reports it below total expenses — but it does sit
// above NOI, which is why it is an operating deduction rather than capital.
stream cre.opex.line on entity asset.home_project outflow currency USD {
  schedule every month from 2024-01 to 2052-12
  category operating.expense.opex
  amount = asset.home_project.reserve / 12
}

// ---------------------------------------------------------------------------
// Debt — the first mortgage and its insurance, as the sizing tab states them.
// ---------------------------------------------------------------------------

// The pro forma carries ONE debt line and the workbook defines it as P+I+MIP.
// Only two of the three are debt service, so they are two contracts: the
// loan, and the mortgage insurance on it. `cre.permanent_debt` lowers the
// interest and principal legs from the loan's terms — $150,000 at 4.00%
// amortizing over 180 months — and their sum is the level payment,
// 1,109.5319 a month; `cre.mortgage_insurance` pays 0.450% of the original
// principal a year, 56.25 a month. Together they are the sizing tab's
// 1,165.7819 to the fourth decimal, with nothing transcribed.
//
// THE WORKBOOK PAYS FOURTEEN OF THE FIFTEEN YEARS. Its pro forma carries the
// payment through year 14 and shows zero from year 15, the same one-year
// slip as the affordability period — so the term here ends in 2037, and the
// balance the loan's account still carries after the last payment (13,030)
// is the year the workbook did not pay. A hand-written stream simply stopped;
// a contract shows what stopping leaves behind.
//
// Coverage is measured on the whole published line: the pack's
// `domain.cre.debt_service` subtotal includes mortgage insurance because this
// source, the one that carries it, defines coverage as NOI over P+I+MIP.
contract cre.permanent_debt on entity asset.home_project {
  term 2024-01..2037-12
  terms {
    principal = inputs.first_mortgage
    interest_rate = inputs.mortgage_rate
    amortization_months = 180
    funded_at_close = 0
  }
}

contract cre.mortgage_insurance on entity asset.home_project {
  term 2024-01..2037-12
  terms {
    premium_rate = inputs.mip_rate
    coverage = inputs.first_mortgage
  }
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.10}}
```

## Verified results

Checked period by period: **13 series** across **11 periods** — **138 values** in all, each within the tolerance shown.

- `cre.unit.base_rent.home` — within ±0.05
- `cre.vacancy.loss` — within ±0.05
- `cre.ops.revenue` — within ±0.05
- `cre.opex.line.management` — within ±0.05
- `cre.opex.line.maintenance` — within ±0.05
- `cre.opex.line.utilities` — within ±0.05
- `cre.opex.line.taxes_insurance` — within ±0.05
- `cre.opex.line` — within ±0.05
- `cre.mortgage_insurance.premium` — within ±0.05
- `domain.cre.debt_service` — within ±0.05
- `domain.cre.egi` — within ±0.1
- `domain.cre.noi` — within ±0.1
- `domain.cre.dscr` — within ±1.0e-4

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `domain.cre.noi` | 1,886,475 | ±130 |
| `domain.cre.debt_service` | 195,846 | ±6 |
