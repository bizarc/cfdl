---
id: benchmark-bespoke-buenavista-del-cobre
title: "Bespoke: open-pit copper mine"
slug: "/docs/examples/bespoke-buenavista-del-cobre"
description: "A 41-year open-pit copper mine, modeled from the operator's published production schedule, unit costs and fiscal rates, and compared against the discounted cash flow the operator published."
source: benchmarks/bespoke/buenavista_del_cobre
---

# Bespoke: open-pit copper mine

A 41-year open-pit copper mine, modeled from the operator's published production schedule, unit costs and fiscal rates, and compared against the discounted cash flow the operator published.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

Buenavista del Cobre is an open-pit copper mine in Sonora, Mexico. It has
operated since 1899 and is among the largest copper mines in the world. Its
operator publishes a 41-year plan: what rock is moved each year, at what grade,
what it costs to move and treat, and what the resulting cash is worth.

This case does not reproduce that cash flow. It takes the operator's inputs,
states our own claims about how the asset behaves, and produces our own
statement. The operator's answer is then a comparison, and the difference is
the finding.

The claims are ordinary. A mine produces metal, which varies by year and splits
between three products. Each sells at its own price. Some costs scale with rock
moved, some with rock milled, some with metal sold, and some are fixed. What is
left is EBITDA. Depreciation, a mining duty, an employee profit share and
income tax take EBITDA to net income.

## The reference

The inputs come from the S-K 1300 Technical Report Summary for the mine,
prepared by WSP USA for Southern Copper Corporation, dated 11 February 2025 and
filed as Exhibit 96.6 to the FY2024 Form 10-K. The 41-year production schedule
is Table 13.3, transcribed as `published_production_schedule.csv`. Unit costs
are section 18, prices and the discount rate are section 19.1, and the fiscal
rates are section 19.2.

The comparison target is Table 19.1, the operator's own discounted cash flow,
transcribed as `published_grid.csv`. No part of the model consults it.

An independent implementation of the same claims over the same inputs produces
the expectations this case asserts, so the check is between two implementations
rather than against the operator's answer. The comparison to that answer is
reported below and is deliberately not asserted: it moves whenever the recovery
assumptions move, and pinning it would turn a finding into a target.

## What it exercises

| | |
|---|---|
| Pack | none — written from the bare language |
| Entities | one real asset, carrying its own lifecycle and its one memory |
| Language features | streams reading the period's result through `series_sum`, open-world lifecycle events with published transitions, declared phases, a carryforward recurrence, run-config knobs driving scenarios, a two-file model |
| Conventions | duty on EBITDA, profit share on EBITDA net of depreciation and duty, income tax net of a duty credit, loss carryforward, first year undiscounted |

The second case in the suite written without a pack, after
`ppiaf_toll_highway`. A mine fits none of the four: no generation and no
offtaker, no rent roll, and no pool of obligors. Its revenue is contained metal
at a price, not a margin on sales.

**The model separates data from claims.** `inputs.cfdl` holds the published
physical drivers and nothing else — eleven curves of tonnes, grades and
capital, with a header stating that nothing in the file is a modeling choice.
`model.cfdl` holds the claims: the rates, the streams, the lifecycle and the
fiscal stack.

**EBITDA is a result, not an input.** The fiscal charges read it from the
period's realized streams through `series_sum`. Cross-stream reads are one hop
deep, so each charge derives from EBITDA in closed form rather than chaining
off another charge.

**Four recovery numbers are ours.** The report states no recovery for its cash
flow. They are run-config knobs rather than constants, so the case's declared
uncertainty is explorable rather than buried, and scenarios walk each to its
alternative published basis.

## The result

All 41 periods reproduce across sixteen columns, and three metrics reproduce,
to 1e-5 against the reference.

Against the operator's own statement, life of mine, in US$ M:

| line | ours | theirs | difference |
|---|---:|---:|---:|
| Total revenue | 79,937 | 76,951 | +3.9% |
| Total operating cost | 56,398 | 57,887 | −2.6% |
| EBITDA | 23,539 | 19,062 | +23.5% |
| Income tax | 3,105 | 2,415 | +28.6% |
| Capital | 8,317 | 8,317 | 0.0% |
| **After-tax NPV at 10%** | **3,689** | **3,405** | **+8.3%** |

Revenue lands within 4% and operating cost within 3% of a statement built by
the operator's own consultants from the same physical plan. Capital matches
exactly, because it is a published total apportioned by material moved.

