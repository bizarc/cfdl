"""Generate benchmarks/cre/penzance_highlands from the frozen Penzance input set."""
import csv, datetime as dt, json, tomllib
from pathlib import Path

EVS  = Path("/Users/matthewmccrea/Documents/evs-platform/docs/benchmarks/penzance-highlands")
OUT  = Path("/Users/matthewmccrea/Documents/cfdl/benchmarks/cre/penzance_highlands")
T    = tomllib.load(open(EVS / "inputs" / "highlands.toml", "rb"))

M0, N       = dt.date(2011, 9, 1), 160
T_C0, T_C1  = 79, 117          # construction window (2018-04 .. 2021-06)
T_AUB, T_EVO, T_EXIT = 118, 120, 128
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
// The for-sale and rental product share a basis, which is what makes this
// deal worth modelling rather than a generic development.
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
  "// the facility's recurrence can read them: `next` sees curves, but never a\n"
  "// stream (docs/03 §3.1), and cost here is a pure function of time.\n"
  "//\n"
  "// EVERY period is declared, including the zeros. A step curve is\n"
  "// flat-forward (docs/03 §4): omit the quiet months and the last construction\n"
  "// draw is held forever, which compounds into a balance that never stops.\n"
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

rows = list(csv.DictReader(open(EVS / "inputs" / "pierce_sellout_actual.csv")))
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
entity asset project : CRE.Asset.Portfolio
entity asset east : CRE.Asset.RealProperty {{ asset_class = "mixed_use"  part of asset.project }}
entity asset west : CRE.Asset.RealProperty {{ asset_class = "multifamily"  part of asset.project }}

// The construction facility. `balance` is a fact about the facility, so the
// facility holds it. Equity funds to its commitment first and the loan draws
// the residual — the standard structure, expressed as two recurrences.
entity asset facility : Asset.Financial {{
  equity_funded init min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))
                next min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))

  balance init 0.0
          next max(0.0,
                   prev
                   + max(0.0, curve_value("dev_cost", time.date)
                              - (min(inputs.equity_commitment, curve_value("dev_cost_cum", time.date))
                                 - min(inputs.equity_commitment, curve_value("dev_cost_cum", edate(time.date, -1)))))
                   + prev * inputs.loan_rate / 12.0)
}}

entity party penzance : CRE.Party.Sponsor  {{ name = "Penzance" }}
entity party baupost  : CRE.Party.Investor {{ name = "The Baupost Group" }}
entity party mack     : CRE.Party.Lender   {{ name = "Mack Real Estate Credit Strategies" }}

// ------------------------------------------------------------------ capital
stream cre.land on entity asset.project outflow currency USD {{
  schedule on {ym(0)}
  category investing.capital.capex
  amount = {LAND:.2f}
}}
''')

for name, total in COSTS.items():
    w(f'''
stream cre.{name} on entity asset.project outflow currency USD {{
  schedule every month from {ym(T_C0)} to {ym(T_C1)}
  category investing.capital.construction
  amount = {total:.2f} * (time.t - {T_C0 - 1}.0) * ({T_C1 + 1}.0 - time.t) / {WSUM}.0
}}''')

w(f'''

// SP #445 conditions: utility undergrounding and TDM at permit; public art,
// green building and affordable housing at certificate of occupancy.
stream cre.obligations_permit on entity asset.project outflow currency USD {{
  schedule on {ym(T_C0)}
  category investing.capital.construction
  amount = {OBLIG[T_C0]:.2f}
}}

stream cre.obligations_co on entity asset.project outflow currency USD {{
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
  schedule every month from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.revenue.base_rent
  amount = {occ} * {p["rent"] * (1 + OTHER) * (1 - VAC):.10g}
}}

stream cre.{tag}_retail on entity {ent} inflow currency USD {{
  schedule every month from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.revenue.other
  amount = {p["retail"] * RRENT * ROCC / 12:.10g}
}}

stream cre.{tag}_parking on entity {ent} inflow currency USD {{
  schedule every month from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.revenue.other
  amount = {p["units"] * PRATIO * PRATE:.10g}
}}

stream cre.{tag}_opex on entity {ent} outflow currency USD {{
  schedule every month from {ym(p["deliv"])} to {ym(T_EXIT)}
  category operating.expense.opex
  amount = {p["units"] * OPEX / 12:.10g}
}}

stream cre.{tag}_tax on entity {ent} outflow currency USD {{
  schedule every month from {ym(p["deliv"])} to {ym(T_EXIT)}
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
  category investing.reversion
  amount = {AUB_P:.2f}
}}

stream cre.evo_sale on entity asset.east inflow currency USD {{
  schedule on {ym(T_EXIT)}
  category investing.reversion
  amount = {EVO_P:.2f}
}}

stream cre.sale_costs on entity asset.project outflow currency USD {{
  schedule on {ym(T_EXIT)}
  category investing.selling_costs
  amount = {(AUB_P + EVO_P):.2f} * inputs.cost_of_sale
}}

stream cre.pierce_closings on entity asset.east inflow currency USD {{
  schedule every month from {ym(lo)} to {ym(hi)}
  category investing.reversion
  amount = curve_value("pierce_sellout", time.date) * (1.0 - inputs.condo_selling_cost)
}}
''')
OUT.mkdir(parents=True, exist_ok=True)
(OUT / "model.cfdl").write_text("\n".join(L))
print("wrote", OUT / "model.cfdl", len((OUT / "model.cfdl").read_text().splitlines()), "lines")
