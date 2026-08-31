"""Generate benchmarks/cre/penzance_one_rosslyn from the frozen input set.

The companion case, penzance_highlands, reconstructs a COMPLETED deal. This one
projects an entitled but unbuilt project, so the ratio of fact to forecast is
inverted. Every rate is sourced -- Arlington's 2026 Guidebook for operations,
BLS output-price indices for escalation, recorded trades for the exit -- and the
model says which is which.

Two exit strategies over one set of facts, selected by `inputs.scenario_b`:
  0  merchant build   sell in lease-up ~10 months after delivery. What Penzance
                      and Baupost actually did at The Highlands, where both
                      towers sold at 83% and 60% occupancy.
  1  build to core    stabilize, refinance into permanent debt, hold five years.

The reference workbook is the independent implementation; this reconciles to it.
"""
import datetime as dt
import json
import tomllib
from pathlib import Path

CASE = Path(__file__).resolve().parent
T = tomllib.load(open(CASE / "inputs" / "one_rosslyn.toml", "rb"))
PR, LAND, OB = T["program"], T["land"], T["obligations"]
OP, CP, CN, FI, CD = T["operating"], T["comps"], T["construction"], T["finance"], T["condo"]

M0 = dt.date(2023, 11, 1)
PROJECT = 12               # valuation tail: the year of income the sale is priced on
T_START, T_MONTHS = 37, 42
T_END = T_START + T_MONTHS - 1
T_NE, T_NW, T_S = 79, 79, 85
LEASEUP = 18
T_STAB = T_S + LEASEUP
A_EXIT, B_REFI, B_EXIT = T_S + 10, T_STAB, T_STAB + 60
N = B_EXIT + 1             # the cash horizon closes on the scenario B sale
CONDO0, CONDO_M = T_NE, 24
GROWTH, MARKET_FACTOR = 0.03, 1.00
PERM_SPREAD, PERM_LTV, PERM_AMORT = 0.0175, 0.60, 30
BASE_T = 32                      # 2026-07, the Guidebook's own vintage


def ym(t):
    m = M0.month - 1 + t
    return f"{M0.year + m // 12}-{m % 12 + 1:02d}"


ESC = CN["escalation_realized"] * (1 + CN["escalation_forward_rate"]) ** 2.17
HARD = CN["highlands_buildings_psf"] * ESC * PR["total_gfa_sf"] \
     + CN["highlands_parking_space"] * ESC * PR["parking_spaces"]
CONSTRUCTION = HARD * (1 + CN["soft_ratio"] + CN["contingency_ratio"] + CN["developer_fee_ratio"])
WSUM = sum((t - (T_START - 1)) * ((T_END + 1) - t) for t in range(T_START, T_END + 1))
RENTAL = PR["nw_tower"]["units"] + PR["s_tower"]["units"]
# ACCUMULATE, do not assign: the NE and NW towers deliver in the same month, so
# a dict literal keyed by period silently drops one tranche. That cost $651,858
# of a recorded obligation until the reference model disagreed with the engine.
OBLIG = {}
for _t, _amt in [(T_START, OB["ahif_base_density"] + OB["public_space_rmsa"] + OB["transportation_rmsa"]),
                 (T_NE, OB["ahif_tranche_ne"]), (T_NW, OB["ahif_tranche_nw"]),
                 (T_S, OB["ahif_tranche_s"])]:
    OBLIG[_t] = OBLIG.get(_t, 0.0) + _amt
assert abs(sum(OBLIG.values()) - (OB["ahif_base_density"] + OB["ahif_additional_density"]
           + OB["public_space_rmsa"] + OB["transportation_rmsa"])) < 0.01, "obligations lost"
EQUITY_COMMIT = (LAND["purchase_price_usd"] + CONSTRUCTION + sum(OBLIG.values())) * (1 - FI["ltc"])
LOAN_COMMIT = CONSTRUCTION * FI["ltc"]
CAP, EXP, VAC, RENT = (OP["cap_rate_metro_highrise"], OP["expenses_per_unit"],
                       OP["vacancy_collection"], OP["rent_per_unit_month"])
