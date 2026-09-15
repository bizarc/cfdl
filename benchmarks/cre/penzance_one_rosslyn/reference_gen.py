"""Write benchmarks/cre/penzance_one_rosslyn/model.cfdl from the frozen input set.

A WRITER, not a calculator. It reads inputs/one_rosslyn.toml, fills the model's
assumption block and its schedule dates, and emits the model text. Every figure
the model needs that can be calculated from another is calculated IN THE MODEL
(derived assumptions, field recurrences, stream expressions), not here: the
budget, both commitments, the draw curve, each tower's lease-up, both exits,
the permanent loan, its payment and its payoff, and every month number, which
the model counts from the stated dates with `months_between`. If a value could
be computed in the language, it does not belong in this file.

The one thing it does beyond copying is name the month AFTER a stated date
where a schedule needs it (the first monthly contribution, the first permanent
payment): a schedule clause takes a literal date.

Regenerate: python3 reference_gen.py
"""
import tomllib
from pathlib import Path
from string import Template

CASE = Path(__file__).resolve().parent
T = tomllib.load(open(CASE / "inputs" / "one_rosslyn.toml", "rb"))
PR, LAND, OB, SCH = T["program"], T["land"], T["obligations"], T["schedule"]
OP, CN, FI, CD, JV = T["operating"], T["construction"], T["finance"], T["condo"], T["jv"]


def month_after(ym):
    """The month following a stated `YYYY-MM`, for a schedule that starts then."""
    y, m = (int(x) for x in ym.split("-"))
    return f"{y + m // 12}-{m % 12 + 1:02d}"


def num(x):
    """A stated figure, printed as a number."""
    return repr(float(x))


values = {k: num(v) for k, v in dict(
    gfa_total_sf=PR["total_gfa_sf"], parking_spaces=PR["parking_spaces"],
    ne_units=PR["ne_tower"]["units"], nw_units=PR["nw_tower"]["units"], s_units=PR["s_tower"]["units"],
    land_price=LAND["purchase_price_usd"],
    ahif_base_density=OB["ahif_base_density"], public_space_rmsa=OB["public_space_rmsa"],
    transportation_rmsa=OB["transportation_rmsa"], ahif_tranche_ne=OB["ahif_tranche_ne"],
    ahif_tranche_nw=OB["ahif_tranche_nw"], ahif_tranche_s=OB["ahif_tranche_s"],
    highlands_buildings_psf=CN["highlands_buildings_psf"], highlands_parking_space=CN["highlands_parking_space"],
    escalation_realized=CN["escalation_realized"], escalation_forward_rate=CN["escalation_forward_rate"],
    escalation_years_forward=CN["escalation_years_forward"], soft_ratio=CN["soft_ratio"],
    contingency_ratio=CN["contingency_ratio"], developer_fee_ratio=CN["developer_fee_ratio"],
    construction_months=SCH["construction_months"], lease_up_months=SCH["lease_up_months"],
    condo_months=SCH["condo_sellout_months"],
    rent_per_unit=OP["rent_per_unit_month"], expenses_per_unit=OP["expenses_per_unit"],
    vacancy=OP["vacancy_collection"], parking_garage_low=OP["parking_garage_low"],
    parking_garage_high=OP["parking_garage_high"], parking_spaces_per_unit=OP["parking_spaces_per_unit"],
    growth=OP["growth"], cap_rate=OP["cap_rate_metro_highrise"],
    pierce_per_unit=CD["pierce_per_unit"], condo_selling_cost=CD["selling_cost"],
    sofr=FI["sofr"], construction_spread=FI["construction_spread"], ltc=FI["ltc"], ust10=FI["ust10"],
    perm_spread=FI["perm_spread"], perm_ltv=FI["perm_ltv"], perm_amortization_years=FI["perm_amortization_years"],
    pref_rate=JV["pref_rate"], sponsor_share=JV["sponsor_share"], promote_share=JV["promote_share"],
).items()}
values.update(
    grid_periods=str(SCH["grid_periods"]), valuation_tail=str(SCH["valuation_tail"]),
    d_grid_start=SCH["grid_start"], d_grid_second=month_after(SCH["grid_start"]),
    d_construction_start=SCH["construction_start"], d_construction_end=SCH["construction_end"],
    d_delivery_nw=SCH["delivery_nw"], d_delivery_s=SCH["delivery_s"],
    d_merchant_sale=SCH["merchant_sale"], d_condo_sellout_end=SCH["condo_sellout_end"],
    d_stabilization=SCH["stabilization"], d_first_perm_payment=month_after(SCH["stabilization"]),
    d_core_exit=SCH["core_exit"], d_operations_end=SCH["operations_end"], d_rent_vintage=SCH["rent_vintage"],
)

