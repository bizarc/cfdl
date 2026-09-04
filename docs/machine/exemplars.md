<!-- GENERATED benchmark exemplars by tools/gen-machine-docs.py — do not edit by hand. Regenerate: make machine-docs -->

# CFDL benchmark exemplars

CFDL 0.9.0. 18 of the registered benchmark
cases, curated so every core-language mechanism and every
meaningfully-composed pack pattern appears at least once. Each model
compiles, runs, and matches an external reference within stated tolerances
in CI — these are full deals, the step up from the single-purpose valid
corpus. Each entry carries the case's own "what it exercises" grid.

## bespoke/buenavista_del_cobre

A 41-year open-pit copper mine whose production plan is derived from its reserve statement, with the pit's strip ratio drawn from a distribution and the valuation reported as a range.

| | |
|---|---|
| Pack | none — written from the bare language |
| Entities | four depleting stocks, each with tonnage and contained metal |
| Language features | stock recurrences, capacity limits, streams reading the period's result through `series_sum`, a stochastic assumption, a seeded Monte Carlo, run-config knobs |
| Conventions | duty on EBITDA, profit share on EBITDA net of depreciation and duty, income tax net of a duty credit, loss carryforward, first year undiscounted |

The second case in the suite written without a pack, and the first to assert a
Monte Carlo result.

**The stocks are the model.** Each of the four reserve classes is an entity
carrying two balances: tonnage and contained metal. A period draws
`min(capacity, remaining)`, and both balances fall together. The copper
concentrator steps from 74 to 43 million tonnes a year when Concentrator I is
taken offline, which is a property of that stock's processing route.

**Grade is a consequence.** Because contained metal is a stock alongside
tonnage, the grade of any period is the remaining metal over the remaining
rock. The exception is the mine's own published head-grade policy, which sets
the first three years; the balance of the reserve carries the rest of the life.
Nothing about grade is fitted.

**The strip ratio is a distribution.** Waste is what must move to reach ore.
The reserve statement pins the life-of-mine ratio at 0.83. An operating pit
strips overburden before it reaches ore and the ratio climbs as the pit
deepens, so the ratio is drawn from a triangular distribution across the range
an operating pit exhibits, with the published figure as its mode. The seed is
declared, so the draws are reproducible.

```cfdl
// Buenavista del Cobre — the mine plan derived from the reserve, not imported.
//
// THE CLAIM. A mine is four depleting stocks of rock, each with its own
// contained metal and its own processing capacity. Each period draws what
// capacity allows, or the remainder if less, and carries the balance forward.
// Mine life is therefore an OUTPUT: the mill runs until its stock is gone.
//
// Grade is not assumed. Contained metal is a stock alongside tonnage, so the
// grade of any period is the remaining metal over the remaining rock. The one
// exception is the mine's own published head-grade policy for its first three
// years, after which the balance carries the rest of the life.
//
// WHAT IS IMPORTED. Nothing but the reserve statement: four tonnages, their
// contained metal, two mill capacities, the strip ratio, and the head-grade
// policy. About twenty numbers, all from Tables 12.5, 12.7 and 12.8. The
// operator's 41-year production schedule is NOT an input to this model; it is
// a comparison, alongside the cash flow.

version 0.1
model "buenavista-del-cobre"
time calendar annual from 2025-01 for 41

// --- the reserve, Table 12.8 (effective 31 December 2024) ------------------
assume reserve_cu_mill_t   = 2117.0    // Mt sulfide mill feed, copper plant
assume reserve_cu_mill_cu  = 8774.0    // kt contained copper
assume reserve_cu_mill_mo  = 181.0     // kt contained molybdenum
assume reserve_zn_mill_t   = 296.0     // Mt sulfide mill feed, zinc plant
assume reserve_zn_mill_cu  = 1798.0
assume reserve_zn_mill_zn  = 1705.0
assume reserve_crushed_t   = 1077.0    // Mt crushed leach
assume reserve_crushed_cu  = 2543.0
assume reserve_rom_t       = 1041.0    // Mt ROM leach
assume reserve_rom_cu      = 3076.0

// --- capacity, section 14 and Table 13.3 headline -------------------------
assume cap_cu_mill_full    = 74.0      // Mt/yr, both concentrators
assume cap_cu_mill_reduced = 43.0      // Mt/yr, after Concentrator I closes
assume cap_zn_mill         = 7.0       // Mt/yr
// Leach is pit-driven rather than capacity-driven; the reserve total spread
// across the analysis window is the base claim, and its phasing is the
// uncertainty the case measures.
assume rate_crushed_leach  = 26.268292682926827   // 1077 / 41
assume rate_rom_leach      = 25.390243902439025   // 1041 / 41

// --- the pit, Table 12.8 --------------------------------------------------
// Waste per tonne of ore. The reserve statement pins the life-of-mine ratio
// at 0.83; the year-to-year ratio observed in an operating pit varies far more,
// because overburden is stripped before ore is reached and the ratio climbs as
// the pit deepens. Triangular on the observed bounds, with the published
// aggregate as the mode.
assume strip_ratio ~ Triangular(min=0.31, mode=0.83, max=2.08)

// --- prices, section 19.1 -------------------------------------------------
assume price_cu = 3.30
assume price_mo = 10.00
assume price_zn = 1.15

// --- recovery to payable, ours; the report states none for its cash flow --
assume rec_cu_mill  = 0.836
assume rec_cu_leach = 0.260
assume rec_mo       = 0.660
assume rec_zn       = 0.629

// --- unit costs, section 18 -----------------------------------------------
assume cost_mining     = 2.71
assume cost_mill       = 5.83
assume cost_zinc_plant = 10.63
assume cost_crush      = 0.84
assume cost_leach      = 0.40
assume cost_gna        = 0.76
assume accretion       = 34.0
assume closure_total   = 544.0
assume capital_lom     = 8317.0
// total ore in the reserve, the denominator capital is spread over
assume reserve_ore_total = 4531.0   // 2117 + 296 + 1077 + 1041
assume sell_cu = 0.54
assume sell_mo = 1.84
assume sell_zn = 0.40

// --- the Mexican fiscal stack, section 19.2 -------------------------------
assume duty_rate = 0.075
assume ptu_rate  = 0.10
assume tax_rate  = 0.30

// The mine's own head-grade instruction, Table 12.5. After 2027 the balance
// of the reserve carries the grade, which is what conserves contained metal.
curve grade_policy_cu {
  2025-01: 0.50
  2026-01: 0.48
  2027-01: 0.43
  2028-01: 0.0
}

// The same policy on lagged dates. A field's `next` computes period t from
// t-1, so a rule that must know period t-1's grade reads it here.
curve grade_policy_lagged {
  2025-01: 0.50
  2026-01: 0.50
  2027-01: 0.48
  2028-01: 0.43
  2029-01: 0.0
}

// ---------------------------------------------------------------------------
// The four stocks.
// ---------------------------------------------------------------------------

entity asset cu_mill : Asset.Real {
  // tonnage available at the period's open
  tonnes init inputs.reserve_cu_mill_t
    next max(0.0, prev - min(if(time.t - 1 <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), prev))

  // contained cu, drawn with the tonnage at the period's grade
  cu init inputs.reserve_cu_mill_cu
    next max(0.0, prev
                  - min(if(time.t - 1 <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), prev.asset.cu_mill.tonnes)
                    * (if(time.t - 1 <= 2, curve_value("grade_policy_lagged", time.date),
                       if(prev.asset.cu_mill.tonnes > 0.0,
                          prev / prev.asset.cu_mill.tonnes / 10.0, 0.0)))
                    * 10.0)

  // contained mo, drawn with the tonnage at the period's grade
  mo init inputs.reserve_cu_mill_mo
    next max(0.0, prev
                  - min(if(time.t - 1 <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), prev.asset.cu_mill.tonnes)
                    * (if(prev.asset.cu_mill.tonnes > 0.0,
                       prev / prev.asset.cu_mill.tonnes / 10.0, 0.0))
                    * 10.0)

}

entity asset zn_mill : Asset.Real {
  // tonnage available at the period's open
  tonnes init inputs.reserve_zn_mill_t
    next max(0.0, prev - min(inputs.cap_zn_mill, prev))

  // contained cu, drawn with the tonnage at the period's grade
  cu init inputs.reserve_zn_mill_cu
    next max(0.0, prev
                  - min(inputs.cap_zn_mill, prev.asset.zn_mill.tonnes)
                    * (if(prev.asset.zn_mill.tonnes > 0.0,
                       prev / prev.asset.zn_mill.tonnes / 10.0, 0.0))
                    * 10.0)

  // contained zn, drawn with the tonnage at the period's grade
  zn init inputs.reserve_zn_mill_zn
    next max(0.0, prev
                  - min(inputs.cap_zn_mill, prev.asset.zn_mill.tonnes)
                    * (if(prev.asset.zn_mill.tonnes > 0.0,
                       prev / prev.asset.zn_mill.tonnes / 10.0, 0.0))
                    * 10.0)

}

entity asset crushed : Asset.Real {
  // tonnage available at the period's open
  tonnes init inputs.reserve_crushed_t
    next max(0.0, prev - min(inputs.rate_crushed_leach, prev))

  // contained cu, drawn with the tonnage at the period's grade
  cu init inputs.reserve_crushed_cu
    next max(0.0, prev
                  - min(inputs.rate_crushed_leach, prev.asset.crushed.tonnes)
                    * (if(prev.asset.crushed.tonnes > 0.0,
                       prev / prev.asset.crushed.tonnes / 10.0, 0.0))
                    * 10.0)

}

entity asset rom : Asset.Real {
  // tonnage available at the period's open
  tonnes init inputs.reserve_rom_t
    next max(0.0, prev - min(inputs.rate_rom_leach, prev))

  // contained cu, drawn with the tonnage at the period's grade
  cu init inputs.reserve_rom_cu
    next max(0.0, prev
                  - min(inputs.rate_rom_leach, prev.asset.rom.tonnes)
                    * (if(prev.asset.rom.tonnes > 0.0,
                       prev / prev.asset.rom.tonnes / 10.0, 0.0))
                    * 10.0)

}

// ---------------------------------------------------------------------------
// The pit. Waste is what must move to reach the ore, at the published strip
// ratio; it carries no metal and no revenue, only cost.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Revenue: payable metal at its price. Contained metal is drawn in kilotonnes,
// so x 2204.6 / 1000 converts to millions of pounds.
// ---------------------------------------------------------------------------

stream mine.revenue.copper on entity asset.cu_mill inflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = inputs.price_cu * 2204.6 / 1000.0
           * (inputs.rec_cu_mill * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) * (if(time.t <= 2, curve_value("grade_policy_cu", time.date),
                if(asset.cu_mill.tonnes > 0.0, asset.cu_mill.cu / asset.cu_mill.tonnes / 10.0, 0.0))) * 10.0 + min(inputs.cap_zn_mill, asset.zn_mill.tonnes) * (if(asset.zn_mill.tonnes > 0.0, asset.zn_mill.cu / asset.zn_mill.tonnes / 10.0, 0.0)) * 10.0)
              + inputs.rec_cu_leach * (min(inputs.rate_crushed_leach, asset.crushed.tonnes) * (if(asset.crushed.tonnes > 0.0, asset.crushed.cu / asset.crushed.tonnes / 10.0, 0.0)) * 10.0 + min(inputs.rate_rom_leach, asset.rom.tonnes) * (if(asset.rom.tonnes > 0.0, asset.rom.cu / asset.rom.tonnes / 10.0, 0.0)) * 10.0))
}

stream mine.revenue.molybdenum on entity asset.cu_mill inflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = inputs.price_mo * 2204.6 / 1000.0 * inputs.rec_mo * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) * (if(asset.cu_mill.tonnes > 0.0, asset.cu_mill.mo / asset.cu_mill.tonnes / 10.0, 0.0)) * 10.0)
}

stream mine.revenue.zinc on entity asset.zn_mill inflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = inputs.price_zn * 2204.6 / 1000.0 * inputs.rec_zn * (min(inputs.cap_zn_mill, asset.zn_mill.tonnes) * (if(asset.zn_mill.tonnes > 0.0, asset.zn_mill.zn / asset.zn_mill.tonnes / 10.0, 0.0)) * 10.0)
}

// ---------------------------------------------------------------------------
// Cost.
// ---------------------------------------------------------------------------

stream mine.opex.mining on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.cost_mining * (1.0 + inputs.strip_ratio) * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) + min(inputs.cap_zn_mill, asset.zn_mill.tonnes) + min(inputs.rate_crushed_leach, asset.crushed.tonnes) + min(inputs.rate_rom_leach, asset.rom.tonnes))
}

stream mine.opex.processing on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.cost_mill * min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes)
           + inputs.cost_zinc_plant * min(inputs.cap_zn_mill, asset.zn_mill.tonnes)
           + inputs.cost_crush * min(inputs.rate_crushed_leach, asset.crushed.tonnes)
           + inputs.cost_leach * (min(inputs.rate_crushed_leach, asset.crushed.tonnes)
                                  + min(inputs.rate_rom_leach, asset.rom.tonnes))
}

stream mine.opex.selling on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.expense.opex
  amount = 2204.6 / 1000.0
           * (inputs.sell_cu * (inputs.rec_cu_mill * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) * (if(time.t <= 2, curve_value("grade_policy_cu", time.date),
                if(asset.cu_mill.tonnes > 0.0, asset.cu_mill.cu / asset.cu_mill.tonnes / 10.0, 0.0))) * 10.0 + min(inputs.cap_zn_mill, asset.zn_mill.tonnes) * (if(asset.zn_mill.tonnes > 0.0, asset.zn_mill.cu / asset.zn_mill.tonnes / 10.0, 0.0)) * 10.0)
                                + inputs.rec_cu_leach * (min(inputs.rate_crushed_leach, asset.crushed.tonnes) * (if(asset.crushed.tonnes > 0.0, asset.crushed.cu / asset.crushed.tonnes / 10.0, 0.0)) * 10.0 + min(inputs.rate_rom_leach, asset.rom.tonnes) * (if(asset.rom.tonnes > 0.0, asset.rom.cu / asset.rom.tonnes / 10.0, 0.0)) * 10.0))
              + inputs.sell_mo * inputs.rec_mo * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) * (if(asset.cu_mill.tonnes > 0.0, asset.cu_mill.mo / asset.cu_mill.tonnes / 10.0, 0.0)) * 10.0)
              + inputs.sell_zn * inputs.rec_zn * (min(inputs.cap_zn_mill, asset.zn_mill.tonnes) * (if(asset.zn_mill.tonnes > 0.0, asset.zn_mill.zn / asset.zn_mill.tonnes / 10.0, 0.0)) * 10.0))
}

stream mine.opex.gna on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.cost_gna * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes)
                              + min(inputs.cap_zn_mill, asset.zn_mill.tonnes))
}

stream mine.opex.accretion on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.accretion
}

// ---------------------------------------------------------------------------
// EBITDA to net income. Each charge reads the period's realized EBITDA.
// ---------------------------------------------------------------------------

stream mine.fiscal.duty on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.income_tax.paid
  amount = inputs.duty_rate * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t))
}

stream mine.fiscal.profit_share on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.income_tax.paid
  amount = inputs.ptu_rate
             * max(0.0, (1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t))
                        - inputs.capital_lom * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) + min(inputs.cap_zn_mill, asset.zn_mill.tonnes) + min(inputs.rate_crushed_leach, asset.crushed.tonnes) + min(inputs.rate_rom_leach, asset.rom.tonnes)) / inputs.reserve_ore_total)
}

stream mine.fiscal.income_tax on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.income_tax.paid
  amount = max(0.0,
               inputs.tax_rate
                 * ((1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t)) - inputs.capital_lom * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) + min(inputs.cap_zn_mill, asset.zn_mill.tonnes) + min(inputs.rate_crushed_leach, asset.crushed.tonnes) + min(inputs.rate_rom_leach, asset.rom.tonnes)) / inputs.reserve_ore_total
                    - inputs.ptu_rate * max(0.0, (1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t))
                                                 - inputs.capital_lom * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) + min(inputs.cap_zn_mill, asset.zn_mill.tonnes) + min(inputs.rate_crushed_leach, asset.crushed.tonnes) + min(inputs.rate_rom_leach, asset.rom.tonnes)) / inputs.reserve_ore_total))
               - inputs.tax_rate * inputs.duty_rate * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t)))
}

stream mine.noncash.accretion_addback on entity asset.cu_mill inflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.accretion
}

stream mine.capital.sustaining on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category investing.capital.capex
  amount = inputs.capital_lom * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) + min(inputs.cap_zn_mill, asset.zn_mill.tonnes) + min(inputs.rate_crushed_leach, asset.crushed.tonnes) + min(inputs.rate_rom_leach, asset.rom.tonnes)) / inputs.reserve_ore_total
}

stream mine.capital.closure on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2061-01 to 2065-01
  category investing.capital.capex
  amount = inputs.closure_total / 5.0
}
```

## bespoke/ppiaf_toll_highway

