---
id: benchmark-cre-hud-home-multifamily
title: "CRE: HOME-funded affordable multifamily"
slug: "/docs/examples/cre-hud-home-multifamily"
source: benchmarks/cre/hud_home_multifamily
---

# CRE: HOME-funded affordable multifamily

A 29-year affordable multifamily underwriting from HUD's HOME Multifamily template, with restricted rents reverting to market at year 15 and a first mortgage that matures before the hold ends.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

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
compounding at its own rate.

## The result

Net operating income and debt service reproduce the template's own figures:
`domain.cre.noi` = **1,886,475** and `domain.cre.debt_service` = **195,846**.

Asserted: ten stream columns across 29 years, plus the two aggregates.

## The delta

The per-period tolerance is 0.5 — half a dollar — because the template publishes
money to whole dollars while compounding on unrounded balances. Its debt service
coverage ratio, which the template quotes to sixteen figures, agrees to five
decimal places and is asserted far more tightly than the money lines.

The template's mortgage payment is principal, interest **and** mortgage
insurance. Insurance is not debt service, and the coverage ratio is computed
without it.

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
// WHY THESE ARE NATIVE STREAMS. Two CRE pack rules nearly fit and do not:
// `cre.property_opex` emits a single un-suffixed stream, so a property cannot
// have more than one expense line; and `cre.vacancy_loss` takes a CONSTANT
// `potential_gross_year`, so vacancy cannot track a rent roll that grows. Both
// The streams below are named into
// the pack's taxonomy so `--pack cre` domain metrics still aggregate them,
// which is the same posture benchmarks/cre/mit_rentleg_plaza takes.
//
// Rounding: the workbook rounds every pro forma line to whole dollars, and
// computes rent loss from the ROUNDED gross rent. We carry full precision, so
// agreement is to the dollar rather than to the cent. That is the source's
// floor, not ours.

version 0.1
model "hud-home-multifamily"
use pack "cre" version "0.1.0"
time calendar annual from 2024-01 for 29

