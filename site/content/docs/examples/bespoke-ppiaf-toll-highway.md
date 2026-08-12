---
id: benchmark-bespoke-ppiaf-toll-highway
title: "Bespoke: tolled highway PPP concession"
slug: "/docs/examples/bespoke-ppiaf-toll-highway"
source: benchmarks/bespoke/ppiaf_toll_highway
---

# Bespoke: tolled highway PPP concession

A 125 km toll highway concession from the World Bank's highway PPP toolkit, financed with three debt tranches and topped up each year by an availability subsidy sized to hold debt service cover at 1.30x.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

# A tolled highway concession — no pack, and a subsidy the model solves for itself

## The case

A 125 km, 2x2-lane tolled highway, built over four years and operated for
forty-six more under a fifty-year concession. Ten thousand vehicles a day use
it at opening, split evenly between two categories paying 0.13 and 0.25 USD per
vehicle per kilometre, and traffic grows 3% a year for the life of the deal.

Almost every mechanic in it is a phase change. Construction draws on three debt
tranches at once — 80% at 4.0% over twenty years, 10% at 4.5% over fifteen, 10%
at 5.0% over ten — with interest capitalising into each balance rather than
being paid, and each tranche's grace period ending in a different year, so the
first tranche starts repaying in 2014 and the other two in 2015. Operating cost
is a regressive scale: the first ten thousand vehicles a day cost nothing to
serve, the next ten thousand cost 0.60 each, the next 0.30, everything above
0.15 — and traffic crosses two of those thresholds before the concession ends.
Corporate tax is levied on the smaller of the year's profit and the profit
accumulated to date, and paid a year late.

And the road does not pay for itself. The contracting authority tops it up each
year with an availability subsidy sized to hold the annual debt service cover
ratio at exactly 1.30x. It pays 21.7m in 2014, rises to 64.9m by 2017, then
falls away as traffic growth outruns the fixed costs and the two short tranches
retire — and stops entirely after 2023, five years before the last tranche is
repaid.

## The reference

The World Bank and PPIAF's *Numerical Model for Financial Simulation of Highway
PPP Projects*, run at the case-study defaults that ship inside it. The workbook
is the toolkit's own teaching model for exactly this deal, and it carries a
complete set of cached values: a fifty-year cash flow waterfall, income
statement, three per-tranche repayment schedules, a funding-during-construction
table and a results sheet. Every figure asserted here is one of those cached
values, so the comparison is period by period rather than against a single
answer.

**Not vendored.** The workbook and the user guide are freely downloadable from
the toolkit, but neither carries an explicit reuse grant, so neither is
committed here. They were fetched once outside the repository and only their
output numbers were carried across. See SOURCE.md.

## What it exercises

| | |
|---|---|
| Pack | **none** — written from the bare language |
| Declared | five entities, nine declared fields, twenty-one native streams |
| Language features | declared state with `init`/`next`, cross-field `prev` reads, a state that snapshots and then holds, `min`/`max`/`pow` |
| Conventions | mid-year drawdown with capitalised interest, constant P+I annuities off three different grace periods, VAT stripped from an inclusive toll, tax in arrears with loss carryforward, a regressive cost scale, an ADSCR-targeted subsidy |

**This is the first case in the suite with no pack**, and that is half the
point of it. A toll road is none of the four: it has no generation and no
offtaker, so `energy` does not describe it; no rent roll, so `cre` does not; no
pool of obligors, so `credit` does not; and its revenue is a traffic count
times a distance times a tariff rather than a margin on sales, so `opco` does
not. Every other benchmark demonstrates that a pack works. This one
demonstrates something no pack-based case can: that the language underneath is
enough to build an asset class nobody has written a pack for.

**The ADSCR-targeted subsidy needs no solver.** The reference computes the
subsidy as an output — the amount that makes cover come out at 1.30x — and read
naively that is a fixed point, because the subsidy sits inside cash available
for debt service, cash available for debt service is net of corporate tax, and
tax is charged on a profit that includes the subsidy. It is not circular,
because tax is paid one year in arrears:

    subsidy(t)  = max(0, 1.30 * debt_service(t) - (revenue(t) - opex(t) - tax_paid(t)))
    tax_paid(t) = 30% * min(pbt(t-1), cumulative_pbt(t-1))

Everything on the right is finished before period *t* is evaluated, so the
subsidy falls out arithmetically once a period. This is the same move as the
tax equity flip, where an IRR hurdle became a discounted running sum: the
circularity is in how the spreadsheet is wired, not in the deal.

