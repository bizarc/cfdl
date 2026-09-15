"""Write benchmarks/cre/penzance_highlands/model.cfdl from the frozen input set.

A WRITER, not a calculator. It reads inputs/highlands.toml and the recorded
condominium closings in inputs/pierce_sellout_actual.csv, fills the model's
assumption block, its schedule dates and the one curve, and emits the model
text. Every figure the model needs that can be calculated from another is
calculated IN THE MODEL: the budget, the obligations, the equity commitment,
the draw curve, each tower's rent, the facility and the split, and every
month number, which the model counts from the stated dates with
`months_between`. If a value could be computed in the language, it does not
belong in this file.

The curve is data, not a calculation: 102 recorded closings by month, with
the quiet months in the range declared as zero. The only thing the writer
does beyond copying is name the month after or before a stated date where a
schedule needs it (the first monthly contribution, the last month of
predevelopment, the first month of the condominium tail): a schedule clause
and a phase take literal dates.

Regenerate: python3 reference_gen.py
"""
import csv
import tomllib
from pathlib import Path
from string import Template

CASE = Path(__file__).resolve().parent
T = tomllib.load(open(CASE / "inputs" / "highlands.toml", "rb"))
SITE, CAP, SCH, DEBT, EQ = T["site"], T["capital"], T["schedule"], T["debt"], T["equity"]
OPS, LU, EXIT, BW, BE = T["operations"], T["leaseup"], T["exit"], T["building"]["west"], T["building"]["east"]
AUB, EVO = T["asset"]["aubrey"], T["asset"]["evo"]
OBL = {o["item"]: o["amount_usd"] for o in T["obligation"]}


def month_after(ym):
    y, m = (int(x) for x in ym.split("-"))
    return f"{y + m // 12}-{m % 12 + 1:02d}"


def month_before(ym):
    y, m = (int(x) for x in ym.split("-"))
    return f"{y - (1 if m == 1 else 0)}-{(m - 2) % 12 + 1:02d}"


def months(first, last):
    """Every YYYY-MM from first to last inclusive."""
    y, m = (int(x) for x in first.split("-"))
    y2, m2 = (int(x) for x in last.split("-"))
    while (y, m) <= (y2, m2):
        yield f"{y}-{m:02d}"
        y, m = (y + 1, 1) if m == 12 else (y, m + 1)


def num(x):
    return repr(float(x))


closings = {r["year_month"]: float(r["gross_proceeds"])
            for r in csv.DictReader(open(CASE / "inputs" / "pierce_sellout_actual.csv"))}
rows = "\n".join(f"  {ym}: {closings.get(ym, 0.0):.2f}"
                 for ym in months(SCH["condo_first_close"], SCH["condo_last_close"]))