A 125 km toll highway concession from the World Bank's highway PPP toolkit, financed with three debt tranches and topped up each year by an availability subsidy sized to hold debt service cover at 1.30x.

| | |
|---|---|
| Pack | **none** — written from the bare language |
| Declared | five entities, nine declared fields, twenty-one native streams |
| Language features | declared state with `init`/`next`, cross-field `prev` reads, a state that snapshots and then holds, `min`/`max`/`pow` |
| Conventions | mid-year drawdown with capitalized interest, constant P+I annuities off three different grace periods, VAT stripped from an inclusive toll, tax in arrears with loss carryforward, a regressive cost scale, an ADSCR-targeted subsidy |

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

```cfdl
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
assume equity_pct          = 0.10      // of construction cost, excluding capitalized interest
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
// During construction the tranches DRAW and interest CAPITALIZES: the year's
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
// concessionaire costs, fees, and the interest capitalized into each tranche —
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
  category financing.equity.contribution
  amount = inputs.equity_pct * inputs.construction_real
             * if(time.t == 1, 0.1, if(time.t == 2, 0.3, if(time.t == 3, 0.5, 0.1)))
             * pow(1.0 + inputs.inflation, time.t)
}

// Each tranche's cash drawdown. Capitalized interest is not drawn cash — it is
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
  category operating.income_tax.paid
  amount = inputs.tax_rate
             * max(0.0, min(prev.asset.project.pbt, asset.project.cum_pbt))
}

// ---------------------------------------------------------------------------
// Operating period: the availability subsidy.
//
// This is the line the whole case exists for. The authority pays whatever it
// takes to hold cover at 1.30x, and nothing once the project covers itself —
// which is why 2013 is unsubsidized at 1.77x, and why the payments stop dead
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
  category financing.debt.interest_paid
  amount = prev.asset.tranche1.balance * inputs.rate_t1
}

stream infra.debt.interest_t2 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.debt.interest_paid
  amount = prev.asset.tranche2.balance * inputs.rate_t2
}

stream infra.debt.interest_t3 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.debt.interest_paid
  amount = prev.asset.tranche3.balance * inputs.rate_t3
}

stream infra.debt.principal_t1 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.debt.principal
  amount = prev.asset.tranche1.balance - asset.tranche1.balance
}

stream infra.debt.principal_t2 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.debt.principal
  amount = prev.asset.tranche2.balance - asset.tranche2.balance
}

stream infra.debt.principal_t3 on entity asset.concession outflow currency USD {
  schedule every year from 2013-01 to 2058-01
  category financing.debt.principal
  amount = prev.asset.tranche3.balance - asset.tranche3.balance
}
```

## cre/office_two_tenant

An institutional two-tenant office DCF: free rent, anniversary escalations, recoveries above expense stops, tenant improvements and leasing commissions, probability-blended rollover, and a forward-NOI exit over ten years.

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.lease_unit` (two instances), `cre.rollover`, `cre.vacancy_loss`, `cre.opex_line`, `cre.permanent_debt`, `cre.exit_forward` |
| Language features | multiple instances of one contract type, per-period subtotals |
| Conventions | free rent, anniversary escalation, recoveries above an expense stop, tenant improvements and leasing commissions, probability-blended rollover with downtime, a forward-NOI exit |

More of the CRE pack's contract surface than any other case.

```cfdl
version 0.1
model "office-two-tenant"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 120 project 12

entity asset tower : CRE.Asset.RealProperty

// Tenant A: 5-year lease, 3 months free, 3% anniversary escalations,
// recoveries above a full stop at 40% pro-rata, $200k TI/LC.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2030-12
  terms {
    rent_year = 480000
    free_rent_months = 3
    escalation = 0.03
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 300000
    pro_rata_share = 0.40
    ti_total = 120000
    lc_total = 80000
  }
}

// Tenant A rollover: window starts AT EXPIRY; during the 3 downtime months
// only the renewal scenario (70%) pays. Runs through the projection tail so
// exit valuation sees a full forward year.
contract cre.rollover.tenant_a on entity asset.tower {
  term 2031-01..2036-12
  terms {
    renewal_probability = 0.7
    renewal_rent_year = 520000
    market_rent_year = 560000
    market_escalation = 0.03
    downtime_months = 3
    renewal_ti_lc = 100000
    new_ti_lc = 350000
  }
}

// Tenant B: 7-year lease from mid-2026, 2.5% escalations, $180k stop at 30%.
contract cre.lease_unit.tenant_b on entity asset.tower {
  term 2026-07..2033-06
  terms {
    rent_year = 360000
    escalation = 0.025
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 180000
    pro_rata_share = 0.30
    ti_total = 100000
    lc_total = 50000
  }
}

contract cre.vacancy_loss on entity asset.tower {
  term 2026-01..2036-12
  terms {
    rate = 0.02
    potential_gross_year = 900000
  }
}

contract cre.opex_line on entity asset.tower {
  term 2026-01..2036-12
  terms {
    amount_year = 300000
    growth_rate = 0.025
  }
}

// Sale at the end of the hold; NOI for the valuation year is DERIVED from
// the modeled streams over the 12 projection months after the sale date.
contract cre.exit_forward on entity asset.tower {
  term 2035-12..2035-12
  terms {
    cap_rate = 0.065
    selling_costs = 0.02
  }
}

// Permanent debt: $6m at 5.50%, 25-year amortization, 10-year hold.
//
// Was a hand-written stream computing its own `pmt`. `cre.permanent_debt`
// states the loan instead of its arithmetic, and reproduces the same
// 4,421,429.94 of debt service — the payment is identical because with one
// combined stream principal is the plug.
//
// The balloon stays off (the default): the unamortized $4.5m is repaid out of
// the sale, not as debt service, and folding it into the final period would
// make that period's DSCR meaningless.
// `funded_at_close = 0`: the reference model's cash flow starts
// post-financing — it nets rents against debt service and never books the
// draw — so the proceeds the contract funds by default are excluded here to
// state what the source states.
contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2035-12
  terms {
    principal = 6000000
    interest_rate = 0.055
    amortization_months = 300
    funded_at_close = 0
  }
}

// Views by master type: the leases, the debt and the disposal, whichever
// pack contract each was lowered from. A slice is a view and moves no number.
slice leases {
  type Contract.Lease
}

slice debt_service {
  type Contract.Debt
}

slice debt_interest {
  type Contract.Debt
  line interest
}

slice disposal {
  type Contract.Sale
}
```

## cre/retail_strip

A retail strip center with base-year expense gross-ups, percentage rent over a breakpoint, and staggered tenant rollover across a ten-year hold.

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.lease_unit` (two instances), `cre.percentage_rent`, `cre.vacancy_loss`, `cre.opex_line`, `cre.exit` |
| Language features | multiple instances of one contract type |
| Conventions | a base-year expense stop with a 95% gross-up, percentage rent over a breakpoint, net leases, staggered rollover |

A gross-up implemented as a flat recovery understates income in every year the
center sits below the gross-up threshold.

```cfdl
version 0.1
model "retail-strip"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 84

entity asset strip_center : CRE.Asset.RealProperty

// Anchor grocer: base-year stop (stop = year-0 grossed-up opex), 95% gross-up,
// 60% pro-rata, plus percentage rent above a $12M breakpoint.
contract cre.lease_unit.anchor on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    rent_year = 540000
    escalation = 0.02
    opex_year = 240000
    opex_escalation = 0.03
    expense_stop_year = 228000
    gross_up_factor = 0.95
    pro_rata_share = 0.60
    ti_total = 150000
    lc_total = 60000
  }
}

contract cre.percentage_rent.anchor on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    sales_year = 11500000
    sales_growth = 0.03
    breakpoint_year = 12000000
    overage_pct = 0.02
  }
}

// Inline shops: single lease with net recoveries (no stop), 30% share.
contract cre.lease_unit.shops on entity asset.strip_center {
  term 2026-07..2031-06
  terms {
    rent_year = 288000
    free_rent_months = 2
    escalation = 0.025
    opex_year = 240000
    opex_escalation = 0.03
    gross_up_factor = 0.95
    pro_rata_share = 0.30
    ti_total = 90000
    lc_total = 40000
  }
}

contract cre.vacancy_loss on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    rate = 0.03
    potential_gross_year = 850000
  }
}

contract cre.opex_line on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    amount_year = 240000
    growth_rate = 0.03
  }
}

contract cre.exit on entity asset.strip_center {
  term 2032-12..2032-12
  terms {
    income = 640000
    cap_rate = 0.0675
    selling_costs = 0.015
  }
}
```

## cre/one_lincoln_street_contract

The same published construction schedule as the native case, declared as one cre.construction_loan contract — equity first, the facility behind it, interest on the drawn balance.

| | |
|---|---|
| Pack | `cre` |
| Declared | one curve, one contract |
| Contracts | `cre.construction_loan` |
| Language features | a pack rule declaring a field, reading a model curve by name, and re-deriving an opening balance rather than carrying one |
| Conventions | equity-first funding, a facility drawing only once the commitment depletes, interest on the drawn balance with ratable draw timing |

The draw schedule stays a `curve` in the model rather than becoming a term. A
development's funding profile is per-deal data — sixteen published quarters
here, an S-curve or a contractor's schedule on the next deal — and all three are
the same object. What the contract adds is the funding convention.

The curve is stated ANNUALIZED — each point is the exhibit's quarterly figure
times four — and the contract divides by periods-per-year, the same convention
every annual quantity in the pack follows. On this quarterly model that divides
straight back to the published number. It matters because a curve is a level: a
step curve returns its last point on every date, so a schedule stated as
per-period totals would be correct here and would fund three times the money if
the same deal were run monthly.

```cfdl
// One Lincoln Street, Boston — the same construction period schedule as the
// primitive-built case of the same name, expressed as a PACK CONTRACT.
//
// The two cases are a matched pair and the point is that they agree. The other
// one builds the funding waterfall from primitives — a curve, a field and three
// native streams — and proves the LANGUAGE can express a depleting equity
// commitment with no domain vocabulary at all. This one declares
// `cre.construction_loan` and proves the PACK lowers to exactly that, against
// the same published exhibit and the same numbers.
//
// That ordering matters. A contract that reproduces what the language already
// validated is a convenience for whoever writes the next development model; a
// contract validated only against itself would be the pack marking its own
// homework. Neither case replaces the other, and if they ever disagree the
// contract is wrong.
//
// THE DRAW SCHEDULE STAYS A CURVE. It is per-deal data — sixteen published
// quarters here — so the contract names it rather than parameterizing its
// shape. Everything the contract adds is the funding CONVENTION: equity first,
// the facility behind it, interest on the drawn balance.

version 0.1
model "one-lincoln-street-contract"
use pack "cre" version "0.1.0"
time calendar quarterly from 2000-01 for 16

entity asset tower : CRE.Asset.RealProperty

// Exhibit 6's quarterly funding requirement, totalling $285,145,000 — stated
// as an ANNUALIZED rate, which is how a CRE contract reads a curve. Each point
// is the exhibit's quarterly figure x 4, so on this quarterly model the
// contract divides straight back to the published number. Stating per-period
// totals instead would be correct here and wrong the moment the same schedule
// were run monthly, because a curve is a level: it returns its last point on
// every date, so a quarterly figure would repeat three times a quarter.
curve required_funding step {
  2000-01: 19932000
  2000-04: 37116000
  2000-07: 33612000
  2000-10: 56460000
  2001-01: 76088000
  2001-04: 84836000
  2001-07: 92820000
  2001-10: 117720000
  2002-01: 88684000
  2002-04: 88584000
  2002-07: 67372000
  2002-10: 48476000
  2003-01: 154876000
  2003-04: 60340000
  2003-07: 39880000
  2003-10: 73784000
}

// Exhibit 7's three stated drivers, and nothing else. The equity/debt split,
// the opening balances and the interest are all derived by the pack.
contract cre.construction_loan on entity asset.tower {
  term 2000-01..2003-10
  terms {
    draw_curve        = "required_funding"
    equity_commitment = 110738000
    interest_rate              = 0.08
    // "Funding is assumed to occur ratably throughout the quarter", so a
    // quarter's own draw earns half a quarter of interest.
    draw_accrual_fraction = 0.5
  }
}
```

## cre/penzance_highlands

A 160-month ground-up CRE development: land in 2011, a 39-month build on a parabolic draw curve, a $380M facility that funds equity first and capitalizes interest, two rental towers sold in lease-up, and a 34-month condominium sellout.

| | |
|---|---|
| Pack | `cre` |
| Declared | 4 curves, 8 entities, 28 streams, 3 accounts, 1 waterfall, 7 tiers, 4 metrics, 8 field recurrences |
| What the deal requires | carrying a balance across periods, a recorded schedule of closings, an ordered priority of payments, cash that accumulates between distributions, a return measured per partner, where in the period the cash falls |
| Conventions | equity-first funding, capitalized construction interest, a facility retired out of disposal proceeds, sale in lease-up |

The facility carries five balances — equity funded, interest, draw, repayment
and the outstanding balance — each advancing from the month before it, so every
month resolves from one already settled. It is built directly rather than from
the pack's construction loan, and the behavior is the same.

Every cost schedule states a value for **every** month, including the quiet
ones, so a run of zeros is declared rather than inferred.

