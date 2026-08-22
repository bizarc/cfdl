---
id: benchmark-cre-acre-retail-development
title: "CRE: retail development"
slug: "/docs/examples/cre-acre-retail-development"
description: "A ground-up retail development on a twelve-suite rent roll, reconciled against A.CRE's own workbook: an S-curve construction draw, NNN recoveries whose management fee closes a circular loop in closed form, and a merchant-build exit."
source: benchmarks/cre/acre_retail_development
---

# CRE: retail development

A ground-up retail development on a twelve-suite rent roll, reconciled against A.CRE's own workbook: an S-curve construction draw, NNN recoveries whose management fee closes a circular loop in closed form, and a merchant-build exit.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A ground-up neighborhood retail center: four buildings, twelve suites, 91,500
rentable square feet on 7.43 acres. Land closes at month 0, construction runs
eighteen months on an S-curve, and the twelve suites commence in five cohorts
between months 19 and 31, each with its own free-rent gap before rent starts.
Leases are triple net, so the tenants reimburse a pro-rata share of operating
expenses — and the management fee inside those expenses is struck as a
percentage of the revenue the reimbursement is part of. Property taxes phase in
at half, then three quarters, then whole. The developer sells at month 43 in a
merchant build.

## The reference

The **A.CRE Retail Development Model v2.2**, in its shipped default state —
Spencer Burton's workbook, not a case written around it. Every expected value
is the workbook's own monthly grid.

**Not redistributable.** A.CRE distributes on a pay-what-you-are-able basis and states no
redistribution rights, so the workbook is a validation target and cannot be
vendored. It is catalogued as source 107 in `research/`, with a transcript of
the author's video walkthrough beside it.

Everything the model states is a DRIVER the workbook states: budget amounts
with a start month, an end month and a spread method; a twelve-suite rent roll;
four expense lines with a fixed/variable split; a tax phase-in vector. No
monthly figure is copied across, and no result is fitted.

## What it exercises

| | |
|---|---|
| Pack | `cre` (categories and metrics; the contracts do not fit — see below) |
| Contract types | none — twenty-eight hand-written streams |
| Language features | entity fields held between events, `set entity` from an event guard, phase-2 `series_sum` closing an algebraic loop, `curve` for a tabulated shape, `round_down` as floor, a twelve-month projection tail |
| Conventions | S-curve draws, NNN recoveries with a circular management fee, fixed/variable expense splits, a tax phase-in during lease-up, reserve-based rollover, escalations on the operating anniversary rather than the calendar year |

Three things the CRE pack cannot say, which is why this case is written in the
bare language. `cre.lease_unit`'s `escalation` steps on lease anniversaries and
has no word for the **every-five-years** step the anchor and both outparcels
carry. `cre.property_opex` has no **fixed/variable split**, so it cannot express
an expense a quarter of which is owed at nil occupancy. And nothing in the pack
phases an expense in over its first two operating years.

## The result

Every period of every line **equals the workbook's value at the precision the
results file publishes**. Effective gross income and net operating income are
identical across all 44 periods; net cash flow differs by at most one unit in
the last published digit.

Lifetime: net cash flow **−16,494,895.325053**, net operating income
**3,183,823.635072** — both agreeing with the workbook to 3.7e-7.

Asserted: net cash flow, effective gross income and net operating income per
period across 44 months, plus the two lifetime figures.

Per period, because the pieces can offset. Recoveries, the management fee and
the vacancy deduction are three expressions of one algebraic identity; a sign
or a share wrong in two of them nets to a correct annual NOI.

## The delta

None that belongs to the model. The residual is the **engine's own output
quantization**: `round_amount` publishes every amount to six decimals, so the
largest residual a correct model can show is 5e-7, and lines whose exact
monthly value does not terminate within six decimals sit exactly there.
`841852.50 / 18` is 46,769.58333… and publishes as 46,769.583333 — a 3.3e-7
difference that is decimal representation, not disagreement. Lines that come
out exact — insurance, CAM, property tax, the offsite draw — show zero.

NOTES.md derives every line's residual from its own arithmetic and predicts
each one before measuring it.

It is not decimal-versus-float. Running the case with
`"arithmetic": "excel_compat"`, which puts every expression in IEEE-754 float64
the way the workbook computes, produces a **byte-identical ledger hash**.

What the case does NOT cover is the financing stack: capitalized construction
interest sized to a 70% loan-to-cost test is a solve, and the workbook resolves
it with a VBA iteration macro. The management fee's loop looked like a second
one and was not — it is linear, so it has a closed form, and the language
states directly what the spreadsheet has to iterate toward.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0}}
// A.CRE Retail Development Model v2.2 — the shipped default case.
//
// A 4-building, 12-suite neighborhood center on 7.43 acres, 91,500 SF NRA.
// The analysis opens 2024-05 (A.CRE's "month 0", the closing) and the merchant
// build sells at month 43, 2027-12. Source workbook:
// research/Retail-Development-Model-v2.2-0ahk71.xlsm.
//
// WHAT THIS CASE ASSERTS, AND WHAT IT LEAVES OUT. Every figure here is DERIVED
// from the workbook's declared drivers — budget amounts with a start month, an
// end month and a spread method; a twelve-suite rent roll; four expense lines
// with a fixed/variable split; a tax phase-in vector. No monthly number is
// copied in. What is deliberately absent is the financing stack: capitalized
// construction interest sized to a 70% loan-to-cost test is a SOLVE, the
// workbook resolves it with a VBA iteration macro, and CFDL has no construct
// for one. The property-level operating
// model below stands on its own and is what this case reconciles.
//
// HOW CLOSE IT LANDS. Every period of every line equals the workbook's value
// rounded to six decimals, and six decimals is exactly what the results file
// carries: `round_amount` in crates/cfdl-engine/src/results.rs is a global
// determinism policy that quantizes every published amount to 1e-6. So the
// observed residual — never worse than 5e-7 — IS that quantization, not a
// modeling or arithmetic difference. The underlying agreement is at least
// 5e-9, which is where the measurement stops rather than where it fails.
//
// Checked rather than assumed: running this case with
// `"arithmetic": "excel_compat"` in the run configuration — which puts every
// expression in IEEE-754 float64, the way the workbook computes — produces a
// BYTE-IDENTICAL ledger hash. The two modes do not differ here because the
// quantization happens after the arithmetic in either one.
//
// THE MANAGEMENT FEE IS NOT ACTUALLY CIRCULAR. The workbook iterates it too:
// the fee is 3% of effective gross revenue, EGR includes recovery income,
// recoveries reimburse a pro-rata share of total opex, and total opex includes
// the fee. But that fixed point is LINEAR, so it has a closed form. Writing
//   R = base rent    O = other income    C = CAM + insurance + property tax
//   s = share of expenses recovered this period        v = vacancy rate
//   f = management fee rate
// then EGR = (1-v)(R + O + sC) + (1-v)s(f * EGR), so
//   EGR = (1-v)(R + O + sC) / (1 - f*s*(1-v))
// which is what the streams below evaluate. Excel iterates because it is wired
// cell to cell and cannot see the algebra; the language states it directly and
// lands on the same number. Measured against Excel's own iterated fee at month
// 31, the closed form differs by 4.9e-8 of a dollar — Excel's macro converges,
// and the algebra agrees with what it converges to.

version 0.1
model "acre-retail-development"
use pack "cre" version "0.1.0"

// Month 0 is the closing; month 43 is the sale. The twelve-month projection
// tail carries A.CRE's reversion pro forma window (its months 43-54), computed
// for valuation and excluded from cash results.
time calendar monthly from 2024-05 for 44 project 12

phase construction from 2024-06 to 2025-11
phase lease_up from 2025-12 to 2026-11
phase operations from 2026-12 to 2027-12