values = {k: num(v) for k, v in dict(
    land_price=SITE["tenure"]["land_price_usd"],
    hard_buildings=CAP["hard_buildings_usd"], hard_garage=CAP["hard_garage_usd"], fire_station=CAP["fire_station_usd"],
    park_and_street=CAP["park_and_street_usd"], soft_costs=CAP["soft_costs_usd"], contingency=CAP["contingency_usd"],
    developer_fee=CAP["developer_fee_usd"],
    utility_fund=OBL["Underground utility fund"], tdm_buy_in=OBL["TDM program buy-in"], public_art=OBL["Public art contribution"],
    green_building_fund=OBL["Green Building Fund"], affordable_housing=CAP["affordable_housing_usd"],
    construction_months=len(list(months(SCH["construction_start"], SCH["construction_end"]))),
    loan_commitment=DEBT["construction_loan_usd"], loan_rate=DEBT["loan_rate"], equity_share_of_cost=EQ["equity_share_of_cost"],
    aubrey_units=AUB["units"], evo_units=EVO["units"], aubrey_rent=LU["aubrey_rent_per_unit_mo"], evo_rent=LU["evo_rent_per_unit_mo"],
    aubrey_assessed=LU["aubrey_assessed_2022_usd"], evo_assessed=LU["evo_assessed_2022_usd"],
    aubrey_retail_sf=BW["gfa_retail_sf"], evo_retail_sf=BE["gfa_retail_sf"],
    other_income_pct=OPS["other_income_pct"], vacancy=OPS["vacancy_pct"], expenses_per_unit=OPS["opex_per_unit_annual"],
    parking_per_space=OPS["parking_per_space_mo"], parking_ratio=T["parking"]["residential_ratio"],
    retail_rent_psf=OPS["retail_rent_psf_year"], retail_occupancy=OPS["retail_occupancy"], effective_tax_rate=OPS["effective_tax_rate"],
    units_per_month=LU["units_per_month"],
    aubrey_price=EXIT["aubrey_price_usd"], evo_price=EXIT["evo_price_usd"],
    cost_of_sale=CAP["cost_of_sale"], condo_selling_cost=CAP["condo_selling_cost"],
    pref_rate=EQ["pref_rate"], sponsor_share=EQ["sponsor_share"], promote_share=EQ["promote_share"],
).items()}
values.update(
    grid_periods=str(SCH["grid_periods"]), pierce_sellout_rows=rows,
    d_grid_start=SCH["grid_start"], d_grid_second=month_after(SCH["grid_start"]),
    d_predevelopment_end=month_before(SCH["construction_start"]),
    d_construction_start=SCH["construction_start"], d_construction_end=SCH["construction_end"],
    d_delivery_aubrey=SCH["delivery_aubrey"], d_delivery_evo=SCH["delivery_evo"], d_sale=SCH["sale"],
    d_condo_tail_start=month_after(SCH["sale"]),
    d_condo_first_close=SCH["condo_first_close"], d_condo_last_close=SCH["condo_last_close"],
    d_grid_end=SCH["grid_end"],
)

