"""Generate benchmarks/cre/penzance_highlands from the frozen Penzance input set."""
import csv, datetime as dt, json, tomllib
from pathlib import Path

CASE = Path(__file__).resolve().parent
OUT  = CASE
T    = tomllib.load(open(CASE / "inputs" / "highlands.toml", "rb"))

M0, N       = dt.date(2011, 9, 1), 160
T_C0, T_C1  = 79, 117          # construction window (2018-04 .. 2021-06)
T_AUB, T_EVO, T_EXIT = 118, 120, 128
T_DIST      = 153              # 2024-06, the last condominium closing
WSUM        = sum((t - 78) * (118 - t) for t in range(T_C0, T_C1 + 1))

def ym(t):
    m = M0.month - 1 + t
    return f"{M0.year + m // 12}-{m % 12 + 1:02d}"

LAND   = 67_000_000.0
COSTS  = {"hard_buildings": 301_693_050.00, "hard_garage": 33_184_000.00,
          "fire_station": 7_454_800.00, "park_and_street": 14_000_000.00,
          "soft_costs": 60_576_414.50, "contingency": 17_816_592.50,
          "developer_fee": 17_994_713.10}
OBLIG  = {T_C0: 157_303.0, T_EVO: 12_252_500.0}

def cost_at(t):
    """Total development cost in period t — a pure function of time."""
    c = LAND if t == 0 else 0.0
    if T_C0 <= t <= T_C1:
        c += sum(COSTS.values()) * (t - 78) * (118 - t) / WSUM
    return c + OBLIG.get(t, 0.0)

# ---- rental parameters (Arlington Guidebook + SP445 program) ----------------
A = dict(units=331, rent=3522.0, retail=9_788,  assess=229_060_900, deliv=T_AUB, ent="west")
E = dict(units=455, rent=3273.0, retail=13_891, assess=290_055_300, deliv=T_EVO, ent="east")
OTHER, VAC, OPEX, PRATE, PRATIO = 0.04, 0.08, 7511.0, 150.0, 0.80
RRENT, ROCC, TAXR, RAMP = 48.0, 0.90, 0.0113, 25.0

L = []
w = L.append
w(f'''version 0.1
model "penzance-highlands" currency USD
use pack "cre" version "0.1.0"
time calendar monthly from {ym(0)} for {N}

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

phase predevelopment from {ym(0)} to {ym(T_C0 - 1)}
phase construction   from {ym(T_C0)} to {ym(T_C1)}
phase lease_up       from {ym(T_AUB)} to {ym(T_EXIT)}
phase condo_tail     from {ym(T_EXIT + 1)} to {ym(N - 1)}

assume loan_rate        = 0.065
assume loan_commitment  = 380000000
assume equity_commitment = 186245280.585
assume condo_selling_cost = 0.05
assume cost_of_sale     = 0.01
assume pref_rate        = 0.08
assume sponsor_share    = 0.10
''')

# ---- curves: the deterministic cost path, so a recurrence can read it -------
w("// Development cost per period and its running total. Declared as curves so\n"
  "// the facility's recurrence can read them. Cost here is a pure function of\n"
  "// time.\n"
  "//\n"
  "// EVERY period is declared, including the zeros. A step curve is\n"
  "// flat-forward (docs/03 §4): omit the quiet months and the last construction\n"
  "// draw is held forward indefinitely and the balance compounds without end.\n"
  "curve dev_cost {")
for t in range(N):
    w(f"  {ym(t)}: {cost_at(t):.4f}")
w("}\n")
cum = 0.0
w("curve dev_cost_cum {")
for t in range(N):
    cum += cost_at(t)
    w(f"  {ym(t)}: {cum:.4f}")
w("}\n")

rows = list(csv.DictReader(open(CASE / "inputs" / "pierce_sellout_actual.csv")))
sched = {int(r["month_index"]) - 1: float(r["gross_proceeds"]) for r in rows}
lo, hi = min(sched), max(sched)
w("// Pierce condo closings — every one of 102 recorded sales, by month.\n"
  "// 34 months, not the smooth absorption an assumption would give.\n"
  "curve pierce_sellout {")
for t in range(lo, hi + 1):
    w(f"  {ym(t)}: {sched.get(t, 0.0):.2f}")