// Occupancy is a FACT ABOUT THE BUILDING, so it lives on the building. Two
// fields walk up as leases commence; the events below are the rent roll's
// commencement months. Neither declares `next`, so each HOLDS between events —
// which is what occupancy does. A field that holds plus an event that moves it
// is the pair a curve was standing in for.
//
// The two walks are three months apart because every lease in this roll sets
// its recovery start three months after its lease start. Expenses vary with
// space LEASED; reimbursement follows space that has begun RECOVERING.
entity asset center : CRE.Asset.RealProperty {
  rentable_area = 91500

  // The property opens in predevelopment and moves through its declared
  // lifecycle. The events below are the deal's turning points; before this
  // they were implicit in a dozen schedule dates, and a reader had to infer
  // when the building was finished from the month a cost stream stopped.
  // `deterministic.transitions` now states them.
  state predevelopment


  // --- Derived costs: what the WORKBOOK derives, derived here too -------
  //
  // Each field is a cell A.CRE computes rather than states, so stating the
  // product here instead would be copying an answer. Fields cannot read each
  // other (E1127_FIELD_RULE_READS_FIELD — a field names this period's close,
  // which does not exist yet inside a rule), so each chain is written out in
  // full from the atomic inputs rather than composed. That is the cost of the
  // rule, and it is the reason `total_hard_costs` appears three times below.

  // I34 = Units * 150, I35 = I34 * 1.5%, I38 = I34 + I35 + I36.
  total_hard_costs init inputs.nra * inputs.construction_psf * (1 + inputs.other_hard_pct)
                        + inputs.offsite

  // I46 = SUM(I41:I44, I38, I31) * 3.5%. Architecture and construction
  // management are themselves percentages of hard costs, so the whole base is
  // (hard * (1 + 6% + 2%)) + marketing + construction-period tax + land.
  //
  // THIS IS THE FIELD THAT PROMPTED THE AUDIT. It was `assume development_fee
  // = 647792.075`, the figure the workbook DISPLAYS. The workbook's actual
  // cached value is 647792.0750000001 — the float64 residue of its own
  // SUM(...)*0.035 — so the stated literal could never reproduce Excel's path
  // bit-for-bit even in excel_compat mode. Derived, it does.
  development_fee init (inputs.nra * inputs.construction_psf * (1 + inputs.other_hard_pct)
                        + inputs.offsite)
                       * (1 + inputs.arch_pct + inputs.constr_mgmt_pct)
                       * inputs.development_fee_pct
                       + (inputs.marketing_leasing + inputs.const_period_tax
                          + inputs.land_purchase + inputs.land_closing + inputs.land_diligence)
                         * inputs.development_fee_pct

  // --- The rent roll, summed ---------------------------------------------
  //
  // A.CRE's `Units` named range is E108: the rent roll's SF column totalled,
  // not a typed constant. `rentable_area` above states the same number, and
  // this field publishes the sum so any drift between the two is visible in
  // results rather than silent.
  nra_from_rent_roll init 65000 + 2500 + 2500 + 2500 + 1500 + 1500 + 1500 + 1500
                          + 3000 + 3000 + 4500 + 2500

  // D148 = $H$108, the roll's weighted average rent per foot: total annual
  // base rent over total area. It drives the leasing reserve below.
  market_rent_psf init (65000 * 14 + 2500 * 28 + 2500 * 25 + 2500 * 25
                        + 1500 * 30 + 1500 * 30 + 1500 * 30 + 1500 * 30
                        + 3000 * 27 + 3000 * 24 + 4500 * 110 + 2500 * 85)
                       / inputs.nra

  // I154 = I151 + H151, the probability-weighted cost of one turnover cycle
  // spread over the lease term. Per scenario the cost is downtime rent, plus
  // the improvement allowance, plus commission over the term — each weighted
  // by that scenario's probability, then annualized by dividing by the term.
  // The renewal leg carries no downtime, which is why its first term vanishes.
  leasing_reserve_year init
      ((65000 * 14 + 2500 * 28 + 2500 * 25 + 2500 * 25
        + 1500 * 30 + 1500 * 30 + 1500 * 30 + 1500 * 30
        + 3000 * 27 + 3000 * 24 + 4500 * 110 + 2500 * 85)
       * (inputs.downtime_new_months / 12) * (1 - inputs.renewal_probability)
       + inputs.ti_new_psf * inputs.nra * (1 - inputs.renewal_probability)
       + (65000 * 14 + 2500 * 28 + 2500 * 25 + 2500 * 25
          + 1500 * 30 + 1500 * 30 + 1500 * 30 + 1500 * 30
          + 3000 * 27 + 3000 * 24 + 4500 * 110 + 2500 * 85)
         * inputs.lease_term_years * inputs.lc_new_pct * (1 - inputs.renewal_probability)
       + inputs.ti_renewal_psf * inputs.nra * inputs.renewal_probability
       + (65000 * 14 + 2500 * 28 + 2500 * 25 + 2500 * 25
          + 1500 * 30 + 1500 * 30 + 1500 * 30 + 1500 * 30
          + 3000 * 27 + 3000 * 24 + 4500 * 110 + 2500 * 85)
         * inputs.lease_term_years * inputs.lc_renewal_pct * inputs.renewal_probability)
      / inputs.lease_term_years
}

// ---------------------------------------------------------------------------
// Drivers — every one of these is a blue input cell in the workbook.
// ---------------------------------------------------------------------------

// Development budget. A.CRE types the FIRST column of each of these and
// computes the rest; so does this model. Percentages and per-foot rates are
// inputs, and every product they generate is a field on the building above.
assume nra                  = 91500        // I18 / the `Units` named range
assume land_purchase        = 3000000      // I27, typed
assume land_closing         = 30000        // I28, typed
assume land_diligence       = 25000        // I29, typed
assume construction_psf     = 150          // the 150 inside `=Units*150` at I34
assume other_hard_pct       = 0.015        // I35 = I34 * 1.5%
assume offsite              = 100000       // I36, typed
assume offsite_months       = 2            // I36 spreads over months 1-2
assume arch_pct             = 0.06         // I41 = I38 * 6%
assume constr_mgmt_pct      = 0.02         // I42 = I38 * 2%
assume marketing_leasing    = 75000        // I43, typed
assume const_period_tax     = 225000       // I44, typed
assume financing_fees       = 130000       // I53, typed — a CARRY cost and a
                                           // levered item: it funds through the
                                           // facility, not the property
assume construction_rate    = 0.09         // E61/E65, typed
assume debt_ltc_target      = 0.70         // C61/C65, typed
assume exit_cap             = 0.07         // I165, typed
assume selling_costs        = 0.02         // I166, typed
// F52, typed and labeled "% of Lease-Up Income to Use to Pay Interest".
// Applying operating cash to interest is a BUSINESS DECISION, not a mechanic —
// a lender may or may not require it and a sponsor may or may not choose it
// while the asset is still stabilizing. A.CRE makes it a knob; so does this.
assume leaseup_income_to_interest = 1.0

// The one number A.CRE's macro solves for. I60 is a hardcoded plug cell; the
// loan is then Project_Cost_Total - Equity_Total, and the macro drives I60
// until debt / total uses equals the target above. Declared as an input so the
// model can VERIFY the fixed point it has no construct to find.
assume equity_commitment    = 6163520.202706008
assume development_fee_pct  = 0.035        // I46 = SUM(I41:I44,I38,I31) * 3.5%
assume soft_cost_months     = 18

// Operating drivers. A.CRE types dollars PER FOOT for the expense lines and
// multiplies by `Units`; the annual figures are nowhere typed.
assume operations_begin_t   = 19           // D91, typed — the month
                                           // operations open, and the anniversary
                                           // every escalator steps on