PARK = 100.0


def esc(t):
    return (1 + GROWTH) ** ((t - BASE_T) / 12)


def dev(t):
    c = LAND["purchase_price_usd"] if t == 0 else 0.0
    if T_START <= t <= T_END:
        c += CONSTRUCTION * (t - (T_START - 1)) * ((T_END + 1) - t) / WSUM
    return c + OBLIG.get(t, 0.0)


def occ(t, d):
    return 0.0 if t < d else min(1.0, (t - d + 1) / LEASEUP) * (1 - VAC)


def units(t):
    return PR["nw_tower"]["units"] * occ(t, T_NW) + PR["s_tower"]["units"] * occ(t, T_S)


def condo(t):
    return (PR["ne_tower"]["units"] / CONDO_M * CD["pierce_per_unit"] * (1 - CD["selling_cost"])
            if CONDO0 <= t < CONDO0 + CONDO_M else 0.0)


def stabilized_value(t):
    """What a buyer prices: stabilized NOI at t's rent level over the guideline cap."""
    noi_u = (RENT * esc(t) + PARK * 0.5 * esc(t)) * 12 * (1 - VAC) - EXP * esc(t)
    return noi_u / CAP * MARKET_FACTOR * RENTAL


A_VALUE, B_VALUE = stabilized_value(A_EXIT), stabilized_value(B_EXIT)
A_EXIT_T, B_EXIT_T = A_EXIT, B_EXIT
HELD = f"if(inputs.scenario_b > 0.5, 1.0, if(time.t <= {A_EXIT}, 1.0, 0.0))"

PERM_RATE = FI["ust10"] + PERM_SPREAD
PERM_PRINCIPAL = stabilized_value(B_REFI) * PERM_LTV
_i = PERM_RATE / 12
PERM_PMT = PERM_PRINCIPAL * _i / (1 - (1 + _i) ** (-PERM_AMORT * 12))

# Permanent balance at the scenario-B exit, after 60 amortizing payments.
_bal = PERM_PRINCIPAL
for _ in range(B_EXIT - B_REFI):
    _bal += _bal * _i - PERM_PMT
PERM_PAYOFF = _bal

L = []
w = L.append
w(f'''version 0.1
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
// Sourcing, strongest first:
//   program        Arlington County Board Report, SP #419 -- fact
//   land           deed 20230100013266, 2023-11-14, $52,000,000 -- fact
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
// THE EXIT IS DERIVED, NOT PICKED. The sale is valued on the twelve months of
// income that follow it, read from the projection tail, divided by the County's
// guideline loaded cap and adjusted by a stated market factor. The factor is 1.00 -- the guideline basis itself. The two market
// observations available bracket it: The Highlands sold at +32% to this basis
// in 2022, and Central Place at -11.6% in 2026. Choosing either as the base
// would import a market call the record does not support.
//
// At the scenario-A exit the south tower is roughly 58% leased, so that sale is
// priced on stabilized NOI. In-place income would understate it by about $140M.
// The Highlands towers sold at 83% and 60% occupancy.

time calendar monthly from {ym(0)} for {N} project {PROJECT}

// 0 = merchant build, 1 = build to core. Every scenario-dependent stream is
// weighted by this, so one model carries both and run.json selects.
assume scenario_b        = 1.0

// The market's premium or discount to the County's guideline basis. 1.00 IS
// the guideline basis. The two observations available bracket it: The
// Highlands sold at +32% to this basis in 2022, Central Place at -11.6% in
// 2026. It scales the EXIT only. A lender sizes the permanent loan off the
// appraised basis at stabilization, not off the price a later buyer pays.
assume market_factor     = 1.0

assume rent_per_unit     = {RENT}.0
assume expenses_per_unit = {EXP}.0
assume vacancy           = {VAC}
assume parking_per_space = {PARK}
assume growth            = {GROWTH}
assume cap_rate          = {CAP}
assume loan_rate         = {FI["loan_rate"]}
assume loan_commitment   = {round(LOAN_COMMIT, 2)}
assume equity_commitment = {round(EQUITY_COMMIT, 2)}
assume pref_rate         = 0.08
assume sponsor_share     = 0.10
''')

