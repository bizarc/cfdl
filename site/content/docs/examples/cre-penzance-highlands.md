---
id: benchmark-cre-penzance-highlands
title: "CRE: mixed-use development with a construction facility"
slug: "/docs/examples/cre-penzance-highlands"
description: "A 160-month ground-up CRE development: land in 2011, a 39-month build on a parabolic draw curve, a $380M facility that funds equity first and capitalizes interest, two rental towers sold in lease-up, and a 34-month condominium sellout."
source: benchmarks/cre/penzance_highlands
---

# CRE: mixed-use development with a construction facility

A 160-month ground-up CRE development: land in 2011, a 39-month build on a parabolic draw curve, a $380M facility that funds equity first and capitalizes interest, two rental towers sold in lease-up, and a 34-month condominium sellout.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A ground-up mixed-use development in Rosslyn, Virginia — The Highlands, by
Penzance with The Baupost Group, on Arlington County site plan **SP #445**. Land
was bought in September 2011, construction ran 39 months, and the deal exited
in three pieces: two rental towers sold on one day in May 2022 and a
condominium sold unit by unit over the 34 months to June 2024.

Two towers sit over one shared podium and carry **both for-sale and rental
product on a single construction basis**:

| | Units | Tenure | Exit |
|---|---|---|---|
| East / North Tower — Pierce | 104 | for sale | 102 recorded closings |
| East / South Tower — Evo | 455 | rental | $334,642,240 |
| West — Aubrey | 331 | rental | $266,455,000 |

Seven of the twelve parcels are **ground-leased from the County**, not owned,
and the site carries in-kind public obligations — a fire station, a public park
and a new public street — that are pure cost with no revenue.

## The reference

Public record, and an independent spreadsheet implementation built from it.
Both read the same frozen input set, so they tie by construction rather than by
transcription.

Fact, from Arlington County: the program, unit mix by tower, GFA, parking, FAR
and the quantified public obligations (site plan SP #445, 2017 approval with
2018 and 2020 amendments); the land basis of **$67,000,000** recorded
2011-09-30 and all 102 condominium closings (assessment roll); both tower sale
prices and the operating parameters — 5.15% guideline loaded cap, $7,511/unit
expenses, 8% vacancy, $150/space parking (Commercial Guidebook, 2022 and 2023).

Rents are **derived, not assumed**: each tower's 1/1/2022 assessed value times
the guideline loaded cap gives the assessor's own NOI, solved back to
$3,522/unit for Aubrey and $3,273/unit for Evo.

Assumed: construction cost, debt pricing, lease-up pace, and the JV tiers. The
Penzance/Baupost terms are private, so the tier percentages here are stated
placeholders rather than the real split.

The distribution is a **once-at-end** waterfall, at the final condominium
closing in 2024-06. A development JV does not distribute while the deal is
live, so the preferred return and the capital are cumulative balances the
venture carries, per `docs/17` §10. Two consequences follow from the pot rather
than from the deal. `available` is *this period's* netted cash, which on a
once-at-end schedule is one month rather than the deal, so the pot is
`series_sum("cre.*", 0, time.t)` — the streams' own running sum. And there is no
return-of-capital tier: contributions are outflows inside those streams, so the
running sum has already recovered the capital, and what survives to the end is
profit. The preference accrues from construction start, not from the 2011 land
purchase — compounding the land for 12.75 years consumes the entire promote.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | 4 curves, 8 entities, 28 streams, 1 waterfall, 8 field recurrences |
| Language features | entity field recurrences (`init`/`next`/`prev`), `curve` lookups, a once-at-end `waterfall` drawing a cumulative pot with `series_sum`, `part of` roll-up, `start` placement |
| Conventions | equity-first funding, capitalized construction interest, a facility retired out of disposal proceeds, sale in lease-up |

The facility is five recurrences on one entity — `equity_funded`, `interest`,
`draw`, `repay`, `balance` — each reading only `prev` and the cost curves, never
a stream, which is what keeps it acyclic. It is hand-built rather than the
pack's `cre.construction_loan` contract, and so does not use that contract's
`capitalize_interest` election; the behavior is the same, the implementation
independent.

Every cost curve declares **every** period, including the zeros. A step curve is
flat-forward, so omitting the quiet months holds the last construction draw
forward for ever and the balance never stops compounding.

## The result

The facility ties to the workbook to the cent:

| | |
|---|---|
| Peak debt | 370,411,950.94 |
| Peak equity | 186,245,280.59 |
| Capitalized interest | 48,448,594.10 |
| `model.total` (levered) | 196,361,512.48 |
| `model.irr` | 0.110161 |
| `model.moic` | 2.04664 |