## The result

**Exact.** Twenty-five series across fifty-one periods and seven financing-plan
totals reproduce the workbook's cached values.

| | model | reference |
|---|---:|---:|
| total uses / sources | 796.229877 | 796.229877 |
| 1st tranche at financial close | 577.459550 | 577.459550 |
| 2nd tranche | 72.852262 | 72.852262 |
| 3rd tranche | 73.526569 | 73.526569 |
| 1st tranche annuity (P+I) | 51.937347 | 51.937347 |
| subsidy, 2014 | 21.697430 | 21.697430 |
| subsidy, nominal, whole concession | 351.951289 | 351.951289 |
| ADSCR, 2013 (unsubsidised) | 1.769033 | 1.769033 |

Asserted: the works, equity, fee and per-tranche drawdown lines through
construction; all three tranche balances across all fifty years; per-tranche
interest and principal; both toll revenue lines; five operating cost lines; the
subsidy; corporate tax; profit before tax; and the depreciable capital base —
1,322 figures in total.

## The delta

The declared state agrees to **2.7e-12** — machine epsilon over a fifty-year
recursion. The cash streams agree to **8.9e-7**, which is not a modelling
difference: the results file publishes stream amounts rounded to six decimal
places, and these are USD millions, so 8.9e-7 is fifty cents on figures in the
hundreds of millions. The per-period tolerance is set at 1e-5 to sit just above
that rounding floor.

One thing the case does **not** assert is the reference's equity IRR, project
IRR and NPV. Those need the dividend policy — distributable reserves are the
lesser of the cash balance and cumulated retained profit — and a balance sheet
to carry cash between years, neither of which is modelled here. The spine that
determines them is: revenue, cost, tax, subsidy and all three debt schedules
are all asserted, so anything downstream would be arithmetic on numbers that
already agree.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
// A 125 km tolled highway concession, built against the World Bank / PPIAF
// highway PPP numerical model's own case study.
//
// WHY THIS CASE IS PACK-FREE. Every other benchmark in this suite instantiates
// a domain pack. A toll road is not any of them: it has no generation and no
// offtaker, so `energy` does not fit; no rent roll, so `cre` does not; no pool
// of obligors, so `credit` does not; and its revenue is a traffic count times a
// distance times a tariff rather than a margin on sales, so `opco` does not.
// It is written here from bare entities, states and streams. That makes it the
// suite's evidence for a claim no pack-based case can make: that the language
// on its own is enough to express an asset class nobody has written a pack for.
//
// THE ADSCR-TARGETED SUBSIDY IS A RECURRENCE, NOT A CIRCULARITY. The reference
// runs in "Mode 1": the contracting authority tops the project up each year to
// whatever it takes to hold the annual debt service cover ratio at 1.30, so the
// subsidy is an output of the model, not an input. Read naively that is a
// fixed point — the subsidy sits inside cash available for debt service, and
// cash available for debt service is net of corporate tax, and tax is levied on
// a profit that includes the subsidy.
//
// It is not circular, because tax is paid one year in arrears. The tax
// settled in year n is 30% of year n-1's profit, and that profit is finished
// before year n is evaluated. So:
//
//     subsidy(t) = max(0, 1.30 * debt_service(t)
//                          - (revenue(t) - opex(t) - tax_paid(t)))
//     tax_paid(t) = 30% * min(pbt(t-1), cumulative_pbt(t-1))
//
// and every term on the right is settled. The subsidy falls out arithmetically
// once a period, with no solver, exactly as the tax equity flip's IRR hurdle
// does. See CASE.md.
//
// THE TIMELINE STARTS AT THE STUDY YEAR. 2008 is period 0 and carries nothing:
// it is the year the costs and tolls are quoted in, so every escalation is
// pow(1.02, time.t) with no off-by-one. Construction is 2009-2012 (t=1..4) and
// operation 2013-2058 (t=5..50).

version 0.1
model "ppiaf-toll-highway"
time calendar annual from 2008-01 for 51

phase construction from 2009-01 to 2012-12
phase operation    from 2013-01 to 2058-12

// --- the road -------------------------------------------------------------
assume length_km          = 125.0
assume cost_per_km        = 5.5        // USD million, 2008 terms
assume construction_real  = 687.5      // length_km * cost_per_km