# ---- curves ----------------------------------------------------------------
for name, fn, note in [
    ("dev_cost", dev, "Development cost by period, land and obligations included.\n"
                      "// EVERY period is declared, including the zeros: a step curve is\n"
                      "// flat-forward, so an omitted month holds the last draw forward."),
    ("units_occupied", units, "Rental units in occupancy, net of the 5% vacancy and\n"
                              "// collection allowance the Guidebook states."),
    ("condo_proceeds", condo, "Condominium sellout, net of selling costs."),
]:
    w(f"\n// {note}\ncurve {name} {{")
    for t in range(N):
        w(f"  {ym(t)}: {fn(t):.4f}")
    w("}\n")

w("// Cumulative development cost, which is what the equity commitment is\n"
  "// measured against.\ncurve dev_cost_cum {")
c = 0.0
for t in range(N):
    c += dev(t)
    w(f"  {ym(t)}: {c:.2f}")
w("}\n")

# The two scenario-specific liquidity events, as separate curves. A curve name
# cannot be selected at runtime, so the facility composes them with the switch.
for name, when, amount, note in [
    ("exit_a_proceeds", A_EXIT, A_VALUE, "Scenario A: the lease-up sale."),
    ("refi_proceeds", B_REFI, PERM_PRINCIPAL, "Scenario B: permanent loan proceeds at stabilization."),
]:
    w(f"// {note}\ncurve {name} {{")
    for t in range(N):
        w(f"  {ym(t)}: {amount if t == when else 0.0:.2f}")
    w("}\n")