assume vacancy_rate         = 0.05
assume mgmt_fee_rate        = 0.03
assume cam_psf              = 5            // H137, typed; I137 = H137 * Units
assume cam_fixed_share      = 0.25
assume insurance_psf        = 2            // H139, typed
assume property_tax_psf     = 4            // H140, typed
assume capex_reserve_psf    = 1            // H155, typed
assume expense_growth       = 0.02
assume parking_income_year  = 50000
assume pct_rent_year        = 25000
assume other_income_year    = 15000
assume other_income_growth  = 0.02

// Market leasing assumptions (rows 147-152), which generate the leasing
// reserve. A.CRE types the renewal probability and derives the new-lease
// probability as its complement, so this model does too.
assume renewal_probability  = 0.75         // E147, typed; D147 = 1 - E147
assume ti_new_psf           = 10           // D149, typed
assume ti_renewal_psf       = 5            // E149, typed
assume lc_new_pct           = 0.06         // D150, typed
assume lc_renewal_pct       = 0.04         // E150, typed
assume lease_term_years     = 10           // D151 / E151, typed
assume downtime_new_months  = 6            // D152, typed; E152 is 0

// ---------------------------------------------------------------------------
// Curves.
//
// A.CRE's S-curve is a NORMAL CDF DIFFERENCE, not a sampled bell: for an
// n-month window at steepness k, month m takes
//     PHI(m; n/2, n/k) - PHI(m-1; n/2, n/k),
// normalized to sum to one. Steepness is 6 on the Data tab, so n=18 gives
// mu=9, sigma=3. These eighteen weights are that formula evaluated. Truncating
// them at sixteen decimal places costs 4.7e-10 of a dollar on 13,725,000 —
// three orders of magnitude below the results file's own precision.
// They are a curve rather than an expression because CFDL has no erf() and no
// normal CDF — the single place in this model where a shape had to be
// tabulated instead of stated. The formula above is the specification.
//
// A DISTRIBUTION CANNOT DO THIS. `assume x ~ Normal(...)` supplies a scalar
// DRAW — the mean in a deterministic run, one sample per Monte Carlo trial. It
// is not a density that can be evaluated at a point, so it cannot supply a
// shape across eighteen months.
// ---------------------------------------------------------------------------

curve scurve_18 step {
  2024-06: 0.0024871974618758
  2024-07: 0.0060011499420441
  2024-08: 0.0129698191863143
  2024-09: 0.0251080068226115
  2024-10: 0.0435384122871466
  2024-11: 0.0676266122671597
  2024-12: 0.0940913109664
  2025-01: 0.1172653952873943
  2025-02: 0.1309120957790538
  2025-03: 0.1309120957790538
  2025-04: 0.1172653952873943
  2025-05: 0.0940913109663999
  2025-06: 0.0676266122671597
  2025-07: 0.0435384122871466
  2025-08: 0.0251080068226115
  2025-09: 0.0129698191863143
  2025-10: 0.0060011499420441
  2025-11: 0.0024871974618758
}






// Property taxes are owed at a fraction of stabilized during lease-up: half in
// operating year one, three quarters in year two, whole thereafter (rows 91-92).
curve tax_phase_in step {
  2024-05: 0.0
  2025-12: 0.5
  2026-12: 0.75
  2027-12: 1.0
}

// The turning points. Each is the first period its guard holds, which is what
// an event is for; the dates come from the phases rather than being restated.
event center.break_ground when time.phase == "construction" {
  set entity asset.center.status = "construction"
}

event center.open when time.phase == "lease_up" {
  set entity asset.center.status = "lease_up"
}

event center.stabilize when time.phase == "operations" {
  set entity asset.center.status = "operating"
}

event center.sell when time.t == 43 {
  set entity asset.center.status = "disposed"
}

// ---------------------------------------------------------------------------
// Development period — months 0 to 18.
// ---------------------------------------------------------------------------

// Land, closing costs and diligence all settle at month 0, the analysis date.
//
// The $130,000 financing fee does NOT belong here. A.CRE books it in Carry
// Costs (I53) and its unlevered cash flow (row 214 = -199 + 205 + 208) leaves
// all three carry costs out — fee, capitalized interest and operating
// shortfall are financing, not property. This case asserts the UNLEVERED
// property cash flow, so the fee arrives with the construction facility or
// not at all.
stream center.land on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month due from 2024-05 to 2024-05
  amount = inputs.land_purchase + inputs.land_closing + inputs.land_diligence
}


// The two S-curve lines take the same eighteen weights; only the amount
// differs, which is exactly what the workbook's METHOD column says.
stream center.construction_costs on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = inputs.nra * inputs.construction_psf
           * curve_value("scurve_18", time.date)
}

stream center.other_hard_costs on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = inputs.nra * inputs.construction_psf * inputs.other_hard_pct
           * curve_value("scurve_18", time.date)
}

// Offsite improvements are straight-line over two months, not eighteen.
stream center.offsite on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month from 2024-06 to 2024-07
  amount = inputs.offsite / inputs.offsite_months
}

// The soft cost lines are all straight-line across the build.
stream center.arch_engineering on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = asset.center.total_hard_costs * inputs.arch_pct / inputs.soft_cost_months
}

stream center.construction_mgmt on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = asset.center.total_hard_costs * inputs.constr_mgmt_pct / inputs.soft_cost_months
}

stream center.marketing_leasing on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = inputs.marketing_leasing / inputs.soft_cost_months
}

stream center.const_period_tax on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = inputs.const_period_tax / inputs.soft_cost_months
}

stream center.development_fee on entity asset.center outflow currency USD {
  category investing.capital.construction
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = asset.center.development_fee / inputs.soft_cost_months
}

// ---------------------------------------------------------------------------
// The rent roll — twelve suites, each its own stream.
//
// A suite's rent starts at its RENT START month, which sits after its lease
// start by the free-rent period the leasing brief negotiated. It then steps by
// its own growth rate on its OWN FREQUENCY: the anchor and the two outparcels
// step every five years, the inline shops annually. That frequency is why
// these are hand-written streams rather than `cre.lease_unit` contracts — the
// pack's `escalation` term steps on lease anniversaries and has no word for
// "every five years", so three of these twelve leases are not expressible in
// the pack's vocabulary. See NOTES.md.
//
// The step count is round_down((t - rent_start) / (12 * frequency), 0), which
// is the floor CFDL spells with round_down.
// ---------------------------------------------------------------------------
stream rent.anchor_walbert on entity asset.anchor_walbert inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-03 to 2028-12
  amount = 65000.0 * 14.0 / time.ppy * pow(1.1, round_down(time.elapsed_years / 5, 0))
  active in state leased
}

stream rent.suite_525_102 on entity asset.suite_525_102 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-06 to 2028-12
  amount = 2500.0 * 28.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.suite_525_103 on entity asset.suite_525_103 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-06 to 2028-12
  amount = 2500.0 * 25.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.suite_525_104 on entity asset.suite_525_104 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-09 to 2028-12
  amount = 2500.0 * 25.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.suite_625_101 on entity asset.suite_625_101 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-09 to 2028-12
  amount = 1500.0 * 30.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.suite_625_102 on entity asset.suite_625_102 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-12 to 2028-12
  amount = 1500.0 * 30.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.suite_625_103 on entity asset.suite_625_103 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-12 to 2028-12
  amount = 1500.0 * 30.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.suite_625_104 on entity asset.suite_625_104 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2027-03 to 2028-12
  amount = 1500.0 * 30.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.suite_725_101 on entity asset.suite_725_101 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2027-03 to 2028-12
  amount = 3000.0 * 27.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.suite_725_102 on entity asset.suite_725_102 inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2027-03 to 2028-12
  amount = 3000.0 * 24.0 / time.ppy * pow(1.025, time.elapsed_years)
  active in state leased
}