## The delta

**The cost side reconstructs; the revenue side does not.** Mining, processing
and general costs all fall within a few percent of the published lines, using
nothing but published unit rates and published tonnages. That is evidence the
report discloses enough to rebuild what it costs to run this mine.

**Copper is the whole difference, and its levers are not equal.** Moving each
input across its full published range moves our after-tax NPV by:

| lever | move |
|---|---:|
| copper price, US$3.30 to the market study's US$3.87 | +2,589 |
| leach recovery, 26% to the secondary-zone chemistry of 57% | +2,592 |
| leach recovery, 26% to the mixed-zone floor of 36% | +838 |
| mill recovery, 83.6% to 78.3% | −666 |
| payability, 96.7% to 95% | −226 |
| molybdenum and zinc recovery, across their whole published ranges | under 50 |

Molybdenum and zinc do not matter. Mill recovery matters modestly. **Price and
leach recovery each move the valuation by roughly US$2.6 bn**, and both are
choices rather than measurements.

**Two unstated judgments carry the case.** The price deck of US$3.30 per pound
was, in the report's own words, "provided by SCC" — the operator — while the
Wood Mackenzie market study the same report contains averages US$3.87 over its
published years. And the leach circuit treats 35% of the contained copper at a
recovery the report never states; the soluble-species chemistry in Table 11.7
implies 36% to 57%, while the operator's economics imply materially less.

Our model is 8.3% above the operator's valuation. Set leach recovery to what
the published chemistry supports and the difference grows rather than closes.
Take the market study's own price and it grows further. So the difference is
not arithmetic. The operator's valuation rests on a price below its own market
study and a leach recovery below its own ore chemistry, and the report explains
neither.

## What the case does not claim

The 0.5% additional royalty on precious-metal receipts, confirmed in the parent
Form 10-K, is not modeled: this mine's published revenue carries only copper,
molybdenum and zinc. The market price curves are not used, because Table 16.2
runs to 2034 and Table 16.4 to 2029 against a mine that runs to 2065, and
extending them would mean inventing three decades. Working capital is not
modeled, since the stated day-counts net to zero over the life. The annual
capital programme is published only as life-of-mine totals, so it is
apportioned by material moved.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.rec_cu_mill":0.836,"cfg.rec_cu_leach":0.26,"cfg.rec_mo":0.66,"cfg.rec_zn":0.629}},"scenarios":{"lever_0":{"parameters":{"cfg.price_cu":3.87,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.rec_cu_mill":0.836,"cfg.rec_cu_leach":0.26,"cfg.rec_mo":0.66,"cfg.rec_zn":0.629}},"lever_1":{"parameters":{"cfg.price_cu":2.8,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.rec_cu_mill":0.836,"cfg.rec_cu_leach":0.26,"cfg.rec_mo":0.66,"cfg.rec_zn":0.629}},"lever_2":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.rec_cu_mill":0.783,"cfg.rec_cu_leach":0.26,"cfg.rec_mo":0.66,"cfg.rec_zn":0.629}},"lever_3":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.rec_cu_mill":0.838,"cfg.rec_cu_leach":0.26,"cfg.rec_mo":0.66,"cfg.rec_zn":0.629}},"lever_4":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.rec_cu_mill":0.836,"cfg.rec_cu_leach":0.36,"cfg.rec_mo":0.66,"cfg.rec_zn":0.629}},"lever_5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.rec_cu_mill":0.836,"cfg.rec_cu_leach":0.57,"cfg.rec_mo":0.66,"cfg.rec_zn":0.629}},"lever_6":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.rec_cu_mill":0.8209519999999999,"cfg.rec_cu_leach":0.25532,"cfg.rec_mo":0.66,"cfg.rec_zn":0.629}}}}
// Buenavista del Cobre — a 41-year copper mine, modeled from the operator's
// published inputs rather than from its published answer.
//
// THE CLAIM. A mine produces metal, which varies by year and splits between
// three products. Each sells at its own price. Some costs scale with rock
// moved, some with rock milled, some with metal sold, and some are fixed.
// What is left is EBITDA. Depreciation, a mining duty, an employee profit
// share and income tax take EBITDA to net income. That is the whole model.
//
// WHAT IS AN INPUT AND WHAT IS OURS. The published physical drivers live in
// `inputs.cfdl` and nothing there is a modeling choice: tonnes and grades from
// Table 13.3 of the S-K 1300 technical report, and the capital programme from
// Table 18.1. Every rate below is a published unit cost or a stated fiscal
// rate. The four RECOVERY numbers are ours -- the report states no recovery
// for its cash flow -- so they are run-config knobs, declared with their basis
// and varied by scenario rather than buried as constants.
//
// EBITDA IS A RESULT. The fiscal charges read it from the period's realized
// streams through series_sum; it is never authored. Cross-stream reads are one
// hop deep, so each charge derives from EBITDA in closed form.