Interest is stated **gross** — an outflow at `financing.interest` against a
matching draw at `financing.debt_proceeds` — rather than folded silently into
the balance. The legs net to zero in cash, so the balance grows by the accrual
either way, but `domain.cre.debt_service` then sees the real interest.

The payoff is categorized `investing.reversion`, not
`financing.debt_principal`. It is retired entirely out of sale and condominium
proceeds, and `financing.debt_principal` folds into `domain.cre.debt_service`,
where a $394M bullet makes every coverage ratio in the disposal period
meaningless. The pack says the same of a permanent loan's balloon.

## The delta

**`model.npv` is deliberately not asserted.** The model carries financing
streams, so its NPV is levered and would need a cost of equity rather than a
project rate — the unlevered PV at 10% is −171,050 while the financing streams
contribute +9,229,459, so essentially all of a reported NPV would be that
artifact. And no source document for this deal states a discount rate: the site
plan record and both guidebooks publish *capitalization* rates, which value a
stabilized year rather than a stream. Run as scenarios, NPV swings from
+84,206,257 at 4.46% to −46,033,231 at 20% while `model.irr` and `model.moic`
do not move at all, both being solved from the cash flows. `one_lincoln_street`
and `hud_home_multifamily` omit `model.npv` for the same reason.

**Placement is `start` throughout.** Every recurring schedule here is
expense-like — construction capex, operating revenue and opex, funding draws,
condominium closings — which `12_payment_timing.md` §6 places at the period's
open. It is also what makes the case tie: at the `end` default the model returns
an IRR of 11.1454% against the workbook's 11.0161%. The totals are identical
either way, because a sum does not care where inside a period its cash sits,
which is why the per-period series are asserted alongside the metrics.

**Both towers sold in lease-up.** Delivery is mid-2021 and the recorded exits
are May 2022, so on any plausible pace neither tower had stabilized. The
model carries the ramp rather than a stabilized year.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"as_of":"2011-09-01"}}
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
// The for-sale and rental product share a basis, which is what makes this
// deal worth modeling rather than a generic development.
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
// the facility's recurrence can read them: `next` sees curves, but never a
// stream (docs/03 §3.1), and cost here is a pure function of time.
//
// EVERY period is declared, including the zeros. A step curve is
// flat-forward (docs/03 §4): omit the quiet months and the last construction
// draw is held forever, which compounds into a balance that never stops.
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
entity asset project : CRE.Asset.Portfolio
entity asset east : CRE.Asset.RealProperty { asset_class = "mixed_use"  part of asset.project }
entity asset west : CRE.Asset.RealProperty { asset_class = "multifamily"  part of asset.project }

entity party penzance : CRE.Party.Sponsor  { name = "Penzance" }
entity party baupost  : CRE.Party.Investor { name = "The Baupost Group" }
entity party mack     : CRE.Party.Lender   { name = "Mack Real Estate Credit Strategies" }

// ------------------------------------------------------------------ capital
stream cre.land on entity asset.project outflow currency USD {
  schedule on 2011-09
  category investing.capital.capex
  amount = 67000000.00
}


stream cre.hard_buildings on entity asset.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 301693050.00 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.hard_garage on entity asset.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 33184000.00 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.fire_station on entity asset.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 7454800.00 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.park_and_street on entity asset.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 14000000.00 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.soft_costs on entity asset.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 60576414.50 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.contingency on entity asset.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 17816592.50 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}

stream cre.developer_fee on entity asset.project outflow currency USD {
  schedule every month start from 2018-04 to 2021-06
  category investing.capital.construction
  amount = 17994713.10 * (time.t - 78.0) * (118.0 - time.t) / 10660.0
}


// SP #445 conditions: utility undergrounding and TDM at permit; public art,
// green building and affordable housing at certificate of occupancy.
stream cre.obligations_permit on entity asset.project outflow currency USD {
  schedule on 2018-04
  category investing.capital.construction
  amount = 157303.00
}

stream cre.obligations_co on entity asset.project outflow currency USD {
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
  category investing.reversion
  amount = 266455000.00
}

stream cre.evo_sale on entity asset.east inflow currency USD {
  schedule on 2022-05
  category investing.reversion
  amount = 334642240.00
}

stream cre.sale_costs on entity asset.project outflow currency USD {
  schedule on 2022-05
  category investing.selling_costs
  amount = 601097240.00 * inputs.cost_of_sale
}

stream cre.pierce_closings on entity asset.east inflow currency USD {
  schedule every month start from 2021-08 to 2024-06
  category investing.reversion
  amount = curve_value("pierce_sellout", time.date) * (1.0 - inputs.condo_selling_cost)
}