stream rent.outparcel_whosaburger on entity asset.outparcel_whosaburger inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-03 to 2028-12
  amount = 4500.0 * 110.0 / time.ppy * pow(1.075, round_down(time.elapsed_years / 5, 0))
  active in state leased
}

stream rent.outparcel_coffee on entity asset.outparcel_coffee inflow currency USD {
  category operating.revenue.base_rent
  schedule every month from 2026-03 to 2028-12
  amount = 2500.0 * 85.0 / time.ppy * pow(1.1, round_down(time.elapsed_years / 5, 0))
  active in state leased
}

// ---------------------------------------------------------------------------
// Other income and the three directly-stated expenses.
//
// Both the expense escalation and the other-income growth step on the
// OPERATING anniversary (months 19, 31, 43), not the calendar year, so each
// line applies its own rate over round_down((t - 19) / 12, 0). That is an
// expression, not a curve: the escalator is a rate the workbook states, and a
// rate compounded on a stated anniversary is exactly what pow() is for.
// Parking varies with occupancy; percentage rent and the other line are fixed.
// ---------------------------------------------------------------------------
stream other_income.parking on entity asset.center inflow currency USD {
  category operating.revenue.other
  schedule every month from 2025-12 to 2028-12
  amount = inputs.parking_income_year / time.ppy            * (if(asset.anchor_walbert.status == "leased", asset.anchor_walbert.rentable_area, 0)
              + if(asset.suite_525_102.status == "leased", asset.suite_525_102.rentable_area, 0)
              + if(asset.suite_525_103.status == "leased", asset.suite_525_103.rentable_area, 0)
              + if(asset.suite_525_104.status == "leased", asset.suite_525_104.rentable_area, 0)
              + if(asset.suite_625_101.status == "leased", asset.suite_625_101.rentable_area, 0)
              + if(asset.suite_625_102.status == "leased", asset.suite_625_102.rentable_area, 0)
              + if(asset.suite_625_103.status == "leased", asset.suite_625_103.rentable_area, 0)
              + if(asset.suite_625_104.status == "leased", asset.suite_625_104.rentable_area, 0)
              + if(asset.suite_725_101.status == "leased", asset.suite_725_101.rentable_area, 0)
              + if(asset.suite_725_102.status == "leased", asset.suite_725_102.rentable_area, 0)
              + if(asset.outparcel_whosaburger.status == "leased", asset.outparcel_whosaburger.rentable_area, 0)
              + if(asset.outparcel_coffee.status == "leased", asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area            * pow(1.0 + inputs.other_income_growth, round_down((time.t - inputs.operations_begin_t) / time.ppy, 0))
}

stream other_income.percentage_rent on entity asset.center inflow currency USD {
  category operating.revenue.percentage_rent
  schedule every month from 2025-12 to 2028-12
  amount = inputs.pct_rent_year / time.ppy            * pow(1.0 + inputs.other_income_growth, round_down((time.t - inputs.operations_begin_t) / time.ppy, 0))
}

stream other_income.other on entity asset.center inflow currency USD {
  category operating.revenue.other
  schedule every month from 2025-12 to 2028-12
  amount = inputs.other_income_year / time.ppy            * pow(1.0 + inputs.other_income_growth, round_down((time.t - inputs.operations_begin_t) / time.ppy, 0))
}

// CAM is 25% fixed: a quarter is owed at nil occupancy and the rest tracks
// the space actually leased. Insurance is wholly fixed. Property taxes are
// wholly fixed too, but phase in over the first two operating years.
stream opex_base.cam on entity asset.center outflow currency USD {
  category operating.expense.opex
  schedule every month from 2025-12 to 2028-12
  amount = inputs.cam_psf * inputs.nra / time.ppy            * (inputs.cam_fixed_share               + (1.0 - inputs.cam_fixed_share) * (if(asset.anchor_walbert.status == "leased", asset.anchor_walbert.rentable_area, 0)
              + if(asset.suite_525_102.status == "leased", asset.suite_525_102.rentable_area, 0)
              + if(asset.suite_525_103.status == "leased", asset.suite_525_103.rentable_area, 0)
              + if(asset.suite_525_104.status == "leased", asset.suite_525_104.rentable_area, 0)
              + if(asset.suite_625_101.status == "leased", asset.suite_625_101.rentable_area, 0)
              + if(asset.suite_625_102.status == "leased", asset.suite_625_102.rentable_area, 0)
              + if(asset.suite_625_103.status == "leased", asset.suite_625_103.rentable_area, 0)
              + if(asset.suite_625_104.status == "leased", asset.suite_625_104.rentable_area, 0)
              + if(asset.suite_725_101.status == "leased", asset.suite_725_101.rentable_area, 0)
              + if(asset.suite_725_102.status == "leased", asset.suite_725_102.rentable_area, 0)
              + if(asset.outparcel_whosaburger.status == "leased", asset.outparcel_whosaburger.rentable_area, 0)
              + if(asset.outparcel_coffee.status == "leased", asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area)            * pow(1.0 + inputs.expense_growth, round_down((time.t - inputs.operations_begin_t) / time.ppy, 0))
}

stream opex_base.insurance on entity asset.center outflow currency USD {
  category operating.expense.opex
  schedule every month from 2025-12 to 2028-12
  amount = inputs.insurance_psf * inputs.nra / time.ppy            * pow(1.0 + inputs.expense_growth, round_down((time.t - inputs.operations_begin_t) / time.ppy, 0))
}

stream opex_base.property_tax on entity asset.center outflow currency USD {
  category operating.expense.opex
  schedule every month from 2025-12 to 2028-12
  amount = inputs.property_tax_psf * inputs.nra / time.ppy            * curve_value("tax_phase_in", time.date)            * pow(1.0 + inputs.expense_growth, round_down((time.t - inputs.operations_begin_t) / time.ppy, 0))
}

// ---------------------------------------------------------------------------
// The three lines that close the fee loop.
//
// Each is written from the closed form in the header comment, reading only
// phase-1 series, because a phase-2 stream may not read another phase-2
// stream. The algebra is therefore repeated rather than named — CFDL has no
// let-binding, and this is the one place in the model where that costs
// readability. The shared sub-expression, every time, is
//
//   EGR = (1-v)(R + O + sC) / (1 - f*s*(1-v))
//
// with R the rent roll, O other income, C the three stated expenses (negated,
// since series_sum returns signed amounts and those are outflows), s the share
// of expenses recovered this period and v the vacancy rate.
// ---------------------------------------------------------------------------
stream opex.management_fee on entity asset.center outflow currency USD {
  category operating.expense.opex
  schedule every month from 2025-12 to 2028-12
  amount = inputs.mgmt_fee_rate            * (1.0 - inputs.vacancy_rate)            * (series_sum("rent.*", time.t, time.t)               + series_sum("other_income.*", time.t, time.t)               + (if(time.t >= asset.anchor_walbert.recovery_start_month, asset.anchor_walbert.rentable_area, 0)
              + if(time.t >= asset.suite_525_102.recovery_start_month, asset.suite_525_102.rentable_area, 0)
              + if(time.t >= asset.suite_525_103.recovery_start_month, asset.suite_525_103.rentable_area, 0)
              + if(time.t >= asset.suite_525_104.recovery_start_month, asset.suite_525_104.rentable_area, 0)
              + if(time.t >= asset.suite_625_101.recovery_start_month, asset.suite_625_101.rentable_area, 0)
              + if(time.t >= asset.suite_625_102.recovery_start_month, asset.suite_625_102.rentable_area, 0)
              + if(time.t >= asset.suite_625_103.recovery_start_month, asset.suite_625_103.rentable_area, 0)
              + if(time.t >= asset.suite_625_104.recovery_start_month, asset.suite_625_104.rentable_area, 0)
              + if(time.t >= asset.suite_725_101.recovery_start_month, asset.suite_725_101.rentable_area, 0)
              + if(time.t >= asset.suite_725_102.recovery_start_month, asset.suite_725_102.rentable_area, 0)
              + if(time.t >= asset.outparcel_whosaburger.recovery_start_month, asset.outparcel_whosaburger.rentable_area, 0)
              + if(time.t >= asset.outparcel_coffee.recovery_start_month, asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area                 * -series_sum("opex_base.*", time.t, time.t))            / (1.0 - inputs.mgmt_fee_rate                   * (if(time.t >= asset.anchor_walbert.recovery_start_month, asset.anchor_walbert.rentable_area, 0)
              + if(time.t >= asset.suite_525_102.recovery_start_month, asset.suite_525_102.rentable_area, 0)
              + if(time.t >= asset.suite_525_103.recovery_start_month, asset.suite_525_103.rentable_area, 0)
              + if(time.t >= asset.suite_525_104.recovery_start_month, asset.suite_525_104.rentable_area, 0)
              + if(time.t >= asset.suite_625_101.recovery_start_month, asset.suite_625_101.rentable_area, 0)
              + if(time.t >= asset.suite_625_102.recovery_start_month, asset.suite_625_102.rentable_area, 0)
              + if(time.t >= asset.suite_625_103.recovery_start_month, asset.suite_625_103.rentable_area, 0)
              + if(time.t >= asset.suite_625_104.recovery_start_month, asset.suite_625_104.rentable_area, 0)
              + if(time.t >= asset.suite_725_101.recovery_start_month, asset.suite_725_101.rentable_area, 0)
              + if(time.t >= asset.suite_725_102.recovery_start_month, asset.suite_725_102.rentable_area, 0)
              + if(time.t >= asset.outparcel_whosaburger.recovery_start_month, asset.outparcel_whosaburger.rentable_area, 0)
              + if(time.t >= asset.outparcel_coffee.recovery_start_month, asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area                   * (1.0 - inputs.vacancy_rate))
}

// Recoveries reimburse the recovered share of TOTAL operating expenses — the
// three stated lines plus the management fee, which is why this reads the
// same closed form rather than a running total.
stream ops.recoveries on entity asset.center inflow currency USD {
  category operating.revenue.recovery
  schedule every month from 2025-12 to 2028-12
  amount = (if(time.t >= asset.anchor_walbert.recovery_start_month, asset.anchor_walbert.rentable_area, 0)
              + if(time.t >= asset.suite_525_102.recovery_start_month, asset.suite_525_102.rentable_area, 0)
              + if(time.t >= asset.suite_525_103.recovery_start_month, asset.suite_525_103.rentable_area, 0)
              + if(time.t >= asset.suite_525_104.recovery_start_month, asset.suite_525_104.rentable_area, 0)
              + if(time.t >= asset.suite_625_101.recovery_start_month, asset.suite_625_101.rentable_area, 0)
              + if(time.t >= asset.suite_625_102.recovery_start_month, asset.suite_625_102.rentable_area, 0)
              + if(time.t >= asset.suite_625_103.recovery_start_month, asset.suite_625_103.rentable_area, 0)
              + if(time.t >= asset.suite_625_104.recovery_start_month, asset.suite_625_104.rentable_area, 0)
              + if(time.t >= asset.suite_725_101.recovery_start_month, asset.suite_725_101.rentable_area, 0)
              + if(time.t >= asset.suite_725_102.recovery_start_month, asset.suite_725_102.rentable_area, 0)
              + if(time.t >= asset.outparcel_whosaburger.recovery_start_month, asset.outparcel_whosaburger.rentable_area, 0)
              + if(time.t >= asset.outparcel_coffee.recovery_start_month, asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area            * (-series_sum("opex_base.*", time.t, time.t)               + inputs.mgmt_fee_rate                 * (1.0 - inputs.vacancy_rate)                 * (series_sum("rent.*", time.t, time.t)                    + series_sum("other_income.*", time.t, time.t)                    + (if(time.t >= asset.anchor_walbert.recovery_start_month, asset.anchor_walbert.rentable_area, 0)
              + if(time.t >= asset.suite_525_102.recovery_start_month, asset.suite_525_102.rentable_area, 0)
              + if(time.t >= asset.suite_525_103.recovery_start_month, asset.suite_525_103.rentable_area, 0)
              + if(time.t >= asset.suite_525_104.recovery_start_month, asset.suite_525_104.rentable_area, 0)
              + if(time.t >= asset.suite_625_101.recovery_start_month, asset.suite_625_101.rentable_area, 0)
              + if(time.t >= asset.suite_625_102.recovery_start_month, asset.suite_625_102.rentable_area, 0)
              + if(time.t >= asset.suite_625_103.recovery_start_month, asset.suite_625_103.rentable_area, 0)
              + if(time.t >= asset.suite_625_104.recovery_start_month, asset.suite_625_104.rentable_area, 0)
              + if(time.t >= asset.suite_725_101.recovery_start_month, asset.suite_725_101.rentable_area, 0)
              + if(time.t >= asset.suite_725_102.recovery_start_month, asset.suite_725_102.rentable_area, 0)
              + if(time.t >= asset.outparcel_whosaburger.recovery_start_month, asset.outparcel_whosaburger.rentable_area, 0)
              + if(time.t >= asset.outparcel_coffee.recovery_start_month, asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area                      * -series_sum("opex_base.*", time.t, time.t))                 / (1.0 - inputs.mgmt_fee_rate                        * (if(time.t >= asset.anchor_walbert.recovery_start_month, asset.anchor_walbert.rentable_area, 0)
              + if(time.t >= asset.suite_525_102.recovery_start_month, asset.suite_525_102.rentable_area, 0)
              + if(time.t >= asset.suite_525_103.recovery_start_month, asset.suite_525_103.rentable_area, 0)
              + if(time.t >= asset.suite_525_104.recovery_start_month, asset.suite_525_104.rentable_area, 0)
              + if(time.t >= asset.suite_625_101.recovery_start_month, asset.suite_625_101.rentable_area, 0)
              + if(time.t >= asset.suite_625_102.recovery_start_month, asset.suite_625_102.rentable_area, 0)
              + if(time.t >= asset.suite_625_103.recovery_start_month, asset.suite_625_103.rentable_area, 0)
              + if(time.t >= asset.suite_625_104.recovery_start_month, asset.suite_625_104.rentable_area, 0)
              + if(time.t >= asset.suite_725_101.recovery_start_month, asset.suite_725_101.rentable_area, 0)
              + if(time.t >= asset.suite_725_102.recovery_start_month, asset.suite_725_102.rentable_area, 0)
              + if(time.t >= asset.outparcel_whosaburger.recovery_start_month, asset.outparcel_whosaburger.rentable_area, 0)
              + if(time.t >= asset.outparcel_coffee.recovery_start_month, asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area                        * (1.0 - inputs.vacancy_rate)))
}

// Vacancy and credit loss is 5% of potential gross income, and potential
// gross income includes the recoveries above — so it resolves through the fee
// as well.
stream ops.vacancy_loss on entity asset.center outflow currency USD {
  category operating.deduction.vacancy
  schedule every month from 2025-12 to 2028-12
  amount = inputs.vacancy_rate            * (series_sum("rent.*", time.t, time.t)               + series_sum("other_income.*", time.t, time.t)               + (if(time.t >= asset.anchor_walbert.recovery_start_month, asset.anchor_walbert.rentable_area, 0)
              + if(time.t >= asset.suite_525_102.recovery_start_month, asset.suite_525_102.rentable_area, 0)
              + if(time.t >= asset.suite_525_103.recovery_start_month, asset.suite_525_103.rentable_area, 0)
              + if(time.t >= asset.suite_525_104.recovery_start_month, asset.suite_525_104.rentable_area, 0)
              + if(time.t >= asset.suite_625_101.recovery_start_month, asset.suite_625_101.rentable_area, 0)
              + if(time.t >= asset.suite_625_102.recovery_start_month, asset.suite_625_102.rentable_area, 0)
              + if(time.t >= asset.suite_625_103.recovery_start_month, asset.suite_625_103.rentable_area, 0)
              + if(time.t >= asset.suite_625_104.recovery_start_month, asset.suite_625_104.rentable_area, 0)
              + if(time.t >= asset.suite_725_101.recovery_start_month, asset.suite_725_101.rentable_area, 0)
              + if(time.t >= asset.suite_725_102.recovery_start_month, asset.suite_725_102.rentable_area, 0)
              + if(time.t >= asset.outparcel_whosaburger.recovery_start_month, asset.outparcel_whosaburger.rentable_area, 0)
              + if(time.t >= asset.outparcel_coffee.recovery_start_month, asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area                 * (-series_sum("opex_base.*", time.t, time.t)                    + inputs.mgmt_fee_rate                      * (1.0 - inputs.vacancy_rate)                      * (series_sum("rent.*", time.t, time.t)                         + series_sum("other_income.*", time.t, time.t)                         + (if(time.t >= asset.anchor_walbert.recovery_start_month, asset.anchor_walbert.rentable_area, 0)
              + if(time.t >= asset.suite_525_102.recovery_start_month, asset.suite_525_102.rentable_area, 0)
              + if(time.t >= asset.suite_525_103.recovery_start_month, asset.suite_525_103.rentable_area, 0)
              + if(time.t >= asset.suite_525_104.recovery_start_month, asset.suite_525_104.rentable_area, 0)
              + if(time.t >= asset.suite_625_101.recovery_start_month, asset.suite_625_101.rentable_area, 0)
              + if(time.t >= asset.suite_625_102.recovery_start_month, asset.suite_625_102.rentable_area, 0)
              + if(time.t >= asset.suite_625_103.recovery_start_month, asset.suite_625_103.rentable_area, 0)
              + if(time.t >= asset.suite_625_104.recovery_start_month, asset.suite_625_104.rentable_area, 0)
              + if(time.t >= asset.suite_725_101.recovery_start_month, asset.suite_725_101.rentable_area, 0)
              + if(time.t >= asset.suite_725_102.recovery_start_month, asset.suite_725_102.rentable_area, 0)
              + if(time.t >= asset.outparcel_whosaburger.recovery_start_month, asset.outparcel_whosaburger.rentable_area, 0)
              + if(time.t >= asset.outparcel_coffee.recovery_start_month, asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area                           * -series_sum("opex_base.*", time.t, time.t))                      / (1.0 - inputs.mgmt_fee_rate                             * (if(time.t >= asset.anchor_walbert.recovery_start_month, asset.anchor_walbert.rentable_area, 0)
              + if(time.t >= asset.suite_525_102.recovery_start_month, asset.suite_525_102.rentable_area, 0)
              + if(time.t >= asset.suite_525_103.recovery_start_month, asset.suite_525_103.rentable_area, 0)
              + if(time.t >= asset.suite_525_104.recovery_start_month, asset.suite_525_104.rentable_area, 0)
              + if(time.t >= asset.suite_625_101.recovery_start_month, asset.suite_625_101.rentable_area, 0)
              + if(time.t >= asset.suite_625_102.recovery_start_month, asset.suite_625_102.rentable_area, 0)
              + if(time.t >= asset.suite_625_103.recovery_start_month, asset.suite_625_103.rentable_area, 0)
              + if(time.t >= asset.suite_625_104.recovery_start_month, asset.suite_625_104.rentable_area, 0)
              + if(time.t >= asset.suite_725_101.recovery_start_month, asset.suite_725_101.rentable_area, 0)
              + if(time.t >= asset.suite_725_102.recovery_start_month, asset.suite_725_102.rentable_area, 0)
              + if(time.t >= asset.outparcel_whosaburger.recovery_start_month, asset.outparcel_whosaburger.rentable_area, 0)
              + if(time.t >= asset.outparcel_coffee.recovery_start_month, asset.outparcel_coffee.rentable_area, 0))
             / asset.center.rentable_area                             * (1.0 - inputs.vacancy_rate))))
}

// ---------------------------------------------------------------------------
// Reserves.
//
// The leasing reserve is A.CRE's stand-in for rollover: it charges the
// annualized probability-weighted cost of keeping the center leased rather
// than modeling second-generation leases, which is why no `cre.rollover`
// appears anywhere in this model.
// ---------------------------------------------------------------------------
stream reserve.leasing on entity asset.center outflow currency USD {
  category investing.capital.leasing
  schedule every month from 2026-12 to 2028-12
  amount = asset.center.leasing_reserve_year / time.ppy            * pow(1.0 + inputs.expense_growth, round_down((time.t - inputs.operations_begin_t) / time.ppy, 0))
}

// The capital reserve starts at OPERATIONS begin, twelve months before the
// leasing reserve, which waits for stabilization. Two reserves, two start
// months: a repair bill does not wait for the last suite to sign.
stream reserve.capex on entity asset.center outflow currency USD {
  category investing.capital.capex
  schedule every month from 2025-12 to 2028-12
  amount = inputs.capex_reserve_psf * inputs.nra / time.ppy            * pow(1.0 + inputs.expense_growth, round_down((time.t - inputs.operations_begin_t) / time.ppy, 0))
}


// ---------------------------------------------------------------------------
// The reversion.
//
// WITHOUT THIS THERE IS NO NPV. A present value needs the outlay, the operating
// flows and the terminal value; a model carrying only the first two discounts
// a deal that never sells and reports a number that is not a return.
//
// A.CRE's reversion NOI is DERIVED, not stated: rows 169-182 column I sum the
// actual monthly rows over months 43-54 with OFFSET, so the exit prices the
// twelve months FOLLOWING the sale, not the trailing year. That is
// `cre.exit_forward` exactly — the window comes from the modeled streams and
// cannot drift from them — and it is why this model declares a twelve-month
// projection tail. `cre.exit` would take a stated number and reintroduce the
// drift the workbook avoided.
//
// Note the workbook's OTHER column: H is the STABILIZED (untrended) pro forma,
// year-one rents with no escalation and full property tax, used to strike the
// stabilized value and the development spread. It is not the exit basis.
// ---------------------------------------------------------------------------

// `cre.exit_forward` CANNOT BE USED HERE, and it fails silently. Its lowering
// sums a fixed list of PACK stream names — cre.unit.base_rent.*,
// cre.property.opex, cre.vacancy.loss and six more. This model is hand-written
// (the pack cannot say every-five-years escalation, a fixed/variable expense
// split, or a tax phase-in), so not one selector matches, the window sums to
// zero, and the contract prices the sale at nothing without a diagnostic. A
// selector that matches nothing must be allowed to sum to zero, so there is no
// error to raise — see E5022_UNKNOWN_SERIES_REFERENCE.
//
// Written by hand instead, over this model's own NOI subtotal.
stream center.exit_proceeds on entity asset.center inflow currency USD {
  category investing.reversion
  schedule every month from 2027-12 to 2027-12
  // The window is time.t .. time.t+11 — the SALE MONTH AND ELEVEN FORWARD.
  // A.CRE labels it "Month 43 - 54" (I167) and the pack's own exit_forward
  // rule uses t+1 .. t+12 instead. On this deal the two differ by 15,650 of
  // proceeds, so the convention is worth stating rather than inheriting.
  //
  // It reads ONLY phase-1 streams. It cannot read recoveries, the management
  // fee or the vacancy deduction — those are phase-2, and a cross-stream read
  // may only see streams that read none (E5002, and the engine says so rather
  // than quietly summing zero). So the closed form from the header is applied
  // to the WINDOW SUMS instead of per period. That is valid only because the
  // recovery share is 1.0 throughout months 43-54 and every term is linear in
  // the period's amounts, so summing commutes with the algebra. Verified
  // against the workbook: 2,083,611.0542 of forward NOI either way.
  amount = ((1 - inputs.vacancy_rate)
            * (series_sum("rent.*", time.t, time.t + 11)
               + series_sum("other_income.*", time.t, time.t + 11)
               - series_sum("opex_base.*", time.t, time.t + 11))
            / (1 - inputs.mgmt_fee_rate * (1 - inputs.vacancy_rate))
            * (1 - inputs.mgmt_fee_rate)
            + series_sum("opex_base.*", time.t, time.t + 11))
           / inputs.exit_cap * (1 - inputs.selling_costs)
}

// ===========================================================================
// THE CONSTRUCTION FACILITY
//
// On its own entity, because a balance lives on it and because the property's
// unlevered cash flow must not move. `entity.asset.center.total` is what this
// case asserts; nothing below may change it.
//
// THE SOLVE IS NOT HERE, AND CANNOT BE. The lender funds 70% of total project
// cost; total cost includes capitalized interest; interest depends on the
// balance, which depends on how much equity funds first. So the loan is sized
// by an equation whose answer is also one of its inputs:
//
//     E = 0.30 * (P + F + S(E) + I(E))
//
// That is ONE SCALAR over the whole horizon, not a per-period fixed point. LTC
// moves with time only because carry costs accumulate, so there is nothing to
// solve until the funding period has an end date. A.CRE resolves it by
// hardcoding I60 and driving it with a macro. CFDL has no solve construct, so
// the commitment is declared and the model verifies the fixed point instead:
// `ltc_achieved` lands on 0.70 when the commitment is right and does not when
// it is wrong.
//
// Because equity funds first, I(E) is LINEAR in E for any E between two
// consecutive cumulative-funding points, so two runs and a secant step find the
// commitment exactly rather than by iteration. See run.json and NOTES.md.
//
// OPERATING CASH FLOW ARRIVES AS A CURVE, AND IT IS THIS MODEL'S OWN OUTPUT.
// The facility needs cash flow from operations twice: the lease-up deficit is
// advanced by the loan, and interest stops capitalizing once operations cover
// it. A field's rule cannot read a stream (docs/03 §3.1), so it cannot see NOI.
// But the dependency is ONE-WAY — the loan does not change the rent roll — so
// the curve below is generated from a first pass of this same model. It is a
// handoff, not a fitted value; NOTES.md records how to regenerate it.
// ===========================================================================

curve operating_cash_flow step {
  2024-05: 0.0
  2025-12: -64063.271858
  2026-03: 109330.129773
  2026-06: 123265.836091
  2026-09: 134662.078575
  2026-12: 127471.872497
  2027-03: 149718.710543
  2027-06: 149980.545228
  2027-09: 150192.977141
  2027-12: 149565.115977
}


// ---------------------------------------------------------------------------
// The rent roll, at unit grain.
//
// Each suite is a `CRE.Asset.Unit` that is `part of` the center, so the
// building's totals include its suites' because they ARE its suites — the
// relation aggregates, not a shared name prefix. Each carries its own area and
// opens `vacant` in the `cre.unit` lifecycle the pack declares.
//
// This is the grain the deal is actually written at: twelve leases, twelve
// commencements, twelve areas. Before this the roll was twelve streams hung on
// the building with the areas appearing only inside their own rent
// expressions, and occupancy was a counter with five hand-totalled cohort
// figures in it.
// ---------------------------------------------------------------------------
entity asset anchor_walbert : CRE.Asset.Unit {
  rentable_area = 65000
  part of asset.center
  state vacant
  recovery_start_month = 22
}
entity asset suite_525_102 : CRE.Asset.Unit {
  rentable_area = 2500
  part of asset.center
  state vacant
  recovery_start_month = 25
}
entity asset suite_525_103 : CRE.Asset.Unit {
  rentable_area = 2500
  part of asset.center
  state vacant
  recovery_start_month = 25
}
entity asset suite_525_104 : CRE.Asset.Unit {
  rentable_area = 2500
  part of asset.center
  state vacant
  recovery_start_month = 28
}
entity asset suite_625_101 : CRE.Asset.Unit {
  rentable_area = 1500
  part of asset.center
  state vacant
  recovery_start_month = 28
}
entity asset suite_625_102 : CRE.Asset.Unit {
  rentable_area = 1500
  part of asset.center
  state vacant
  recovery_start_month = 31
}
entity asset suite_625_103 : CRE.Asset.Unit {
  rentable_area = 1500
  part of asset.center
  state vacant
  recovery_start_month = 31
}
entity asset suite_625_104 : CRE.Asset.Unit {
  rentable_area = 1500
  part of asset.center
  state vacant
  recovery_start_month = 34
}
entity asset suite_725_101 : CRE.Asset.Unit {
  rentable_area = 3000
  part of asset.center
  state vacant
  recovery_start_month = 34
}
entity asset suite_725_102 : CRE.Asset.Unit {
  rentable_area = 3000
  part of asset.center
  state vacant
  recovery_start_month = 34
}
entity asset outparcel_whosaburger : CRE.Asset.Unit {
  rentable_area = 4500
  part of asset.center
  state vacant
  recovery_start_month = 22
}
entity asset outparcel_coffee : CRE.Asset.Unit {
  rentable_area = 2500
  part of asset.center
  state vacant
  recovery_start_month = 22
}

// A lease commences: the suite becomes `leased`, and the center's leased area
// grows by that suite's own area. Recovery starts three months later on every
// lease in this roll, which is why there are two events per suite rather than
// one with two effects.

// A lease commences. Each event writes only its OWN suite's status, so the
// twelve that share a period write twelve different fields and cannot collide.
// An earlier draft funnelled them into one `leased_sf` counter on the center;
// the spec is explicit that declaration order decides which write wins when two
// events set the same field, so three suites commencing together recorded only
// the last one's area. Measured: 30 where 1,230 was intended, and reversing the
// declarations gave 1,000 — the same three events, three different answers.

event lease.anchor_walbert when time.t == 19 {
  set entity asset.anchor_walbert.status = "leased"
}

event lease.suite_525_102 when time.t == 22 {
  set entity asset.suite_525_102.status = "leased"
}

event lease.suite_525_103 when time.t == 22 {
  set entity asset.suite_525_103.status = "leased"
}

event lease.suite_525_104 when time.t == 25 {
  set entity asset.suite_525_104.status = "leased"
}

event lease.suite_625_101 when time.t == 25 {
  set entity asset.suite_625_101.status = "leased"
}

event lease.suite_625_102 when time.t == 28 {
  set entity asset.suite_625_102.status = "leased"
}

event lease.suite_625_103 when time.t == 28 {
  set entity asset.suite_625_103.status = "leased"
}

event lease.suite_625_104 when time.t == 31 {
  set entity asset.suite_625_104.status = "leased"
}

event lease.suite_725_101 when time.t == 31 {
  set entity asset.suite_725_101.status = "leased"
}

event lease.suite_725_102 when time.t == 31 {
  set entity asset.suite_725_102.status = "leased"
}

event lease.outparcel_whosaburger when time.t == 19 {
  set entity asset.outparcel_whosaburger.status = "leased"
}

event lease.outparcel_coffee when time.t == 19 {
  set entity asset.outparcel_coffee.status = "leased"
}

entity asset cloan : Asset.Financial {
  // Cumulative funding required: land and the financing fee at close, then the
  // monthly development draw. The increment is RECOMPUTED from the same drivers
  // the cost streams use, because a field cannot read a stream.
  // `prev.asset.center.total_hard_costs` reads a CONSTANT field one period
  // back — for a value that never changes the previous period's value IS the
  // value, which is how a derived constant is reused inside a recurrence
  // without restating its chain.
  funding_required_to_date
      init inputs.land_purchase + inputs.land_closing + inputs.land_diligence
           + inputs.financing_fees
      next prev + if(time.t >= 1 and time.t <= inputs.soft_cost_months,
              (prev.asset.center.total_hard_costs - inputs.offsite)
                * curve_value("scurve_18", time.date)
              + (prev.asset.center.total_hard_costs
                   * (inputs.arch_pct + inputs.constr_mgmt_pct)
                 + inputs.marketing_leasing + inputs.const_period_tax
                 + prev.asset.center.development_fee) / inputs.soft_cost_months,
              0)
           + if(time.t >= 1 and time.t <= inputs.offsite_months,
                inputs.offsite / inputs.offsite_months, 0)

  // THE LOAN HAS THREE BALANCES, not one, and A.CRE tracks them separately
  // (rows 72, 73, 74) because they are different obligations that happen to
  // share a facility. Rolling them into a single number would make the
  // interest reserve indistinguishable from principal, and would let the
  // shortfall reserve read as though operating losses were themselves debt.
  //
  // 1. PRINCIPAL. Equity funds first, so the split at any date is a function of
  //    cumulative funding against the commitment — no per-period min-chain.
  principal_balance
      init 0
      next prev
           + max(0, prev.asset.cloan.funding_required_to_date + if(time.t >= 1 and time.t <= inputs.soft_cost_months,
              (prev.asset.center.total_hard_costs - inputs.offsite)
                * curve_value("scurve_18", time.date)
              + (prev.asset.center.total_hard_costs
                   * (inputs.arch_pct + inputs.constr_mgmt_pct)
                 + inputs.marketing_leasing + inputs.const_period_tax
                 + prev.asset.center.development_fee) / inputs.soft_cost_months,
              0)
           + if(time.t >= 1 and time.t <= inputs.offsite_months,
                inputs.offsite / inputs.offsite_months, 0)
                    - inputs.equity_commitment)
           - max(0, prev.asset.cloan.funding_required_to_date
                    - inputs.equity_commitment)

  // 2. INTEREST RESERVE. Interest accrues on the whole balance carried in.
  //    Whatever the sponsor elects to cover from lease-up income is paid; the
  //    remainder capitalizes into this reserve. At 0% election it all
  //    capitalizes, which is the more common construction-loan structure.
  interest_reserve
      init 0
      next prev
           + max(0, (prev.asset.cloan.principal_balance + prev
                     + prev.asset.cloan.shortfall_reserve)
                    * inputs.construction_rate / time.ppy
                    - inputs.leaseup_income_to_interest
                      * max(0, curve_value("operating_cash_flow", time.date)))

  // 3. OPERATING SHORTFALL RESERVE. A separate advance the lender makes to
  //    fund lease-up deficits. It is a loan against operating losses, which is
  //    why it is its own balance and not netted into principal.
  shortfall_reserve
      init 0
      next prev + max(0, 0 - curve_value("operating_cash_flow", time.date))

  // THE FIXED POINT, VERIFIED. Total uses is the commitment plus the balance,
  // so achieved LTC is the balance over their sum. Reading `prev` rather than
  // the current balance is exact here because the balance is flat from month 22
  // on, once operations cover the interest — a field may not read another field
  // at the current period, and for a settled balance it does not need to.
  ltc_achieved
      init 0
      next (prev.asset.cloan.principal_balance + prev.asset.cloan.interest_reserve
            + prev.asset.cloan.shortfall_reserve)
           / (inputs.equity_commitment + prev.asset.cloan.principal_balance
              + prev.asset.cloan.interest_reserve
              + prev.asset.cloan.shortfall_reserve)
}

// --- The facility's cash ---------------------------------------------------
//
// Everything that moves is a stream. The fields above carry state; these carry
// money. All sit on `cloan`, so the property's unlevered total stays untouched.

// Equity funds first. At close it takes the whole of land plus the fee; after
// that it takes each month's draw until the commitment is exhausted, which
// happens in month 7. Two streams because `prev` does not exist in the first
// period (E1129_PREV_IN_FIRST_PERIOD).
stream cloan.equity_at_close on entity asset.cloan inflow currency USD {
  category financing.equity
  schedule every month due from 2024-05 to 2024-05
  amount = min(inputs.equity_commitment, asset.cloan.funding_required_to_date)
}

stream cloan.equity_contribution on entity asset.cloan inflow currency USD {
  category financing.equity
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = min(inputs.equity_commitment, asset.cloan.funding_required_to_date)
           - min(inputs.equity_commitment, prev.asset.cloan.funding_required_to_date)
}

// The loan takes the balance of each month's development draw.
stream cloan.principal_draw on entity asset.cloan inflow currency USD {
  category financing.debt_proceeds
  schedule every month from phase_start("construction") to phase_end("construction")
  amount = max(0, asset.cloan.funding_required_to_date - inputs.equity_commitment)
           - max(0, prev.asset.cloan.funding_required_to_date - inputs.equity_commitment)
}

// Interest accrues on the balance carried into the month. What operations
// cannot cover capitalizes; the rest is paid from operating cash.
stream cloan.interest on entity asset.cloan outflow currency USD {
  category financing.debt_service
  schedule every month from 2024-06 to 2027-12
  amount = (prev.asset.cloan.principal_balance
             + prev.asset.cloan.interest_reserve
             + prev.asset.cloan.shortfall_reserve)
           * inputs.construction_rate / time.ppy
}

// The lease-up deficit, advanced by the lender (A.CRE's EW59).
stream cloan.shortfall_advance on entity asset.cloan inflow currency USD {
  category financing.debt_proceeds
  schedule every month from 2024-06 to 2027-12
  amount = max(0, 0 - curve_value("operating_cash_flow", time.date))
}

// The facility is retired from sale proceeds at month 43.
stream cloan.payoff on entity asset.cloan outflow currency USD {
  category financing.debt_proceeds
  schedule every month from 2027-12 to 2027-12
  amount = prev.asset.cloan.principal_balance
           + prev.asset.cloan.interest_reserve
           + prev.asset.cloan.shortfall_reserve
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.0
  }
}
```

## Verified results

Checked period by period: **3 series** across **44 periods** — **132 values** in all, each within the tolerance shown.

- `domain.cre.egi` — within ±0.000001
- `domain.cre.noi` — within ±0.000001
- `domain.cre.leasing_costs` — within ±0.00001

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `domain.cre.noi` | 3,183,823.64 | ±0.0001 |
| `stream.center.construction_costs.total` | -13,725,000 | ±0.0001 |
| `stream.center.development_fee.total` | -647,792.08 | ±0.0001 |
| `stream.cloan.principal_draw.total` | 13,122,616.87 | ±0.0001 |
| `stream.cloan.shortfall_advance.total` | 192,189.82 | ±0.0001 |
| `stream.cloan.interest.total` | -3,439,700.65 | ±0.0001 |
| `stream.cloan.payoff.total` | -14,381,551.36 | ±0.0001 |
| `stream.center.exit_proceeds.total` | 29,170,554.76 | ±0.0001 |