version 0.1
model "buenavista-del-cobre"
time calendar annual from 2025-01 for 41

import "inputs.cfdl"

phase full_rate from 2025-01 to 2035-12
phase reduced_plant from 2036-01 to 2060-12
phase reclamation from 2061-01 to 2065-12

// --- prices, section 19.1 (the owner's deck) -------------------------------
// Prices and recoveries are run-config knobs; base values are in run.json.
// Prices: section 19.1, the owner's deck of US$3.30 / 10.00 / 1.15 per lb.

// --- RECOVERY TO PAYABLE — the four numbers the report does not state ------
// Mill copper: the flotation circuit's tailings grade is near-constant across
// the three published operating years (0.0678, 0.0713, 0.0718 % Cu), giving
// recovery (g - 0.0703)/g at the plan's grades, times 96.7% payability.
assume rec_cu_mill  = 0.836
// Leach copper: the stated rule recovers 95% of acid-soluble and 65% of
// cyanide-soluble copper. Mixed and secondary zone chemistry (Table 11.7)
// implies 36-61%; the operation's own economics imply materially less. Held
// at the low end pending disclosure.
assume rec_cu_leach = 0.260
// Molybdenum: Table 14.1 basic design parameter, 66.0%, at 100% payability.
assume rec_mo       = 0.660
// Zinc: 74% from the resource statement, times 85% payability.
assume rec_zn       = 0.629

// --- unit costs, section 18 ------------------------------------------------
assume cost_mining     = 2.71    // US$/t moved, life-of-mine average, T18.7
assume cost_mill       = 5.83    // US$/t milled, T18.8
assume cost_zinc_plant = 10.63   // US$/t milled, T18.9
assume cost_crush      = 0.84    // US$/t crushed, T18.8
assume cost_leach      = 0.40    // US$/t leached, T18.8
assume cost_gna        = 0.76    // US$/t milled, 18.3.1
assume accretion       = 34.0    // US$ M/yr, 18.3.3
assume closure_total   = 544.0   // US$ M, final year, 18.3.3
assume sell_cu = 0.54            // US$/lb payable, T12.4 (transport-inclusive)
assume sell_mo = 1.84
assume sell_zn = 0.40

// --- the Mexican fiscal stack, section 19.2 --------------------------------
assume duty_rate = 0.075
assume ptu_rate  = 0.10
assume tax_rate  = 0.30


// ---------------------------------------------------------------------------
// The mine's lifecycle. Its type declares no lifecycle, so the states are
// open-world: each event writes `status`, the write is published in
// deterministic.transitions, and streams gate on it with `active when`.
// The eras are the ones the report states -- Concentrator I is taken offline
// at the end of 2035, cutting ore processed by 40%, and the reclamation
// outlay falls in the last five years. One period is one year, so t=11 is
// 2036 and t=36 is 2061.
// ---------------------------------------------------------------------------

event concentrator_one_offline when time.t >= 11 {
  set entity asset.mine.status = "reduced"
}

event closure_era_opens when time.t >= 36 {
  set entity asset.mine.status = "closing"
}