```cfdl
version 0.1
model "penzance-highlands" currency USD
use pack "cre" version "0.1.0"
time calendar monthly from 2011-09 for 160

// ===========================================================================
// THE HIGHLANDS — Rosslyn, Virginia.  Penzance / The Baupost Group.
// Arlington County Site Plan SP #445.  Land 2011-09-30, delivered 2021,
// both rental towers sold to Cortland 2022-05-17, condo sellout to 2024-06.
//
// Two towers over one shared podium and one construction facility:
//   east  = Pierce (104 for-sale condos) + Evo (455 rental)
//   west  = Aubrey (331 rental)
// The for-sale and rental product share a basis. Their costs are therefore not
// separable, which is the modeling problem this case addresses.
//
// Program, obligations, land basis, both sale prices and the whole condo
// sellout are recorded fact — Arlington's site-plan record, deed register,
// assessment roll and Commercial Guidebook. Cost, debt pricing and the JV
// split are stated assumptions. Provenance is in CASE.md.
// ===========================================================================

phase predevelopment from 2011-09 to 2018-03
phase construction   from 2018-04 to 2021-06
phase lease_up       from 2021-07 to 2022-05
phase condo_tail     from 2022-06 to 2024-12

assume loan_rate        = 0.065
assume loan_commitment  = 380000000
assume equity_commitment = 186245280.585
assume condo_selling_cost = 0.05
assume cost_of_sale     = 0.01
assume pref_rate        = 0.08
assume sponsor_share    = 0.10

// Development cost per period and its running total. Declared as curves so
// the facility's recurrence can read them. Cost here is a pure function of
// time.
//
// EVERY period is declared, including the zeros. A step curve is
// flat-forward (docs/03 §4): omit the quiet months and the last construction
// draw is held forward indefinitely and the balance compounds without end.
curve dev_cost {
  2011-09: 67000000.0000
  2011-10: 0.0000
  2011-11: 0.0000
  2011-12: 0.0000
  2012-01: 0.0000
  2012-02: 0.0000
  2012-03: 0.0000
  2012-04: 0.0000
  2012-05: 0.0000
  2012-06: 0.0000
  2012-07: 0.0000
  2012-08: 0.0000
  2012-09: 0.0000
  2012-10: 0.0000
  2012-11: 0.0000
  2012-12: 0.0000
  2013-01: 0.0000
  2013-02: 0.0000
  2013-03: 0.0000
  2013-04: 0.0000
  2013-05: 0.0000
  2013-06: 0.0000
  2013-07: 0.0000
  2013-08: 0.0000
  2013-09: 0.0000
  2013-10: 0.0000
  2013-11: 0.0000
  2013-12: 0.0000
  2014-01: 0.0000
  2014-02: 0.0000
  2014-03: 0.0000
  2014-04: 0.0000
  2014-05: 0.0000
  2014-06: 0.0000
  2014-07: 0.0000
  2014-08: 0.0000
  2014-09: 0.0000
  2014-10: 0.0000
  2014-11: 0.0000
  2014-12: 0.0000
  2015-01: 0.0000
  2015-02: 0.0000
  2015-03: 0.0000
  2015-04: 0.0000
  2015-05: 0.0000
  2015-06: 0.0000
  2015-07: 0.0000
  2015-08: 0.0000
  2015-09: 0.0000
  2015-10: 0.0000
  2015-11: 0.0000
  2015-12: 0.0000
  2016-01: 0.0000
  2016-02: 0.0000
  2016-03: 0.0000
  2016-04: 0.0000
  2016-05: 0.0000
  2016-06: 0.0000
  2016-07: 0.0000
  2016-08: 0.0000
  2016-09: 0.0000
  2016-10: 0.0000
  2016-11: 0.0000
  2016-12: 0.0000
  2017-01: 0.0000
  2017-02: 0.0000
  2017-03: 0.0000
  2017-04: 0.0000
  2017-05: 0.0000
  2017-06: 0.0000
  2017-07: 0.0000
  2017-08: 0.0000
  2017-09: 0.0000
  2017-10: 0.0000
  2017-11: 0.0000
  2017-12: 0.0000
  2018-01: 0.0000
  2018-02: 0.0000
  2018-03: 0.0000
  2018-04: 1813594.1101
  2018-05: 3227644.2146
  2018-06: 4714059.3134
  2018-07: 6115536.4066
  2018-08: 7432075.4941
  2018-09: 8663676.5760
  2018-10: 9810339.6523
  2018-11: 10872064.7229
  2018-12: 11848851.7878
  2019-01: 12740700.8471
  2019-02: 13547611.9007
  2019-03: 14269584.9487
  2019-04: 14906619.9911
  2019-05: 15458717.0278
  2019-06: 15925876.0589
  2019-07: 16308097.0843
  2019-08: 16605380.1040
  2019-09: 16817725.1182
  2019-10: 16945132.1266
  2019-11: 16987601.1295
  2019-12: 16945132.1266
  2020-01: 16817725.1182
  2020-02: 16605380.1040
  2020-03: 16308097.0843
  2020-04: 15925876.0589
  2020-05: 15458717.0278
  2020-06: 14906619.9911
  2020-07: 14269584.9487
  2020-08: 13547611.9007
  2020-09: 12740700.8471
  2020-10: 11848851.7878
  2020-11: 10872064.7229
  2020-12: 9810339.6523
  2021-01: 8663676.5760
  2021-02: 7432075.4941
  2021-03: 6115536.4066
  2021-04: 4714059.3134
  2021-05: 3227644.2146
  2021-06: 1656291.1101
  2021-07: 0.0000
  2021-08: 0.0000
  2021-09: 12252500.0000
  2021-10: 0.0000
  2021-11: 0.0000
  2021-12: 0.0000
  2022-01: 0.0000
  2022-02: 0.0000
  2022-03: 0.0000
  2022-04: 0.0000
  2022-05: 0.0000
  2022-06: 0.0000
  2022-07: 0.0000
  2022-08: 0.0000
  2022-09: 0.0000
  2022-10: 0.0000
  2022-11: 0.0000
  2022-12: 0.0000
  2023-01: 0.0000
  2023-02: 0.0000
  2023-03: 0.0000
  2023-04: 0.0000
  2023-05: 0.0000
  2023-06: 0.0000
  2023-07: 0.0000
  2023-08: 0.0000
  2023-09: 0.0000
  2023-10: 0.0000
  2023-11: 0.0000
  2023-12: 0.0000
  2024-01: 0.0000
  2024-02: 0.0000
  2024-03: 0.0000
  2024-04: 0.0000
  2024-05: 0.0000
  2024-06: 0.0000
  2024-07: 0.0000
  2024-08: 0.0000
  2024-09: 0.0000
  2024-10: 0.0000
  2024-11: 0.0000
  2024-12: 0.0000
}

curve dev_cost_cum {
  2011-09: 67000000.0000
  2011-10: 67000000.0000
  2011-11: 67000000.0000
  2011-12: 67000000.0000
  2012-01: 67000000.0000
  2012-02: 67000000.0000
  2012-03: 67000000.0000
  2012-04: 67000000.0000
  2012-05: 67000000.0000
  2012-06: 67000000.0000
  2012-07: 67000000.0000
  2012-08: 67000000.0000
  2012-09: 67000000.0000
  2012-10: 67000000.0000
  2012-11: 67000000.0000
  2012-12: 67000000.0000
  2013-01: 67000000.0000
  2013-02: 67000000.0000
  2013-03: 67000000.0000
  2013-04: 67000000.0000
  2013-05: 67000000.0000
  2013-06: 67000000.0000
  2013-07: 67000000.0000
  2013-08: 67000000.0000
  2013-09: 67000000.0000
  2013-10: 67000000.0000
  2013-11: 67000000.0000
  2013-12: 67000000.0000
  2014-01: 67000000.0000
  2014-02: 67000000.0000
  2014-03: 67000000.0000
  2014-04: 67000000.0000
  2014-05: 67000000.0000
  2014-06: 67000000.0000
  2014-07: 67000000.0000
  2014-08: 67000000.0000
  2014-09: 67000000.0000
  2014-10: 67000000.0000
  2014-11: 67000000.0000
  2014-12: 67000000.0000
  2015-01: 67000000.0000
  2015-02: 67000000.0000
  2015-03: 67000000.0000
  2015-04: 67000000.0000
  2015-05: 67000000.0000
  2015-06: 67000000.0000
  2015-07: 67000000.0000
  2015-08: 67000000.0000
  2015-09: 67000000.0000
  2015-10: 67000000.0000
  2015-11: 67000000.0000
  2015-12: 67000000.0000
  2016-01: 67000000.0000
  2016-02: 67000000.0000
  2016-03: 67000000.0000
  2016-04: 67000000.0000
  2016-05: 67000000.0000
  2016-06: 67000000.0000
  2016-07: 67000000.0000
  2016-08: 67000000.0000
  2016-09: 67000000.0000
  2016-10: 67000000.0000
  2016-11: 67000000.0000
  2016-12: 67000000.0000
  2017-01: 67000000.0000
  2017-02: 67000000.0000
  2017-03: 67000000.0000
  2017-04: 67000000.0000
  2017-05: 67000000.0000
  2017-06: 67000000.0000
  2017-07: 67000000.0000
  2017-08: 67000000.0000
  2017-09: 67000000.0000
  2017-10: 67000000.0000
  2017-11: 67000000.0000
  2017-12: 67000000.0000
  2018-01: 67000000.0000
  2018-02: 67000000.0000
  2018-03: 67000000.0000
  2018-04: 68813594.1101
  2018-05: 72041238.3247
  2018-06: 76755297.6381
  2018-07: 82870834.0447
  2018-08: 90302909.5389
  2018-09: 98966586.1149
  2018-10: 108776925.7672
  2018-11: 119648990.4900
  2018-12: 131497842.2778
  2019-01: 144238543.1249
  2019-02: 157786155.0256
  2019-03: 172055739.9744
  2019-04: 186962359.9655
  2019-05: 202421076.9933
  2019-06: 218346953.0522
  2019-07: 234655050.1364
  2019-08: 251260430.2405
  2019-09: 268078155.3586
  2019-10: 285023287.4853
  2019-11: 302010888.6147
  2019-12: 318956020.7414
  2020-01: 335773745.8595
  2020-02: 352379125.9636
  2020-03: 368687223.0478
  2020-04: 384613099.1067
  2020-05: 400071816.1345
  2020-06: 414978436.1256
  2020-07: 429248021.0744
  2020-08: 442795632.9751
  2020-09: 455536333.8222
  2020-10: 467385185.6100
  2020-11: 478257250.3328
  2020-12: 488067589.9851
  2021-01: 496731266.5611
  2021-02: 504163342.0553
  2021-03: 510278878.4619
  2021-04: 514992937.7753
  2021-05: 518220581.9899
  2021-06: 519876873.1000
  2021-07: 519876873.1000
  2021-08: 519876873.1000
  2021-09: 532129373.1000
  2021-10: 532129373.1000
  2021-11: 532129373.1000
  2021-12: 532129373.1000
  2022-01: 532129373.1000
  2022-02: 532129373.1000
  2022-03: 532129373.1000
  2022-04: 532129373.1000
  2022-05: 532129373.1000
  2022-06: 532129373.1000
  2022-07: 532129373.1000
  2022-08: 532129373.1000
  2022-09: 532129373.1000
  2022-10: 532129373.1000
  2022-11: 532129373.1000
  2022-12: 532129373.1000
  2023-01: 532129373.1000
  2023-02: 532129373.1000
  2023-03: 532129373.1000
  2023-04: 532129373.1000
  2023-05: 532129373.1000
  2023-06: 532129373.1000
  2023-07: 532129373.1000
  2023-08: 532129373.1000
  2023-09: 532129373.1000
  2023-10: 532129373.1000
  2023-11: 532129373.1000
  2023-12: 532129373.1000
  2024-01: 532129373.1000
  2024-02: 532129373.1000
  2024-03: 532129373.1000
  2024-04: 532129373.1000
  2024-05: 532129373.1000
  2024-06: 532129373.1000
  2024-07: 532129373.1000
  2024-08: 532129373.1000
  2024-09: 532129373.1000
  2024-10: 532129373.1000
  2024-11: 532129373.1000
  2024-12: 532129373.1000
}

// Pierce condo closings — every one of 102 recorded sales, by month.
// 34 months, not the smooth absorption an assumption would give.
curve pierce_sellout {
  2021-08: 2056400.00
  2021-09: 8109000.00
  2021-10: 6243000.00
  2021-11: 16820000.00
  2021-12: 22430000.00
  2022-01: 23646000.00
  2022-02: 11617000.00
  2022-03: 4729000.00
  2022-04: 7200000.00
  2022-05: 2750000.00
  2022-06: 2407000.00
  2022-07: 5735000.00
  2022-08: 3673000.00
  2022-09: 10380320.00
  2022-10: 1500000.00
  2022-11: 1813000.00
  2022-12: 6830500.00
  2023-01: 2496000.00
  2023-02: 0.00
  2023-03: 2845000.00
  2023-04: 12874000.00
  2023-05: 2357000.00
  2023-06: 4589000.00
  2023-07: 0.00
  2023-08: 2320000.00
  2023-09: 6060000.00
  2023-10: 3250000.00
  2023-11: 2160000.00
  2023-12: 3548000.00
  2024-01: 0.00
  2024-02: 1823000.00
  2024-03: 0.00
  2024-04: 0.00
  2024-05: 3950000.00
  2024-06: 3975000.00
}


// ---------------------------------------------------------------- structure
entity container project : CRE.Container.Portfolio
entity asset east : CRE.Asset.RealProperty { asset_class = "mixed_use"  part of container.project }
entity asset west : CRE.Asset.RealProperty { asset_class = "multifamily"  part of container.project }

entity party penzance : CRE.Party.Sponsor  { name = "Penzance" }
entity party baupost  : CRE.Party.Investor { name = "The Baupost Group" }
entity party mack     : CRE.Party.Lender   { name = "Mack Real Estate Credit Strategies" }

// ------------------------------------------------------------------ capital
stream cre.land on entity container.project outflow currency USD {
  schedule on 2011-09
  category investing.capital.capex
  amount = 67000000.00
}


stream cre.hard_buildings on entity container.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 301693050.00 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.hard_garage on entity container.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 33184000.00 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.fire_station on entity container.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 7454800.00 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.park_and_street on entity container.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 14000000.00 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.soft_costs on entity container.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 60576414.50 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.contingency on entity container.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 17816592.50 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.developer_fee on entity container.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 17994713.10 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}


// SP #445 conditions: utility undergrounding and TDM at permit; public art,
// green building and affordable housing at certificate of occupancy.
stream cre.obligations_permit on entity container.project outflow currency USD {
  schedule on 2018-04
  category investing.capital.construction
  amount = 157303.00
}

stream cre.obligations_co on entity container.project outflow currency USD {
  schedule on 2021-09
  category investing.capital.construction
  amount = 12252500.00
}


// ---- aubrey: delivered 2021-07, sold 2022-05 — still in lease-up
stream cre.aubrey_rent on entity asset.west inflow currency USD {
  schedule every month start from 2021-07 to 2022-05
  category operating.revenue.base_rent
  amount = min(331.0, max(0.0, (time.t - 117.0) * 25.0)) * 3369.8496
}

stream cre.aubrey_retail on entity asset.west inflow currency USD {
  schedule every month start from 2021-07 to 2022-05
  category operating.revenue.other
  amount = 35236.8
}

stream cre.aubrey_parking on entity asset.west inflow currency USD {
  schedule every month start from 2021-07 to 2022-05
  category operating.revenue.other
  amount = 39720
}

stream cre.aubrey_opex on entity asset.west outflow currency USD {
  schedule every month start from 2021-07 to 2022-05
  category operating.expense.opex
  amount = 207178.4167
}

stream cre.aubrey_tax on entity asset.west outflow currency USD {
  schedule every month start from 2021-07 to 2022-05
  category operating.expense.opex
  amount = 215699.0142
}

// ---- evo: delivered 2021-09, sold 2022-05 — still in lease-up
stream cre.evo_rent on entity asset.east inflow currency USD {
  schedule every month start from 2021-09 to 2022-05
  category operating.revenue.base_rent
  amount = min(455.0, max(0.0, (time.t - 119.0) * 25.0)) * 3131.6064
}

stream cre.evo_retail on entity asset.east inflow currency USD {
  schedule every month start from 2021-09 to 2022-05
  category operating.revenue.other
  amount = 50007.6
}

stream cre.evo_parking on entity asset.east inflow currency USD {
  schedule every month start from 2021-09 to 2022-05
  category operating.revenue.other
  amount = 54600
}

stream cre.evo_opex on entity asset.east outflow currency USD {
  schedule every month start from 2021-09 to 2022-05
  category operating.expense.opex
  amount = 284792.0833
}

stream cre.evo_tax on entity asset.east outflow currency USD {
  schedule every month start from 2021-09 to 2022-05
  category operating.expense.opex
  amount = 273135.4075
}


// ---------------------------------------------------------------------- exit
// Both towers sold 2022-05-17 as ONE transaction; only the deed recording
// dates differ (5/18 leasehold, 7/12 fee), which is why the press reported two.
stream cre.aubrey_sale on entity asset.west inflow currency USD {
  schedule on 2022-05
  category investing.disposal.reversion
  amount = 266455000.00
}

stream cre.evo_sale on entity asset.east inflow currency USD {
  schedule on 2022-05
  category investing.disposal.reversion
  amount = 334642240.00
}

stream cre.sale_costs on entity container.project outflow currency USD {
  schedule on 2022-05
  category investing.disposal.selling_costs
  amount = 601097240.00 * inputs.cost_of_sale
}

stream cre.pierce_closings on entity asset.east inflow currency USD {
  schedule every month start from 2021-08 to 2024-06
  category investing.disposal.reversion
  amount = curve_value("pierce_sellout", time.date) * (1.0 - inputs.condo_selling_cost)
}

// Cash available to repay the facility: condo closings net of selling costs,
// plus the two tower sales net of cost of sale. Every period is declared, so
// a quiet month reads zero.
curve loan_proceeds {
  2011-09: 0.0000
  2011-10: 0.0000
  2011-11: 0.0000
  2011-12: 0.0000
  2012-01: 0.0000
  2012-02: 0.0000
  2012-03: 0.0000
  2012-04: 0.0000
  2012-05: 0.0000
  2012-06: 0.0000
  2012-07: 0.0000
  2012-08: 0.0000
  2012-09: 0.0000
  2012-10: 0.0000
  2012-11: 0.0000
  2012-12: 0.0000
  2013-01: 0.0000
  2013-02: 0.0000
  2013-03: 0.0000
  2013-04: 0.0000
  2013-05: 0.0000
  2013-06: 0.0000
  2013-07: 0.0000
  2013-08: 0.0000
  2013-09: 0.0000
  2013-10: 0.0000
  2013-11: 0.0000
  2013-12: 0.0000
  2014-01: 0.0000
  2014-02: 0.0000
  2014-03: 0.0000
  2014-04: 0.0000
  2014-05: 0.0000
  2014-06: 0.0000
  2014-07: 0.0000
  2014-08: 0.0000
  2014-09: 0.0000
  2014-10: 0.0000
  2014-11: 0.0000
  2014-12: 0.0000
  2015-01: 0.0000
  2015-02: 0.0000
  2015-03: 0.0000
  2015-04: 0.0000
  2015-05: 0.0000
  2015-06: 0.0000
  2015-07: 0.0000
  2015-08: 0.0000
  2015-09: 0.0000
  2015-10: 0.0000
  2015-11: 0.0000
  2015-12: 0.0000
  2016-01: 0.0000
  2016-02: 0.0000
  2016-03: 0.0000
  2016-04: 0.0000
  2016-05: 0.0000
  2016-06: 0.0000
  2016-07: 0.0000
  2016-08: 0.0000
  2016-09: 0.0000
  2016-10: 0.0000
  2016-11: 0.0000
  2016-12: 0.0000
  2017-01: 0.0000
  2017-02: 0.0000
  2017-03: 0.0000
  2017-04: 0.0000
  2017-05: 0.0000
  2017-06: 0.0000
  2017-07: 0.0000
  2017-08: 0.0000
  2017-09: 0.0000
  2017-10: 0.0000
  2017-11: 0.0000
  2017-12: 0.0000
  2018-01: 0.0000
  2018-02: 0.0000
  2018-03: 0.0000
  2018-04: 0.0000
  2018-05: 0.0000
  2018-06: 0.0000
  2018-07: 0.0000
  2018-08: 0.0000
  2018-09: 0.0000
  2018-10: 0.0000
  2018-11: 0.0000
  2018-12: 0.0000
  2019-01: 0.0000
  2019-02: 0.0000
  2019-03: 0.0000
  2019-04: 0.0000
  2019-05: 0.0000
  2019-06: 0.0000
  2019-07: 0.0000
  2019-08: 0.0000
  2019-09: 0.0000
  2019-10: 0.0000
  2019-11: 0.0000
  2019-12: 0.0000
  2020-01: 0.0000
  2020-02: 0.0000
  2020-03: 0.0000
  2020-04: 0.0000
  2020-05: 0.0000
  2020-06: 0.0000
  2020-07: 0.0000
  2020-08: 0.0000
  2020-09: 0.0000
  2020-10: 0.0000
  2020-11: 0.0000
  2020-12: 0.0000
  2021-01: 0.0000
  2021-02: 0.0000
  2021-03: 0.0000
  2021-04: 0.0000
  2021-05: 0.0000
  2021-06: 0.0000
  2021-07: 0.0000
  2021-08: 1953580.0000
  2021-09: 7703550.0000
  2021-10: 5930850.0000
  2021-11: 15979000.0000
  2021-12: 21308500.0000
  2022-01: 22463700.0000
  2022-02: 11036150.0000
  2022-03: 4492550.0000
  2022-04: 6840000.0000
  2022-05: 597698767.6000
  2022-06: 2286650.0000
  2022-07: 5448250.0000
  2022-08: 3489350.0000
  2022-09: 9861304.0000
  2022-10: 1425000.0000
  2022-11: 1722350.0000
  2022-12: 6488975.0000
  2023-01: 2371200.0000
  2023-02: 0.0000
  2023-03: 2702750.0000
  2023-04: 12230300.0000
  2023-05: 2239150.0000
  2023-06: 4359550.0000
  2023-07: 0.0000
  2023-08: 2204000.0000
  2023-09: 5757000.0000
  2023-10: 3087500.0000
  2023-11: 2052000.0000
  2023-12: 3370600.0000
  2024-01: 0.0000
  2024-02: 1731850.0000
  2024-03: 0.0000
  2024-04: 0.0000
  2024-05: 3752500.0000
  2024-06: 3776250.0000
  2024-07: 0.0000
  2024-08: 0.0000
  2024-09: 0.0000
  2024-10: 0.0000
  2024-11: 0.0000
  2024-12: 0.0000
}


// ------------------------------------------------------------- the facility
// The $380M construction facility (Mack Real Estate Credit Strategies).
//
// Equity funds to its commitment first, the loan draws the residual, interest
// capitalizes into the balance, and sale and condo proceeds repay it. Four
// recurrences, each a fact about the facility, each reading only values that
// are already finished -- `prev` and the cost curves -- which is what keeps
// the whole thing acyclic by construction.
entity asset facility : Asset.Financial {
  equity_funded init min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))
                next min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))

  interest init 0.0
           next prev.asset.facility.balance * inputs.loan_rate / 12.0

  draw init max(0.0, curve_value("dev_cost", time.date) - inputs.equity_commitment)
       next min(max(0.0, curve_value("dev_cost", time.date) - (min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date)) - min(inputs.equity_commitment, curve_value("dev_cost_cum", edate(time.date, -1))))), max(0.0, inputs.loan_commitment - prev.asset.facility.balance - prev.asset.facility.balance * inputs.loan_rate / 12.0))

  repay init 0.0
        next min(prev.asset.facility.balance + prev.asset.facility.balance * inputs.loan_rate / 12.0 + min(max(0.0, curve_value("dev_cost", time.date) - (min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date)) - min(inputs.equity_commitment, curve_value("dev_cost_cum", edate(time.date, -1))))), max(0.0, inputs.loan_commitment - prev.asset.facility.balance - prev.asset.facility.balance * inputs.loan_rate / 12.0)), max(0.0, curve_value("loan_proceeds", time.date)))

  balance init max(0.0, curve_value("dev_cost", time.date) - inputs.equity_commitment)
          next max(0.0, prev.asset.facility.balance + min(max(0.0, curve_value("dev_cost", time.date) - (min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date)) - min(inputs.equity_commitment, curve_value("dev_cost_cum", edate(time.date, -1))))), max(0.0, inputs.loan_commitment - prev.asset.facility.balance - prev.asset.facility.balance * inputs.loan_rate / 12.0)) + prev.asset.facility.balance * inputs.loan_rate / 12.0 - min(prev.asset.facility.balance + prev.asset.facility.balance * inputs.loan_rate / 12.0 + min(max(0.0, curve_value("dev_cost", time.date) - (min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date)) - min(inputs.equity_commitment, curve_value("dev_cost_cum", edate(time.date, -1))))), max(0.0, inputs.loan_commitment - prev.asset.facility.balance - prev.asset.facility.balance * inputs.loan_rate / 12.0)), max(0.0, curve_value("loan_proceeds", time.date))))
}

// Interest is capitalized: it is repaid inside the principal repayment. That is the convention the reference workbook uses.
stream cre.loan_draw on entity container.project inflow currency USD {
  schedule every month start from 2011-09 to 2024-12
  category financing.debt.proceeds
  amount = asset.facility.draw
}

// Interest capitalizes: the facility funds its own accrual, so the two legs
// net to zero in cash while the balance grows. Stated GROSS rather than folded
// into the balance silently, so `domain.cre.debt_service` sees real interest
// and coverage during the build is measurable instead of absent.
stream cre.loan_interest on entity container.project outflow currency USD {
  schedule every month start from 2011-09 to 2024-12
  category financing.debt.interest_paid
  amount = asset.facility.interest
}

stream cre.loan_interest_funding on entity container.project inflow currency USD {
  schedule every month start from 2011-09 to 2024-12
  category financing.debt.proceeds
  amount = asset.facility.interest
}

// The payoff sits in the reversion. `financing.debt.principal` folds into
// `domain.cre.debt_service`, and a balance retired out of sale proceeds is not
// debt service — it would make every coverage ratio in the disposal period
// meaningless. The cre pack says the same of a permanent loan's balloon.
stream cre.loan_repayment on entity container.project outflow currency USD {
  schedule every month start from 2011-09 to 2024-12
  category investing.disposal.reversion
  amount = asset.facility.repay
}

// ------------------------------------------------------------ the JV capital
// Cash accrues to the venture and is split once, when the last unit closes.
// The preference and the capital are therefore CUMULATIVE balances, carried
// forward rather than re-derived at the distribution --
// 17_ordered_waterfall.md section 10.
//
// Both partners fund pro rata and nothing is returned before the split, so the
// two balances only grow. Their difference is the accrued preference.
//
// The preference accrues from CONSTRUCTION START, not from the 2011 land
// purchase: the venture is formed to build, and the land it is capitalized
// with earns nothing for the seven years before there is anything to build.
// Compounding that $67M from 2011 instead consumes the entire promote, which
// is how the assumption was identified.
entity asset jv : Asset.Financial {
  // The facility's equity funding one period back, so a month's contribution
  // can be differenced without reaching two periods behind.
  funded_prev init 0.0
              next prev.asset.facility.equity_funded

  capital init 0.0
          next prev.asset.jv.capital
             + (prev.asset.facility.equity_funded - prev.asset.jv.funded_prev)

  unreturned init 0.0
             next prev.asset.jv.unreturned * (1.0 + if(time.t >= 79, inputs.pref_rate / 12.0, 0.0))
                + (prev.asset.facility.equity_funded - prev.asset.jv.funded_prev)
}

// ------------------------------------------------------- the venture's cash
//
// A development JV does not distribute while the deal is live, so cash
// accumulates from inception and is allocated once, at the final closing. What
// accumulates is the venture's whole cash position: the equity the partners
// contributed, plus everything the deal earned on it, less every cost.
account deal_cash {
  from series_sum("cre.*", time.t, time.t)
     + (min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))
                - if(time.t == 0, 0.0,
                     min(inputs.equity_commitment,
                         curve_value("dev_cost_cum", edate(time.date, -1)))))
}

// WHAT EACH PARTNER PUT IN, so that what each partner got back can be measured
// against it. The venture funds pro rata -- 90% Baupost, 10% Penzance, the
// same share the tiers split on. Each partner's balance carries its capital out
// on the dates the facility draws it, and its distributions back in when the
// venture allocates.
account baupost_capital {
  owner party.baupost
  from 0.0 - (min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))
                - if(time.t == 0, 0.0,
                     min(inputs.equity_commitment,
                         curve_value("dev_cost_cum", edate(time.date, -1)))))
             * (1.0 - inputs.sponsor_share)
}

account penzance_capital {
  owner party.penzance
  from 0.0 - (min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))
                - if(time.t == 0, 0.0,
                     min(inputs.equity_commitment,
                         curve_value("dev_cost_cum", edate(time.date, -1)))))
             * inputs.sponsor_share
}

// -------------------------------------------------------------- the JV split
// Penzance / Baupost terms are not public; these tiers are stated assumptions.
waterfall jv.distribution on entity container.project {
  schedule on 2024-06 end
  from deal_cash

  // Capital back first: each partner is repaid the capital it has not yet had
  // returned, which is what its own balance carries. The preference is tracked
  // separately, because it compounds.
  pay capital_inv   to party.baupost  = min(0.0 - prev.baupost_capital, remaining)
  pay capital_sp    to party.penzance = min(0.0 - prev.penzance_capital, remaining)

  pay preferred_inv to party.baupost  = (asset.jv.unreturned - asset.jv.capital) * (1.0 - inputs.sponsor_share)
  pay preferred_sp  to party.penzance = (asset.jv.unreturned - asset.jv.capital) * inputs.sponsor_share
  pay promote       to party.penzance = remaining * 0.20
  pay residual_inv  to party.baupost  = remaining * (1.0 - inputs.sponsor_share)
  pay residual_sp   to party.penzance = remaining
}

// -------------------------------------------------------------- the returns
// WHAT EACH PARTNER ACTUALLY EARNED, measured on that partner's own capital in
// and distributions out.
//
// Penzance's figure is all-in: its preferred and residual as a 10% investor,
// and the promote it earned as sponsor. Each tier is reported on its own, so
// the promote can be read separately from the investor return.
metric baupost_irr   = irr(party.baupost)
metric baupost_moic  = moic(party.baupost)
metric penzance_irr  = irr(party.penzance)
metric penzance_moic = moic(party.penzance)
```