MODEL = Template(r'''version 0.1
model "penzance-one-rosslyn" currency USD

use pack "cre" version "0.1.0"

// ONE ROSSLYN -- 1901 & 1911 N Fort Myer Drive, Arlington, Virginia.
// Penzance with The Baupost Group. Site plan SP #419, amendment SPLA24-00040,
// approved 2025-07-19.
//
// The companion case, penzance_highlands, reconstructs a deal that COMPLETED.
// This project is entitled and unbuilt: its program and its land are recorded
// fact, its economics are forecast. The two cases are written to be read
// together, and the contrast is the point.
//
// Every figure the model uses is an `assume` below, labeled with its standing,
// and everything else is arithmetic on those figures: the construction budget,
// both commitments, the draw curve, each tower's lease-up, the value the sale
// is priced on, the permanent loan and its payoff. Nothing is computed outside
// the model and pasted in. `inputs/one_rosslyn.toml` records where each figure
// came from.
//
// Sourcing, strongest first:
//   program        Arlington County Board Report, SP #419 -- fact
//   land           deed 20230100013266, ${d_grid_start}-14, $$52,000,000 -- fact
//   obligations    Board Report; the AHIF tranches are tied to certificates of
//                  occupancy, so their TIMING is entitlement fact -- fact
//   operations     Arlington 2026 Guidebook, MARKET Apartment Guidelines,
//                  High-Rise 9+ (PCC 313), 2010+, Metro -- fact
//   comparables    four recorded Arlington trades, 2022 to 2026 -- fact
//   rent           derived by the County's own method from a 2026 comparable
//   escalation     BLS output-price index WPUIP2312001 -- published
//   growth         3.0%/yr, below BLS CUSR0000SEHA at 10yr 4.23% and 3yr 3.98%
//   cost, debt, pace, condominium pricing, JV tiers -- projection
//
// THE EXIT IS DERIVED, NOT PICKED. Both strategies value the sale over the
// County's guideline loaded cap, adjusted by a stated market factor. The
// build-to-core sale is priced on the twelve months of income that follow it,
// read from the projection tail. The lease-up sale is priced on the stabilized
// income of the building at that month's rent level, by the County's own
// method, because a purchaser underwrites what the asset will produce rather
// than what it collects mid-lease-up. The factor is 1.00 -- the guideline basis
// itself. The two market observations available bracket it: The Highlands sold
// at +32% to this basis in 2022, and Central Place at -11.6% in 2026. Choosing
// either as the base would import a market call the record does not support.
//
// At the lease-up sale the south tower is eleven months into an eighteen-month
// lease-up, so in-place income would understate the price by roughly $$140M.
// The Highlands towers sold before either had stabilized.

time calendar monthly from ${d_grid_start} for ${grid_periods} project ${valuation_tail}

// 0 = merchant build, 1 = build to core. Every scenario-dependent stream is
// weighted by this, so one model carries both and run.json selects.
assume scenario_b        = 1.0

// The market's premium or discount to the County's guideline basis. 1.00 IS
// the guideline basis. It scales the SALE only: a lender sizes the permanent
// loan off the appraised basis at stabilization, not off the price a later
// buyer pays.
assume market_factor     = 1.0

// ------------------------------------------------------------- the program
// FACT. Board Report SP #419, statistical summary. Three towers over one
// podium: the NE tower is for sale, the NW and S towers are rental.
assume gfa_total_sf      = ${gfa_total_sf}
assume parking_spaces    = ${parking_spaces}
assume ne_units          = ${ne_units}
assume nw_units          = ${nw_units}
assume s_units           = ${s_units}
assume rental_units      = inputs.nw_units + inputs.s_units

// ----------------------------------------------------------------- the land
// FACT. Recorded ${d_grid_start}-14, deed 20230100013266, sales code "4-Multiple RPCs",
// so the price is the aggregate across all three parcels. The County's own
// guideline land value for Metro high-rise is $$78,000 per rental unit, putting
// 772 rental units at $$60,216,000: the site was bought 13.6% below guideline.
assume land_price        = ${land_price}

// ------------------------------------------------------- public obligations
// FACT, and the timing is fact too. The affordable-housing tranches are tied
// to the first partial certificate of occupancy for each tower.
assume ahif_base_density    = ${ahif_base_density}
assume public_space_rmsa    = ${public_space_rmsa}
assume transportation_rmsa  = ${transportation_rmsa}
assume ahif_tranche_ne      = ${ahif_tranche_ne}
assume ahif_tranche_nw      = ${ahif_tranche_nw}
assume ahif_tranche_s       = ${ahif_tranche_s}
assume obligations_total    = inputs.ahif_base_density + inputs.public_space_rmsa
                            + inputs.transportation_rmsa + inputs.ahif_tranche_ne
                            + inputs.ahif_tranche_nw + inputs.ahif_tranche_s

// ---------------------------------------------------- the construction basis
// Anchored to the companion case's ACTUAL spend and escalated by a published
// index. Buildings and parking are priced separately: The Highlands built 1.17
// spaces per unit and this project builds 0.51, and a blended rate would charge
// it for parking it does not build. The soft, contingency and fee ratios are
// the companion case's own.
assume highlands_buildings_psf   = ${highlands_buildings_psf}        // FACT: 301,693,050 / 1,183,110 sf
assume highlands_parking_space   = ${highlands_parking_space}      // FACT: 33,184,000 / 1,037 spaces
assume escalation_realized       = ${escalation_realized}       // FACT: BLS WPUIP2312001, 2019-11 to ${d_rent_vintage}
assume escalation_forward_rate   = ${escalation_forward_rate}       // PROJECTION: 36-month trailing rate on the same index
assume escalation_years_forward  = ${escalation_years_forward}         // ${d_rent_vintage} to the projected 2028-09 midpoint
assume soft_ratio                = ${soft_ratio}     // FACT: ratio to hard, companion case
assume contingency_ratio         = ${contingency_ratio}     // FACT
assume developer_fee_ratio       = ${developer_fee_ratio}     // FACT
assume escalation      = inputs.escalation_realized
                       * pow(1.0 + inputs.escalation_forward_rate, inputs.escalation_years_forward)
assume hard_cost       = (inputs.highlands_buildings_psf * inputs.gfa_total_sf
                        + inputs.highlands_parking_space * inputs.parking_spaces) * inputs.escalation
assume construction_cost = inputs.hard_cost
                         * (1.0 + inputs.soft_ratio + inputs.contingency_ratio + inputs.developer_fee_ratio)

// ------------------------------------------------------------------ timing
// PROJECTION. Each date is stated once, in the schedules below and here; the
// month numbers the expressions need are counted from the grid's first month.
assume construction_start   = months_between(parse_date("${d_grid_start}"), parse_date("${d_construction_start}"))
assume construction_months  = ${construction_months}
assume delivery_nw          = months_between(parse_date("${d_grid_start}"), parse_date("${d_delivery_nw}"))
assume delivery_s           = months_between(parse_date("${d_grid_start}"), parse_date("${d_delivery_s}"))
assume lease_up_months      = ${lease_up_months}    // each tower, from its delivery
assume exit_a_month         = months_between(parse_date("${d_grid_start}"), parse_date("${d_merchant_sale}"))
assume refinance_month      = months_between(parse_date("${d_grid_start}"), parse_date("${d_stabilization}"))
assume exit_b_month         = months_between(parse_date("${d_grid_start}"), parse_date("${d_core_exit}"))
assume condo_months         = ${condo_months}    // sellout from the NE delivery
assume rent_base_month      = months_between(parse_date("${d_grid_start}"), parse_date("${d_rent_vintage}"))

// The draw is a parabola over the construction window: the weight on month t
// is (t - (start - 1)) * ((start + months) - t), and the weights sum to
// n(n + 1)(n + 2) / 6 for n months, so each month's draw is its weight's share
// of the budget.
assume draw_weight_sum = inputs.construction_months * (inputs.construction_months + 1.0)
                       * (inputs.construction_months + 2.0) / 6.0

// -------------------------------------------------------------- operations
// FACT: Arlington 2026 Guidebook, MARKET Apartment Guidelines, High-Rise 9+,
// effective age 2010+, Metro. Rent is DERIVED by the County's own method: a
// comparable's assessed value times the guideline loaded cap gives the
// assessor's NOI; add back guideline expenses and gross up for vacancy. Rent,
// parking and expenses escalate from the Guidebook's ${d_rent_vintage} vintage -- this
// project delivers in 2030 and freezing 2026 dollars would understate it.
assume rent_per_unit          = ${rent_per_unit}    // DERIVED, $$/unit/month, 2026 dollars
assume expenses_per_unit      = ${expenses_per_unit}    // FACT, $$/unit/year
assume vacancy                = ${vacancy}      // FACT, vacancy and collection
assume parking_garage_low     = ${parking_garage_low}      // FACT, $$/space/month
assume parking_garage_high    = ${parking_garage_high}     // FACT
assume parking_per_space      = (inputs.parking_garage_low + inputs.parking_garage_high) / 2.0
assume parking_spaces_per_unit = ${parking_spaces_per_unit}      // PROJECTION: 429 spaces over 845 units
assume growth                 = ${growth}      // PROJECTION, rent and expenses
assume cap_rate               = ${cap_rate}    // FACT, guideline loaded cap

// ------------------------------------------------------------ condominium
// PROJECTION, and the weakest input in the case: the NE tower's 73 units
// average 2,273 sq ft, far larger than the Pierce units this is anchored to,
// and no Rosslyn condominium of that size has traded recently.
assume pierce_per_unit     = ${pierce_per_unit}    // FACT: Pierce, 102 recorded closings
assume condo_selling_cost  = ${condo_selling_cost}
assume condo_monthly       = inputs.ne_units / inputs.condo_months
                           * inputs.pierce_per_unit * (1.0 - inputs.condo_selling_cost)

// ----------------------------------------------------------------- finance
// PROJECTION. No instrument is recorded. The index rates are FACT (FRED,
// 2026-08-24); the spreads over them are projections.
assume sofr                 = ${sofr}
assume construction_spread  = ${construction_spread}
assume loan_rate            = inputs.sofr + inputs.construction_spread
assume ltc                  = ${ltc}
assume loan_commitment      = inputs.construction_cost * inputs.ltc
assume equity_commitment    = (inputs.land_price + inputs.construction_cost + inputs.obligations_total)
                            * (1.0 - inputs.ltc)
assume ust10                = ${ust10}
assume perm_spread          = ${perm_spread}
assume perm_rate            = inputs.ust10 + inputs.perm_spread
assume perm_ltv             = ${perm_ltv}
assume perm_amortization_years  = ${perm_amortization_years}
assume perm_amortization_months = inputs.perm_amortization_years * 12.0

// --------------------------------------------------------------- valuation
// Stabilized net operating income by the County's method, in 2026 dollars:
// every rental unit's rent and parking net of the vacancy allowance, less
// guideline expenses on every unit. Escalated to the month it is struck.
assume stabilized_noi_2026 = inputs.rental_units
                           * ((1.0 - inputs.vacancy) * (inputs.rent_per_unit
                              + inputs.parking_per_space * inputs.parking_spaces_per_unit) * 12.0
                              - inputs.expenses_per_unit)

// The lease-up sale: stabilized income at the exit month's rent level, over
// the guideline cap.
assume exit_a_value = inputs.stabilized_noi_2026
                    * pow(1.0 + inputs.growth, (inputs.exit_a_month - inputs.rent_base_month) / 12.0)
                    / inputs.cap_rate

// The permanent loan: sized at loan-to-value on the appraised basis at
// stabilization, a level payment over thirty years.
assume stabilized_value_at_refinance = inputs.stabilized_noi_2026
                    * pow(1.0 + inputs.growth, (inputs.refinance_month - inputs.rent_base_month) / 12.0)
                    / inputs.cap_rate
assume perm_principal = inputs.stabilized_value_at_refinance * inputs.perm_ltv
assume perm_payment   = 0.0 - pmt(inputs.perm_rate / 12.0, inputs.perm_amortization_months, inputs.perm_principal)

// ------------------------------------------------------------------ the JV
// Penzance / Baupost terms are not public; these three rates are stated
// placeholders. Replacing them recomputes every partner figure.
assume pref_rate      = ${pref_rate}
assume sponsor_share  = ${sponsor_share}
assume promote_share  = ${promote_share}


// ---------------------------------------------------------------- structure
entity container project : CRE.Container.Portfolio
entity asset ne : CRE.Asset.RealProperty { asset_class = "condominium"  part of container.project }
entity asset nw : CRE.Asset.RealProperty { asset_class = "multifamily"  part of container.project }
entity asset south : CRE.Asset.RealProperty { asset_class = "multifamily"  part of container.project }

entity party penzance : CRE.Party.Sponsor  { name = "Penzance" }
entity party baupost  : CRE.Party.Investor { name = "The Baupost Group" }

// -------------------------------------------------------------- the program
// The development schedule, stated once. A field may read another field only
// at the prior close, so each of these is held ONE PERIOD AHEAD: the value at
// month t is the schedule for month t + 1, and the facility and the cost
// streams read it as `prev.asset.program.<field>`, which is this month's
// figure.
entity asset program : Asset.Financial {
  // The draw curve: a parabola over the construction window, each month's
  // weight (t - (start - 1)) * ((start + months) - t) as a share of the sum.
  draw_share_ahead init 0.0
                   next if(time.t + 1.0 >= inputs.construction_start
                           and time.t + 1.0 <= inputs.construction_start + inputs.construction_months - 1.0,
                           (time.t + 1.0 - (inputs.construction_start - 1.0))
                             * ((inputs.construction_start + inputs.construction_months) - (time.t + 1.0))
                             / inputs.draw_weight_sum,
                           0.0)

  // The month's whole cost: the budget on the curve, and each obligation on
  // the date the County ties it to. The land is month 0 and funds itself
  // below.
  cost_ahead init 0.0
             next inputs.construction_cost
                    * if(time.t + 1.0 >= inputs.construction_start
                         and time.t + 1.0 <= inputs.construction_start + inputs.construction_months - 1.0,
                         (time.t + 1.0 - (inputs.construction_start - 1.0))
                           * ((inputs.construction_start + inputs.construction_months) - (time.t + 1.0))
                           / inputs.draw_weight_sum,
                         0.0)
                  + if(time.t + 1.0 == inputs.construction_start,
                       inputs.ahif_base_density + inputs.public_space_rmsa + inputs.transportation_rmsa, 0.0)
                  + if(time.t + 1.0 == inputs.delivery_nw, inputs.ahif_tranche_ne + inputs.ahif_tranche_nw, 0.0)
                  + if(time.t + 1.0 == inputs.delivery_s, inputs.ahif_tranche_s, 0.0)

  // Cash that retires the facility: the condominium closings, then whichever
  // liquidity event the scenario reaches -- the sale, or the refinancing.
  proceeds_ahead init 0.0
                 next if(time.t + 1.0 >= inputs.delivery_nw
                         and time.t + 1.0 < inputs.delivery_nw + inputs.condo_months,
                         inputs.condo_monthly, 0.0)
                    + if(time.t + 1.0 == inputs.exit_a_month,
                         inputs.exit_a_value * inputs.market_factor * (1.0 - inputs.scenario_b), 0.0)
                    + if(time.t + 1.0 == inputs.refinance_month,
                         inputs.perm_principal * inputs.scenario_b, 0.0)
}

// ------------------------------------------------------- development cost
stream cre.cost.land on entity container.project outflow currency USD {
  schedule on ${d_grid_start}
  category investing.capital.capex
  amount = inputs.land_price
}

// The budget on the curve, in the companion case's four lines.
stream cre.cost.hard on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.hard_cost * prev.asset.program.draw_share_ahead
}

stream cre.cost.soft on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.hard_cost * inputs.soft_ratio * prev.asset.program.draw_share_ahead
}

stream cre.cost.contingency on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.hard_cost * inputs.contingency_ratio * prev.asset.program.draw_share_ahead
}

stream cre.cost.developer_fee on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.hard_cost * inputs.developer_fee_ratio * prev.asset.program.draw_share_ahead
}

// The obligations, each on the date the County ties it to.
stream cre.cost.obligations_at_start on entity container.project outflow currency USD {
  schedule on ${d_construction_start}
  category investing.capital.construction
  amount = inputs.ahif_base_density + inputs.public_space_rmsa + inputs.transportation_rmsa
}

stream cre.cost.ahif_ne on entity asset.ne outflow currency USD {
  schedule on ${d_delivery_nw}
  category investing.capital.construction
  amount = inputs.ahif_tranche_ne
}

stream cre.cost.ahif_nw on entity asset.nw outflow currency USD {
  schedule on ${d_delivery_nw}
  category investing.capital.construction
  amount = inputs.ahif_tranche_nw
}

stream cre.cost.ahif_s on entity asset.south outflow currency USD {
  schedule on ${d_delivery_s}
  category investing.capital.construction
  amount = inputs.ahif_tranche_s
}

// -------------------------------------------------------------- operations
// Each rental tower leases up from its own delivery, a straight line over
// eighteen months, net of the vacancy and collection allowance. The blended
// occupancy the exit reads is the sum of the two. Under the merchant build
// the venture collects nothing after the sale month.
stream cre.rent.nw on entity asset.nw inflow currency USD {
  schedule every month start from ${d_delivery_nw} to ${d_operations_end}
  category operating.revenue.base_rent
  amount = inputs.nw_units * clamp((time.t - inputs.delivery_nw + 1.0) / inputs.lease_up_months, 0.0, 1.0)
         * (1.0 - inputs.vacancy) * inputs.rent_per_unit
         * pow(1.0 + inputs.growth, (time.t - inputs.rent_base_month) / 12.0)
         * if(inputs.scenario_b > 0.5, 1.0, if(time.t <= inputs.exit_a_month, 1.0, 0.0))
}

stream cre.parking.nw on entity asset.nw inflow currency USD {
  schedule every month start from ${d_delivery_nw} to ${d_operations_end}
  category operating.revenue.other
  amount = inputs.nw_units * clamp((time.t - inputs.delivery_nw + 1.0) / inputs.lease_up_months, 0.0, 1.0)
         * (1.0 - inputs.vacancy) * inputs.parking_per_space * inputs.parking_spaces_per_unit
         * pow(1.0 + inputs.growth, (time.t - inputs.rent_base_month) / 12.0)
         * if(inputs.scenario_b > 0.5, 1.0, if(time.t <= inputs.exit_a_month, 1.0, 0.0))
}

stream cre.opex.nw on entity asset.nw outflow currency USD {
  schedule every month start from ${d_delivery_nw} to ${d_operations_end}
  category operating.expense.opex
  amount = inputs.nw_units * clamp((time.t - inputs.delivery_nw + 1.0) / inputs.lease_up_months, 0.0, 1.0)
         * (1.0 - inputs.vacancy) * inputs.expenses_per_unit / 12.0
         * pow(1.0 + inputs.growth, (time.t - inputs.rent_base_month) / 12.0)
         * if(inputs.scenario_b > 0.5, 1.0, if(time.t <= inputs.exit_a_month, 1.0, 0.0))
}

stream cre.rent.south on entity asset.south inflow currency USD {
  schedule every month start from ${d_delivery_s} to ${d_operations_end}
  category operating.revenue.base_rent
  amount = inputs.s_units * clamp((time.t - inputs.delivery_s + 1.0) / inputs.lease_up_months, 0.0, 1.0)
         * (1.0 - inputs.vacancy) * inputs.rent_per_unit
         * pow(1.0 + inputs.growth, (time.t - inputs.rent_base_month) / 12.0)
         * if(inputs.scenario_b > 0.5, 1.0, if(time.t <= inputs.exit_a_month, 1.0, 0.0))
}

stream cre.parking.south on entity asset.south inflow currency USD {
  schedule every month start from ${d_delivery_s} to ${d_operations_end}
  category operating.revenue.other
  amount = inputs.s_units * clamp((time.t - inputs.delivery_s + 1.0) / inputs.lease_up_months, 0.0, 1.0)
         * (1.0 - inputs.vacancy) * inputs.parking_per_space * inputs.parking_spaces_per_unit
         * pow(1.0 + inputs.growth, (time.t - inputs.rent_base_month) / 12.0)
         * if(inputs.scenario_b > 0.5, 1.0, if(time.t <= inputs.exit_a_month, 1.0, 0.0))
}

stream cre.opex.south on entity asset.south outflow currency USD {
  schedule every month start from ${d_delivery_s} to ${d_operations_end}
  category operating.expense.opex
  amount = inputs.s_units * clamp((time.t - inputs.delivery_s + 1.0) / inputs.lease_up_months, 0.0, 1.0)
         * (1.0 - inputs.vacancy) * inputs.expenses_per_unit / 12.0
         * pow(1.0 + inputs.growth, (time.t - inputs.rent_base_month) / 12.0)
         * if(inputs.scenario_b > 0.5, 1.0, if(time.t <= inputs.exit_a_month, 1.0, 0.0))
}

// The condominium sells out evenly over two years from its delivery.
stream cre.condo_closings on entity asset.ne inflow currency USD {
  schedule every month start from ${d_delivery_nw} to ${d_condo_sellout_end}
  category investing.disposal.reversion
  amount = inputs.condo_monthly
}

// ---------------------------------------------------------- the JV capital
// WHAT EACH PARTNER PUTS IN, as cash. Equity funds first, pro rata -- 90%
// Baupost, 10% Penzance, the same share the tiers split on -- so each month's
// contribution is the step in the equity funded to date, which the facility
// states. Each contribution moves its partner's capital account. The land is
// funded in the first period, where there is no previous period to difference
// against, so it is its own one-shot.
stream cre.equity_contribution.baupost_land on entity container.project inflow currency USD {
  schedule on ${d_grid_start}
  category financing.equity.contribution
  amount = asset.facility.equity_funded * (1.0 - inputs.sponsor_share)
  moves baupost_capital
}

stream cre.equity_contribution.baupost on entity container.project inflow currency USD {
  schedule every month start from ${d_grid_second} to ${d_core_exit}
  category financing.equity.contribution
  amount = (asset.facility.equity_funded - prev.asset.facility.equity_funded) * (1.0 - inputs.sponsor_share)
  moves baupost_capital
}

stream cre.equity_contribution.penzance_land on entity container.project inflow currency USD {
  schedule on ${d_grid_start}
  category financing.equity.contribution
  amount = asset.facility.equity_funded * inputs.sponsor_share
  moves penzance_capital
}

stream cre.equity_contribution.penzance on entity container.project inflow currency USD {
  schedule every month start from ${d_grid_second} to ${d_core_exit}
  category financing.equity.contribution
  amount = (asset.facility.equity_funded - prev.asset.facility.equity_funded) * inputs.sponsor_share
  moves penzance_capital
}

// Each partner's capital is DUE to it from the venture: a contribution lowers
// the balance below zero, and an allocation from the split raises it back.
account baupost_capital due {
  owner party.baupost
}

account penzance_capital due {
  owner party.penzance
}

// ------------------------------------------------------------- the facility
// PROJECTION. No instrument is recorded. Equity funds to its commitment first,
// the facility draws the residual up to its own commitment, interest
// capitalizes into the balance, and the proceeds retire it. Each field reads
// only the prior close -- `prev` -- and the program's schedule, so every month
// resolves from the month before it.
//
// A field may read neither a stream nor a sibling field in the same period,
// so the equity funded to date is stated in closed form -- the cost incurred
// through the month, capped at the commitment; the parabola's weights on
// months 1..m sum to (n + 1) m (m + 1) / 2 - m (m + 1)(2m + 1) / 6 -- and the
// month's draw is spelled out where the repayment and the balance need it.
entity asset facility : Asset.Financial {
  equity_funded init min(inputs.equity_commitment, inputs.land_price)
                next min(inputs.equity_commitment,
                         inputs.land_price
                       + inputs.construction_cost
                         * ((inputs.construction_months + 1.0)
                              * clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months)
                              * (clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0) / 2.0
                            - clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months)
                              * (clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0)
                              * (2.0 * clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0) / 6.0)
                         / inputs.draw_weight_sum
                       + if(time.t >= inputs.construction_start,
                            inputs.ahif_base_density + inputs.public_space_rmsa + inputs.transportation_rmsa, 0.0)
                       + if(time.t >= inputs.delivery_nw, inputs.ahif_tranche_ne + inputs.ahif_tranche_nw, 0.0)
                       + if(time.t >= inputs.delivery_s, inputs.ahif_tranche_s, 0.0))

  interest init 0.0
           next prev.asset.facility.balance * inputs.loan_rate / 12.0

  // What the equity does not fund, up to the room left under the commitment
  // after this month's accrual.
  draw init max(0.0, inputs.land_price - inputs.equity_commitment)
       next min(max(0.0, prev.asset.program.cost_ahead
                          - (min(inputs.equity_commitment,
                                 inputs.land_price
                               + inputs.construction_cost
                                 * ((inputs.construction_months + 1.0)
                                      * clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months)
                                      * (clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0) / 2.0
                                    - clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months)
                                      * (clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0)
                                      * (2.0 * clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0) / 6.0)
                                 / inputs.draw_weight_sum
                               + if(time.t >= inputs.construction_start,
                                    inputs.ahif_base_density + inputs.public_space_rmsa + inputs.transportation_rmsa, 0.0)
                               + if(time.t >= inputs.delivery_nw, inputs.ahif_tranche_ne + inputs.ahif_tranche_nw, 0.0)
                               + if(time.t >= inputs.delivery_s, inputs.ahif_tranche_s, 0.0))
                             - prev.asset.facility.equity_funded)),
                max(0.0, inputs.loan_commitment
                         - prev.asset.facility.balance * (1.0 + inputs.loan_rate / 12.0)))

  // Repaid from the month's proceeds, up to what is outstanding after this
  // month's accrual and draw.
  repay init 0.0
        next min(prev.asset.facility.balance * (1.0 + inputs.loan_rate / 12.0)
                   + min(max(0.0, prev.asset.program.cost_ahead
                          - (min(inputs.equity_commitment,
                                 inputs.land_price
                               + inputs.construction_cost
                                 * ((inputs.construction_months + 1.0)
                                      * clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months)
                                      * (clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0) / 2.0
                                    - clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months)
                                      * (clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0)
                                      * (2.0 * clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0) / 6.0)
                                 / inputs.draw_weight_sum
                               + if(time.t >= inputs.construction_start,
                                    inputs.ahif_base_density + inputs.public_space_rmsa + inputs.transportation_rmsa, 0.0)
                               + if(time.t >= inputs.delivery_nw, inputs.ahif_tranche_ne + inputs.ahif_tranche_nw, 0.0)
                               + if(time.t >= inputs.delivery_s, inputs.ahif_tranche_s, 0.0))
                             - prev.asset.facility.equity_funded)),
                max(0.0, inputs.loan_commitment
                         - prev.asset.facility.balance * (1.0 + inputs.loan_rate / 12.0))),
                 prev.asset.program.proceeds_ahead)

  balance init max(0.0, inputs.land_price - inputs.equity_commitment)
          next max(0.0, prev.asset.facility.balance * (1.0 + inputs.loan_rate / 12.0)
                        + min(max(0.0, prev.asset.program.cost_ahead
                          - (min(inputs.equity_commitment,
                                 inputs.land_price
                               + inputs.construction_cost
                                 * ((inputs.construction_months + 1.0)
                                      * clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months)
                                      * (clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0) / 2.0
                                    - clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months)
                                      * (clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0)
                                      * (2.0 * clamp(time.t - inputs.construction_start + 1.0, 0.0, inputs.construction_months) + 1.0) / 6.0)
                                 / inputs.draw_weight_sum
                               + if(time.t >= inputs.construction_start,
                                    inputs.ahif_base_density + inputs.public_space_rmsa + inputs.transportation_rmsa, 0.0)
                               + if(time.t >= inputs.delivery_nw, inputs.ahif_tranche_ne + inputs.ahif_tranche_nw, 0.0)
                               + if(time.t >= inputs.delivery_s, inputs.ahif_tranche_s, 0.0))
                             - prev.asset.facility.equity_funded)),
                max(0.0, inputs.loan_commitment
                         - prev.asset.facility.balance * (1.0 + inputs.loan_rate / 12.0)))
                        - prev.asset.program.proceeds_ahead)
}

stream cre.loan_draw on entity container.project inflow currency USD {
  schedule every month start from ${d_grid_start} to ${d_core_exit}
  category financing.debt.proceeds
  amount = asset.facility.draw
}

// Interest capitalizes: stated GROSS, an outflow against a matching draw, so
// coverage stays measurable. The two legs net to zero in cash.
stream cre.loan_interest on entity container.project outflow currency USD {
  schedule every month start from ${d_grid_start} to ${d_core_exit}
  category financing.debt.interest_paid
  amount = asset.facility.interest
}

stream cre.loan_interest_funding on entity container.project inflow currency USD {
  schedule every month start from ${d_grid_start} to ${d_core_exit}
  category financing.debt.proceeds
  amount = asset.facility.interest
}

// A balance retired out of disposal proceeds is a reversion, not debt service:
// folding it into debt service makes every coverage ratio in the period
// meaningless. The cre pack says the same of a permanent loan's balloon.
stream cre.loan_repayment on entity container.project outflow currency USD {
  schedule every month start from ${d_grid_start} to ${d_core_exit}
  category investing.disposal.reversion
  amount = asset.facility.repay
}

// ----------------------------------------------------- scenario A: the sale
stream cre.exit_a on entity container.project inflow currency USD {
  schedule on ${d_merchant_sale} end
  category investing.disposal.reversion
  amount = inputs.exit_a_value * inputs.market_factor * (1.0 - inputs.scenario_b)
}

// ------------------------------------------- scenario B: refinance and hold
// Permanent debt at stabilization, sized above from the appraised basis: a
// level payment over thirty years, split into its interest and principal
// legs so coverage reads both, and the balance carried month to month. The
// payoff at the sale is what is left after the last payment.
entity asset perm : Asset.Financial {
  balance init 0.0
          next if(time.t == inputs.refinance_month,
                  inputs.perm_principal * inputs.scenario_b,
                  max(0.0, prev.asset.perm.balance * (1.0 + inputs.perm_rate / 12.0)
                           - if(prev.asset.perm.balance > 0.0, inputs.perm_payment, 0.0)))
}

stream cre.refinance on entity container.project inflow currency USD {
  schedule on ${d_stabilization} end
  category financing.debt.proceeds
  amount = inputs.perm_principal * inputs.scenario_b
}

stream cre.perm_interest on entity container.project outflow currency USD {
  schedule every month end from ${d_first_perm_payment} to ${d_core_exit}
  category financing.debt.interest_paid
  amount = prev.asset.perm.balance * inputs.perm_rate / 12.0
}

stream cre.perm_principal on entity container.project outflow currency USD {
  schedule every month end from ${d_first_perm_payment} to ${d_core_exit}
  category financing.debt.principal
  amount = (inputs.perm_payment - prev.asset.perm.balance * inputs.perm_rate / 12.0) * inputs.scenario_b
}

stream cre.perm_payoff on entity container.project outflow currency USD {
  schedule on ${d_core_exit} end
  category investing.disposal.reversion
  amount = asset.perm.balance
}

// The stabilized sale: the twelve months of income that follow it, read from
// the projection tail, over the guideline cap.
stream cre.exit_b on entity container.project inflow currency USD {
  schedule on ${d_core_exit} end
  category investing.disposal.reversion
  amount = (series_sum("cre.rent.*", time.t + 1, time.t + 12)
          + series_sum("cre.parking.*", time.t + 1, time.t + 12)
          + series_sum("cre.opex.*", time.t + 1, time.t + 12))
         / inputs.cap_rate * inputs.market_factor * inputs.scenario_b
}

// ---------------------------------------------------------- the preference
// Cash accrues to the venture and is split once, so the preference is a
// CUMULATIVE balance carried forward rather than re-derived at the
// distribution -- 17_ordered_waterfall.md section 10. It compounds on the
// capital contributed to date, from CONSTRUCTION START rather than the 2023
// land purchase: the venture is formed to build, and the land earns nothing
// until there is something to build. The contributed capital itself is the
// `equity_funded` account; the difference between the two is the accrued
// preference.
entity asset jv : Asset.Financial {
  // The equity funded one period back, so a month's contribution can be
  // differenced without reaching two periods behind.
  funded_prev init 0.0
              next prev.asset.facility.equity_funded

  capital init 0.0
          next prev.asset.jv.capital
             + (prev.asset.facility.equity_funded - prev.asset.jv.funded_prev)

  unreturned init 0.0
             next prev.asset.jv.unreturned
                    * (1.0 + if(time.t >= inputs.construction_start, inputs.pref_rate / 12.0, 0.0))
                  + (prev.asset.facility.equity_funded - prev.asset.jv.funded_prev)
}

// ------------------------------------------------------- the venture's cash
// A development JV does not distribute while the deal is live, so cash
// accumulates from inception and is allocated once. What accumulates is the
// venture's whole cash position: the equity the partners contributed, plus
// everything the deal earned on it, less every cost.
account deal_cash {
  from series_sum("cre.*", time.t, time.t)
}

// The deal's own cash, with the partners' contributions left out: what the
// workbook ties to, and what the project returns on.
slice deal {
  entity container.project
  except category "financing.equity.contribution"
}

// -------------------------------------------------------------- the JV split
// Capital and the preference come back PRO RATA. The investor's tier of each
// pair is capped at its share of the pot, so a pot too small to return
// everyone's capital -- the 2026-discount scenario is one -- or to pay the
// whole preference -- every scenario at the guideline basis is one -- shorts
// both partners in proportion rather than the sponsor alone. Below the
// promote the partners are pari passu. The companion case pays the investor
// first, and its pot is never short, so the two spellings agree there.
//
// One model, two exits, and a waterfall's schedule is a date. So each strategy
// has its own distribution, on its own final cash event, and every tier is
// weighted by the scenario switch: the strategy not selected allocates nothing
// and leaves the pot untouched for the one that is.
//
// Scenario A: the towers sell in ${d_merchant_sale}, and the condominium sellout runs
// to ${d_condo_sellout_end}. The venture distributes when the last unit closes.
waterfall jv.distribution_a on entity container.project {
  schedule on ${d_condo_sellout_end} end
  from deal_cash

  pay capital_inv   to party.baupost  = min(0.0 - prev.baupost_capital, remaining * (1.0 - inputs.sponsor_share)) * (1.0 - inputs.scenario_b)
  pay capital_sp    to party.penzance = min(0.0 - prev.penzance_capital, remaining) * (1.0 - inputs.scenario_b)
  pay preferred_inv to party.baupost  = min((asset.jv.unreturned - asset.jv.capital) * (1.0 - inputs.sponsor_share), remaining * (1.0 - inputs.sponsor_share)) * (1.0 - inputs.scenario_b)
  pay preferred_sp  to party.penzance = (asset.jv.unreturned - asset.jv.capital) * inputs.sponsor_share * (1.0 - inputs.scenario_b)
  pay promote       to party.penzance = remaining * inputs.promote_share * (1.0 - inputs.scenario_b)
  pay residual_inv  to party.baupost  = remaining * (1.0 - inputs.sponsor_share) * (1.0 - inputs.scenario_b)
  pay residual_sp   to party.penzance = remaining * (1.0 - inputs.scenario_b)
}

// Scenario B: the stabilized asset sells in ${d_core_exit}, the last period of
// the cash horizon, and the venture distributes on the same date.
waterfall jv.distribution_b on entity container.project {
  schedule on ${d_core_exit} end
  from deal_cash

  pay capital_inv   to party.baupost  = min(0.0 - prev.baupost_capital, remaining * (1.0 - inputs.sponsor_share)) * inputs.scenario_b
  pay capital_sp    to party.penzance = min(0.0 - prev.penzance_capital, remaining) * inputs.scenario_b
  pay preferred_inv to party.baupost  = min((asset.jv.unreturned - asset.jv.capital) * (1.0 - inputs.sponsor_share), remaining * (1.0 - inputs.sponsor_share)) * inputs.scenario_b
  pay preferred_sp  to party.penzance = (asset.jv.unreturned - asset.jv.capital) * inputs.sponsor_share * inputs.scenario_b
  pay promote       to party.penzance = remaining * inputs.promote_share * inputs.scenario_b
  pay residual_inv  to party.baupost  = remaining * (1.0 - inputs.sponsor_share) * inputs.scenario_b
  pay residual_sp   to party.penzance = remaining * inputs.scenario_b
}

// -------------------------------------------------------------- the returns
// WHAT EACH PARTNER EARNS, measured on that partner's own capital in and
// distributions out. Penzance's figure is all-in: its preferred and residual
// as a 10% investor, and the promote it earns as sponsor. Each tier is
// reported on its own, so the promote can be read separately.
metric baupost_irr   = irr(party.baupost)
metric baupost_moic  = moic(party.baupost)
metric penzance_irr  = irr(party.penzance)
metric penzance_moic = moic(party.penzance)
''')

(CASE / "model.cfdl").write_text(MODEL.substitute(values))
print(f"wrote model.cfdl  {len((CASE / 'model.cfdl').read_text().splitlines())} lines")