entity asset mine : Asset.Real {
  // Loss carried forward, held as a negative number or zero.
  //
  // A field rule may not read a series (03 section 3.1): state sees only
  // settled things, which is what keeps recurrences free of cycles. So while
  // every stream above reads EBITDA as the period's realized result, this rule
  // cannot, and restates it from the same curves and rates instead. That
  // duplication is the price of the backward-only discipline, and it is the
  // only place in the model where a definition appears twice.
  shelter init min(0.0, (((cfg.price_cu * 2204.6 / 1000.0
                    * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                       + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                  + cfg.price_mo * 2204.6 / 1000.0 * cfg.rec_mo * curve_value("mo_contained", time.date)
                  + cfg.price_zn * 2204.6 / 1000.0 * cfg.rec_zn * curve_value("zn_contained", time.date))
                 - (inputs.cost_mining * curve_value("material_moved", time.date)
                  + inputs.cost_mill * curve_value("ore_milled", time.date)
                  + inputs.cost_zinc_plant * curve_value("ore_zinc_mill", time.date)
                  + inputs.cost_crush * curve_value("ore_crushed", time.date)
                  + inputs.cost_leach * curve_value("ore_leached", time.date)
                  + inputs.cost_gna * (curve_value("ore_milled", time.date) + curve_value("ore_zinc_mill", time.date))
                  + inputs.accretion
                  + 2204.6 / 1000.0
                    * (inputs.sell_cu * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                                         + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                       + inputs.sell_mo * cfg.rec_mo * curve_value("mo_contained", time.date)
                       + inputs.sell_zn * cfg.rec_zn * curve_value("zn_contained", time.date)))) - curve_value("depreciation", time.date)
                 - inputs.duty_rate * ((cfg.price_cu * 2204.6 / 1000.0
                    * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                       + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                  + cfg.price_mo * 2204.6 / 1000.0 * cfg.rec_mo * curve_value("mo_contained", time.date)
                  + cfg.price_zn * 2204.6 / 1000.0 * cfg.rec_zn * curve_value("zn_contained", time.date))
                 - (inputs.cost_mining * curve_value("material_moved", time.date)
                  + inputs.cost_mill * curve_value("ore_milled", time.date)
                  + inputs.cost_zinc_plant * curve_value("ore_zinc_mill", time.date)
                  + inputs.cost_crush * curve_value("ore_crushed", time.date)
                  + inputs.cost_leach * curve_value("ore_leached", time.date)
                  + inputs.cost_gna * (curve_value("ore_milled", time.date) + curve_value("ore_zinc_mill", time.date))
                  + inputs.accretion
                  + 2204.6 / 1000.0
                    * (inputs.sell_cu * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                                         + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                       + inputs.sell_mo * cfg.rec_mo * curve_value("mo_contained", time.date)
                       + inputs.sell_zn * cfg.rec_zn * curve_value("zn_contained", time.date))))
                 - inputs.ptu_rate * max(0.0, (1.0 - inputs.duty_rate) * ((cfg.price_cu * 2204.6 / 1000.0
                    * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                       + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                  + cfg.price_mo * 2204.6 / 1000.0 * cfg.rec_mo * curve_value("mo_contained", time.date)
                  + cfg.price_zn * 2204.6 / 1000.0 * cfg.rec_zn * curve_value("zn_contained", time.date))
                 - (inputs.cost_mining * curve_value("material_moved", time.date)
                  + inputs.cost_mill * curve_value("ore_milled", time.date)
                  + inputs.cost_zinc_plant * curve_value("ore_zinc_mill", time.date)
                  + inputs.cost_crush * curve_value("ore_crushed", time.date)
                  + inputs.cost_leach * curve_value("ore_leached", time.date)
                  + inputs.cost_gna * (curve_value("ore_milled", time.date) + curve_value("ore_zinc_mill", time.date))
                  + inputs.accretion
                  + 2204.6 / 1000.0
                    * (inputs.sell_cu * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                                         + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                       + inputs.sell_mo * cfg.rec_mo * curve_value("mo_contained", time.date)
                       + inputs.sell_zn * cfg.rec_zn * curve_value("zn_contained", time.date)))) - curve_value("depreciation", time.date))))
    next min(0.0, prev + (((cfg.price_cu * 2204.6 / 1000.0
                    * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                       + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                  + cfg.price_mo * 2204.6 / 1000.0 * cfg.rec_mo * curve_value("mo_contained", time.date)
                  + cfg.price_zn * 2204.6 / 1000.0 * cfg.rec_zn * curve_value("zn_contained", time.date))
                 - (inputs.cost_mining * curve_value("material_moved", time.date)
                  + inputs.cost_mill * curve_value("ore_milled", time.date)
                  + inputs.cost_zinc_plant * curve_value("ore_zinc_mill", time.date)
                  + inputs.cost_crush * curve_value("ore_crushed", time.date)
                  + inputs.cost_leach * curve_value("ore_leached", time.date)
                  + inputs.cost_gna * (curve_value("ore_milled", time.date) + curve_value("ore_zinc_mill", time.date))
                  + inputs.accretion
                  + 2204.6 / 1000.0
                    * (inputs.sell_cu * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                                         + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                       + inputs.sell_mo * cfg.rec_mo * curve_value("mo_contained", time.date)
                       + inputs.sell_zn * cfg.rec_zn * curve_value("zn_contained", time.date)))) - curve_value("depreciation", time.date)
                 - inputs.duty_rate * ((cfg.price_cu * 2204.6 / 1000.0
                    * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                       + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                  + cfg.price_mo * 2204.6 / 1000.0 * cfg.rec_mo * curve_value("mo_contained", time.date)
                  + cfg.price_zn * 2204.6 / 1000.0 * cfg.rec_zn * curve_value("zn_contained", time.date))
                 - (inputs.cost_mining * curve_value("material_moved", time.date)
                  + inputs.cost_mill * curve_value("ore_milled", time.date)
                  + inputs.cost_zinc_plant * curve_value("ore_zinc_mill", time.date)
                  + inputs.cost_crush * curve_value("ore_crushed", time.date)
                  + inputs.cost_leach * curve_value("ore_leached", time.date)
                  + inputs.cost_gna * (curve_value("ore_milled", time.date) + curve_value("ore_zinc_mill", time.date))
                  + inputs.accretion
                  + 2204.6 / 1000.0
                    * (inputs.sell_cu * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                                         + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                       + inputs.sell_mo * cfg.rec_mo * curve_value("mo_contained", time.date)
                       + inputs.sell_zn * cfg.rec_zn * curve_value("zn_contained", time.date))))
                 - inputs.ptu_rate * max(0.0, (1.0 - inputs.duty_rate) * ((cfg.price_cu * 2204.6 / 1000.0
                    * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                       + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                  + cfg.price_mo * 2204.6 / 1000.0 * cfg.rec_mo * curve_value("mo_contained", time.date)
                  + cfg.price_zn * 2204.6 / 1000.0 * cfg.rec_zn * curve_value("zn_contained", time.date))
                 - (inputs.cost_mining * curve_value("material_moved", time.date)
                  + inputs.cost_mill * curve_value("ore_milled", time.date)
                  + inputs.cost_zinc_plant * curve_value("ore_zinc_mill", time.date)
                  + inputs.cost_crush * curve_value("ore_crushed", time.date)
                  + inputs.cost_leach * curve_value("ore_leached", time.date)
                  + inputs.cost_gna * (curve_value("ore_milled", time.date) + curve_value("ore_zinc_mill", time.date))
                  + inputs.accretion
                  + 2204.6 / 1000.0
                    * (inputs.sell_cu * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                                         + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
                       + inputs.sell_mo * cfg.rec_mo * curve_value("mo_contained", time.date)
                       + inputs.sell_zn * cfg.rec_zn * curve_value("zn_contained", time.date)))) - curve_value("depreciation", time.date))))

  shelter_in init 0.0
    next prev.asset.mine.shelter
}