MODEL = Template(r'''version 0.1
model "penzance-highlands" currency USD
use pack "cre" version "0.1.0"
time calendar monthly from ${d_grid_start} for ${grid_periods}

// ===========================================================================
// THE HIGHLANDS — Rosslyn, Virginia.  Penzance / The Baupost Group.
// Arlington County Site Plan SP #445.  Land ${d_grid_start}-30, delivered 2021,
// both rental towers sold to Cortland ${d_sale}-17, condo sellout to ${d_condo_last_close}.
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
//
// Every figure the model uses is an `assume` below, labeled with its
// standing, and everything else is arithmetic on those figures: the budget's
// draw, the equity commitment, each tower's lease-up, the facility and the
// split. The one table is the condominium sellout, which is recorded fact.
// `inputs/highlands.toml` records where each figure came from.
// ===========================================================================

phase predevelopment from ${d_grid_start} to ${d_predevelopment_end}
phase construction   from ${d_construction_start} to ${d_construction_end}
phase lease_up       from ${d_delivery_aubrey} to ${d_sale}
phase condo_tail     from ${d_condo_tail_start} to ${d_grid_end}

// ----------------------------------------------------------------- the land
// FACT. Deed 4491/2257, ${d_grid_start}-30, one price across the fee parcels.
assume land_price = ${land_price}

// -------------------------------------------------------------- the budget
// ASSUMPTION, by line, calibrated so that peak debt sits at 97.5% of the
// reported facility. Every line is drawn on one parabola over the 39-month
// construction window.
assume hard_buildings    = ${hard_buildings}
assume hard_garage       = ${hard_garage}
assume fire_station      = ${fire_station}
assume park_and_street   = ${park_and_street}
assume soft_costs        = ${soft_costs}
assume contingency       = ${contingency}
assume developer_fee     = ${developer_fee}
assume construction_cost = inputs.hard_buildings + inputs.hard_garage + inputs.fire_station
                         + inputs.park_and_street + inputs.soft_costs + inputs.contingency
                         + inputs.developer_fee

// SP #445 conditions. FACT except the affordable housing contribution, which
// condition #42 leaves to the developer's election and is an ASSUMPTION.
// Utility undergrounding and TDM at permit; public art, green building and
// affordable housing at certificate of occupancy.
assume utility_fund         = ${utility_fund}
assume tdm_buy_in           = ${tdm_buy_in}
assume public_art           = ${public_art}
assume green_building_fund  = ${green_building_fund}
assume affordable_housing   = ${affordable_housing}
assume obligations_at_permit = inputs.utility_fund + inputs.tdm_buy_in
assume obligations_at_co     = inputs.public_art + inputs.green_building_fund + inputs.affordable_housing
assume total_cost = inputs.land_price + inputs.construction_cost
                  + inputs.obligations_at_permit + inputs.obligations_at_co

// ------------------------------------------------------------------ timing
// Each date is stated once, in the schedules below and here; the month
// numbers the expressions need are counted from the grid's first month.
assume construction_start  = months_between(parse_date("${d_grid_start}"), parse_date("${d_construction_start}"))
assume construction_months = ${construction_months}
assume delivery_aubrey     = months_between(parse_date("${d_grid_start}"), parse_date("${d_delivery_aubrey}"))
assume delivery_evo        = months_between(parse_date("${d_grid_start}"), parse_date("${d_delivery_evo}"))
assume sale_month          = months_between(parse_date("${d_grid_start}"), parse_date("${d_sale}"))
assume condo_first_close   = months_between(parse_date("${d_grid_start}"), parse_date("${d_condo_first_close}"))
assume condo_last_close    = months_between(parse_date("${d_grid_start}"), parse_date("${d_condo_last_close}"))

// The draw is a parabola over the construction window: the weight on month t
// is (t - (start - 1)) * ((start + months) - t), and the weights sum to
// n(n + 1)(n + 2) / 6 for n months, so each month's draw is its weight's share
// of the budget.
assume draw_weight_sum = inputs.construction_months * (inputs.construction_months + 1.0)
                       * (inputs.construction_months + 2.0) / 6.0

// ------------------------------------------------------------- the facility
// The $$380M construction facility is press-reported; its pricing is an
// ASSUMPTION. Equity funds first, to a stated share of total development
// cost; the facility draws the residual and capitalizes interest.
assume loan_commitment      = ${loan_commitment}
assume loan_rate            = ${loan_rate}
assume equity_share_of_cost = ${equity_share_of_cost}
assume equity_commitment    = inputs.total_cost * inputs.equity_share_of_cost

// -------------------------------------------------------------- operations
// FACT from the 2022 Commercial Guidebook: expenses per unit, vacancy, the
// garage rate. Rents are DERIVED, not assumed: each tower's 1/1/2022 assessed
// value times the County's guideline loaded cap gives the assessor's own net
// operating income, solved back to a rent per unit. The other-income share,
// the retail rent and occupancy and the lease-up pace are ASSUMPTIONS.
assume aubrey_units       = ${aubrey_units}
assume evo_units          = ${evo_units}
assume aubrey_rent        = ${aubrey_rent}      // DERIVED, $$/unit/month
assume evo_rent           = ${evo_rent}      // DERIVED
assume aubrey_assessed    = ${aubrey_assessed} // FACT, 1/1/2022
assume evo_assessed       = ${evo_assessed} // FACT
assume aubrey_retail_sf   = ${aubrey_retail_sf}      // FACT
assume evo_retail_sf      = ${evo_retail_sf}     // FACT
assume other_income_pct   = ${other_income_pct}
assume vacancy            = ${vacancy}        // FACT
assume expenses_per_unit  = ${expenses_per_unit}      // FACT, $$/unit/year
assume parking_per_space  = ${parking_per_space}       // FACT, $$/space/month
assume parking_ratio      = ${parking_ratio}        // FACT, spaces per unit
assume retail_rent_psf    = ${retail_rent_psf}        // $$/sf/year
assume retail_occupancy   = ${retail_occupancy}
assume effective_tax_rate = ${effective_tax_rate}      // FACT, on assessed value
assume units_per_month    = ${units_per_month}        // lease-up pace, each tower from its delivery

// ---------------------------------------------------------------- the exit
// FACT: both towers sold ${d_sale}-17 as ONE transaction; only the deed
// recording dates differ, which is why the press reported two. The
// condominium closings are the recorded schedule below. Costs of sale are
// ASSUMPTIONS.
assume aubrey_price       = ${aubrey_price}
assume evo_price          = ${evo_price}
assume cost_of_sale       = ${cost_of_sale}
assume condo_selling_cost = ${condo_selling_cost}

// ------------------------------------------------------------------ the JV
// Penzance / Baupost terms are not public; these three rates are stated
// placeholders. Replacing them recomputes every partner figure.
assume pref_rate     = ${pref_rate}
assume sponsor_share = ${sponsor_share}
assume promote_share = ${promote_share}

// Pierce condo closings — every one of 102 recorded sales, by month. FACT,
// from the assessment roll: 34 months, not the smooth absorption an
// assumption would give. Every month in the range is declared, so a quiet
// month reads zero. A step curve holds its end values outside its points, so
// the curve states where it stops and every read below is gated to the
// closings' own months.
curve pierce_sellout to ${d_grid_end} {
${pierce_sellout_rows}
}


// ---------------------------------------------------------------- structure
entity container project : CRE.Container.Portfolio
entity asset east : CRE.Asset.RealProperty { asset_class = "mixed_use"  part of container.project }
entity asset west : CRE.Asset.RealProperty { asset_class = "multifamily"  part of container.project }

entity party penzance : CRE.Party.Sponsor  { name = "Penzance" }
entity party baupost  : CRE.Party.Investor { name = "The Baupost Group" }
entity party mack     : CRE.Party.Lender   { name = "Mack Real Estate Credit Strategies" }

// -------------------------------------------------------------- the program
// The development schedule, stated once. A field may read another field only
// at the prior close, so each of these is held ONE PERIOD AHEAD: the value at
// month t is the schedule for month t + 1, and the facility and the cost
// streams read it as `prev.asset.program.<field>`, which is this month's
// figure.
entity asset program : Asset.Financial {
  // The draw curve, as each month's share of the weights.
  draw_share_ahead init 0.0
                   next if(time.t + 1.0 >= inputs.construction_start
                           and time.t + 1.0 <= inputs.construction_start + inputs.construction_months - 1.0,
                           (time.t + 1.0 - (inputs.construction_start - 1.0))
                             * ((inputs.construction_start + inputs.construction_months) - (time.t + 1.0))
                             / inputs.draw_weight_sum,
                           0.0)

  // The month's whole cost: the budget on the curve, and each obligation on
  // its date. The land is month 0 and funds itself below.
  cost_ahead init 0.0
             next inputs.construction_cost
                    * if(time.t + 1.0 >= inputs.construction_start
                         and time.t + 1.0 <= inputs.construction_start + inputs.construction_months - 1.0,
                         (time.t + 1.0 - (inputs.construction_start - 1.0))
                           * ((inputs.construction_start + inputs.construction_months) - (time.t + 1.0))
                           / inputs.draw_weight_sum,
                         0.0)
                  + if(time.t + 1.0 == inputs.construction_start, inputs.obligations_at_permit, 0.0)
                  + if(time.t + 1.0 == inputs.delivery_evo, inputs.obligations_at_co, 0.0)

  // Cash that retires the facility: each condominium closing net of its
  // selling cost, and the two tower sales net of the cost of sale.
  proceeds_ahead init 0.0
                 next if(time.t + 1.0 >= inputs.condo_first_close and time.t + 1.0 <= inputs.condo_last_close,
                         curve_value("pierce_sellout", edate(time.date, 1)) * (1.0 - inputs.condo_selling_cost),
                         0.0)
                    + if(time.t + 1.0 == inputs.sale_month,
                         (inputs.aubrey_price + inputs.evo_price) * (1.0 - inputs.cost_of_sale), 0.0)
}

// ------------------------------------------------------------------ capital
stream cre.land on entity container.project outflow currency USD {
  schedule on ${d_grid_start}
  category investing.capital.capex
  amount = inputs.land_price
}

stream cre.hard_buildings on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.hard_buildings * prev.asset.program.draw_share_ahead
}

stream cre.hard_garage on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.hard_garage * prev.asset.program.draw_share_ahead
}

stream cre.fire_station on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.fire_station * prev.asset.program.draw_share_ahead
}

stream cre.park_and_street on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.park_and_street * prev.asset.program.draw_share_ahead
}

stream cre.soft_costs on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.soft_costs * prev.asset.program.draw_share_ahead
}

stream cre.contingency on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.contingency * prev.asset.program.draw_share_ahead
}

stream cre.developer_fee on entity container.project outflow currency USD {
  schedule every month start from ${d_construction_start} to ${d_construction_end}
  category investing.capital.construction
  amount = inputs.developer_fee * prev.asset.program.draw_share_ahead
}

// SP #445 conditions: utility undergrounding and TDM at permit; public art,
// green building and affordable housing at certificate of occupancy.
stream cre.obligations_permit on entity container.project outflow currency USD {
  schedule on ${d_construction_start}
  category investing.capital.construction
  amount = inputs.obligations_at_permit
}

stream cre.obligations_co on entity container.project outflow currency USD {
  schedule on ${d_delivery_evo}
  category investing.capital.construction
  amount = inputs.obligations_at_co
}

// ---- aubrey: delivered ${d_delivery_aubrey}, sold ${d_sale} — still in lease-up. Each
// tower leases up from its own delivery at the stated pace; rent is the
// derived rent plus other income, net of the vacancy allowance.
stream cre.aubrey_rent on entity asset.west inflow currency USD {
  schedule every month start from ${d_delivery_aubrey} to ${d_sale}
  category operating.revenue.base_rent
  amount = min(inputs.aubrey_units, max(0.0, (time.t - (inputs.delivery_aubrey - 1.0)) * inputs.units_per_month))
         * inputs.aubrey_rent * (1.0 + inputs.other_income_pct) * (1.0 - inputs.vacancy)
}

stream cre.aubrey_retail on entity asset.west inflow currency USD {
  schedule every month start from ${d_delivery_aubrey} to ${d_sale}
  category operating.revenue.other
  amount = inputs.aubrey_retail_sf * inputs.retail_rent_psf * inputs.retail_occupancy / 12.0
}

stream cre.aubrey_parking on entity asset.west inflow currency USD {
  schedule every month start from ${d_delivery_aubrey} to ${d_sale}
  category operating.revenue.other
  amount = inputs.aubrey_units * inputs.parking_ratio * inputs.parking_per_space
}

stream cre.aubrey_opex on entity asset.west outflow currency USD {
  schedule every month start from ${d_delivery_aubrey} to ${d_sale}
  category operating.expense.opex
  amount = inputs.aubrey_units * inputs.expenses_per_unit / 12.0
}

stream cre.aubrey_tax on entity asset.west outflow currency USD {
  schedule every month start from ${d_delivery_aubrey} to ${d_sale}
  category operating.expense.opex
  amount = inputs.aubrey_assessed * inputs.effective_tax_rate / 12.0
}

// ---- evo: delivered ${d_delivery_evo}, sold ${d_sale} — still in lease-up
stream cre.evo_rent on entity asset.east inflow currency USD {
  schedule every month start from ${d_delivery_evo} to ${d_sale}
  category operating.revenue.base_rent
  amount = min(inputs.evo_units, max(0.0, (time.t - (inputs.delivery_evo - 1.0)) * inputs.units_per_month))
         * inputs.evo_rent * (1.0 + inputs.other_income_pct) * (1.0 - inputs.vacancy)
}

stream cre.evo_retail on entity asset.east inflow currency USD {
  schedule every month start from ${d_delivery_evo} to ${d_sale}
  category operating.revenue.other
  amount = inputs.evo_retail_sf * inputs.retail_rent_psf * inputs.retail_occupancy / 12.0
}

stream cre.evo_parking on entity asset.east inflow currency USD {
  schedule every month start from ${d_delivery_evo} to ${d_sale}
  category operating.revenue.other
  amount = inputs.evo_units * inputs.parking_ratio * inputs.parking_per_space
}

stream cre.evo_opex on entity asset.east outflow currency USD {
  schedule every month start from ${d_delivery_evo} to ${d_sale}
  category operating.expense.opex
  amount = inputs.evo_units * inputs.expenses_per_unit / 12.0
}

stream cre.evo_tax on entity asset.east outflow currency USD {
  schedule every month start from ${d_delivery_evo} to ${d_sale}
  category operating.expense.opex
  amount = inputs.evo_assessed * inputs.effective_tax_rate / 12.0
}


// ---------------------------------------------------------------------- exit
// Both towers sold ${d_sale}-17 as ONE transaction; only the deed recording
// dates differ (5/18 leasehold, 7/12 fee), which is why the press reported two.
stream cre.aubrey_sale on entity asset.west inflow currency USD {
  schedule on ${d_sale}
  category investing.disposal.reversion
  amount = inputs.aubrey_price
}

stream cre.evo_sale on entity asset.east inflow currency USD {
  schedule on ${d_sale}
  category investing.disposal.reversion
  amount = inputs.evo_price
}

stream cre.sale_costs on entity container.project outflow currency USD {
  schedule on ${d_sale}
  category investing.disposal.selling_costs
  amount = (inputs.aubrey_price + inputs.evo_price) * inputs.cost_of_sale
}

stream cre.pierce_closings on entity asset.east inflow currency USD {
  schedule every month start from ${d_condo_first_close} to ${d_condo_last_close}
  category investing.disposal.reversion
  amount = curve_value("pierce_sellout", time.date) * (1.0 - inputs.condo_selling_cost)
}

// ------------------------------------------------------------- the facility
// The $$380M construction facility (Mack Real Estate Credit Strategies).
//
// Equity funds to its commitment first, the loan draws the residual, interest
// capitalizes into the balance, and sale and condo proceeds repay it. Each
// field reads only the prior close -- `prev` -- and the program's schedule, so
// every month resolves from the month before it.
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
                       + if(time.t >= inputs.construction_start, inputs.obligations_at_permit, 0.0)
                       + if(time.t >= inputs.delivery_evo, inputs.obligations_at_co, 0.0))

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
                               + if(time.t >= inputs.construction_start, inputs.obligations_at_permit, 0.0)
                               + if(time.t >= inputs.delivery_evo, inputs.obligations_at_co, 0.0))
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
                               + if(time.t >= inputs.construction_start, inputs.obligations_at_permit, 0.0)
                               + if(time.t >= inputs.delivery_evo, inputs.obligations_at_co, 0.0))
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
                               + if(time.t >= inputs.construction_start, inputs.obligations_at_permit, 0.0)
                               + if(time.t >= inputs.delivery_evo, inputs.obligations_at_co, 0.0))
                             - prev.asset.facility.equity_funded)),
                max(0.0, inputs.loan_commitment
                         - prev.asset.facility.balance * (1.0 + inputs.loan_rate / 12.0)))
                        - prev.asset.program.proceeds_ahead)
}

stream cre.loan_draw on entity container.project inflow currency USD {
  schedule every month start from ${d_grid_start} to ${d_grid_end}
  category financing.debt.proceeds
  amount = asset.facility.draw
}

// Interest capitalizes: the facility funds its own accrual, so the two legs
// net to zero in cash while the balance grows. Stated GROSS rather than folded
// into the balance silently, so `domain.cre.debt_service` sees real interest
// and coverage during the build is measurable instead of absent.
stream cre.loan_interest on entity container.project outflow currency USD {
  schedule every month start from ${d_grid_start} to ${d_grid_end}
  category financing.debt.interest_paid
  amount = asset.facility.interest
}

stream cre.loan_interest_funding on entity container.project inflow currency USD {
  schedule every month start from ${d_grid_start} to ${d_grid_end}
  category financing.debt.proceeds
  amount = asset.facility.interest
}

// The payoff sits in the reversion. `financing.debt.principal` folds into
// `domain.cre.debt_service`, and a balance retired out of sale proceeds is not
// debt service — it would make every coverage ratio in the disposal period
// meaningless. The cre pack says the same of a permanent loan's balloon.
stream cre.loan_repayment on entity container.project outflow currency USD {
  schedule every month start from ${d_grid_start} to ${d_grid_end}
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
// Compounding that $$67M from 2011 instead consumes the entire promote, which
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
             next prev.asset.jv.unreturned
                    * (1.0 + if(time.t >= inputs.construction_start, inputs.pref_rate / 12.0, 0.0))
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
}

// WHAT EACH PARTNER PUT IN, as cash. The venture funds pro rata -- 90% Baupost,
// 10% Penzance, the same share the tiers split on -- on the dates the facility
// draws equity, which the facility's own field states. Each contribution is a
// stream into the project that moves the partner's capital account, so the
// account carries the capital out as it is paid in and the distributions back
// in when the venture allocates; nothing restates the draw.
stream cre.equity_contribution.baupost_land on entity container.project inflow currency USD {
  schedule on ${d_grid_start}
  category financing.equity.contribution
  amount = asset.facility.equity_funded * (1.0 - inputs.sponsor_share)
  moves baupost_capital
}

stream cre.equity_contribution.baupost on entity container.project inflow currency USD {
  schedule every month start from ${d_grid_second} to ${d_grid_end}
  category financing.equity.contribution
  amount = (asset.facility.equity_funded - prev.asset.facility.equity_funded)
           * (1.0 - inputs.sponsor_share)
  moves baupost_capital
}

stream cre.equity_contribution.penzance_land on entity container.project inflow currency USD {
  schedule on ${d_grid_start}
  category financing.equity.contribution
  amount = asset.facility.equity_funded * inputs.sponsor_share
  moves penzance_capital
}

stream cre.equity_contribution.penzance on entity container.project inflow currency USD {
  schedule every month start from ${d_grid_second} to ${d_grid_end}
  category financing.equity.contribution
  amount = (asset.facility.equity_funded - prev.asset.facility.equity_funded)
           * inputs.sponsor_share
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

// The deal's own cash, with the partners' contributions left out: what the
// workbook ties to, and what the project returns on.
slice deal {
  entity container.project
  except category "financing.equity.contribution"
}

// -------------------------------------------------------------- the JV split
// Penzance / Baupost terms are not public; these tiers are stated assumptions.
waterfall jv.distribution on entity container.project {
  schedule on ${d_condo_last_close} end
  from deal_cash

  // Capital back first: each partner is repaid the capital it has not yet had
  // returned, which is what its own balance carries. The preference is tracked
  // separately, because it compounds.
  pay capital_inv   to party.baupost  = min(0.0 - prev.baupost_capital, remaining)
  pay capital_sp    to party.penzance = min(0.0 - prev.penzance_capital, remaining)

  pay preferred_inv to party.baupost  = (asset.jv.unreturned - asset.jv.capital) * (1.0 - inputs.sponsor_share)
  pay preferred_sp  to party.penzance = (asset.jv.unreturned - asset.jv.capital) * inputs.sponsor_share
  pay promote       to party.penzance = remaining * inputs.promote_share
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
''')

(CASE / "model.cfdl").write_text(MODEL.substitute(values))
print(f"wrote model.cfdl  {len((CASE / 'model.cfdl').read_text().splitlines())} lines")