// --- escalation -----------------------------------------------------------
// Costs inflate; tolls are indexed on their own rate, which happens to equal
// inflation in this case study but is a separate assumption in the source.
assume inflation          = 0.02
assume toll_indexation    = 0.02
assume vat_rate           = 0.196

// --- traffic and tolls ----------------------------------------------------
// Two vehicle categories, 5,000 vehicles a day each at opening, both growing
// 3% a year for the whole concession.
assume traffic_cat1_open  = 5000.0
assume traffic_cat2_open  = 5000.0
assume traffic_growth     = 0.03
assume toll_cat1          = 0.13       // USD per vehicle per km, VAT included
assume toll_cat2          = 0.25

// --- operating costs (2008 terms, USD million) ----------------------------
assume concessionaire_cost = 2.0       // runs from the start of construction
assume operation_cost      = 6.0       // operating period only
assume heavy_maintenance   = 14.609375 // 17% of construction cost every 8 years
assume light_maintenance   = 1.71875   // 0.25% of construction cost a year

// Variable cost is a regressive scale on daily traffic: nothing on the first
// 10,000 vehicles, then 0.60, 0.30 and 0.15 USD a vehicle on each band above.
assume var_band2_rate      = 0.6
assume var_band3_rate      = 0.3
assume var_band4_rate      = 0.15

// --- financing ------------------------------------------------------------
assume equity_pct          = 0.10      // of construction cost, excluding capitalised interest
assume fee_pct             = 0.015     // arrangement fees, % of the debt requirement
assume rate_t1             = 0.04
assume rate_t2             = 0.045
assume rate_t3             = 0.05
assume alloc_t1            = 0.8
assume alloc_t2            = 0.1
assume alloc_t3            = 0.1

assume tax_rate            = 0.30
assume adscr_target        = 1.3
assume depreciation_years  = 46.0

entity asset concession : Asset.Real

// ---------------------------------------------------------------------------
// The debt.
//
// Three tranches, same seniority and currency, each with its own maturity,
// rate and grace period, all measured from the start of construction:
//
//              maturity   grace   first principal   payments   rate
//   tranche 1    20 yrs   5 yrs     2014 (t=6)         15      4.0%
//   tranche 2    15 yrs   6 yrs     2015 (t=7)          9      4.5%
//   tranche 3    10 yrs   6 yrs     2015 (t=7)          4      5.0%
//
// During construction the tranches DRAW and interest CAPITALISES: the year's
// draw is taken mid-year, so it accrues half a year of interest while the
// opening balance accrues a full one. After construction interest is paid in
// cash, and once the grace period ends each tranche repays on a constant
// principal-plus-interest annuity.
//
// The annuity is a state rather than a literal so that nothing is copied back
// in from the answer. It is fixed one period BEFORE the first repayment, off
// the balance standing at the end of grace — a `next` expression may only read
// the previous period, so computing it in the repayment year itself would be
// one period too late. The balance is flat through grace, so the two are the
// same number.
// ---------------------------------------------------------------------------

entity asset tranche1 : Asset.Financial {
  balance init 0.0
    next if(time.t <= 4,
            prev * (1.0 + inputs.rate_t1)
              + inputs.alloc_t1 * 1.015
                * (0.9 * inputs.construction_real
                     * if(time.t == 1, 0.1,
                       if(time.t == 2, 0.3,
                       if(time.t == 3, 0.5,
                       if(time.t == 4, 0.1, 0.0))))
                   + inputs.concessionaire_cost)
                * pow(1.0 + inputs.inflation, time.t)
                * (1.0 + inputs.rate_t1 / 2.0),
            prev - if(time.t >= 6 and time.t <= 20,
                      prev.asset.tranche1.annuity - prev * inputs.rate_t1,
                      0.0))

  annuity init 0.0
    next if(time.t == 5,
            prev.asset.tranche1.balance * inputs.rate_t1
              / (1.0 - pow(1.0 + inputs.rate_t1, -15.0)),
            prev)
}