w("}\n")

w(f'''
// ---------------------------------------------------------------- structure
entity container project : CRE.Container.Portfolio
entity asset east : CRE.Asset.RealProperty {{ asset_class = "mixed_use"  part of container.project }}
entity asset west : CRE.Asset.RealProperty {{ asset_class = "multifamily"  part of container.project }}

entity party penzance : CRE.Party.Sponsor  {{ name = "Penzance" }}
entity party baupost  : CRE.Party.Investor {{ name = "The Baupost Group" }}
entity party mack     : CRE.Party.Lender   {{ name = "Mack Real Estate Credit Strategies" }}

// ------------------------------------------------------------------ capital
stream cre.land on entity container.project outflow currency USD {{
  schedule on {ym(0)}
  category investing.capital.capex
  amount = {LAND:.2f}
}}
''')

for name, total in COSTS.items():
    w(f'''
stream cre.{name} on entity container.project outflow currency USD {{
  schedule every month start from {ym(T_C0)} to {ym(T_C1)}
  category investing.capital.construction
  amount = {total:.2f} * (time.t - {T_C0 - 1}.0) * ({T_C1 + 1}.0 - time.t) / {WSUM}.0
}}''')

w(f'''

// SP #445 conditions: utility undergrounding and TDM at permit; public art,
// green building and affordable housing at certificate of occupancy.
stream cre.obligations_permit on entity container.project outflow currency USD {{
  schedule on {ym(T_C0)}
  category investing.capital.construction
  amount = {OBLIG[T_C0]:.2f}
}}

stream cre.obligations_co on entity container.project outflow currency USD {{
  schedule on {ym(T_EVO)}
  category investing.capital.construction
  amount = {OBLIG[T_EVO]:.2f}
}}
''')

for tag, p in (("aubrey", A), ("evo", E)):
    occ = f'min({p["units"]}.0, max(0.0, (time.t - {p["deliv"] - 1}.0) * {RAMP}))'
    ent = f'asset.{p["ent"]}'
    w(f'''
// ---- {tag}: delivered {ym(p["deliv"])}, sold {ym(T_EXIT)} — still in lease-up
stream cre.{tag}_rent on entity {ent} inflow currency USD {{
  schedule every month start from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.revenue.base_rent
  amount = {occ} * {p["rent"] * (1 + OTHER) * (1 - VAC):.10g}
}}

stream cre.{tag}_retail on entity {ent} inflow currency USD {{
  schedule every month start from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.revenue.other
  amount = {p["retail"] * RRENT * ROCC / 12:.10g}
}}

stream cre.{tag}_parking on entity {ent} inflow currency USD {{
  schedule every month start from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.revenue.other
  amount = {p["units"] * PRATIO * PRATE:.10g}
}}

stream cre.{tag}_opex on entity {ent} outflow currency USD {{
  schedule every month start from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.expense.opex
  amount = {p["units"] * OPEX / 12:.10g}
}}

stream cre.{tag}_tax on entity {ent} outflow currency USD {{
  schedule every month start from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.expense.opex
  amount = {p["assess"] * TAXR / 12:.10g}
}}''')

AUB_P, EVO_P = 266_455_000.0, 334_642_240.0
w(f'''

// ---------------------------------------------------------------------- exit
// Both towers sold 2022-05-17 as ONE transaction; only the deed recording
// dates differ (5/18 leasehold, 7/12 fee), which is why the press reported two.
stream cre.aubrey_sale on entity asset.west inflow currency USD {{
  schedule on {ym(T_EXIT)}
  category investing.disposal.reversion
  amount = {AUB_P:.2f}
}}

stream cre.evo_sale on entity asset.east inflow currency USD {{
  schedule on {ym(T_EXIT)}
  category investing.disposal.reversion
  amount = {EVO_P:.2f}
}}

stream cre.sale_costs on entity container.project outflow currency USD {{
  schedule on {ym(T_EXIT)}
  category investing.disposal.selling_costs
  amount = {(AUB_P + EVO_P):.2f} * inputs.cost_of_sale
}}

stream cre.pierce_closings on entity asset.east inflow currency USD {{
  schedule every month start from {ym(lo)} to {ym(hi)}
  category investing.disposal.reversion
  amount = curve_value("pierce_sellout", time.date) * (1.0 - inputs.condo_selling_cost)
}}
''')