// ---------------------------------------------------------------------------
// Revenue: payable metal at its price. Contained metal is published in
// kilotonnes, so x 2204.6 converts to millions of pounds.
// ---------------------------------------------------------------------------

stream mine.revenue.copper on entity asset.mine inflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = cfg.price_cu * 2204.6 / 1000.0
           * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
              + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
}

stream mine.revenue.molybdenum on entity asset.mine inflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = cfg.price_mo * 2204.6 / 1000.0
           * cfg.rec_mo * curve_value("mo_contained", time.date)
}

stream mine.revenue.zinc on entity asset.mine inflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = cfg.price_zn * 2204.6 / 1000.0
           * cfg.rec_zn * curve_value("zn_contained", time.date)
}

// ---------------------------------------------------------------------------
// Cost: variable with rock moved, rock milled, rock leached, and metal sold;
// then the fixed lines.
// ---------------------------------------------------------------------------

stream mine.opex.mining on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.cost_mining * curve_value("material_moved", time.date)
}

stream mine.opex.processing on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.cost_mill * curve_value("ore_milled", time.date)
           + inputs.cost_zinc_plant * curve_value("ore_zinc_mill", time.date)
           + inputs.cost_crush * curve_value("ore_crushed", time.date)
           + inputs.cost_leach * curve_value("ore_leached", time.date)
}

stream mine.opex.selling on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = 2204.6 / 1000.0
           * (inputs.sell_cu
                * (cfg.rec_cu_mill * curve_value("cu_mill_contained", time.date)
                   + cfg.rec_cu_leach * curve_value("cu_leach_contained", time.date))
              + inputs.sell_mo * cfg.rec_mo * curve_value("mo_contained", time.date)
              + inputs.sell_zn * cfg.rec_zn * curve_value("zn_contained", time.date))
}