entity asset tranche2 : Asset.Financial {
  balance init 0.0
    next if(time.t <= 4,
            prev * (1.0 + inputs.rate_t2)
              + inputs.alloc_t2 * 1.015
                * (0.9 * inputs.construction_real
                     * if(time.t == 1, 0.1,
                       if(time.t == 2, 0.3,
                       if(time.t == 3, 0.5,
                       if(time.t == 4, 0.1, 0.0))))
                   + inputs.concessionaire_cost)
                * pow(1.0 + inputs.inflation, time.t)
                * (1.0 + inputs.rate_t2 / 2.0),
            prev - if(time.t >= 7 and time.t <= 15,
                      prev.asset.tranche2.annuity - prev * inputs.rate_t2,
                      0.0))

  annuity init 0.0
    next if(time.t == 6,
            prev.asset.tranche2.balance * inputs.rate_t2
              / (1.0 - pow(1.0 + inputs.rate_t2, -9.0)),
            prev)
}

entity asset tranche3 : Asset.Financial {
  balance init 0.0
    next if(time.t <= 4,
            prev * (1.0 + inputs.rate_t3)
              + inputs.alloc_t3 * 1.015
                * (0.9 * inputs.construction_real
                     * if(time.t == 1, 0.1,
                       if(time.t == 2, 0.3,
                       if(time.t == 3, 0.5,
                       if(time.t == 4, 0.1, 0.0))))
                   + inputs.concessionaire_cost)
                * pow(1.0 + inputs.inflation, time.t)
                * (1.0 + inputs.rate_t3 / 2.0),
            prev - if(time.t >= 7 and time.t <= 10,
                      prev.asset.tranche3.annuity - prev * inputs.rate_t3,
                      0.0))

  annuity init 0.0
    next if(time.t == 6,
            prev.asset.tranche3.balance * inputs.rate_t3
              / (1.0 - pow(1.0 + inputs.rate_t3, -4.0)),
            prev)
}

// ---------------------------------------------------------------------------
// The project's own books: the depreciable capital base, and the two profit
// figures the tax charge is computed from.
//
// `capital` accumulates every use of funds during construction — works,
// concessionaire costs, fees, and the interest capitalised into each tranche —
// and then holds. It is the depreciation base, and because it stops moving at
// t=4 the operating years can read it through `prev` without a lag.
//
// `pbt` is the year's profit before tax and `cum_pbt` is the profit accumulated
// through the PREVIOUS year, which is the pair the source's tax rule needs:
// the charge is levied on the smaller of the year's profit and the cumulative
// profit, so a loss carried forward shelters later income.
// ---------------------------------------------------------------------------

entity asset project : Asset.Financial {
  capital init 0.0
    next if(time.t <= 4,
            prev
              + inputs.equity_pct * inputs.construction_real
                  * if(time.t == 1, 0.1,
                    if(time.t == 2, 0.3,
                    if(time.t == 3, 0.5,
                    if(time.t == 4, 0.1, 0.0))))
                  * pow(1.0 + inputs.inflation, time.t)
              + 1.015
                * (0.9 * inputs.construction_real
                     * if(time.t == 1, 0.1,
                       if(time.t == 2, 0.3,
                       if(time.t == 3, 0.5,
                       if(time.t == 4, 0.1, 0.0))))
                   + inputs.concessionaire_cost)
                * pow(1.0 + inputs.inflation, time.t)
                * (1.0 + inputs.alloc_t1 * inputs.rate_t1 / 2.0
                       + inputs.alloc_t2 * inputs.rate_t2 / 2.0
                       + inputs.alloc_t3 * inputs.rate_t3 / 2.0)
              + prev.asset.tranche1.balance * inputs.rate_t1
              + prev.asset.tranche2.balance * inputs.rate_t2
              + prev.asset.tranche3.balance * inputs.rate_t3,
            prev)

  pbt init 0.0
    next if(time.t < 5, 0.0,
            (inputs.traffic_cat1_open * inputs.toll_cat1
               + inputs.traffic_cat2_open * inputs.toll_cat2)
              * 365.0 * inputs.length_km / 1000000.0
              * pow(1.0 + inputs.traffic_growth, time.t - 5.0)
              * pow(1.0 + inputs.toll_indexation, time.t)
              / (1.0 + inputs.vat_rate)
            + max(0.0,
                  inputs.adscr_target
                    * (if(time.t >= 6 and time.t <= 20,
                          prev.asset.tranche1.annuity,
                          prev.asset.tranche1.balance * inputs.rate_t1)
                     + if(time.t >= 7 and time.t <= 15,
                          prev.asset.tranche2.annuity,
                          prev.asset.tranche2.balance * inputs.rate_t2)
                     + if(time.t >= 7 and time.t <= 10,
                          prev.asset.tranche3.annuity,
                          prev.asset.tranche3.balance * inputs.rate_t3))
                  - ((inputs.traffic_cat1_open * inputs.toll_cat1
                        + inputs.traffic_cat2_open * inputs.toll_cat2)
                       * 365.0 * inputs.length_km / 1000000.0
                       * pow(1.0 + inputs.traffic_growth, time.t - 5.0)
                       * pow(1.0 + inputs.toll_indexation, time.t)
                       / (1.0 + inputs.vat_rate)
                     - (inputs.concessionaire_cost + inputs.operation_cost
                          + inputs.heavy_maintenance + inputs.light_maintenance)
                       * pow(1.0 + inputs.inflation, time.t)
                     - ((min(10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0), 20000.0)
                           - 10000.0) * inputs.var_band2_rate
                        + max(0.0, min(10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0), 30000.0)
                           - 20000.0) * inputs.var_band3_rate
                        + max(0.0, 10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0)
                           - 30000.0) * inputs.var_band4_rate)
                       * 365.0 / 1000000.0
                       * pow(1.0 + inputs.inflation, time.t)
                     - inputs.tax_rate
                       * max(0.0, min(prev.asset.project.pbt,
                                      prev.asset.project.cum_pbt
                                        + prev.asset.project.pbt))))
            - (inputs.concessionaire_cost + inputs.operation_cost
                 + inputs.heavy_maintenance + inputs.light_maintenance)
              * pow(1.0 + inputs.inflation, time.t)
            - ((min(10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0), 20000.0)
                  - 10000.0) * inputs.var_band2_rate
               + max(0.0, min(10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0), 30000.0)
                  - 20000.0) * inputs.var_band3_rate
               + max(0.0, 10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0)
                  - 30000.0) * inputs.var_band4_rate)
              * 365.0 / 1000000.0
              * pow(1.0 + inputs.inflation, time.t)
            - prev.asset.project.capital / inputs.depreciation_years
            - prev.asset.tranche1.balance * inputs.rate_t1
            - prev.asset.tranche2.balance * inputs.rate_t2
            - prev.asset.tranche3.balance * inputs.rate_t3)

  cum_pbt init 0.0
    next prev + prev.asset.project.pbt
}