# ---- proceeds available to repay the facility (deterministic in time) ------
AUB_P2, EVO_P2 = 266_455_000.0, 334_642_240.0
proceeds = {t: v * 0.95 for t, v in sched.items()}
proceeds[T_EXIT] = proceeds.get(T_EXIT, 0.0) + (AUB_P2 + EVO_P2) * (1 - 0.01)
w("// Cash available to repay the facility: condo closings net of selling costs,")
w("// plus the two tower sales net of cost of sale. Every period is declared, so")
w("// a quiet month reads zero.")
w("curve loan_proceeds {")
for t in range(N):
    w("  %s: %.4f" % (ym(t), proceeds.get(t, 0.0)))
w("}\n")

EQD = ('(min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))'
       ' - min(inputs.equity_commitment, curve_value("dev_cost_cum", edate(time.date, -1))))')
INT = 'prev.asset.facility.balance * inputs.loan_rate / 12.0'
DRAW = ('min(max(0.0, curve_value("dev_cost", time.date) - %s), '
        'max(0.0, inputs.loan_commitment - prev.asset.facility.balance - %s))' % (EQD, INT))
REPAY = ('min(prev.asset.facility.balance + %s + %s, '
         'max(0.0, curve_value("loan_proceeds", time.date)))' % (INT, DRAW))
INIT_DRAW = 'max(0.0, curve_value("dev_cost", time.date) - inputs.equity_commitment)'

w("""
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
           next %s

  draw init %s
       next %s

  repay init 0.0
        next %s

  balance init %s
          next max(0.0, prev.asset.facility.balance + %s + %s - %s)
}

// Interest is capitalized: it is repaid inside the principal repayment. That is the convention the reference workbook uses.
stream cre.loan_draw on entity container.project inflow currency USD {
  schedule every month start from %s to %s
  category financing.debt.proceeds
  amount = asset.facility.draw
}

// Interest capitalizes: the facility funds its own accrual, so the two legs
// net to zero in cash while the balance grows. Stated GROSS rather than folded
// into the balance silently, so `domain.cre.debt_service` sees real interest
// and coverage during the build is measurable instead of absent.
stream cre.loan_interest on entity container.project outflow currency USD {
  schedule every month start from %s to %s
  category financing.debt.interest_paid
  amount = asset.facility.interest
}

stream cre.loan_interest_funding on entity container.project inflow currency USD {
  schedule every month start from %s to %s
  category financing.debt.proceeds
  amount = asset.facility.interest
}

// The payoff sits in the reversion. `financing.debt.principal` folds into
// `domain.cre.debt_service`, and a balance retired out of sale proceeds is not
// debt service — it would make every coverage ratio in the disposal period
// meaningless. The cre pack says the same of a permanent loan's balloon.
stream cre.loan_repayment on entity container.project outflow currency USD {
  schedule every month start from %s to %s
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
             next prev.asset.jv.unreturned * (1.0 + if(time.t >= %d, inputs.pref_rate / 12.0, 0.0))
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
// against it. The venture funds pro rata -- 90%% Baupost, 10%% Penzance, the
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
  schedule on %s end
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
// Penzance's figure is all-in: its preferred and residual as a 10%% investor,
// and the promote it earned as sponsor. Each tier is reported on its own, so
// the promote can be read separately from the investor return.
metric baupost_irr   = irr(party.baupost)
metric baupost_moic  = moic(party.baupost)
metric penzance_irr  = irr(party.penzance)
metric penzance_moic = moic(party.penzance)
""" % (INT, INIT_DRAW, DRAW, REPAY, INIT_DRAW, DRAW, INT, REPAY,
       ym(0), ym(N - 1), ym(0), ym(N - 1), ym(0), ym(N - 1),
       ym(0), ym(N - 1), T_C0, ym(T_DIST)))

OUT.mkdir(parents=True, exist_ok=True)
(OUT / "model.cfdl").write_text("\n".join(L))
print("wrote", OUT / "model.cfdl", len((OUT / "model.cfdl").read_text().splitlines()), "lines")