stream mine.opex.gna on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.cost_gna
           * (curve_value("ore_milled", time.date) + curve_value("ore_zinc_mill", time.date))
}

stream mine.opex.accretion on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.accretion
}

// ---------------------------------------------------------------------------
// The charges that take EBITDA to net income. Each reads the period's EBITDA
// as the realized result of the streams above.
// ---------------------------------------------------------------------------

stream mine.fiscal.duty on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.tax
  amount = inputs.duty_rate * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t))
}

stream mine.fiscal.profit_share on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.tax
  amount = inputs.ptu_rate
             * max(0.0, (1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t)) - curve_value("depreciation", time.date))
}

stream mine.fiscal.income_tax on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.tax
  amount = max(0.0,
               inputs.tax_rate
                 * ((1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t)) - curve_value("depreciation", time.date)
                    - inputs.ptu_rate * max(0.0, (1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t)) - curve_value("depreciation", time.date))
                    + asset.mine.shelter_in)
               - inputs.tax_rate * inputs.duty_rate * (series_sum("mine.revenue.*", time.t, time.t)
                + series_sum("mine.opex.*", time.t, time.t)))
}

// Accretion is charged above but never leaves the bank; the filing strikes its
// cash flow on ARO OUTLAYS, so it is added back.
stream mine.noncash.accretion_addback on entity asset.mine inflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = inputs.accretion
}

// ---------------------------------------------------------------------------
// Capital and closure.
// ---------------------------------------------------------------------------

stream mine.capital.sustaining on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category investing.capital.capex
  amount = curve_value("capital", time.date)
}

stream mine.capital.closure on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category investing.capital.capex
  active when entity.status == "closing"
  amount = inputs.closure_total / 5.0
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.1,
    "parameters": {
      "cfg.price_cu": 3.3,
      "cfg.price_mo": 10.0,
      "cfg.price_zn": 1.15,
      "cfg.rec_cu_mill": 0.836,
      "cfg.rec_cu_leach": 0.26,
      "cfg.rec_mo": 0.66,
      "cfg.rec_zn": 0.629
    }
  },
  "scenarios": {
    "lever_0": {
      "parameters": {
        "cfg.price_cu": 3.87,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.rec_cu_mill": 0.836,
        "cfg.rec_cu_leach": 0.26,
        "cfg.rec_mo": 0.66,
        "cfg.rec_zn": 0.629
      }
    },
    "lever_1": {
      "parameters": {
        "cfg.price_cu": 2.8,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.rec_cu_mill": 0.836,
        "cfg.rec_cu_leach": 0.26,
        "cfg.rec_mo": 0.66,
        "cfg.rec_zn": 0.629
      }
    },
    "lever_2": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.rec_cu_mill": 0.783,
        "cfg.rec_cu_leach": 0.26,
        "cfg.rec_mo": 0.66,
        "cfg.rec_zn": 0.629
      }
    },
    "lever_3": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.rec_cu_mill": 0.838,
        "cfg.rec_cu_leach": 0.26,
        "cfg.rec_mo": 0.66,
        "cfg.rec_zn": 0.629
      }
    },
    "lever_4": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.rec_cu_mill": 0.836,
        "cfg.rec_cu_leach": 0.36,
        "cfg.rec_mo": 0.66,
        "cfg.rec_zn": 0.629
      }
    },
    "lever_5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.rec_cu_mill": 0.836,
        "cfg.rec_cu_leach": 0.57,
        "cfg.rec_mo": 0.66,
        "cfg.rec_zn": 0.629
      }
    },
    "lever_6": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.rec_cu_mill": 0.8209519999999999,
        "cfg.rec_cu_leach": 0.25532,
        "cfg.rec_mo": 0.66,
        "cfg.rec_zn": 0.629
      }
    }
  }
}
```

## Verified results

Checked period by period: **16 series** across **41 periods** — **656 values** in all, each within ±0.00001 of the reference.

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
- `asset.mine.shelter`
- `net_cash_flow`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 3,689.28 | ±0.0001 |
| `stream.mine.revenue.copper.total` | 74,688.94 | ±0.0001 |
| `stream.mine.capital.sustaining.total` | -8,317 | ±0.0001 |