// ---------------------------------------------------------------------------
// Construction period: sources and uses.
// ---------------------------------------------------------------------------

stream infra.construction.works on entity asset.concession outflow currency USD {
  schedule every year from 2009-01 to 2012-01
  category investing.capex
  amount = inputs.construction_real
             * if(time.t == 1, 0.1, if(time.t == 2, 0.3, if(time.t == 3, 0.5, 0.1)))
             * pow(1.0 + inputs.inflation, time.t)
}

// The concessionaire's own overhead. It runs the whole concession, not just
// the build, which is why it is not folded into the works line.
stream infra.opex.concessionaire on entity asset.concession outflow currency USD {
  schedule every year from 2009-01 to 2058-01
  category operating.opex
  amount = inputs.concessionaire_cost * pow(1.0 + inputs.inflation, time.t)
}

// Arrangement fees are 1.5% of the debt requirement, and are themselves funded
// by debt — the drawdown is the requirement grossed up by the fee.
stream infra.funding.fees on entity asset.concession outflow currency USD {
  schedule every year from 2009-01 to 2012-01
  category financing.fees
  amount = inputs.fee_pct
             * (0.9 * inputs.construction_real
                  * if(time.t == 1, 0.1, if(time.t == 2, 0.3, if(time.t == 3, 0.5, 0.1)))
                + inputs.concessionaire_cost)
             * pow(1.0 + inputs.inflation, time.t)
}

stream infra.funding.equity on entity asset.concession inflow currency USD {
  schedule every year from 2009-01 to 2012-01
  category financing.equity
  amount = inputs.equity_pct * inputs.construction_real
             * if(time.t == 1, 0.1, if(time.t == 2, 0.3, if(time.t == 3, 0.5, 0.1)))
             * pow(1.0 + inputs.inflation, time.t)
}

// Each tranche's cash drawdown. Capitalised interest is not drawn cash — it is
// rolled into the balance — so it is visible in the balances, not here.
stream infra.funding.draw_t1 on entity asset.concession inflow currency USD {
  schedule every year from 2009-01 to 2012-01
  category financing.debt_draw
  amount = inputs.alloc_t1 * 1.015
             * (0.9 * inputs.construction_real
                  * if(time.t == 1, 0.1, if(time.t == 2, 0.3, if(time.t == 3, 0.5, 0.1)))
                + inputs.concessionaire_cost)
             * pow(1.0 + inputs.inflation, time.t)
}