## credit/level_pay_pool

A level-payment amortizing loan pool — the constant instalment that splits into shrinking interest and growing principal.

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay`, `credit.purchase` |
| Language features | a pack contract paired with a purchase price |
| Conventions | level-pay amortization, CPR, CDR, loss severity, recovery lag, a servicing strip, a prepayment penalty, purchase at a discount |

```cfdl
version 0.1
model "level-pay-pool"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 126

entity asset buyer : Credit.Asset.LoanPool

// $25m homogeneous level-pay pool, 6.5% note rate, 10-year amortization,
// 8 CPR, 2 CDR, 35% severity, 6-month recovery lag, 50bp servicing strip,
// 1% prepayment penalty. The contract term spans term_months +
// recovery_lag_months so recoveries have periods to land in.
contract credit.pool_level_pay.auto_a on entity asset.buyer {
  term 2026-01..2036-06
  terms {
    principal = 25000000
    interest_rate = 0.065
    term_months = 120
    cpr = 0.08
    cdr = 0.02
    severity = 0.35
    recovery_lag_months = 6
    servicing_fee = 0.005
    prepay_penalty_rate = 0.01
  }
}

// Purchased at a 1-point discount (99.0) at close.
contract credit.purchase.auto_a on entity asset.buyer {
  term 2026-01..2026-01
  terms {
    price = 24750000
  }
}
```

## credit/float_bridge_pool

A floating-rate bridge loan pool priced off a forward curve, where the coupon resets each period rather than being fixed at origination.

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_float_io_bullet`, `credit.purchase` |
| Language features | a declared `curve` read per period by `curve_value` with step interpolation |
| Conventions | a coupon that resets rather than fixing at origination, a binding rate floor, a bullet maturity |

Curves are exercised end to end here: the curve statement, its representation
in the compiled model, and the per-period lookup.

```cfdl
version 0.1
model "float-bridge-pool"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 41

// Forward index curve (flat-forward / step interpolation): the coupon for a
// period uses the last curve point at or before the period date.
curve sofr {
  2026-01: 0.048
  2026-07: 0.045
  2027-01: 0.042
  2027-07: 0.040
  2028-01: 0.0385
}

entity asset buyer : Credit.Asset.LoanPool

// $15m floating IO bridge pool: SOFR + 275, coupon floored at 7.00% (binds
// once SOFR + margin falls below it) and capped at 9.00% (never binds),
// 36-month bullet, 10 CPR, 2.5 CDR, 45% severity, 5-month recovery lag.
// Contract term spans term_months + recovery_lag_months.
contract credit.pool_float_io_bullet.bridge_f on entity asset.buyer {
  term 2026-01..2029-05
  terms {
    principal = 15000000
    index_curve = "sofr"
    margin = 0.0275
    rate_floor = 0.07
    rate_cap = 0.09
    term_months = 36
    cpr = 0.10
    cdr = 0.025
    severity = 0.45
    recovery_lag_months = 5
  }
}

contract credit.purchase.bridge_f on entity asset.buyer {
  term 2026-01..2026-01
  terms {
    price = 15000000
  }
}
```

## credit/mbs_pool_by_loan

The same mortgage pool declared loan by loan, with the published pool schedule asserted against the aggregate the engine rolls up from its children.

| | |
|---|---|
| Pack | `credit` |
| Declared | five typed assets, one of them a parent; four contract instances |
| Language features | **`part of` hierarchy**, typed entity fields, per-instance contract suffixes |
| Conventions | level-pay amortization, SMM on the gross balance, MDR, a lagged recovery |

Two aggregates are asserted, computed by unrelated code:

- `entity.asset.pool.net_cash_flow` — the **hierarchy rollup**, aggregating the
  children a `part of` relation names rather than a matching name prefix.
- `domain.credit.gross_collections` — the **category subtotal**, the pack folding
  four contract instances into one domain line.

Both must reproduce the same published schedule. A defect in either shows as a
divergence between them.

```cfdl
version 0.1
model "mbs-pool-by-loan"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 372

// THE SAME POOL AS `mbs_pool_conventions`, MODELED AT A DIFFERENT GRAIN.
//
// That case declares one $100m pool and asserts the published schedule against
// it. This one declares the SAME $100m as four loans that belong to a pool, and
// asserts the SAME published figures against the pool.
//
// The published numbers are therefore doing two jobs. They still check the
// conventions — level-pay amortization, SMM on the gross balance, MDR, a
// twelve-month recovery lag. And because the pool holds no contract of its own,
// every figure asserted at the pool level is an aggregate the engine computed
// by walking `part of`. A rollup that summed the wrong children, or that
// aggregated by name prefix rather than by the relation, cannot reproduce a
// schedule it did not produce.
//
// The four balances are uneven so that the aggregation is tested: four equal
// loans would agree with the pool under any rule that divided by four.

entity asset pool : Credit.Asset.LoanPool {
  collateral_type            = "residential_mortgage"
  original_balance           = 100000000
  weighted_average_coupon    = 0.08
  weighted_average_maturity  = 360
}

entity asset loan_a : Credit.Asset.Loan {
  original_balance = 40000000
  coupon           = 0.08
  part of asset.pool
}

entity asset loan_b : Credit.Asset.Loan {
  original_balance = 30000000
  coupon           = 0.08
  part of asset.pool
}

entity asset loan_c : Credit.Asset.Loan {
  original_balance = 20000000
  coupon           = 0.08
  part of asset.pool
}

entity asset loan_d : Credit.Asset.Loan {
  original_balance = 10000000
  coupon           = 0.08
  part of asset.pool
}

// Every loan carries the pool's conventions, because the reference's pool is
// homogeneous: one 8% WAC, one term, one hazard pair. The pack takes ANNUAL
// cpr/cdr and converts with `cpr_to_periodic`, so a monthly 1% SMM is stated
// as its annual equivalent, 1 - (1 - 0.01)^12, which converts back to exactly
// 0.01. The same restatement `mbs_pool_conventions` makes.

contract credit.pool_level_pay.a on entity asset.loan_a {
  term 2026-01..2056-12
  terms {
    principal = 40000000
    interest_rate = 0.08
    term_months = 360
    cpr = 0.11361512828387077
    cdr = 0.11361512828387077
    severity = 0.20
    recovery_lag_months = 12
  }
}

contract credit.pool_level_pay.b on entity asset.loan_b {
  term 2026-01..2056-12
  terms {
    principal = 30000000
    interest_rate = 0.08
    term_months = 360
    cpr = 0.11361512828387077
    cdr = 0.11361512828387077
    severity = 0.20
    recovery_lag_months = 12
  }
}

contract credit.pool_level_pay.c on entity asset.loan_c {
  term 2026-01..2056-12
  terms {
    principal = 20000000
    interest_rate = 0.08
    term_months = 360
    cpr = 0.11361512828387077
    cdr = 0.11361512828387077
    severity = 0.20
    recovery_lag_months = 12
  }
}

contract credit.pool_level_pay.d on entity asset.loan_d {
  term 2026-01..2056-12
  terms {
    principal = 10000000
    interest_rate = 0.08
    term_months = 360
    cpr = 0.11361512828387077
    cdr = 0.11361512828387077
    severity = 0.20
    recovery_lag_months = 12
  }
}
```