// Cash available to repay the facility: condo closings net of selling costs,
// plus the two tower sales net of cost of sale. Every period is declared, so
// the step curve cannot hold a value forward into a quiet month.
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

// Interest is capitalized, so it is never a cash line -- it is repaid inside
// the principal repayment. That is the convention the reference workbook uses.
stream cre.loan_draw on entity asset.project inflow currency USD {
  schedule every month start from 2011-09 to 2024-12
  category financing.debt_proceeds
  amount = asset.facility.draw
}

// Interest capitalizes: the facility funds its own accrual, so the two legs
// net to zero in cash while the balance grows. Stated GROSS rather than folded
// into the balance silently, so `domain.cre.debt_service` sees real interest
// and coverage during the build is measurable instead of absent.
stream cre.loan_interest on entity asset.project outflow currency USD {
  schedule every month start from 2011-09 to 2024-12
  category financing.interest
  amount = asset.facility.interest
}

stream cre.loan_interest_funding on entity asset.project inflow currency USD {
  schedule every month start from 2011-09 to 2024-12
  category financing.debt_proceeds
  amount = asset.facility.interest
}

// The payoff sits in the reversion. `financing.debt_principal` folds into
// `domain.cre.debt_service`, and a balance retired out of sale proceeds is not
// debt service — it would make every coverage ratio in the disposal period
// meaningless. The cre pack says the same of a permanent loan's balloon.
stream cre.loan_repayment on entity asset.project outflow currency USD {
  schedule every month start from 2011-09 to 2024-12
  category investing.reversion
  amount = asset.facility.repay
}

// ------------------------------------------------------------ the JV capital
// A development JV does not distribute while the deal is live. Cash accrues to
// the venture and is split once, when the last unit closes, so the preference
// and the capital are CUMULATIVE quantities carried as balances rather than
// re-derived at the distribution -- 17_ordered_waterfall.md section 10.
//
// Both partners fund pro rata and nothing is returned before the split, so the
// two balances only grow. Their difference is the accrued preference.
//
// The preference accrues from CONSTRUCTION START, not from the 2011 land
// purchase: the venture is formed to build, and the land it is capitalized
// with earns nothing for the seven years before there is anything to build.
// Compounding that $67M from 2011 instead consumes the entire promote, which
// is how the assumption announced itself.
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

// -------------------------------------------------------------- the JV split
// Penzance / Baupost terms are not public; these tiers are stated assumptions.
//
// `available` is THIS period's netted cash, so on a once-at-end waterfall it
// would draw only the final month. The pot is the streams' own running sum
// since inception instead -- cumulative net cash, after every cost, every draw
// and the facility payoff. The waterfall is named outside the `cre.` prefix so
// the selector cannot match its own steps (E1342).
//
// There is no return-of-capital tier, and that is a property of the pot rather
// than of the deal. Contributions are outflows inside the streams, so a running
// sum of them has ALREADY recovered the capital: what survives to the end is
// profit. Paying capital back out of profit would recover it a second time and
// leave nothing for the promote -- which is exactly what it did.
waterfall jv.distribution on entity asset.project {
  schedule on 2024-06 end
  from series_sum("cre.*", 0, time.t)

  pay preferred_inv to party.baupost  = (asset.jv.unreturned - asset.jv.capital) * (1.0 - inputs.sponsor_share)
  pay preferred_sp  to party.penzance = (asset.jv.unreturned - asset.jv.capital) * inputs.sponsor_share
  pay promote       to party.penzance = remaining * 0.20
  pay residual_inv  to party.baupost  = remaining * (1.0 - inputs.sponsor_share)
  pay residual_sp   to party.penzance = remaining
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.10,
    "as_of": "2011-09-01"
  }
}
```

## Verified results

Checked period by period: **8 series** across **160 periods** — **1162 values** in all, each within the tolerance shown.

- `net_cash_flow` — within ±0.01
- `domain.cre.debt_service` — within ±0.01
- `domain.cre.dscr` — within ±1.0e-6
- `domain.cre.egi` — within ±0.01
- `domain.cre.leasing_costs` — within ±0.01
- `domain.cre.noi` — within ±0.01
- `domain.cre.opex_total` — within ±0.01
- `domain.cre.pgr` — within ±0.01

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.total` | 196,361,512.48 | ±1 |
| `model.irr` | 0.110161 | ±0.00001 |
| `model.moic` | 2.04664 | ±0.0001 |
| `domain.cre.debt_service` | 48,448,594.1 | ±1 |
| `entity.party.baupost.total` | 160,851,859.44 | ±1 |
| `entity.party.penzance.total` | 35,509,653.04 | ±1 |