stream infra.funding.draw_t2 on entity asset.concession inflow currency USD {
  schedule every year from 2009-01 to 2012-01
  category financing.debt_draw
  amount = inputs.alloc_t2 * 1.015
             * (0.9 * inputs.construction_real
                  * if(time.t == 1, 0.1, if(time.t == 2, 0.3, if(time.t == 3, 0.5, 0.1)))
                + inputs.concessionaire_cost)
             * pow(1.0 + inputs.inflation, time.t)
}

stream infra.funding.draw_t3 on entity asset.concession inflow currency USD {
  schedule every year from 2009-01 to 2012-01
  category financing.debt_draw
  amount = inputs.alloc_t3 * 1.015
             * (0.9 * inputs.construction_real
                  * if(time.t == 1, 0.1, if(time.t == 2, 0.3, if(time.t == 3, 0.5, 0.1)))
                + inputs.concessionaire_cost)
             * pow(1.0 + inputs.inflation, time.t)
}

// ---------------------------------------------------------------------------
// Operating period: toll revenue.
//
// Tolls are quoted per vehicle per km including VAT, in study-year money. The
// road is 125 km, so a vehicle paying the category 1 rate end to end pays
// 16.25 USD. Revenue is indexed on the toll index, and VAT is stripped out
// because it belongs to the state, not the concessionaire.
// ---------------------------------------------------------------------------

stream infra.revenue.toll_cat1 on entity asset.concession inflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category operating.revenue
  amount = inputs.traffic_cat1_open * pow(1.0 + inputs.traffic_growth, time.t - 5.0)
             * 365.0 * inputs.toll_cat1 * inputs.length_km / 1000000.0
             * pow(1.0 + inputs.toll_indexation, time.t)
             / (1.0 + inputs.vat_rate)
}

stream infra.revenue.toll_cat2 on entity asset.concession inflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category operating.revenue
  amount = inputs.traffic_cat2_open * pow(1.0 + inputs.traffic_growth, time.t - 5.0)
             * 365.0 * inputs.toll_cat2 * inputs.length_km / 1000000.0
             * pow(1.0 + inputs.toll_indexation, time.t)
             / (1.0 + inputs.vat_rate)
}

// ---------------------------------------------------------------------------
// Operating period: costs.
// ---------------------------------------------------------------------------

stream infra.opex.operations on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category operating.opex
  amount = inputs.operation_cost * pow(1.0 + inputs.inflation, time.t)
}

// Periodic resurfacing is 17% of construction cost every eight years, but the
// source charges it as a level annual accrual of one eighth rather than as a
// lump in the year the work happens.
stream infra.opex.heavy_maintenance on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category operating.opex
  amount = inputs.heavy_maintenance * pow(1.0 + inputs.inflation, time.t)
}

stream infra.opex.light_maintenance on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category operating.opex
  amount = inputs.light_maintenance * pow(1.0 + inputs.inflation, time.t)
}

// The regressive scale: the first 10,000 vehicles a day cost nothing to serve,
// the next 10,000 cost 0.60 each, the next 10,000 0.30, everything above 0.15.
// Traffic passes 20,000 in 2037 and 30,000 in 2051, so all three bands bind
// before the concession ends.
stream infra.opex.variable on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category operating.opex
  amount = ((min(10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0), 20000.0) - 10000.0)
              * inputs.var_band2_rate
            + max(0.0, min(10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0), 30000.0) - 20000.0)
              * inputs.var_band3_rate
            + max(0.0, 10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0) - 30000.0)
              * inputs.var_band4_rate)
           * 365.0 / 1000000.0
           * pow(1.0 + inputs.inflation, time.t)
}

// Corporate tax is levied on the smaller of the year's profit and the profit
// accumulated to date, and settled the following year.
stream infra.tax.corporate on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category operating.tax
  amount = inputs.tax_rate
             * max(0.0, min(prev.asset.project.pbt, asset.project.cum_pbt))
}

// ---------------------------------------------------------------------------
// Operating period: the availability subsidy.
//
// This is the line the whole case exists for. The authority pays whatever it
// takes to hold cover at 1.30x, and nothing once the project covers itself —
// which is why 2013 is unsubsidised at 1.77x, and why the payments stop dead
// after 2028 when the last tranche is repaid and debt service goes to zero.
// ---------------------------------------------------------------------------