w(f'''
// ---------------------------------------------------------------- structure
entity container project : CRE.Container.Portfolio
entity asset ne : CRE.Asset.RealProperty {{ asset_class = "condominium"  part of container.project }}
entity asset nw : CRE.Asset.RealProperty {{ asset_class = "multifamily"  part of container.project }}
entity asset south : CRE.Asset.RealProperty {{ asset_class = "multifamily"  part of container.project }}

entity party penzance : CRE.Party.Sponsor  {{ name = "Penzance" }}
entity party baupost  : CRE.Party.Investor {{ name = "The Baupost Group" }}

// FACT. Recorded 2023-11-14, deed 20230100013266, sales code "4-Multiple RPCs",
// so the price is the aggregate across all three parcels. The County's own
// guideline land value for Metro high-rise is $78,000 per rental unit, putting
// 772 rental units at $60,216,000: the site was bought 13.6% below guideline.
stream cre.development on entity container.project outflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N-1)}
  category investing.capital.capex
  amount = curve_value("dev_cost", time.date)
}}

// ------------------------------------------------------------- operations
// Rent is DERIVED: a comparable's assessed value times the guideline loaded cap
// gives the assessor's own NOI; add back guideline expenses and gross up for
// vacancy. Escalated from the Guidebook's 2026-07 vintage, NOT frozen there --
// this project delivers in 2030 and freezing 2026 dollars would understate it.
stream cre.rent on entity container.project inflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N - 1 + PROJECT)}
  category operating.revenue.base_rent
  amount = curve_value("units_occupied", time.date) * inputs.rent_per_unit
         * pow(1.0 + inputs.growth, (time.t - {BASE_T}) / 12.0) * {HELD}
}}

stream cre.parking on entity container.project inflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N - 1 + PROJECT)}
  category operating.revenue.other
  amount = curve_value("units_occupied", time.date) * inputs.parking_per_space * 0.5
         * pow(1.0 + inputs.growth, (time.t - {BASE_T}) / 12.0) * {HELD}
}}

stream cre.opex on entity container.project outflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N - 1 + PROJECT)}
  category operating.expense.opex
  amount = curve_value("units_occupied", time.date) * inputs.expenses_per_unit / 12.0
         * pow(1.0 + inputs.growth, (time.t - {BASE_T}) / 12.0) * {HELD}
}}

// The weakest input in the case, and it carries the most weight: the NE tower's
// 73 units average 2,273 sq ft, far larger than the Pierce units this is
// anchored to, and no Rosslyn condominium of that size has traded recently.
stream cre.condo_closings on entity container.project inflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N-1)}
  category investing.disposal.reversion
  amount = curve_value("condo_proceeds", time.date)
}}

// ------------------------------------------------------------- the facility
// PROJECTION. No instrument is recorded. SOFR at 2026-08-24 was 3.65% (FRED);
// the spread over it is a projection. Equity funds to its commitment first, the
// facility draws the residual, and interest capitalizes into the balance.
// Each recurrence reads `prev`, the cost curves and the inputs, so every month
// resolves from the month before it.
entity asset facility : Asset.Financial {{
  equity_funded init min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))
                next min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))

  interest init 0.0
           next prev.asset.facility.balance * inputs.loan_rate / 12.0

  draw init max(0.0, curve_value("dev_cost", time.date) - inputs.equity_commitment)
       next {{DRAW_EXPR}}

  // The facility is repaid from condominium closings, then from whichever
  // liquidity event the scenario reaches: the sale, or the refinancing.
  repay init 0.0
        next min(prev.asset.facility.balance + prev.asset.facility.balance * inputs.loan_rate / 12.0 + {{DRAW_EXPR}},
                 max(0.0, curve_value("condo_proceeds", time.date)
                        + curve_value("exit_a_proceeds", time.date) * (1.0 - inputs.scenario_b)
                        + curve_value("refi_proceeds", time.date) * inputs.scenario_b))

  balance init max(0.0, curve_value("dev_cost", time.date) - inputs.equity_commitment)
          next max(0.0, prev.asset.facility.balance + {{DRAW_EXPR}}
                      + prev.asset.facility.balance * inputs.loan_rate / 12.0
                      - min(prev.asset.facility.balance + prev.asset.facility.balance * inputs.loan_rate / 12.0 + {{DRAW_EXPR}},
                            max(0.0, curve_value("condo_proceeds", time.date)
                                   + curve_value("exit_a_proceeds", time.date) * (1.0 - inputs.scenario_b)
                                   + curve_value("refi_proceeds", time.date) * inputs.scenario_b)))
}}

stream cre.loan_draw on entity container.project inflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N-1)}
  category financing.debt.proceeds
  amount = asset.facility.draw
}}

// Interest capitalizes: stated GROSS, an outflow against a matching draw, so
// coverage stays measurable. The two legs net to zero in cash.
stream cre.loan_interest on entity container.project outflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N-1)}
  category financing.debt.interest_paid
  amount = asset.facility.interest
}}

stream cre.loan_interest_funding on entity container.project inflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N-1)}
  category financing.debt.proceeds
  amount = asset.facility.interest
}}

// A balance retired out of disposal proceeds is a reversion, not debt service:
// folding it into debt service makes every coverage ratio in the period
// meaningless. The cre pack says the same of a permanent loan's balloon.
stream cre.loan_repayment on entity container.project outflow currency USD {{
  schedule every month start from {ym(0)} to {ym(N-1)}
  category investing.disposal.reversion
  amount = asset.facility.repay
}}

// ----------------------------------------------------- scenario A: the sale
stream cre.exit_a on entity container.project inflow currency USD {{
  schedule on {ym(A_EXIT)} end
  category investing.disposal.reversion
  amount = {A_VALUE:.2f} * inputs.market_factor * (1.0 - inputs.scenario_b)
}}

// ------------------------------------------- scenario B: refinance and hold
// Permanent debt at stabilization: the 10-year Treasury at 2026-08-24 was
// 4.70% (FRED) and the spread over it is a projection. Sized at 60% of the
// stabilized value, amortizing over 30 years.
stream cre.refinance on entity container.project inflow currency USD {{
  schedule on {ym(B_REFI)} end
  category financing.debt.proceeds
  amount = {PERM_PRINCIPAL:.2f} * inputs.scenario_b
}}

stream cre.perm_debt_service on entity container.project outflow currency USD {{
  schedule every month end from {ym(B_REFI + 1)} to {ym(B_EXIT)}
  category financing.debt.service
  amount = {PERM_PMT:.2f} * inputs.scenario_b
}}

stream cre.exit_b on entity container.project inflow currency USD {{
  schedule on {ym(B_EXIT)} end
  category investing.disposal.reversion
  amount = (series_sum("cre.rent", time.t + 1, time.t + 12)
          + series_sum("cre.parking", time.t + 1, time.t + 12)
          + series_sum("cre.opex", time.t + 1, time.t + 12))
         / inputs.cap_rate * inputs.market_factor * inputs.scenario_b
}}

stream cre.perm_payoff on entity container.project outflow currency USD {{
  schedule on {ym(B_EXIT)} end
  category investing.disposal.reversion
  amount = {PERM_PAYOFF:.2f} * inputs.scenario_b
}}
'''.replace("{DRAW_EXPR}",
            'min(max(0.0, curve_value("dev_cost", time.date) - '
            '(min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date)) - '
            'min(inputs.equity_commitment, curve_value("dev_cost_cum", edate(time.date, -1))))), '
            'max(0.0, inputs.loan_commitment - prev.asset.facility.balance - '
            'prev.asset.facility.balance * inputs.loan_rate / 12.0))'))

