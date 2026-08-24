---
id: benchmark-bespoke-buenavista-del-cobre
title: "Bespoke: open-pit copper mine"
slug: "/docs/examples/bespoke-buenavista-del-cobre"
description: "A 41-year open-pit copper mine whose production plan is derived from its reserve statement, with the pit's strip ratio drawn from a distribution and the valuation reported as a range."
source: benchmarks/bespoke/buenavista_del_cobre
---

# Bespoke: open-pit copper mine

A 41-year open-pit copper mine whose production plan is derived from its reserve statement, with the pit's strip ratio drawn from a distribution and the valuation reported as a range.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

Buenavista del Cobre is an open-pit copper mine in Sonora, Mexico. It has
operated since 1899 and is among the largest copper mines in the world. Its
operator publishes a reserve: how much rock remains, at what grade, and where
each class of rock is processed.

A mine is a set of depleting stocks. Each period draws what its processing
capacity allows, or the remainder if less, and carries the balance forward.
Mine life is therefore a result rather than an input: the mill runs until its
stock is gone.

This case derives the mine's 41-year production plan from the reserve, values
it, and compares both against what the operator published. The comparisons are
reported. Neither drives the model.

## The reference

The inputs come from the S-K 1300 Technical Report Summary for the mine,
prepared by WSP USA for Southern Copper Corporation, dated 11 February 2025 and
filed as Exhibit 96.6 to the FY2024 Form 10-K.

About twenty numbers: four reserve tonnages and their contained metal (Table
12.8), two mill capacities, the strip ratio, the head-grade policy (Table
12.5), unit costs (section 18), and prices, the discount rate and the fiscal
rates (section 19).

Two published tables are comparisons rather than inputs. The operator's
production schedule is `published_production_schedule.csv` and its discounted
cash flow is `published_grid.csv`. The model reads neither.

An independent implementation of the same claims over the same inputs produces
the expectations this case asserts.

## What it exercises

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

## The result

All 41 periods reproduce across fifteen columns, two metrics reproduce, and
five Monte Carlo aggregates reproduce, to 1e-5 against the reference.

**The derived plan against the operator's**, over the life of the mine:

| line | ours Mt | theirs Mt | difference |
|---|---:|---:|---:|
| copper mill feed | 2,104 | 2,104 | 0.0% |
| zinc mill feed | 287 | 287 | 0.0% |
| crushed leach | 1,077 | 1,080 | −0.3% |
| ROM leach | 1,041 | 1,039 | +0.2% |
| waste | 3,742 | 3,769 | −0.7% |
| contained copper, kt | 16,083 | 16,192 | −0.7% |

Capacity against a depleting stock reproduces both mill schedules exactly, year
for year. Mass balance holds contained copper to within a percent.

**The valuation against the operator's**, in US$ M:

| line | ours | theirs | difference |
|---|---:|---:|---:|
| total revenue | 79,527 | 76,951 | +3.3% |
| total operating cost | 59,167 | 57,887 | +2.2% |
| capital | 8,277 | 8,317 | −0.5% |
| after-tax NPV at 10% | 3,120 | 3,405 | −8.4% |

Over 500 trials on the strip ratio, the after-tax NPV has a mean of 3,132, a
median of 3,236, and a standard deviation of 820, ranging from 1,110 to 4,632.
The operator's 3,405 sits inside that range.

## The delta

**The plan derives; the pit sequence does not.** Both mill schedules are exact
because they are capacity against a stock, and both capacities are published.
The leach and waste lines match in total and not in shape: ours are smooth,
while the operator's leach tonnage swings between 8 and 154 million tonnes a
year. Nothing in the report describes the pit sequence that produces that
swing.

**That swing is worth a quarter of the valuation.** The strip ratio observed
across an operating pit spans 0.31 to 2.08. Drawn across that range, the
after-tax NPV has a standard deviation of 820 against a mean of 3,132. How much
waste moves in which year is the largest single uncertainty in this asset, and
it is larger than any question about metallurgy or price.

**Four recovery numbers are ours.** The report states no recovery for its cash
flow. Mill copper, leach copper, molybdenum and zinc recovery are declared with
their basis and are run-config knobs, so they can be moved without editing the
model. Across their full published ranges, molybdenum and zinc together move
the valuation by less than 50; leach copper moves it by up to 2,592.

**The price is the operator's.** Section 19.1 records that the deck of
US$3.30 per pound of copper was provided by the operator. The Wood Mackenzie
market study the same report contains averages US$3.87 per pound over its
published years. Moving to that study's own base case raises the valuation by
about 2,589.

## What the case does not claim

The additional royalty on precious-metal receipts is not modeled: this mine's
published revenue carries only copper, molybdenum and zinc. The market price
curves are not used, because they cover ten and five years of a 41-year life.
Working capital is not modeled, because the stated day counts net to zero over
the life. The pit sequence is not modeled, because the report does not describe
it; its effect is measured instead.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.1},"monte_carlo":{"trial_count":500,"seed":20250211}}
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
  category operating.tax
  amount = inputs.duty_rate * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t))
}

stream mine.fiscal.profit_share on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.tax
  amount = inputs.ptu_rate
             * max(0.0, (1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t))
                        - inputs.capital_lom * (min(if(time.t <= 10, inputs.cap_cu_mill_full, inputs.cap_cu_mill_reduced), asset.cu_mill.tonnes) + min(inputs.cap_zn_mill, asset.zn_mill.tonnes) + min(inputs.rate_crushed_leach, asset.crushed.tonnes) + min(inputs.rate_rom_leach, asset.rom.tonnes)) / inputs.reserve_ore_total)
}

stream mine.fiscal.income_tax on entity asset.cu_mill outflow currency USD {
  schedule every year start from 2025-01 to 2065-01
  category operating.tax
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

## Run configuration

```json
{
  "deterministic": { "annual_discount_rate": 0.10 },
  "monte_carlo": { "trial_count": 500, "seed": 20250211 }
}
```

## Verified results

Checked period by period: **15 series** across **41 periods** — **615 values** in all, each within ±0.00001 of the reference.

- `mine.revenue.copper`
- `mine.revenue.molybdenum`
- `mine.revenue.zinc`
- `mine.opex.mining`
- `mine.opex.processing`
- `mine.opex.selling`
- `mine.opex.gna`
- `mine.opex.accretion`
- `mine.fiscal.duty`
- `mine.fiscal.profit_share`
- `mine.fiscal.income_tax`
- `mine.noncash.accretion_addback`
- `mine.capital.sustaining`
- `mine.capital.closure`
- `net_cash_flow`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 3,119.75 | ±0.0001 |
| `stream.mine.capital.sustaining.total` | -8,276.62 | ±0.0001 |