stream infra.subsidy.availability on entity asset.concession inflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category operating.subsidy
  amount = max(0.0,
               inputs.adscr_target
                 * (if(time.t >= 6 and time.t <= 20,
                       asset.tranche1.annuity,
                       prev.asset.tranche1.balance * inputs.rate_t1)
                  + if(time.t >= 7 and time.t <= 15,
                       asset.tranche2.annuity,
                       prev.asset.tranche2.balance * inputs.rate_t2)
                  + if(time.t >= 7 and time.t <= 10,
                       asset.tranche3.annuity,
                       prev.asset.tranche3.balance * inputs.rate_t3))
               - (inputs.traffic_cat1_open * inputs.toll_cat1
                    + inputs.traffic_cat2_open * inputs.toll_cat2)
                   * 365.0 * inputs.length_km / 1000000.0
                   * pow(1.0 + inputs.traffic_growth, time.t - 5.0)
                   * pow(1.0 + inputs.toll_indexation, time.t)
                   / (1.0 + inputs.vat_rate)
               + (inputs.concessionaire_cost + inputs.operation_cost
                    + inputs.heavy_maintenance + inputs.light_maintenance)
                 * pow(1.0 + inputs.inflation, time.t)
               + ((min(10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0), 20000.0) - 10000.0)
                    * inputs.var_band2_rate
                  + max(0.0, min(10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0), 30000.0) - 20000.0)
                    * inputs.var_band3_rate
                  + max(0.0, 10000.0 * pow(1.0 + inputs.traffic_growth, time.t - 5.0) - 30000.0)
                    * inputs.var_band4_rate)
                 * 365.0 / 1000000.0
                 * pow(1.0 + inputs.inflation, time.t)
               + inputs.tax_rate
                 * max(0.0, min(prev.asset.project.pbt, asset.project.cum_pbt)))
}

// ---------------------------------------------------------------------------
// Operating period: debt service.
// ---------------------------------------------------------------------------

stream infra.debt.interest_t1 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.interest
  amount = prev.asset.tranche1.balance * inputs.rate_t1
}

stream infra.debt.interest_t2 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.interest
  amount = prev.asset.tranche2.balance * inputs.rate_t2
}

stream infra.debt.interest_t3 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.interest
  amount = prev.asset.tranche3.balance * inputs.rate_t3
}

stream infra.debt.principal_t1 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.debt_principal
  amount = prev.asset.tranche1.balance - asset.tranche1.balance
}

stream infra.debt.principal_t2 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.debt_principal
  amount = prev.asset.tranche2.balance - asset.tranche2.balance
}

stream infra.debt.principal_t3 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.debt_principal
  amount = prev.asset.tranche3.balance - asset.tranche3.balance
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.08
  }
}
```

## Verified results

Checked period by period: **26 series** across **51 periods** — **1322 values** in all, each within ±0.00001 of the reference.

- `infra.construction.works`
- `infra.funding.equity`
- `infra.funding.fees`
- `infra.funding.draw_t1`
- `infra.funding.draw_t2`
- `infra.funding.draw_t3`
- `asset.tranche1.balance`
- `asset.tranche2.balance`
- `asset.tranche3.balance`
- `infra.debt.interest_t1`
- `infra.debt.interest_t2`
- `infra.debt.interest_t3`
- `infra.debt.principal_t1`
- `infra.debt.principal_t2`
- `infra.debt.principal_t3`
- `infra.revenue.toll_cat1`
- `infra.revenue.toll_cat2`
- `infra.opex.concessionaire`
- `infra.opex.operations`
- `infra.opex.heavy_maintenance`
- `infra.opex.light_maintenance`
- `infra.opex.variable`
- `infra.subsidy.availability`
- `infra.tax.corporate`
- `asset.project.pbt`
- `asset.project.capital`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `stream.infra.construction.works.total` | -723.914961 | ±0.00001 |
| `stream.infra.funding.equity.total` | 72.391496 | ±0.00001 |
| `stream.infra.funding.fees.total` | -9.898973 | ±0.00001 |
| `stream.infra.debt.principal_t1.total` | -577.45955 | ±0.00001 |
| `stream.infra.debt.principal_t2.total` | -72.852262 | ±0.00001 |
| `stream.infra.debt.principal_t3.total` | -73.526569 | ±0.00001 |
| `stream.infra.subsidy.availability.total` | 351.951289 | ±0.00001 |