## credit/fnma_remic_2019_2_g3

Security Group 3 of a Fannie Mae REMIC: a seasoned mortgage pool passing through to a single class, with the coupon stripped between it and an interest-only class that carries no principal.

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay` |
| Language features | two waterfalls over one collateral, one for principal and one for interest; entity fields carrying class balances |
| Conventions | PSA on a pool seasoned past the ramp, a servicing and guaranty strip, a stripped coupon, a notional interest-only class |

**This is the first case in the repo where a coupon is stripped.** Every other
credit case pays interest at the rate the asset earns. Here three rates are in
play at once and none of them is the mortgage rate:

- the loans carry a **5.451%** weighted average coupon;
- **0.451%** is the servicing and guaranty strip, carried as `servicing_fee`, so
  what reaches the trust is 5.00% exactly;
- that 5.00% is then split 3.25% / 1.75% between a principal class and a
  notional one.

The interest waterfall is the test. It declares a residual step, and if the
strip is right that step takes nothing:

```cfdl
pay ab_interest to party.ab_holders = asset.ab.balance * (0.0325 / 12.0)
pay io_interest to party.io_holders = asset.io.balance * (0.05 / 12.0)
pay residual    to party.residual   = remaining
```

**The residual is zero in all 361 periods, to ten decimal places.**

A note on why the class balances are fields at all. AB is a pass-through, so its
balance is the pool's, and IO's is a fixed fraction of AB's — neither carries
state of its own, and neither is copied. The field says what the class *is*
(`next prev.asset.pool.balance`) and derives the number. That the balances land
one period behind the distributions is not a workaround here but the deal's own
convention: the supplement strikes interest on "the outstanding balance of that
Certificate immediately prior to that Distribution Date", which is precisely the
number these fields hold.

```cfdl
version 0.1
model "fnma-remic-2019-2-g3"
use pack "credit" version "0.1.0"
time calendar monthly from 2019-02 for 361

// GROUP 3 OF A FANNIE MAE REMIC, against the issuer's own decrement table.
//
// Fannie Mae REMIC Trust 2019-2 has three groups. Groups 1 and 2 are STRUCTURED
// COLLATERAL — their assets are seventeen tranches of other REMICs issued
// between 2002 and 2006, so the cash arriving at those groups is another
// instrument's output and has to be supplied rather than derived. Group 3 is
// the one backed directly by mortgage-backed securities, so it is complete in
// this document and is the group modeled here.
//
// A REMIC IS A FUNCTION FROM DOLLARS RECEIVED TO DOLLARS ALLOCATED, and this
// group's function is one line: everything to AB. What makes it worth a case is
// not the waterfall but the STRIP. The pool passes through at 5.00% against a
// 5.451% weighted average coupon; AB takes 3.25% of that, and the remaining
// 1.75% is sold separately as IO — a notional class with no principal, whose
// balance is 35.0000000674% of AB's and which therefore shrinks exactly as AB
// does.
//
// The two coupons reconstruct the pass-through rate to nine decimal places:
//
//     3.25%  +  0.350000000674 x 5.00%  =  5.00000000337%
//
// so the interest waterfall below should exhaust the pool's interest and leave
// the residual class nothing. That identity is asserted, and it is the reason
// this deal is more than a single pass-through pool.
//
// A FIELD CARRIES THE OPENING BALANCE. A recurrence may read the previous
// period's fields and no stream at all (docs/14), so a class balance at t is
// the balance FOLLOWING the distribution at t-1. That is not a workaround here,
// it is the deal's own convention: the supplement says interest on each
// certificate is "one month's interest on the outstanding balance of that
// Certificate immediately prior to that Distribution Date", which is exactly
// the number these fields hold.
//
// The published decrement table states balances outstanding after each January's
// distribution, so `expected.csv` asserts them one row later, and the timeline
// carries 361 periods so the January 2049 row has somewhere to land.
//
// Reference: Prospectus Supplement dated 24 January 2019 to the REMIC
// Prospectus dated 1 November 2018. See SOURCE.md.

entity asset trust : Credit.Asset.LoanPool {
  collateral_type = "residential"
}

// The Group 3 MBS. `balance` restates the level-pay amortization the pack
// already applies — one step per period, at the mortgage rate, against the PSA
// curve — because a field cannot read a stream. It is not an independent
// number: `expected.csv` asserts the pack's own cumulative principal against the
// same published column, so both are pinned to the issuer's figures rather than
// to each other.
//
// The loans are SEASONED PAST THE RAMP. Weighted average loan age is 175
// months, so `min(age, 30)` is 30 in every period and 198% PSA is a flat
// 11.88% CPR throughout. The ramp is written out in full anyway, because what
// the model should say is "this pool prepays at 198% PSA", not "this pool
// prepays at 11.88% CPR" — the second is a consequence, and it stops being true
// the moment the seasoning changes.
entity asset pool : Credit.Asset.LoanPool {
  collateral_type = "residential"
  part of asset.trust

  balance init 148372434.0
               * (1.0 - ((-pmt(0.0045425, 173.0, 1.0)) - 0.0045425))
               * (1.0 - cpr_to_periodic(min(1.98 * 0.002 * max(1.0, min(176.0, 30.0)), 1.0), 12.0))
          next if(time.t < 173.0,
                  prev * (1.0 - ((-pmt(0.0045425, 173.0 - time.t, 1.0)) - 0.0045425))
                       * (1.0 - cpr_to_periodic(min(1.98 * 0.002 * max(1.0, min(time.t + 176.0, 30.0)), 1.0), 12.0)),
                  0.0)
}

// AB — the pass-through class. It takes every dollar of principal, so its
// balance IS the pool's balance one period back. Nothing is copied: the field
// says "AB is a pass-through" and derives the number rather than tracking it.
entity asset ab : Credit.Asset.Tranche {
  seniority = 1
  original_balance = 148372434.0
  balance init 148372434.0
          next prev.asset.pool.balance
}

// IO — a notional class. It has no principal and receives none; the balance
// exists only to strike its interest, and it is a fixed fraction of AB's.
entity asset io : Credit.Asset.Tranche {
  seniority = 1
  original_balance = 51930352.0
  balance init 51930352.0
          next prev.asset.pool.balance * 0.350000000674
}

entity party ab_holders : Credit.Party.Investor { name = "Class AB holders" }
entity party io_holders : Credit.Party.Investor { name = "Class IO holders" }
entity party residual : Credit.Party.Investor { name = "Classes R and RL" }

// THE GROUP 3 MBS. $148,372,434 of Fannie Mae certificates passing through at
// 5.00%, against mortgage loans the supplement assumes at a 5.451% weighted
// average coupon, 173 months remaining and 175 months of seasoning as of
// 1 January 2019. 198% PSA is the pricing speed of the seven the decrement
// table publishes.
//
// `rate` is the MORTGAGE rate, because that is what amortizes the loans and so
// sets the principal the trust passes through. The 0.451% between it and the
// 5.00% pass-through rate is the servicing and guaranty strip, and it is
// carried as `servicing_fee` so that what reaches the trust is 5.00% exactly —
// which is what makes the AB and IO coupons add up below.
contract credit.pool_level_pay.g3 on entity asset.pool {
  term 2019-02..2033-06
  terms {
    principal = 148372434
    interest_rate = 0.05451
    term_months = 173
    age_months = 175
    psa_speed = 1.98
    servicing_fee = 0.00451
  }
}

// ---------------------------------------------------------------------------
// Distributions of principal
//
//   "The Group 3 Principal Distribution Amount to AB until retired."
//
// That is the entire priority of payments for this group. IO receives no
// principal, and the residual classes are entitled to nothing until AB is gone.
// ---------------------------------------------------------------------------
// NARROWER THAN `available`, deliberately: the supplement distributes
// principal as its own amount, so this waterfall draws that slice rather than
// the group's whole cash. `docs/03` §3.2 keeps the `from` expression free
// for exactly this.
waterfall g3.principal on entity asset.trust {
  schedule every month from 2019-02 to 2033-06

  from series_sum("credit.pool.sched_principal.*", time.t, time.t)
       + series_sum("credit.pool.prepay.*", time.t, time.t)

  pay ab_principal to party.ab_holders = remaining
}

// ---------------------------------------------------------------------------
// Distributions of interest
//
// One month's interest on the balance each certificate carried into the
// distribution date. AB at 3.25% and IO at 5.00% of a notional that is
// 35.0000000674% of AB's balance — together, the 5.00% the pool passes through.
//
// The residual step is the test. If the two coupons are right it takes nothing,
// and `expected.csv` asserts that it takes nothing.
// ---------------------------------------------------------------------------
// NARROWER THAN `available`, deliberately: the supplement distributes
// interest as its own amount, so this waterfall draws that slice rather than
// the group's whole cash. `docs/03` §3.2 keeps the `from` expression free
// for exactly this.
waterfall g3.interest on entity asset.trust {
  schedule every month from 2019-02 to 2033-06

  // What the TRUST receives, which is not what the loans pay. The pack's
  // interest line is gross, at the 5.451% mortgage coupon; the servicing and
  // guaranty strip is a separate outflow and is stored negative, so adding it
  // nets it off and leaves the 5.00% pass-through rate exactly.
  from series_sum("credit.pool.interest.*", time.t, time.t)
       + series_sum("credit.pool.servicing.*", time.t, time.t)

  pay ab_interest to party.ab_holders = asset.ab.balance * (0.0325 / 12.0)
  pay io_interest to party.io_holders = asset.io.balance * (0.05 / 12.0)
  pay residual    to party.residual   = remaining
}
```

## credit/auto_abs_tranches

The note classes of an auto ABS: the trust as a container, collections as accounts, and ordered waterfalls paying seven classes by seniority, reconciled against the issuer's published percent-outstanding grid at every distribution date.

The trust as a container, the notes as claims on its cash, and the priority of
payments as two ordered allocations from the two amounts the indenture defines.

Interest collected, net of the servicer's fee and the trust's own expense, is
one account; principal collected is another. Each class is a `credit.note`
the trust issued — a face, a coupon, and the account its holder's principal
is paid into — and each distribution date allocates the first account to the
classes' interest and the second to their principal, by seniority. Every
holder owns an account that receives its principal, and that account IS the
class's position: what a class is still owed is its face less what its
holder has been paid, which the note lowers as its claim, so a step pays the
claim and says which note and line it pays:

```cfdl
pay a3_principal to party.a3_holders for contract credit.note.a3 line principal =
      min(remaining, container.trust.credit_note_claim_a3)
```

Declaration order is seniority. A retired class contributes zero because its
claim is zero, without being switched off. Nothing restates the waterfall and
no class carries a balance of its own.

The published grid is therefore asserted directly, not by differencing: each
class's percent outstanding is its face less its account, and the account
balances sit in `expected.csv` beside the payments they explain.

```cfdl
version 0.1
model "auto-abs-tranches"
use pack "credit" version "0.1.0"
time calendar monthly from 2018-10 for 64

// THE NOTE CLASSES OF AN AUTO ABS, against the issuer's published grid.
//
// `auto_abs_wal` reconciles this deal's COLLATERAL — 43 sub-pools amortizing
// to an aggregate the issuer states to the cent. It stops there, and says so:
// the per-class columns need a sequential-pay waterfall. This case is that
// axis: the same 43 sub-pools, one priority of payments, seven note classes.
//
// THE TRUST IS A CONTAINER, AND THE NOTES ARE CLAIMS ON ITS CASH. Each month
// the receivables pay interest and principal; the trust's fees come out of
// those collections; interest is paid on each
// class at its coupon; and principal repays the classes strictly in order of
// seniority. Interest collected and principal collected are the two amounts the
// indenture defines, each an account the trust holds, and each distribution
// date allocates them by the priority of payments.
//
// A CLASS'S POSITION IS ITS HOLDER'S ACCOUNT. What a class has been repaid is
// the principal allocated to its holder so far, so what it is still owed is
// its face less that account. No class carries a balance of its own, and
// nothing restates the waterfall: each step reads the account the previous
// distributions filled. Declaration order IS seniority, and a retired class
// contributes zero because its claim is zero.
//
// The one-month gap between collection and distribution is the deal's own
// convention: receivables pay on the last day of the month and the notes pay
// on the 15th of the next.
//
// NO LOSSES ARE ASSUMED, by the exhibit's own terms — it states the receivables
// prepay at a constant ABS rate "with no defaults, losses or repurchases". So
// overcollateralization never has to build and no trigger can trip. The $13.75m
// by which the pool exceeds the notes, and interest collected beyond interest
// paid, accumulate in the trust's own accounts.

entity container trust : Container.SPV

entity asset p01 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p02 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p03 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p04 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p06 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p07 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p08 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p09 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p11 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p12 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p13 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p14 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p16 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p17 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p18 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p19 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p21 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p22 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p23 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p24 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p26 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p27 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p28 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p29 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p31 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p32 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p33 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p34 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p36 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p37 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p38 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p39 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p40 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p41 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p42 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p43 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p44 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p45 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p46 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p47 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p48 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p49 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p50 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity party servicer : Credit.Party.Servicer { name = "Servicer" }
entity party a1_holders : Credit.Party.Investor { name = "Class A-1 noteholders" }
entity party a2_holders : Credit.Party.Investor { name = "Class A-2 noteholders" }
entity party a3_holders : Credit.Party.Investor { name = "Class A-3 noteholders" }
entity party a4_holders : Credit.Party.Investor { name = "Class A-4 noteholders" }
entity party b_holders : Credit.Party.Investor { name = "Class B noteholders" }
entity party c_holders : Credit.Party.Investor { name = "Class C noteholders" }
entity party d_holders : Credit.Party.Investor { name = "Class D noteholders" }

