---
id: benchmark-cre-mit-rentleg-plaza
title: "CRE: rent-regulated plaza"
slug: "/docs/examples/cre-mit-rentleg-plaza"
source: benchmarks/cre/mit_rentleg_plaza
---

# CRE: rent-regulated plaza

A five-year office acquisition and disposition from MIT's real estate finance course, valued on a levered before-tax cash flow with an exit at a stated cap rate.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A 30,000 rentable square foot office building with two suites, acquired and held
for five years. The two suites sit at different expense stops, so recoveries
differ between them; the stop resets to a new base year when a suite re-lets.
Operating expenses vary with occupancy, rollover at expiry is
probability-weighted, market rent spikes once during the hold, and the building
is sold at ten times forward net operating income net of a 5% commission.

## The reference

Problem Set 1 from MIT OpenCourseWare's real estate finance and investment
course. It publishes the full pro forma table **and** the answer: a present
value at 12% of **$2,292,810**.

**Redistributable.** Released under CC BY-NC-SA 4.0.

The source publishes both the working and the answer, so the case checks every
intermediate line as well as the result.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.exit_forward` |
| Declared | seven native streams |
| Language features | native streams alongside a pack contract |
| Conventions | two expense stops at different levels, a base-year stop reset on re-lease, occupancy-varying operating expenses, probability-weighted rollover, a forward-NOI reversion |

## The result

Every pro forma line reproduces, and so does the published answer:
`model.npv` = **2,292,810.18** against the problem set's $2,292,810.

Asserted: eight stream columns across the five-year table, plus the present
value and the undiscounted total.

## The delta

The 18 cents is the source's rounding, not the engine's — the problem set states
its answer to the dollar. Every per-period line agrees inside a one-cent
tolerance.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.12}}
// Rentleg Plaza — MIT OCW 11.431J/15.426J Problem Set 1, Part C.
//
// External reference case. 30,000 NRSF office, two suites, 5-year annual
// pro forma (2001-2005) with a 2006 projection year for the reversion.
// The problem set publishes the answer: PV @ 12% = $2,292,810.
//
// Source: MIT OpenCourseWare 11.431J Fall 2006, Problem Set 1 (CC BY-NC-SA).
// Footnote markers below (MIT fn N) refer to the footnotes under the
// published pro forma table, which define how each tagged number is computed.
//
// NOTE ON PACK USE: the reversion is a CRE pack contract. The operating
// streams are still native, because two pack features this deal needs do not
// exist yet: occupancy-varying opex (MIT fn 7 splits it 81% fixed / 19%
// variable) and an expense stop that resets to a computed later-year value
// (fn 5). They are named to the pack's stream taxonomy so `--pack cre` domain
// metrics aggregate them and `cre.exit_forward` can read them. See NOTES.md.
//
// NOTE ON THE 2006 COLUMN: the reversion needs 2006 NOI, one year past the
// hold. It is derived by `cre.exit_forward` from the modelled streams over the
// `project 1` tail. That was impossible when this file was written — E2103
// measured a native stream against the cash horizon, so the operating streams
// could not reach the tail — and the 2006 NOI was carried inline, duplicating
// the opex formula. See NOTES.md.

version 0.1
model "mit-rentleg-plaza"
use pack "cre" version "0.1.0"
time calendar annual from 2001-01 for 5 project 1

entity asset rentleg : CRE.Asset.RealProperty

// ---------------------------------------------------------------------------
// Inputs — every figure below is stated in the problem set.
// ---------------------------------------------------------------------------

assume building_sf      = 30000
assume suite_100_sf     = 20000
assume suite_200_sf     = 10000

assume suite_100_rent_psf = 15.00   // in-place lease, signed 1/99, expires 12/03
assume market_rent_psf    = 14.00   // prevailing market rent, soft market
assume rent_spike         = 0.20    // one-time 20% step in 2004, flat thereafter

assume opex_psf_full   = 4.81       // at 100% occupancy, projected 2001
assume opex_growth     = 0.04
assume opex_pct_fixed  = 0.81       // remaining 19% varies directly with occupancy

assume capex_psf       = 1.00       // general capital improvements, uninflated

assume suite_100_stop_psf = 4.00    // MIT fn 4 — stop in the in-place lease
assume suite_200_stop_psf = 5.00    // MIT fn 6 — stop in the new Suite 200 lease

assume renewal_prob    = 0.50       // Suite 100 rollover at 12/03
assume ti_new_psf      = 10.00
assume ti_renew_psf    = 3.00
assume lc_new_pct      = 0.06
assume lc_renew_pct    = 0.03
assume downtime_years  = 0.50       // 6 months vacancy if the tenant leaves
assume abatement_months = 5         // 1 month free per year of a 5-year term


// Suite 100's replacement lease resets its stop to actual 2004 opex/SF
// (MIT fn 5). Opex per SF is closed-form in t, so the 2004 value is stated
// here as a constant. The engine has no way to read another period's phase-1
// value, so the opex formula is necessarily duplicated — see NOTES.md.
assume opex_psf_2004 = 4.81 * pow(1.04, 3) * (0.81 + (5.0 / 6.0) * 0.19)


// ---------------------------------------------------------------------------
// Potential gross revenue — rent roll
// ---------------------------------------------------------------------------

// Suite 100: contract rent at $15.00/SF through 2003, then re-leased at the
// post-spike market rent of $16.80/SF from 2004.
stream cre.unit.base_rent.suite_100 on entity asset.rentleg inflow currency USD {
  schedule every year from 2001-01 to 2006-01
  category operating.revenue.base_rent
  amount = inputs.suite_100_sf * if(time.t <= 2,
             inputs.suite_100_rent_psf,
             inputs.market_rent_psf * (1 + inputs.rent_spike))
}

// Suite 200: vacant in 2001 (offset by the vacancy line below), then a 5-year
// lease signed 1/02 at the then-current $14.00/SF, flat through 2006.
stream cre.unit.base_rent.suite_200 on entity asset.rentleg inflow currency USD {
  schedule every year from 2001-01 to 2006-01
  category operating.revenue.base_rent
  amount = inputs.suite_200_sf * inputs.market_rent_psf
}

// ---------------------------------------------------------------------------
// Deductions from potential gross revenue
// ---------------------------------------------------------------------------

// MIT fn 1 — Suite 200 vacant all of 2001.
// MIT fn 2 — Suite 100 expected 2004 vacancy: 50% x 6mo = 25% of its PGR.
stream cre.vacancy.loss on entity asset.rentleg outflow currency USD {
  schedule every year from 2001-01 to 2006-01
  category operating.deduction.vacancy
  amount = if(time.t == 0,
            inputs.suite_200_sf * inputs.market_rent_psf,
            if(time.t == 3,
              (1 - inputs.renewal_prob) * inputs.downtime_years
                * inputs.suite_100_sf * inputs.market_rent_psf * (1 + inputs.rent_spike),
              0))
}

// MIT fn 3 — 5 months free rent on the new Suite 200 lease, taken in 2002.
stream cre.abatement.suite_200 on entity asset.rentleg outflow currency USD {
  schedule every year from 2001-01 to 2006-01
  category operating.deduction.abatement
  amount = if(time.t == 1,
            (inputs.abatement_months / 12) * inputs.suite_200_sf * inputs.market_rent_psf,
            0)
}

// ---------------------------------------------------------------------------
// Operating expenses
//
// $4.81/SF at full occupancy, growing 4%/yr, of which 81% is fixed and 19%
// varies directly with occupancy. Occupancy: 2/3 in 2001 (Suite 200 dark),
// full in 2002-03, 5/6 in 2004 (MIT fn 7 — Suite 100 dark 0.25yr on 20k SF
// of 30k), full thereafter.
// ---------------------------------------------------------------------------

stream cre.property.opex on entity asset.rentleg outflow currency USD {
  schedule every year from 2001-01 to 2006-01
  category operating.expense.opex
  amount = inputs.building_sf * inputs.opex_psf_full
           * pow(1 + inputs.opex_growth, time.t)
           * (inputs.opex_pct_fixed
              + (1 - inputs.opex_pct_fixed)
                * if(time.t == 0, 2.0 / 3.0, if(time.t == 3, 5.0 / 6.0, 1.0)))
}

// ---------------------------------------------------------------------------
// Expense reimbursements
//
// Full-service leases with an expense stop: the tenant pays its pro-rata share
// of actual opex per SF above the stop stated in its own lease. Stops are NOT
// grossed up to full occupancy here — the problem set is explicit that the
// stop is tested against actual building opex however full or vacant it is.
// ---------------------------------------------------------------------------

// MIT fn 4 (2001-03, $4.00 stop) and fn 5 (2005-06, stop reset to actual
// 2004 opex/SF, which makes the 2004 reimbursement exactly zero).
stream cre.unit.recoveries.suite_100 on entity asset.rentleg inflow currency USD {
  schedule every year from 2001-01 to 2006-01
  category operating.revenue.recovery
  amount = inputs.suite_100_sf
           * max(0,
               (inputs.building_sf * inputs.opex_psf_full
                 * pow(1 + inputs.opex_growth, time.t)
                 * (inputs.opex_pct_fixed
                    + (1 - inputs.opex_pct_fixed)
                      * if(time.t == 0, 2.0 / 3.0, if(time.t == 3, 5.0 / 6.0, 1.0)))
               ) / inputs.building_sf
               - if(time.t <= 2, inputs.suite_100_stop_psf, inputs.opex_psf_2004))
}

// MIT fn 6 — $5.00/SF stop, running from the 2002 lease commencement.
stream cre.unit.recoveries.suite_200 on entity asset.rentleg inflow currency USD {
  schedule every year from 2002-01 to 2006-01
  category operating.revenue.recovery
  amount = inputs.suite_200_sf
           * max(0,
               (inputs.building_sf * inputs.opex_psf_full
                 * pow(1 + inputs.opex_growth, time.t)
                 * (inputs.opex_pct_fixed
                    + (1 - inputs.opex_pct_fixed)
                      * if(time.t == 3, 5.0 / 6.0, 1.0))
               ) / inputs.building_sf
               - inputs.suite_200_stop_psf)
}

// ---------------------------------------------------------------------------
// Leasing and capital expenditures (below NOI)
// ---------------------------------------------------------------------------

// Suite 200 lease-up in 2002: $10/SF TI, plus a 6% commission struck on
// cumulative lease revenue net of the free-rent concession (MIT fn 9):
//   0.06 * (5 * $14 - (5/12) * $14) * 10,000 SF
stream cre.unit.ti_lc.suite_200 on entity asset.rentleg outflow currency USD {
  schedule every year from 2002-01 to 2002-01
  category investing.capital.leasing
  amount = inputs.suite_200_sf
           * (inputs.ti_new_psf
              + inputs.lc_new_pct
                * (5 * inputs.market_rent_psf
                   - (inputs.abatement_months / 12) * inputs.market_rent_psf))
}

// Suite 100 rollover in 2004, probability-weighted across renew / re-let.
// MIT fn 8  — TI:  (50% * $10 + 50% * $3) * 20,000 SF
// MIT fn 10 — LC:  (50% * 6% + 50% * 3%) * (5 * $16.80) * 20,000 SF
//                  (no abatement deduction: concessions are gone by 2004)
stream cre.unit.ti_lc.suite_100 on entity asset.rentleg outflow currency USD {
  schedule every year from 2004-01 to 2004-01
  category investing.capital.leasing
  amount = inputs.suite_100_sf
           * ((inputs.renewal_prob * inputs.ti_renew_psf
               + (1 - inputs.renewal_prob) * inputs.ti_new_psf)
              + (inputs.renewal_prob * inputs.lc_renew_pct
                 + (1 - inputs.renewal_prob) * inputs.lc_new_pct)
                * 5 * inputs.market_rent_psf * (1 + inputs.rent_spike))
}

// $1.00/SF/yr of general capital improvements, uninflated, over the hold.
stream cre.capex on entity asset.rentleg outflow currency USD {
  schedule every year from 2001-01 to 2005-01
  category investing.capital.capex
  amount = inputs.building_sf * inputs.capex_psf
}

// ---------------------------------------------------------------------------
// Reversion — MIT fn 11
//
// Sale at the end of 2005 for 10x the FOLLOWING year's NOI, net of a 5%
// selling commission. The 2006 NOI is derived from the modeled streams over
// the projection tail rather than restated as an input. Outflow streams carry
// a negative sign, so the terms simply add.
// ---------------------------------------------------------------------------

// MIT fn 11 — the reversion, as a CRE pack contract.
//
// This is the acceptance test for annual-calendar lowering, and it depends on
// three behaviours at once:
//
//   the CRE pack lowers on any calendar it declares, not monthly alone;
//   a native stream is measured against the projection horizon, so the
//     operating streams reach the `project 1` tail this contract reads;
//   a disposal settles at period end, so the exit discounts from the end of
//     2005 rather than its start — five years, not four, worth $207,783 here.
//
// `exit_cap = 0.10` is MIT's "10 times the following year's NOI"; the 5%
// selling commission is fn 11. The forward NOI is derived from the modelled
// streams, not restated.
contract cre.exit_forward on entity asset.rentleg {
  term 2005-01..2005-01
  terms {
    exit_cap = 0.10
    selling_costs = 0.05
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.12
  }
}
```

## Verified results

Checked period by period: **12 series** across **5 periods** — **60 values** in all, each within ±0.01 of the reference.

- `cre.abatement.suite_200`
- `cre.capex`
- `cre.exit.proceeds`
- `cre.property.opex`
- `cre.unit.base_rent.suite_100`
- `cre.unit.base_rent.suite_200`
- `cre.unit.recoveries.suite_100`
- `cre.unit.recoveries.suite_200`
- `cre.unit.ti_lc.suite_100`
- `cre.unit.ti_lc.suite_200`
- `cre.vacancy.loss`
- `net_cash_flow`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 2,292,810.18 | ±1 |
| `model.total` | 3,852,483.13 | ±1 |