(CASE / "run.json").write_text(json.dumps({
    "deterministic": {"annual_discount_rate": 0.10, "as_of": str(M0),
                      "parameters": {"inputs.scenario_b": 1.0}},
    "scenarios": {
        "merchant_build": {"parameters": {"inputs.scenario_b": 0.0}},
        "build_to_core": {"parameters": {"inputs.scenario_b": 1.0}},
        # The market factor, across the range the record supports. 0.884 is
        # where Central Place traded in 2026; 1.321 is where The Highlands
        # traded in 2022. 1.00 is the County's guideline basis.
        "merchant_at_2026_discount": {
            "parameters": {"inputs.scenario_b": 0.0, "inputs.market_factor": 0.884}},
        "merchant_at_2022_premium": {
            "parameters": {"inputs.scenario_b": 0.0, "inputs.market_factor": 1.321}},
        "core_at_2026_discount": {
            "parameters": {"inputs.scenario_b": 1.0, "inputs.market_factor": 0.884}},
        "core_at_2022_premium": {
            "parameters": {"inputs.scenario_b": 1.0, "inputs.market_factor": 1.321}},
    },
}, indent=2) + "\n")
(CASE / "model.cfdl").write_text("\n".join(L) + "\n")

print(f"wrote model.cfdl  {len((CASE / 'model.cfdl').read_text().splitlines())} lines")
print(f"  construction    {CONSTRUCTION:>16,.2f}")
print(f"  equity commit   {EQUITY_COMMIT:>16,.2f}")
print(f"  loan commit     {LOAN_COMMIT:>16,.2f}")
print(f"  A exit {ym(A_EXIT)} {A_VALUE:>16,.2f}")
print(f"  B refi {ym(B_REFI)} {PERM_PRINCIPAL:>16,.2f}   pmt {PERM_PMT:,.2f}")
print(f"  B exit {ym(B_EXIT)} {B_VALUE:>16,.2f}   payoff {PERM_PAYOFF:,.2f}")