contract credit.pool_level_pay.p01 on entity asset.p01 {
  term 2018-10..2020-03
  terms {
    principal = 5616021.32
    interest_rate = 0.00000
    term_months = 18
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p02 on entity asset.p02 {
  term 2018-10..2021-01
  terms {
    principal = 2616054.82
    interest_rate = 0.00000
    term_months = 28
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p03 on entity asset.p03 {
  term 2018-10..2022-06
  terms {
    principal = 4635948.89
    interest_rate = 0.00000
    term_months = 45
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p04 on entity asset.p04 {
  term 2018-10..2022-12
  terms {
    principal = 2205909.75
    interest_rate = 0.00000
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p06 on entity asset.p06 {
  term 2018-10..2019-11
  terms {
    principal = 147440.15
    interest_rate = 0.00915
    term_months = 14
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p07 on entity asset.p07 {
  term 2018-10..2021-03
  terms {
    principal = 216238.15
    interest_rate = 0.00992
    term_months = 30
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p08 on entity asset.p08 {
  term 2018-10..2022-07
  terms {
    principal = 354043.75
    interest_rate = 0.00907
    term_months = 46
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p09 on entity asset.p09 {
  term 2018-10..2022-12
  terms {
    principal = 342126.24
    interest_rate = 0.00905
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p11 on entity asset.p11 {
  term 2018-10..2020-02
  terms {
    principal = 610459.31
    interest_rate = 0.01906
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p12 on entity asset.p12 {
  term 2018-10..2021-04
  terms {
    principal = 1144291.74
    interest_rate = 0.01951
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p13 on entity asset.p13 {
  term 2018-10..2022-02
  terms {
    principal = 699535.89
    interest_rate = 0.01949
    term_months = 41
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p14 on entity asset.p14 {
  term 2018-10..2022-12
  terms {
    principal = 201897.47
    interest_rate = 0.01869
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p16 on entity asset.p16 {
  term 2018-10..2020-02
  terms {
    principal = 13918351.08
    interest_rate = 0.02594
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p17 on entity asset.p17 {
  term 2018-10..2021-04
  terms {
    principal = 26181002.53
    interest_rate = 0.02626
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p18 on entity asset.p18 {
  term 2018-10..2022-02
  terms {
    principal = 28740527.64
    interest_rate = 0.02684
    term_months = 41
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p19 on entity asset.p19 {
  term 2018-10..2022-12
  terms {
    principal = 9735143.46
    interest_rate = 0.02794
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p21 on entity asset.p21 {
  term 2018-10..2020-02
  terms {
    principal = 14533243.98
    interest_rate = 0.03678
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p22 on entity asset.p22 {
  term 2018-10..2021-04
  terms {
    principal = 26195374.46
    interest_rate = 0.03667
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p23 on entity asset.p23 {
  term 2018-10..2022-03
  terms {
    principal = 37348352.52
    interest_rate = 0.03671
    term_months = 42
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p24 on entity asset.p24 {
  term 2018-10..2023-01
  terms {
    principal = 19509631.08
    interest_rate = 0.03673
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p26 on entity asset.p26 {
  term 2018-10..2020-02
  terms {
    principal = 12183065.19
    interest_rate = 0.04661
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p27 on entity asset.p27 {
  term 2018-10..2021-04
  terms {
    principal = 20323443.61
    interest_rate = 0.04674
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p28 on entity asset.p28 {
  term 2018-10..2022-03
  terms {
    principal = 32071657.98
    interest_rate = 0.04690
    term_months = 42
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p29 on entity asset.p29 {
  term 2018-10..2023-01
  terms {
    principal = 20332473.43
    interest_rate = 0.04674
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p31 on entity asset.p31 {
  term 2018-10..2020-02
  terms {
    principal = 6428613.14
    interest_rate = 0.05572
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p32 on entity asset.p32 {
  term 2018-10..2021-05
  terms {
    principal = 16325861.98
    interest_rate = 0.05566
    term_months = 32
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p33 on entity asset.p33 {
  term 2018-10..2022-04
  terms {
    principal = 34020451.15
    interest_rate = 0.05608
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p34 on entity asset.p34 {
  term 2018-10..2023-01
  terms {
    principal = 22175932.04
    interest_rate = 0.05615
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p36 on entity asset.p36 {
  term 2018-10..2020-03
  terms {
    principal = 4214767.90
    interest_rate = 0.06583
    term_months = 18
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p37 on entity asset.p37 {
  term 2018-10..2021-05
  terms {
    principal = 10197295.25
    interest_rate = 0.06567
    term_months = 32
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p38 on entity asset.p38 {
  term 2018-10..2022-04
  terms {
    principal = 28511150.24
    interest_rate = 0.06580
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p39 on entity asset.p39 {
  term 2018-10..2023-01
  terms {
    principal = 21518975.29
    interest_rate = 0.06583
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p40 on entity asset.p40 {
  term 2018-10..2024-01
  terms {
    principal = 210992.57
    interest_rate = 0.06671
    term_months = 64
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p41 on entity asset.p41 {
  term 2018-10..2020-02
  terms {
    principal = 2314366.62
    interest_rate = 0.07537
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p42 on entity asset.p42 {
  term 2018-10..2021-04
  terms {
    principal = 6049009.56
    interest_rate = 0.07527
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p43 on entity asset.p43 {
  term 2018-10..2022-04
  terms {
    principal = 17752272.88
    interest_rate = 0.07538
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p44 on entity asset.p44 {
  term 2018-10..2023-02
  terms {
    principal = 17560641.20
    interest_rate = 0.07526
    term_months = 53
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p45 on entity asset.p45 {
  term 2018-10..2024-01
  terms {
    principal = 133227.13
    interest_rate = 0.07709
    term_months = 64
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p46 on entity asset.p46 {
  term 2018-10..2020-02
  terms {
    principal = 4089106.53
    interest_rate = 0.09923
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p47 on entity asset.p47 {
  term 2018-10..2021-04
  terms {
    principal = 9761650.69
    interest_rate = 0.09773
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p48 on entity asset.p48 {
  term 2018-10..2022-05
  terms {
    principal = 26285138.49
    interest_rate = 0.09619
    term_months = 44
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p49 on entity asset.p49 {
  term 2018-10..2023-02
  terms {
    principal = 29949234.04
    interest_rate = 0.09622
    term_months = 53
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p50 on entity asset.p50 {
  term 2018-10..2023-11
  terms {
    principal = 279866.82
    interest_rate = 0.09836
    term_months = 62
    cpr = 0
    cdr = 0
  }
}

// ---------------------------------------------------------------------------
// The notes: seven classes, each a structured note the trust issued — a face,
// a coupon, and the account its holder's principal is paid into. Faces are
// each class's balance at the exhibit's cut-off, which is the base its
// percent-outstanding grid is stated on: A-1 was paid in full in January 2018
// and is carried at zero; A-2 had amortized to 112,026,644 (the trust's Form
// 10-D); the rest stood at their original principal. Coupons and the 30/360
// day count are the exhibit's. A note lowers no cash of its own: it lowers
// its claim — face less what its holder's account has received — and the
// interest due on it, and the priority of payments below pays both.
// ---------------------------------------------------------------------------
contract credit.note a1 on entity container.trust {
  term 2018-10..2024-01
  terms {
    face = 0.00
    coupon = 0.0110
    principal_account = a1_principal
  }
  parties {
    holder = party.a1_holders
  }
}
contract credit.note a2 on entity container.trust {
  term 2018-10..2024-01
  terms {
    face = 112026644.00
    coupon = 0.0153
    principal_account = a2_principal
  }
  parties {
    holder = party.a2_holders
  }
}
contract credit.note a3 on entity container.trust {
  term 2018-10..2024-01
  terms {
    face = 271370000.00
    coupon = 0.0174
    principal_account = a3_principal
  }
  parties {
    holder = party.a3_holders
  }
}
contract credit.note a4 on entity container.trust {
  term 2018-10..2024-01
  terms {
    face = 86010000.00
    coupon = 0.0201
    principal_account = a4_principal
  }
  parties {
    holder = party.a4_holders
  }
}
contract credit.note b on entity container.trust {
  term 2018-10..2024-01
  terms {
    face = 22220000.00
    coupon = 0.0224
    principal_account = b_principal
  }
  parties {
    holder = party.b_holders
  }
}
contract credit.note c on entity container.trust {
  term 2018-10..2024-01
  terms {
    face = 18510000.00
    coupon = 0.0237
    principal_account = c_principal
  }
  parties {
    holder = party.c_holders
  }
}
contract credit.note d on entity container.trust {
  term 2018-10..2024-01
  terms {
    face = 13750000.00
    coupon = 0.0291
    principal_account = d_principal
  }
  parties {
    holder = party.d_holders
  }
}

// ---------------------------------------------------------------------------
// The trust's expenses, one item each. The servicing fee is 1.00% per annum
// on the pool balance the trust carried into the month — the initial pool
// less the principal collected so far — and the administration fee is $1,500
// a month. Neither the servicer nor the administrator is modeled as a payee:
// the fees leave the trust's cash before anything reaches the notes, which is
// all the published grid depends on.
// ---------------------------------------------------------------------------
assume initial_pool = 537640787.96

stream credit.trust.servicing_fee on entity container.trust outflow currency USD {
  schedule every month from 2018-10 to 2024-01
  category operating.expense.servicing
  amount = 0.01 / 12.0 * (inputs.initial_pool
             - if(time.t == 0.0, 0.0,
                  series_sum("credit.pool.sched_principal.*", 0, time.t - 1)
                  + series_sum("credit.pool.prepay.*", 0, time.t - 1)))
}

stream credit.trust.admin_fee on entity container.trust outflow currency USD {
  schedule every month from 2018-10 to 2024-01
  category operating.expense.servicing
  amount = 1500.0
}

// ---------------------------------------------------------------------------
// The accounts. The indenture defines two amounts on each distribution date,
// and each is a location cash sits in: AVAILABLE INTEREST — the interest
// collected, net of the trust's fees — and the
// PRINCIPAL DISTRIBUTABLE AMOUNT — the principal collected. Each holder owns an
// account that receives its principal, which IS the class's position, and a
// separate interest account beside it, so what has been repaid and what has
// been earned are never mixed.
// ---------------------------------------------------------------------------
account interest_collections {
  from series_sum("credit.pool.interest.*", time.t, time.t)
     + series_sum("credit.trust.servicing_fee", time.t, time.t)
     + series_sum("credit.trust.admin_fee", time.t, time.t)
}

account principal_collections {
  from series_sum("credit.pool.sched_principal.*", time.t, time.t)
     + series_sum("credit.pool.prepay.*", time.t, time.t)
}

account a1_principal { owner party.a1_holders }
account a2_principal { owner party.a2_holders }
account a3_principal { owner party.a3_holders }
account a4_principal { owner party.a4_holders }
account b_principal { owner party.b_holders }
account c_principal { owner party.c_holders }
account d_principal { owner party.d_holders }

account a1_interest { from 0.0 }
account a2_interest { from 0.0 }
account a3_interest { from 0.0 }
account a4_interest { from 0.0 }
account b_interest { from 0.0 }
account c_interest { from 0.0 }
account d_interest { from 0.0 }

// ---------------------------------------------------------------------------
// Interest, on each distribution date: every class its interest due — its
// coupon on the claim it carried in, which the note lowers as a field.
// Each step names the note and the line it pays. Interest collected beyond
// the coupons stays in the trust's interest account.
// ---------------------------------------------------------------------------
waterfall notes.interest on entity container.trust {
  schedule every month from 2018-10 to 2024-01
  from interest_collections

  pay a1_interest to account a1_interest for contract credit.note.a1 line interest =
        min(remaining, container.trust.credit_note_interest_due_a1)
  pay a2_interest to account a2_interest for contract credit.note.a2 line interest =
        min(remaining, container.trust.credit_note_interest_due_a2)
  pay a3_interest to account a3_interest for contract credit.note.a3 line interest =
        min(remaining, container.trust.credit_note_interest_due_a3)
  pay a4_interest to account a4_interest for contract credit.note.a4 line interest =
        min(remaining, container.trust.credit_note_interest_due_a4)
  pay b_interest to account b_interest for contract credit.note.b line interest =
        min(remaining, container.trust.credit_note_interest_due_b)
  pay c_interest to account c_interest for contract credit.note.c line interest =
        min(remaining, container.trust.credit_note_interest_due_c)
  pay d_interest to account d_interest for contract credit.note.d line interest =
        min(remaining, container.trust.credit_note_interest_due_d)
}

// ---------------------------------------------------------------------------
// Principal, strictly by seniority. A step pays its note's claim — face less
// what the holder's account has received, lowered by the note — out of what
// remains, so nothing reaches a class until every class above it is gone.
// The $13.75m by which the pool exceeds the notes stays in the principal
// account as the trust's own cash.
// ---------------------------------------------------------------------------
waterfall notes.principal on entity container.trust {
  schedule every month from 2018-10 to 2024-01
  from principal_collections

  pay a1_principal to party.a1_holders for contract credit.note.a1 line principal =
        min(remaining, container.trust.credit_note_claim_a1)
  pay a2_principal to party.a2_holders for contract credit.note.a2 line principal =
        min(remaining, container.trust.credit_note_claim_a2)
  pay a3_principal to party.a3_holders for contract credit.note.a3 line principal =
        min(remaining, container.trust.credit_note_claim_a3)
  pay a4_principal to party.a4_holders for contract credit.note.a4 line principal =
        min(remaining, container.trust.credit_note_claim_a4)
  pay b_principal to party.b_holders for contract credit.note.b line principal =
        min(remaining, container.trust.credit_note_claim_b)
  pay c_principal to party.c_holders for contract credit.note.c line principal =
        min(remaining, container.trust.credit_note_claim_c)
  pay d_principal to party.d_holders for contract credit.note.d line principal =
        min(remaining, container.trust.credit_note_claim_d)
}
```

## energy/utility_pv_singleowner

A utility-scale photovoltaic project in a single-owner structure, carrying its own tax position rather than allocating to an investor.

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.ppa`, `energy.om`, `energy.debt_service`, `energy.itc`, `energy.macrs_shield`, `energy.capex` |
| Language features | pack contracts across a full capital structure; term units |
| Conventions | production degradation, price escalation, level-pay debt, an investment tax credit, MACRS with a basis reduction |

More of the energy pack's contract surface than any other case.

```cfdl
version 0.1
model "utility-pv-singleowner"
use pack "energy" version "0.1.0"

// 100 MW-AC utility-scale PV, single-owner project finance, reconciled against
// the national laboratory's open-source project-finance model.
//
// Period 0 is the construction year and carries only capex, so periods 1..25
// line up index-for-index with the reference's annual operating years. That
// alignment is the point: no cadence translation sits between the two models,
// so a divergence is a convention difference and nothing else.
//
// Every escalation here is NOMINAL, which is what the pack's `escalation` term
// means. The reference states O&M escalation as a REAL rate carried on top of
// an inflation assumption, so this case is run at zero inflation, where the two
// coincide exactly. See NOTES.md — the conversion is additive, not compounded.
time calendar annual from 2025-01 for 26

entity asset pv : Energy.Asset.GenerationFacility

// THE DEAL'S TWO TAX INPUTS, STATED ONCE. Installed cost and the credit rate
// drive the capex, the credit and the depreciable basis, so changing either
// propagates instead of leaving a hand-computed figure behind.
assume installed_cost = 100000000
assume itc_rate       = 0.30

// 100 MW-AC / 250 GWh year one — a 28.5% net capacity factor.
contract energy.capex on entity asset.pv {
  term 2025-01..2025-01
  terms { amount = inputs.installed_cost }
}

// $45/MWh (4.5 c/kWh) escalating 2%/yr; 0.5%/yr module degradation.
contract energy.ppa on entity asset.pv {
  term 2026-01..2050-01
  terms {
    quantity = 250000 "MWh/yr"
    price = 45 "USD/MWh"
    escalation = 0.02
    degradation = 0.005
  }
}

// $15/kW-yr fixed O&M on 100,000 kW, escalating 2%/yr.
contract energy.om on entity asset.pv {
  term 2026-01..2050-01
  terms {
    fee_year = 1500000
    escalation = 0.02
  }
}

// 60% debt at 6% over 18 years, level annual payments.
// funded_at_close = 0: the reference's cash flow starts post-financing —
// it nets operations against debt service and never books the draw — so the
// proceeds the contract funds by default are excluded to state what the
// source states.
contract energy.debt_service on entity asset.pv {
  term 2026-01..2043-01
  terms {
    interest_rate = 0.06
    term_months = 216
    principal = 60000000
    funded_at_close = 0
  }
}

// The ITC on the full installed cost, taken in the first operating year.
contract energy.itc on entity asset.pv {
  term 2026-01..2026-01
  terms { amount = inputs.installed_cost * inputs.itc_rate }
}

// 5-year MACRS on the REDUCED basis: taking the investment credit removes
// half of it from what may be depreciated, so 100m becomes 85m here.
//
// The reduction is STATED, not pre-computed. The pack takes `basis` as an
// input rather than deriving it, because basis adjustments are jurisdictional
// and a wrong default is worse than none — but that is a reason for the model
// to say which adjustment applies, not a reason to paste in the answer. A
// hardcoded 85,000,000 goes stale the moment the cost or the credit rate
// moves, and nothing objects: the full basis overstates the shield by 17.6%.
contract energy.macrs_shield on entity asset.pv {
  term 2026-01..2050-01
  terms {
    basis = inputs.installed_cost * (1 - 0.5 * inputs.itc_rate)
    tax_rate = 0.21
    life = 5
  }
}
```

## energy/merchant_capacity

A merchant generator earning both energy and capacity revenue, exposed to price rather than to a contracted offtake.

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.merchant`, `energy.capacity`, `energy.ptc`, `energy.om`, `energy.debt_service`, `energy.macrs_shield`, `energy.capex` |
| Language features | pack contracts; term units on the credit rate |
| Conventions | merchant pricing with escalation, a flat capacity payment, a ten-year production credit with an inflation adjustment, MACRS on full basis |

```cfdl
// 100 MW-AC merchant renewable project with a capacity contract and the
// production tax credit, reconciled against the national laboratory's
// open-source project-finance model.
//
// COMPANION TO utility_pv_singleowner, AND A NARROWER CLAIM. That case
// validated the arithmetic; this one validates the WIRING. `energy.merchant`
// and `energy.ptc` are the same expression as `energy.ppa` with different term
// names, and `energy.capacity` is a single division — so agreement here shows
// the terms reach the right places and the contracts compose, not that a new
// formula is correct. NOTES.md says so; do not read it as more than that.
//
// THE ONE GENUINELY NEW THING is the production tax credit's STAIRCASE. The
// inflation-adjusted credit is published rounded to the nearest 0.1 cent per
// kWh, so it steps once a year and holds. The pack carried it continuously
// until now — wrong by up to 1.8% in a single year, alternating sign, which is
// why it read as noise. This case asserts the rounded path at a non-zero
// escalation, which is precisely the case that used to fail.
//
// Investment credit and production credit are mutually exclusive, so unlike the
// PV case there is no basis reduction here: MACRS runs on the full $100m.

version 0.1
model "merchant-capacity"
use pack "energy" version "0.1.0"

time calendar annual from 2025-01 for 26

entity asset wind : Energy.Asset.GenerationFacility

contract energy.capex on entity asset.wind {
  term 2025-01..2025-01
  terms { amount = 100000000 }
}

// Merchant energy: $45/MWh escalating 2%/yr, 0.5%/yr degradation.
contract energy.merchant on entity asset.wind {
  term 2026-01..2050-01
  terms {
    quantity = 250000
    price = 45
    escalation = 0.02
    degradation = 0.005
  }
}

// A flat capacity contract — no escalation, which is what the rule supports.
contract energy.capacity on entity asset.wind {
  term 2026-01..2050-01
  terms { price = 4000000 }
}

// Production tax credit: $27.50/MWh base, 2.5%/yr inflation adjustment, ten
// years statutory. round_step = 0.10 is the rule's default and is the
// statutory 0.1 c/kWh tick stated on this rule's $/MWh basis.
contract energy.ptc on entity asset.wind {
  term 2026-01..2035-01
  terms {
    quantity = 250000 "MWh/yr"
    amount = 27.50 "USD/MWh"
    escalation = 0.025
    degradation = 0.005
  }
}

contract energy.om on entity asset.wind {
  term 2026-01..2050-01
  terms {
    fee_year = 1500000
    escalation = 0.02
  }
}

// funded_at_close = 0: the reference's cash flow starts post-financing —
// it nets operations against debt service and never books the draw — so the
// proceeds the contract funds by default are excluded to state what the
// source states.
contract energy.debt_service on entity asset.wind {
  term 2026-01..2043-01
  terms {
    interest_rate = 0.06
    term_months = 216
    principal = 60000000
    funded_at_close = 0
  }
}

// Full basis: no investment credit was taken, so nothing reduces it.
contract energy.macrs_shield on entity asset.wind {
  term 2026-01..2050-01
  terms {
    basis = 100000000
    tax_rate = 0.21
    life = 5
  }
}

// Views by master type: everything sold as output, and every tax attribute.
slice supply {
  type Contract.Supply
}

slice tax {
  type Contract.Tax
}
```

## energy/tax_equity_flip

A tax-equity partnership whose flip date is derived from the investor's return rather than stated, reconciled against an external model.

| | |
|---|---|
| Pack | `energy` |
| Declared | two typed assets, two parties, three states, one event, one waterfall |
| Language features | **a declared lifecycle**, **an event whose guard is a computed value**, the transition log, a waterfall reading its owner's state |
| Conventions | 98/2 pre-flip and 5/95 post-flip sharing, an investment credit at 30%, MACRS on a basis reduced by half the credit, level-pay debt |

The lifecycle sits on the partnership **interest** rather than the plant: the
panels do not change when the structure flips, the claim on their cash does.

**The test needs no solver.** The criterion is an internal rate of return
reaching 8%, which this language cannot compute mid-model. It does not need to
— at a fixed hurdle the two statements are one:

    IRR through period n >= 8%   <=>   NPV at 8% through period n >= 0

A discounted running sum is a recurrence, so the test is arithmetic evaluated
once a period. Nor can it be circular: the test at period *t* reads flows
through *t-1*, and every one of those periods is still pre-flip by
construction, so the sharing percentages it depends on are settled before it
is evaluated.

```cfdl
version 0.1
model "tax-equity-flip"
use pack "energy" version "0.1.0"
time calendar annual from 2026-01 for 26

// A TAX-EQUITY PARTNERSHIP FLIP, where the flip date is DERIVED.
//
// A tax investor funds most of the equity and takes 98% of the cash and the
// tax attributes. When its after-tax return reaches a target, the structure
// flips: it drops to 5% and the sponsor takes the rest. The flip is not a date
// in a contract — it is a test, and when it lands depends on how the project
// performs.
//
// THE FLIP IS AN EVENT WRITING A NUMBER, and the date is an output. The
// investor's share of cash and tax is a term of the contract between the
// parties: stated at signing, changed when a condition is met. So it is a
// field of the stake, and the event sets it.
//
// It was first built as a lifecycle, `pre_flip` -> `post_flip`, with the
// percentages looked up from whichever state the stake was in. That is two
// facts — a state name and the number it implies — kept in step by nobody. A
// lifecycle earns its place when the phases differ in WHICH RULES APPLY, as a
// building under construction differs from one in operation. Here they differ
// by a number, and a number is a field.
//
// THE TEST NEEDS NO SOLVER. The criterion is an internal rate of return
// reaching 8%, and this language cannot compute an IRR mid-model. It does not
// need to: at a fixed hurdle the two statements are the same one.
//
//     IRR through period n >= 8%   <=>   NPV at 8% through period n >= 0
//
// A discounted running sum is a recurrence, which is a declared state, so the
// test is arithmetic evaluated once a period — a discrete test rather than a
// search, the same shape as an ordered waterfall's tiers.
//
// AND IT CANNOT BE CIRCULAR. The test at period t reads flows through t-1, and
// every one of those periods is by construction still pre-flip: the flip has
// not fired yet, or the test would not still be running. So the sharing
// percentages the test depends on are settled before it is evaluated.

entity asset project : Energy.Asset.GenerationFacility {
  technology         = "solar_pv"
  nameplate_capacity = 100000.0
  state in_service

  // What the plant throws off after operating costs and debt service. A fact
  // about the project, so it belongs to the project.
  cash init 0.0
       next inputs.energy_year_one * inputs.ppa_price
       * pow(1.0 + inputs.ppa_escalation, time.t - 1.0)
       * pow(1.0 - inputs.degradation, time.t - 1.0)
       - inputs.capacity_kw * inputs.om_per_kw
       * pow(1.0 + inputs.om_escalation, time.t - 1.0)
       - if(time.t <= inputs.debt_term,
       0.0 - pmt(inputs.debt_rate, inputs.debt_term, inputs.debt_amount),
       0.0)
}

// THE PARTNERSHIP, which is the thing that allocates the project's cash. Not
// the plant: when the flip happens nothing about the solar farm changes — same
// panels, same output — only who has a right to the money.
//
// There is no separate "stake" object between the partnership and its terms.
// The sharing percentage is a term of the partnership, so it is a field of the
// partnership.
entity asset partnership : Energy.Asset.ProjectInterest {
  interest_type = "tax_equity"

  // The investor's share of cash and of tax attributes, as the contract
  // states it at signing.
  investor_share init 0.98

  // How far the investor is toward its target, as the discounted value of
  // everything the partnership has returned it. The test that moves the share.
  return_position init 0.0 - inputs.investor_equity
                  next prev
                     + if(time.t >= 2.0 and prev < 0.0,
                     inputs.preflip_share
                     * ( prev.asset.project.cash
                     - inputs.tax_rate
                     * ( prev.asset.project.cash
                     + if(time.t - 1.0 <= inputs.debt_term,
                     (0.0 - pmt(inputs.debt_rate, inputs.debt_term, inputs.debt_amount))
                     + ipmt(inputs.debt_rate, time.t - 1.0, inputs.debt_term,
                     inputs.debt_amount),
                     0.0)
                     - macrs_rate(time.t - 2.0, 5)
                     * (inputs.installed_cost
                     - 0.5 * inputs.itc_rate * inputs.installed_cost) )
                     + if(time.t - 1.0 == 1.0, inputs.itc_rate * inputs.installed_cost, 0.0) )
                     / pow(1.0 + inputs.hurdle, time.t - 1.0),
                     0.0)
}

entity party sponsor      : Party { name = "Sponsor" }
entity party tax_investor : Party { name = "Tax investor" }

// ---------------------------------------------------------------------------
// The deal
// ---------------------------------------------------------------------------

assume energy_year_one = 250000000.0     // kWh in the first operating year
assume ppa_price       = 0.045           // $/kWh
assume ppa_escalation  = 0.02
assume degradation     = 0.005

assume capacity_kw     = 100000.0
assume om_per_kw       = 15.0
assume om_escalation   = 0.02

assume debt_amount     = 60000000.0
assume debt_rate       = 0.06
assume debt_term       = 18.0

// The equipment is $100m; the reference capitalizes $3.1m of financing into
// the installed cost, so the credit and depreciation are taken on the larger
// figure. Both bases follow from it: the credit on all of it, depreciation on
// it less half the credit, which is the rule that catches people out.
assume installed_cost  = 103100000.0
assume itc_rate        = 0.30
assume tax_rate        = 0.21

assume preflip_share   = 0.98
assume postflip_share  = 0.05

assume hurdle          = 0.08
assume investor_equity = 42238000.0      // 98% of $43.1m of equity

// ---------------------------------------------------------------------------
// The project, before anybody is paid
// ---------------------------------------------------------------------------



// THE TEST, as one recurrence.
//
// At period t this holds the investor's discounted after-tax position through
// period t-1: its share of cash, of the tax saved on the loss depreciation
// creates, and of the credit in the first operating year.
//
// It is one state rather than two because a state's `next` may read another
// state's PREVIOUS value and not its current one — so the flow of period t-1
// is exactly what is reachable here, and that is the flow the closed test
// wants. The lag is the deal's own convention: the year's books close, the
// return is tested, and the new sharing applies to the year that follows.
//
// Computed at the PRE-FLIP shares, and it stops accumulating the moment it
// turns non-negative — which is the period the flip fires. A test that has
// passed has no further question to answer, and stopping it keeps the series
// readable: its final value is the position that triggered the flip, not a
// running total at shares that stopped applying.
//
// Interest comes from `ipmt` rather than from a balance carried alongside.
// A balance state would hold the CLOSING figure, and interest is charged on
// the opening one — an off-by-one this states outright rather than works
// around.


// When the investor's return reaches its target, its share drops to 5% and the
// sponsor takes the rest — from the following period, which is the deal's own
// convention: the year's books close, the return is tested, the new split
// applies to the year that follows.
event flip when asset.partnership.return_position >= 0.0 {
  set entity asset.partnership.investor_share = 0.05
}

// ---------------------------------------------------------------------------
// What each partner receives
//
// The waterfall is owned by the INTEREST, so its steps read the lifecycle that
// governs the split. The investor takes its share and the sponsor takes the
// residual, which is what "the sponsor gets the rest" means.
// ---------------------------------------------------------------------------

// NOT `from available`: the pot is the PROJECT's cash — a sibling the
// partnership holds an interest in, not a child — and the project carries it
// as a field the deal itself tracks. Rehoming it as streams would move the
// case's asserted figures, so it stays until the case is rebuilt.
waterfall partnership.distribution on entity asset.partnership {
  schedule every year from 2027-01 to 2051-01
  from asset.project.cash

  pay investor to party.tax_investor = remaining * asset.partnership.investor_share
  pay sponsor  to party.sponsor = remaining
}
```

## opco/lbo_buyout

A leveraged buyout: entry at a stated multiple, debt paid down out of operating cash flow, and an exit that returns the sponsor's equity.

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.working_capital_policy`, `opco.term_debt`, `opco.cash_taxes`, `opco.acquisition`, `opco.exit_ebitda` |
| Language features | pack contracts across an entry, a hold and an exit |
| Conventions | entry at a multiple, days-based working capital, debt amortization from operating cash flow, an exit on trailing EBITDA |

The widest span of the opco pack's contract surface: entry, hold and exit
rather than one mechanic.

```cfdl
version 0.1
model "lbo-buyout"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity asset target : OpCo.Asset.Enterprise

// $12M-revenue services business bought at 8.0x run-rate EBITDA with a
// 5.0x term loan; sold after 5 years at 8.5x trailing-12 EBITDA.

contract opco.revenue_line on entity asset.target {
  term 2026-01..2030-12
  terms {
    amount = 1000000
    growth_rate = 0.06
  }
}

contract opco.opex_line on entity asset.target {
  term 2026-01..2030-12
  terms {
    amount = 650000
    growth_rate = 0.04
  }
}

// DSO 45 / DPO 30 / DIO 10; ending balance released at exit.
contract opco.working_capital_policy on entity asset.target {
  term 2026-01..2030-12
  terms {
    ar_days = 45
    ap_days = 30
    inv_days = 10
    release_at_end = 1
  }
}

// Maintenance capex at 3% of revenue.
contract opco.capex_line on entity asset.target {
  term 2026-01..2030-12
  terms {
    pct_of_revenue = 0.03
  }
}

// 5.0x leverage: $21m term loan, 8.5%, 12 months IO then 7-year
// amortization, balloon at exit.
contract opco.term_debt on entity asset.target {
  term 2026-01..2030-12
  terms {
    principal = 21000000
    interest_rate = 0.085
    interest_only_months = 12
    amortization_months = 84
  }
}

// Cash taxes at 26% on EBITDA - D&A - interest (no NOL carryforward).
contract opco.cash_taxes on entity asset.target {
  term 2026-01..2030-12
  terms {
    tax_rate = 0.26
    da_monthly = 150000
  }
}

// Entry at 8.0x annualized run-rate EBITDA = 8.0 * (350k * 12) = $33.6m.
contract opco.acquisition on entity asset.target {
  term 2026-01..2026-01
  terms {
    price = 33600000
  }
}

// Exit at 8.5x trailing-12 EBITDA net of 1.5% selling costs.
contract opco.exit_ebitda on entity asset.target {
  term 2030-12..2030-12
  terms {
    multiple = 8.5
    selling_costs = 0.015
  }
}

// Views by master type: the purchase, the debt and the exit as the deal's
// three agreements rather than as pack stream names.
slice acquisition {
  type Contract.Purchase
}

slice debt {
  type Contract.Debt
}

slice debt_interest {
  type Contract.Debt
  line interest
}

slice exit {
  type Contract.Sale
}
```

## opco/lbo_financing_cases

One sponsor buyout run at three capital structures, with the published five-year multiple and return reproduced for each.

| | |
|---|---|
| Pack | `opco` |
| Declared | two states, five curves, two native streams, three run scenarios |
| Language features | **run-config scenarios**, `cfg.*` parameters, declared state with `init`/`next`, curves |
| Conventions | average-balance interest, payment-in-kind accrual, a 100% cash sweep, tranche sizing to a debt increment, a sponsor cheque struck as the plug |

The financing case is the **run configuration**, not the model: the deterministic run is
Base and two scenarios override the tranche sizes, coupons and the sponsor's
cheque. That is what the source's own case switch does.

Sizes are not stated as inputs. Each tranche is its leverage multiple times LTM
EBITDA rounded to a $25m increment, and the sponsor's cheque is whatever
balances sources against uses. Base checks the rule — its published $275m,
$175m and $100m are what 3.0x, 2.0x and 1.0x round to — and the other two
structures are derived rather than transcribed.

```cfdl
version 0.1
model "lbo-financing-cases"
use pack "opco" version "0.1.0"
time calendar annual from 2016-01 for 6

// A sponsor buyout of a mid-market business, run at THREE capital structures.
//
// The operating case is identical in all three — same revenue path, same
// margin, same capex, same working capital. Only the financing changes, which
// is what the reference's own "financing case" switch does. Here that switch is
// the run configuration: the deterministic run is Base, and two scenarios override the
// tranche sizes and coupons.
//
// WHAT IS ASSERTED IS THE ENDPOINT. The reference publishes a period-by-period
// debt schedule for Base only, but it publishes MoIC and IRR for all three. So
// this case asserts returns, and nothing in between is anchored: the operating
// build, the debt schedule, the sweep, the PIK accrual and the exit all have to
// be right to land on a published multiple.
//
// Sizes are not stated as inputs. The reference sizes each tranche as its
// leverage multiple times LTM EBITDA, rounded to a $25m increment, and the
// sponsor's cheque is the plug that balances sources against uses. Both rules
// are reproduced below, which is why the three cases differ only in leverage
// and pricing.

entity asset target : OpCo.Asset.Enterprise
entity party sponsor : OpCo.Party.Sponsor { name = "Sponsor" }
entity party mgmt : OpCo.Party.Management { name = "Management" }

// --- The operating case, identical across financing cases -------------------
// EBIT, D&A, capex and the working-capital movement the reference derives from
// its revenue build. Held as curves because they are inputs to the recursion
// below, not outputs of it.

curve ebitda step {
  2017-01: 94.5
  2018-01: 100.17
  2019-01: 107.1819
  2020-01: 113.612814
  2021-01: 119.2934547
}

curve ebit step {
  2017-01: 76.65
  2018-01: 81.249
  2019-01: 86.93643
  2020-01: 92.1526158
  2021-01: 96.76024659
}

curve dna step {
  2017-01: 17.85
  2018-01: 18.921
  2019-01: 20.24547
  2020-01: 21.4601982
  2021-01: 22.53320811
}

curve capex step {
  2017-01: 17.85
  2018-01: 18.921
  2019-01: 20.24547
  2020-01: 21.4601982
  2021-01: 22.53320811
}

curve delta_nwc step {
  2017-01: -2.9
  2018-01: -3.654
  2019-01: -4.51878
  2020-01: -4.1443668
  2021-01: -3.66085734
}

curve libor step {
  2017-01: 0.005
  2018-01: 0.011
  2019-01: 0.015
  2020-01: 0.0215
  2021-01: 0.025
}

// --- The financing case ------------------------------------------------------
// Every knob a scenario overrides. The deterministic run is Base at 6.0x.

assume ltm_ebitda        = 90.0
assume debt_increment    = 25.0
assume tax_rate          = 0.35
assume purchase_equity   = 720.0        // 8.0x entry
assume transaction_costs = 15.0
assume rollover          = 36.0         // 5% management rollover
assume exit_multiple     = 8.0
assume cash_at_exit      = 5.0          // the minimum cash balance

// Tranche sizes, coupons, the annual fee amortization and the sponsor's cheque
// all arrive from the run configuration, because they are what a financing case IS.
// The deterministic run is Base; the two scenarios are the other structures.


assume commitment_fee = 0.35   // 0.35% on a $100m undrawn revolver
assume interest_income = 0.0125 // 0.25% on the $5m minimum cash balance

// --- The debt schedule -------------------------------------------------------
// Subordinated notes pay in kind for three years. A PIK coupon on an average
// balance is self-referential and collapses to a constant growth factor:
//
//     B = B0 + r (B0 + B) / 2   ->   B = B0 (1 + r/2) / (1 - r/2)

entity asset sub_notes : Asset.Financial {
  balance init cfg.sub_size
          next if(time.t <= 3,
                  prev * (1 + cfg.sub_rate / 2) / (1 - cfg.sub_rate / 2),
                  prev)
}

// Term Loan B, in closed form. Every dollar of free cash flow after the 1%
// mandatory amortization sweeps against it, and interest accrues on the average
// balance — so the balance depends on the interest that depends on the balance.
// Collecting terms solves it in one substitution.
entity asset tlb : Asset.Financial {
  balance init cfg.tlb_size
          next max(0.0,
       (prev * (1 + (1 - inputs.tax_rate) * (curve_value("libor", time.date) + cfg.tlb_spread) / 2)
        - (1 - inputs.tax_rate) * (curve_value("ebit", time.date)
             - (inputs.commitment_fee + cfg.senior_size * cfg.senior_rate
                + if(time.t <= 3,
                     prev.asset.sub_notes.balance * ((1 + cfg.sub_rate / 2) / (1 - cfg.sub_rate / 2) - 1),
                     cfg.sub_rate * prev.asset.sub_notes.balance)
                + cfg.fee_amort - inputs.interest_income))
        - (curve_value("dna", time.date)
           + if(time.t <= 3,
                prev.asset.sub_notes.balance * ((1 + cfg.sub_rate / 2) / (1 - cfg.sub_rate / 2) - 1),
                0.0)
           + cfg.fee_amort
           + curve_value("delta_nwc", time.date)
           - curve_value("capex", time.date)))
       / (1 - (1 - inputs.tax_rate) * (curve_value("libor", time.date) + cfg.tlb_spread) / 2))
}

// --- The sponsor's cash flows ------------------------------------------------
// Two points: the cheque at close, and the proceeds at exit. MoIC and IRR fall
// out of them, which is what the reference publishes for all three cases.

stream opco.sponsor.investment on entity asset.target outflow currency USD {
  schedule on 2016-01
  category investing.acquisition.purchase
  amount = cfg.sponsor_equity
}

// Exit enterprise value less net debt is the equity; the sponsor's preferred
// converts one-for-one at this exit level, so sponsor and management divide it
// in proportion to what each put in.
stream opco.sponsor.proceeds on entity asset.target inflow currency USD {
  schedule on 2021-01
  category investing.disposal.proceeds
  amount = (inputs.exit_multiple * curve_value("ebitda", time.date)
            - (asset.tlb.balance + cfg.senior_size + asset.sub_notes.balance
               - inputs.cash_at_exit))
           * cfg.sponsor_equity / (cfg.sponsor_equity + inputs.rollover)
}
```

## opco/lbo_option_pool_exit

A leveraged buyout's exit waterfall, splitting proceeds between an accruing preferred, rolled-over management equity and a laddered management option pool.

| | |
|---|---|
| Pack | `opco` |
| Declared | seven options, two states, four native streams |
| Language features | options with an exercise test and a payoff; declared state read by an option guard |
| Conventions | a preferred accruing 8% and converting one-for-one, a management rollover, laddered option strikes, dilution at exit |

Two tranches are out of the money, so the case asserts a non-exercise as well
as an exercise.

```cfdl
// A sponsor LBO's exit waterfall: an accruing convertible preferred, a
// management rollover, and a seven-tranche management stock option pool.
//
// COMPANION TO lbo_circular_interest, AND THE OTHER KIND OF CIRCULARITY. That
// case showed the debt schedule's loop is LINEAR, so it collects into a closed
// form. This one is the case that case's notes said was out of reach: a
// DISCRETE fixed point.
//
// An option tranche is exercised if it is in the money — if the exit
// consideration per share exceeds its strike. But exercising a tranche adds
// both its strike proceeds and its shares to the pool, which MOVES the value
// per share. So which options exercise depends on the value per share, and the
// value per share depends on which options exercise. There is no algebra that
// collects this: the unknown is a SET, not a number.
//
// IT IS STILL CLOSED, BECAUSE THE STRIKES ARE ORDERED. Any exercising set must
// be a prefix of the tranches sorted by strike — if a $20.00 option is in the
// money then so is every cheaper one. That reduces the search from 2^7 subsets
// to 8 candidate prefixes, and exactly one of them is self-consistent. So the
// fixed point is resolved by a finite ordered test rather than by iterating:
//
//     V(j) = (exit equity + cumulative strike proceeds through j)
//            / (preferred shares + rollover shares + cumulative option shares)
//
//     take the largest j whose own strike is below its own V(j)
//
// which is the `if` chain on `value_per_share` below. Verified against all six
// published exit multiples; the consistent prefix is unique at each one.
//
// This is a real structure, not a teaching abstraction: a management option
// pool struck above the sponsor's entry price is how nearly every sponsor deal
// pays management, and the strikes are laddered precisely so that later
// tranches only pay in better outcomes.
//
// Modeled at the 8.0x exit multiple — the same multiple the deal was entered
// at, so it is the case where the sponsor's return comes from deleveraging and
// growth rather than from multiple expansion. The other five published columns
// are reconciled in NOTES.md.

version 0.1
model "lbo-option-pool-exit"
use pack "opco" version "0.1.0"

// A single exit period. Everything here is struck at one instant — this is a
// waterfall, not a cash flow schedule, and the schedule is the other case.
time calendar annual from 2021-01 for 1

// Cash the option pool returns at exit — strike proceeds on exercised options,
// stated by the source's option-pool schedule (the 44.500 column).
assume option_pool_proceeds = 44.500

entity asset target : OpCo.Asset.Enterprise {
  // Both quantities describe the enterprise AT EXIT, so they hang on it rather
  // than floating as model variables. `value_per_share` is what every option's
  // exercise test reads.
  exit_equity init 575.6158451632398

  value_per_share init if(25.00 < (575.6158451632398 + 103.875) / 32.203133120640006,
          (575.6158451632398 + 103.875) / 32.203133120640006,
          if(22.50 < (575.6158451632398 + 72.625) / 30.953133120640006,
          (575.6158451632398 + 72.625) / 30.953133120640006,
          if(20.00 < (575.6158451632398 + inputs.option_pool_proceeds) / 29.703133120640006,
          (575.6158451632398 + inputs.option_pool_proceeds) / 29.703133120640006,
          if(17.50 < (575.6158451632398 + 29.500) / 28.953133120640006,
          (575.6158451632398 + 29.500) / 28.953133120640006,
          if(15.00 < (575.6158451632398 + 20.750) / 28.453133120640006,
          (575.6158451632398 + 20.750) / 28.453133120640006,
          if(14.00 < (575.6158451632398 + 13.250) / 27.953133120640006,
          (575.6158451632398 + 13.250) / 27.953133120640006,
          if(12.50 < (575.6158451632398 + 6.250) / 27.453133120640006,
          (575.6158451632398 + 6.250) / 27.453133120640006,
          575.6158451632398 / 26.953133120640006)))))))
}

// ---------------------------------------------------------------------------
// Exit. LTM adjusted EBITDA at the end of the five-year hold, at 8.0x, less
// net debt carried out of the debt schedule.
// ---------------------------------------------------------------------------
assume exit_ebitda      = 119.29345470000001
assume exit_multiple    = 8.0
assume net_debt         = 378.7317924367603

// Sponsor preferred: $158.9375m at $10.00/share, accruing 8% for five years,
// converting one-for-one. 158.9375 * 1.08^5 = 233.53133.
assume preferred_shares = 23.353133120640006

// Management rollover: $36m at $10.00/share.
assume rollover_shares  = 3.6


// ---------------------------------------------------------------------------
// The option pool. Seven tranches, laddered by strike ($mm of proceeds and
// millions of shares).
//
//   strike   shares   cumulative shares   cumulative proceeds
//   12.50     0.50           0.50                 6.250
//   14.00     0.50           1.00                13.250
//   15.00     0.50           1.50                20.750
//   17.50     0.50           2.00                29.500
//   20.00     0.75           2.75                44.500
//   22.50     1.25           4.00                72.625
//   25.00     1.25           5.25               103.875
// ---------------------------------------------------------------------------

// Exit equity value, before any option proceeds.


// The resolved value per share.
//
// Walks the prefixes from the largest down and takes the first one that is
// self-consistent — the first j whose own strike sits below the value per
// share that exercising through j would produce. Descending order is what
// makes "largest consistent j" fall out of a plain `if` chain.
//
// The denominators are (26.953133120640006 + cumulative option shares), where
// 26.953133 is the preferred plus rollover shares that exist regardless.

// ---------------------------------------------------------------------------
// The options themselves. Each tests its own strike against the resolved value
// per share — the economically real test, and now expressible directly:
// `exercise when` reads `asset.target.value_per_share`, the value the model derives
// above, rather than a constant restated for the engine's benefit.
//
// The payoff is the tranche's intrinsic value at exit: shares * (value - strike).
// ---------------------------------------------------------------------------

option mgmt_options_12_50 type OpCo.Contract.EquityOption {
  exercise when asset.target.value_per_share > 12.50
  payoff 0.50 * (asset.target.value_per_share - 12.50)
}

option mgmt_options_14_00 type OpCo.Contract.EquityOption {
  exercise when asset.target.value_per_share > 14.00
  payoff 0.50 * (asset.target.value_per_share - 14.00)
}

option mgmt_options_15_00 type OpCo.Contract.EquityOption {
  exercise when asset.target.value_per_share > 15.00
  payoff 0.50 * (asset.target.value_per_share - 15.00)
}

option mgmt_options_17_50 type OpCo.Contract.EquityOption {
  exercise when asset.target.value_per_share > 17.50
  payoff 0.50 * (asset.target.value_per_share - 17.50)
}

option mgmt_options_20_00 type OpCo.Contract.EquityOption {
  exercise when asset.target.value_per_share > 20.00
  payoff 0.75 * (asset.target.value_per_share - 20.00)
}

// Out of the money at 8.0x: the value per share resolves to $20.88, below both
// remaining strikes. Included precisely so the case asserts a NON-exercise as
// well as an exercise — an option model that only ever fires is not tested.
option mgmt_options_22_50 type OpCo.Contract.EquityOption {
  exercise when asset.target.value_per_share > 22.50
  payoff 1.25 * (asset.target.value_per_share - 22.50)
}

option mgmt_options_25_00 type OpCo.Contract.EquityOption {
  exercise when asset.target.value_per_share > 25.00
  payoff 1.25 * (asset.target.value_per_share - 25.00)
}

// ---------------------------------------------------------------------------
// The reported lines.
// ---------------------------------------------------------------------------

// Total cash to shareholders: exit equity plus the strike proceeds the
// exercised tranches pay in.
stream opco.exit.equity_value on entity asset.target inflow currency USD {
  schedule every year from 2021-01 to 2021-01
  category investing.disposal.proceeds
  amount = asset.target.exit_equity
}

stream opco.exit.option_proceeds on entity asset.target inflow currency USD {
  schedule every year from 2021-01 to 2021-01
  category investing.disposal.proceeds
  amount = 44.500
}

// THE SPLIT, AS A WATERFALL. Total cash to shareholders is exit equity plus
// the strike proceeds the exercised tranches pay in. The sponsor takes its
// converted preferred shares at the resolved value; management takes what is
// left, which is its rollover plus the exercised option shares at the same
// value.
//
// WHY A WATERFALL AND NOT TWO STREAMS. Written as two independent amounts,
// nothing checked that they add up to the cash available — both could be wrong
// together and every gate would pass. As a waterfall the adding-up is
// structural: management's step is `remaining`, so the two exhaust the pot by
// construction, and an error in the sponsor's share moves management's figure
// away from the published one instead of hiding.
//
// It is gross of the strikes paid in: those are already inside the pot, so
// netting them here would double-count.
entity party sponsor : OpCo.Party.Sponsor    { name = "Sponsor" }
entity party mgmt    : OpCo.Party.Management { name = "Management" }

waterfall opco.exit on entity asset.target {
  schedule on 2021-01
  from asset.target.exit_equity + inputs.option_pool_proceeds

  pay sponsor_proceeds    to party.sponsor = 23.353133120640006 * asset.target.value_per_share
  pay management_proceeds to party.mgmt    = remaining
}
```

## opco/damodaran_fcff

A free cash flow to firm valuation following Damodaran's published method, with reinvestment driven by growth and return on capital.

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.capex_line`, `opco.cash_taxes` |
| Declared | two curves |
| Language features | pack contracts driven by curves; declared state inside the pack's growth rules |
| Conventions | a declining growth path, margin-driven operating expense, cash taxes, capital expenditure as a share of revenue |

The reference publishes the **drivers** rather than only the results, which is
what a pack rule consumes, so the pack's lowering is checked and not only the
engine's arithmetic.

```cfdl
// Damodaran's FCFF Simple Ginzu — the reference implementation of textbook
// intrinsic valuation, and the first opco case built from PACK CONTRACTS.
//
// WHY THIS SOURCE. benchmarks/opco/banker_dcf_conventions reconciles a banker's
// DCF, but that filing publishes the RESULT — per-year unlevered cash flow — so
// the model had to hand-write six native streams and validated the engine's
// discounting rather than the pack. This source publishes the DRIVERS: revenue
// growth, operating margin, tax rate and a sales-to-capital ratio, and every
// line they produce. That is what a pack rule consumes, so this case is built
// entirely from opco contracts and takes the pack off 0-of-10.
//
// THE DRIVERS CONVERGE, which is the whole character of intrinsic valuation:
// growth decays toward the riskfree rate and the effective tax rate climbs
// toward the marginal one as the firm matures. Both paths below are DERIVED
// from the stated inputs (5% -> 4.58%, 17.5% -> 25%, linearly over years 6-10),
// not read off the output — verified to reproduce the published growth and tax
// rows exactly.
//
// WHAT IS ASSERTED, AND WHY NOT ALL TEN YEARS. The curves carry a PER-PERIOD
// rate, which is the right interface and the one that will be correct once a
// stream can read its own prior period. Until then the rules
// compound with pow(1 + g, t), which applies one period's rate as though it had
// held throughout — exact while the rate is flat, drifting once it moves. So
// years 1-5 are asserted and years 6-10 are not; NOTES.md carries the measured
// drift, which is the delta 5.1 is expected to close.
//
// Reinvestment funds NEXT year's growth, so its exact window closes a year
// earlier than revenue's. Also asserted only where it is exact.
//
// NOT ASSERTED AT ALL: value, NPV, per-share price. The cost of capital
// converges 7.055% -> 8.81% and the engine takes a single scalar discount rate,
// so a term structure is inexpressible. Discounting at a flat rate and calling
// the result agreement would be worse than saying so.

version 0.1
model "damodaran-fcff"
use pack "opco" version "0.1.0"
time calendar annual from 2026-01 for 10

entity asset firm : OpCo.Asset.Enterprise

// Revenue growth: 5% while the firm is growing, decaying to the riskfree rate
// by the terminal year.
curve revenue_growth linear {
  2026-01: 0.0500000000
  2027-01: 0.0500000000
  2028-01: 0.0500000000
  2029-01: 0.0500000000
  2030-01: 0.0500000000
  2031-01: 0.0491600000
  2032-01: 0.0483200000
  2033-01: 0.0474800000
  2034-01: 0.0466400000
  2035-01: 0.0458000000
}

// Effective tax rate climbing to the marginal rate over the same window.
curve tax_rate linear {
  2026-01: 0.1750000000
  2027-01: 0.1750000000
  2028-01: 0.1750000000
  2029-01: 0.1750000000
  2030-01: 0.1750000000
  2031-01: 0.1900000000
  2032-01: 0.2050000000
  2033-01: 0.2200000000
  2034-01: 0.2350000000
  2035-01: 0.2500000000
}

contract opco.revenue_line.core on entity asset.firm {
  term 2026-01..2035-01
  terms {
    amount = 22853.6700000000
    growth_rate = curve_value("revenue_growth", time.date)
  }
}

// Operating margin is flat at 14.063%, so operating cost is the complement of
// revenue and follows the same path.
contract opco.opex_line.operating on entity asset.firm {
  term 2026-01..2035-01
  terms {
    amount = 19639.7250000000
    growth_rate = curve_value("revenue_growth", time.date)
  }
}

// Cash taxes on EBIT. The rule reads revenue and opex from base streams and
// opex is signed negative, so their sum is EBIT; no debt and no D&A here.
contract opco.cash_taxes.federal on entity asset.firm {
  term 2026-01..2035-01
  terms {
    tax_rate = curve_value("tax_rate", time.date)
  }
}

// Reinvestment = revenue * growth / sales-to-capital, which funds NEXT year's
// growth. With a flat growth rate it is itself a geometric series on the same
// curve.
contract opco.capex_line.reinvestment on entity asset.firm {
  term 2026-01..2035-01
  terms {
    amount = 668.8079047797
    growth_rate = curve_value("revenue_growth", time.date)
  }
}
```