entity asset home_project : CRE.Asset.RealProperty {
  // THE OPERATING LINES ARE THE PROPERTY'S, each escalating on the trend.
  // The trend is the shared assumption; the amounts are facts about this
  // building, so they belong to it.
  // management, escalating on the trend.
  opex_management init inputs.opex_management
       next round_to(prev * (1 + inputs.opex_trend), 1)
  // maintenance, escalating on the trend.
  opex_maintenance init inputs.opex_maintenance
       next round_to(prev * (1 + inputs.opex_trend), 1)
  // utilities, escalating on the trend.
  opex_utilities init inputs.opex_utilities
       next round_to(prev * (1 + inputs.opex_trend), 1)
  // taxes and ins, escalating on the trend.
  opex_taxes_ins init inputs.opex_taxes_ins
       next round_to(prev * (1 + inputs.opex_trend), 1)

  // The replacement reserve, on the same trend.
  reserve init inputs.reserve_y1
          next round_to(prev * (1 + inputs.opex_trend), 1)
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
// The first mortgage, from the workbook's First Mortgage Sizing tab. Its
// published payment is labeled "Calculated Monthly P+I+MIP Payment" — the
// three are one line on the pro forma, and only two of them are debt service.
assume first_mortgage     = 150000.00   // sizing tab, calculated loan amount
assume mip_rate           = 0.0045      // sizing tab, 0.450% of original principal
// Displayed to four places; the pro forma rounds the annual figure to whole
// dollars anyway, so the digits beyond these are immaterial to the result.
assume pi_mip_monthly     = 1165.7819   // sizing tab, monthly P+I+MIP

// Restriction runs through year 14; year 15 is the first at market rents. The
// assumptions tab states a 15-year affordability period, and the workbook's own
// switch fires one year earlier than that label reads — see NOTES.md.
assume restricted_years   = 14

// ---------------------------------------------------------------------------
// Revenue
// ---------------------------------------------------------------------------

stream cre.unit.base_rent.home on entity asset.home_project inflow currency USD {
  schedule every year from 2024-01 to 2052-01
  category operating.revenue.base_rent
  amount = if(time.t < inputs.restricted_years,
            inputs.rent_restricted_y1 * pow(1 + inputs.rent_trend, time.t),
            inputs.rent_market_y1 * pow(1 + inputs.rent_trend, time.t))
}

stream cre.ops.revenue on entity asset.home_project inflow currency USD {
  schedule every year from 2024-01 to 2052-01
  category operating.revenue.other
  amount = inputs.other_income_y1 * pow(1 + inputs.other_trend, time.t)
}

// Vacancy tracks the active rent track, so it steps at the cliff too.
stream cre.vacancy.loss on entity asset.home_project outflow currency USD {
  schedule every year from 2024-01 to 2052-01
  category operating.deduction.vacancy
  amount = inputs.vacancy_rate *
           if(time.t < inputs.restricted_years,
             inputs.rent_restricted_y1 * pow(1 + inputs.rent_trend, time.t),
             inputs.rent_market_y1 * pow(1 + inputs.rent_trend, time.t))
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
// A declared state expresses that recurrence directly.
// One state per sub-line, because each is rounded on its own before the sum.


// One stream per PUBLISHED sub-line. The workbook reports these four
// separately — it escalates and rounds each on its own before summing — and
// until `cre.property_opex` took a suffix a model could declare exactly one
// expense line, so they had to be added together here and the four published
// figures could not be checked against anything.
//
// The states were already per-sub-line for the rounding reason, so this is a
// decomposition and not a change: the four sum to what the single stream
// carried, to the cent.
stream cre.property.opex.management on entity asset.home_project outflow currency USD {
  schedule every year from 2024-01 to 2052-01
  category operating.expense.opex
  amount = asset.home_project.opex_management
}

stream cre.property.opex.maintenance on entity asset.home_project outflow currency USD {
  schedule every year from 2024-01 to 2052-01
  category operating.expense.opex
  amount = asset.home_project.opex_maintenance
}

stream cre.property.opex.utilities on entity asset.home_project outflow currency USD {
  schedule every year from 2024-01 to 2052-01
  category operating.expense.opex
  amount = asset.home_project.opex_utilities
}

stream cre.property.opex.taxes_insurance on entity asset.home_project outflow currency USD {
  schedule every year from 2024-01 to 2052-01
  category operating.expense.opex
  amount = asset.home_project.opex_taxes_ins
}

// The replacement reserve is its own published line and is semantically not an
// operating expense — HUD reports it below total expenses — but it does sit
// above NOI, which is why it is an operating deduction rather than capital.
stream cre.ops.expense on entity asset.home_project outflow currency USD {
  schedule every year from 2024-01 to 2052-01
  category operating.expense.opex
  amount = asset.home_project.reserve
}

// ---------------------------------------------------------------------------
// Debt — level annual payment for 14 years, then the first mortgage matures.
// Named to match what domain.cre.debt_service reads.
// ---------------------------------------------------------------------------

// The pro forma carries ONE debt line and the workbook defines it as P+I+MIP,
// so it was modeled as one number and the two published components could not
// be checked separately. Mortgage insurance is not a payment on the debt —
// and coverage here is measured against the whole line, which is
// what `financing.*` folds to.
//
// MIP is the sizing tab's stated 0.450% of original principal, flat and exact.
// Debt service is the residual.
//
// THE ROUND IS THE WORKBOOK'S, NOT A FUDGE. The pro forma's debt cell is
// `=ROUND(...,0)`, so 13,989 is what it COMPUTES and not what it displays, and
// the DSCR it publishes is that rounded line divided into a rounded NOI. Using
// the sizing tab's unrounded 13,989.3828 instead would be more precise and less
// accurate — it would leave a 0.38 residual against every published debt line.
//
// Written as the workbook's arithmetic rather than as its answer: the same
// `round_to` this model already uses for the expense recurrence, applied to the
// published monthly payment. So the derivation is visible and tracks the sizing
// inputs, instead of a 13,989 constant that would not. The 0.38 the round
// discards belongs to the P&I leg, which is the leg the workbook rounded.
stream loan.permanent_debt_service on entity asset.home_project outflow currency USD {
  schedule every year from 2024-01 to 2037-01
  category financing.debt_service
  amount = round_to(inputs.pi_mip_monthly * 12, 1) - inputs.first_mortgage * inputs.mip_rate
}

stream loan.mortgage_insurance on entity asset.home_project outflow currency USD {
  schedule every year from 2024-01 to 2037-01
  category financing.mortgage_insurance
  amount = inputs.first_mortgage * inputs.mip_rate
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.10}}
```

## Verified results

Checked period by period: **13 series** across **11 periods** — **138 values** in all, each within the tolerance shown.

- `cre.unit.base_rent.home` — within ±0.5
- `cre.vacancy.loss` — within ±0.5
- `cre.ops.revenue` — within ±0.5
- `cre.property.opex.management` — within ±0.5
- `cre.property.opex.maintenance` — within ±0.5
- `cre.property.opex.utilities` — within ±0.5
- `cre.property.opex.taxes_insurance` — within ±0.5
- `cre.ops.expense` — within ±0.5
- `loan.permanent_debt_service` — within ±0.5
- `loan.mortgage_insurance` — within ±0.5
- `domain.cre.egi` — within ±1.0
- `domain.cre.noi` — within ±1.0
- `domain.cre.dscr` — within ±1.0e-4

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `domain.cre.noi` | 1,886,475 | ±130 |
| `domain.cre.debt_service` | 195,846 | ±1 |
