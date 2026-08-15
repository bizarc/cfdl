#!/usr/bin/env python3
"""Independent reference for buenavista_del_cobre.

Year-by-year implementation of the Mexican fiscal stack and cash conventions
behind Table 19.1 of the S-K 1300 technical report summary.

WHERE THE FISCAL STRUCTURE COMES FROM. Buenavista's Table 19.1 prints EBITDA
and pre-tax gross income but not the charges between them, so the structure
below was not read off it. It was read off **La Caridad/Pilares** (Exhibit 96.7
of the same Form 10-K, same author, same template, dated within a fortnight),
whose Table 19.1 prints every intermediate line explicitly: Depreciation,
Royalty (Derechos de Mineria), PTU Employee Sharing, Minimum tax, Income tax,
and both add-backs. Fitting those ten published lines gives:

    royalty      = 7.5% * ebitda
    ptu          = 10% * max(0, ebitda - depreciation - royalty)
    gross_income = ebitda - depreciation - royalty - ptu
    minimum_tax  = 30% * royalty
    income_tax   = 30% * gross_income
    total_taxes  = income_tax - minimum_tax
    niat         = (gross_income - total_taxes) + depreciation + accretion

which reproduces all ten of La Caridad's printed lines to within 0.77 US$ M,
and is exactly what the reports state in prose -- "30% on pre-tax gross income
less 30% of the royalty" is total_taxes, and the "Minimum tax" row is the
royalty credit. Applied to Buenavista it reproduces the printed tax, net income
and after-tax cash flow to within 1.20 US$ M across all 21 annual columns.

Depreciation is not printed for Buenavista. It is recovered by inverting the
above from the two lines that are printed, EBITDA and gross income. The
inversion branches because PTU is floored at zero:

    ebitda - dep - royalty = gross_income / 0.9   when gross income is positive
                           = gross_income         otherwise

PROVENANCE: generated implementation, cross-validated against a second mine.
Regenerate: python3 reference_gen.py
"""
from __future__ import annotations

import csv
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent

# Section 19.1, Principal Assumptions.
PRICE = {"copper": 3.30, "molybdenum": 10.00, "zinc": 1.15}   # US$ per pound
DISCOUNT = 0.10

# Section 19.2, and confirmed line-by-line against La Caridad's Table 19.1.
DUTY_RATE, PTU_RATE, TAX_RATE = 0.075, 0.10, 0.30

FIRST_YEAR, ANNUAL, BUCKET_YEARS = 2025, 21, 5
PERIODS = 41

COST_LINES = [
    ("mining", "Mining"),
    ("concentrator", "Concentrator and Plants"),
    ("smelting", "Smelting/TCRC, Freight and Sales"),
    ("gna", "G&A and Home Office Cost"),
    ("decommissioning", "Plant Decommissioning"),
    ("accretion", "On-going and final accretion"),
]
METALS = [("copper", "Copper"), ("molybdenum", "Molybdenum"), ("zinc", "Zinc")]

# The filing rounds every cell to US$ 1 M. A derived line is a sum or
# difference of rounded cells, so its bound is (cells + 1) * 0.5.
ROUNDING_BOUND = 2.5


def load():
    series, totals = {}, {}
    with open(HERE / "published_grid.csv", newline="") as fh:
        for row in csv.DictReader(fh):
            cells = list(row.values())[3:]
            series[row["line_item"]] = [float(c) if c.strip() else 0.0 for c in cells]
            totals[row["line_item"]] = float(row["total_avg"])
    return series, totals


def spread(values):
    """25 printed columns -> 41 annual periods; the four buckets divide evenly.

    The intra-bucket profile is not published. This is the only assumption the
    reference makes that the filing does not state.
    """
    out = list(values[:ANNUAL])
    for bucket in values[ANNUAL:]:
        out += [bucket / BUCKET_YEARS] * BUCKET_YEARS
    return out


def npv(series, rate=DISCOUNT):
    """The filing discounts its first year at par, so 2025 is undiscounted."""
    return sum(cash / (1.0 + rate) ** t for t, cash in enumerate(series))


def drivers(published):
    """Declared inputs: payable metal, cost lines, capital, and depreciation."""
    d = {
        f"payable.{key}": [v / PRICE[key] for v in spread(published[label])]
        for key, label in METALS
    }
    d.update({f"cost.{key}": spread(published[label]) for key, label in COST_LINES})
    d["accretion"] = spread(published["On-going and final accretion"])
    d["capex"] = spread(published["Total Capex"])
    d["closure"] = spread(published["Closure"])
    d["working_capital"] = spread(published["Working Capital"])

    ebitda, gross = spread(published["EBITDA"]), spread(published["Pre Tax Gross Income"])
    d["depreciation"] = [
        ebitda[t] - DUTY_RATE * ebitda[t]
        - (gross[t] / (1.0 - PTU_RATE) if gross[t] > 0 else gross[t])
        for t in range(PERIODS)
    ]
    return d


def run(d, price=None, opex_factor=1.0, capex_factor=1.0):
    """The model. One pass per period; no solver anywhere in it."""
    price = {**PRICE, **(price or {})}
    out = {k: [] for k in (
        "revenue.total", "opex.total", "ebitda", "depreciation", "royalty", "ptu",
        "gross_income", "minimum_tax", "income_tax", "total_taxes", "niat", "shelter",
        "accretion_addback", "pre_tax_cash_flow", "after_tax_cash_flow")}
    for key, _ in METALS:
        out[f"revenue.{key}"] = []
    for key, _ in COST_LINES:
        out[f"opex.{key}"] = []

    shelter = 0.0
    for t in range(PERIODS):
        revenue = 0.0
        for key, _ in METALS:
            line = price[key] * d[f"payable.{key}"][t]
            out[f"revenue.{key}"].append(line)
            revenue += line
        opex = 0.0
        for key, _ in COST_LINES:
            line = opex_factor * d[f"cost.{key}"][t]
            out[f"opex.{key}"].append(line)
            opex += line

        ebitda = revenue - opex
        # Depreciation follows the capital that creates it.
        dep = capex_factor * d["depreciation"][t]
        royalty = DUTY_RATE * ebitda
        ptu = PTU_RATE * max(0.0, ebitda - dep - royalty)
        gross = ebitda - dep - royalty - ptu

        # A loss shelters later income: the filing prints no tax in 2043-2045
        # although gross income is positive, because 2037-2042 ran at a loss.
        available = gross + shelter
        shelter = min(0.0, available)
        out["shelter"].append(shelter)
        minimum_tax = TAX_RATE * royalty
        income_tax = TAX_RATE * available
        total_taxes = max(0.0, income_tax - minimum_tax)

        # Depreciation and ARO accretion are non-cash: the filing strikes its
        # cash flow on revenue less operating cost, taxes, capital and ARO
        # OUTLAYS, so both are added back.
        addback = opex_factor * d["accretion"][t]
        niat = (gross - total_taxes) + dep + addback
        capital = capex_factor * d["capex"][t] + d["closure"][t] + d["working_capital"][t]

        for k, v in (("revenue.total", revenue), ("opex.total", opex), ("ebitda", ebitda),
                     ("depreciation", dep), ("royalty", royalty), ("ptu", ptu),
                     ("gross_income", gross), ("minimum_tax", minimum_tax),
                     ("income_tax", income_tax), ("total_taxes", total_taxes),
                     ("niat", niat), ("accretion_addback", addback),
                     ("pre_tax_cash_flow", ebitda - capital),
                     ("after_tax_cash_flow", niat - capital)):
            out[k].append(v)
    return out


def check_table(ref, published, totals):
    pairs = [("TOTAL REVENUE", "revenue.total"), ("Total Operating Cost", "opex.total"),
             ("EBITDA", "ebitda"), ("Pre Tax Gross Income", "gross_income"),
             ("Total Taxes", "total_taxes"), ("NET INCOME AFTER TAXES", "niat"),
             ("Pre Tax Cash Flow", "pre_tax_cash_flow"),
             ("After Tax Cash Flow", "after_tax_cash_flow")]
    print("Table 19.1 -- derived line vs the filing, 21 printed annual columns (US$ M)\n")
    print(f"  {'line':26} {'max |diff|':>10} {'mean':>7}  worst year")
    ok = True
    for label, key in pairs:
        want = spread(published[label])
        diffs = [abs(ref[key][t] - want[t]) for t in range(ANNUAL)]
        worst = max(range(ANNUAL), key=lambda t: diffs[t])
        flag = "" if max(diffs) <= ROUNDING_BOUND else "   <-- outside rounding"
        ok &= max(diffs) <= ROUNDING_BOUND
        print(f"  {label:26} {max(diffs):10.2f} {sum(diffs) / ANNUAL:7.2f}  "
              f"{FIRST_YEAR + worst}{flag}")
    for label, key, want in (("Pre-Tax NPV", "pre_tax_cash_flow", 5826.0),
                             ("After-Tax NPV", "after_tax_cash_flow", 3405.0)):
        got = npv(ref[key])
        print(f"\n  {label} @ {DISCOUNT:.0%}: reference {got:,.1f} vs filing {want:,.0f} "
              f"({(got / want - 1) * 100:+.2f}%)")
    return ok


def check_sensitivities(d):
    """All 78 published points of Table 19.2. Every one is an after-tax NPV."""
    grid = {}
    with open(HERE / "published_sensitivities.csv", newline="") as fh:
        for row in csv.DictReader(fh):
            grid[row["variable"]] = [float(v) for v in list(row.values())[1:]]
    steps = [-30, -25, -20, -15, -10, -5, 0, 5, 10, 15, 20, 25, 30]

    def value(var, f):
        if var == "Operating Cost":
            return npv(run(d, opex_factor=f)["after_tax_cash_flow"])
        if var == "Capital Cost":
            return npv(run(d, capex_factor=f)["after_tax_cash_flow"])
        keys = {"Commodity Prices": [k for k, _ in METALS], "Copper Price": ["copper"],
                "Molybdenum Price": ["molybdenum"], "Zinc Price": ["zinc"]}[var]
        return npv(run(d, price={k: PRICE[k] * f for k in keys})["after_tax_cash_flow"])

    print("\n\nTable 19.2 -- all 78 published sensitivity points (after-tax NPV, US$ M)\n")
    print(f"  {'variable':18} {'max |diff|':>10} {'mean':>7}  worst step")
    results, worst_all = {}, 0.0
    for var, pub in grid.items():
        diffs, row = [], {}
        for step, want in zip(steps, pub):
            got = value(var, 1 + step / 100)
            row[f"{step:+d}%"] = round(got, 6)
            diffs.append((abs(got - want), step))
        results[var] = row
        worst = max(diffs)
        worst_all = max(worst_all, worst[0])
        print(f"  {var:18} {worst[0]:10.1f} {sum(x for x, _ in diffs) / 13:7.1f}  "
              f"{worst[1]:+d}%")
    print(f"\n  worst of all 78: {worst_all:,.1f} US$ M "
          f"({worst_all / 3405 * 100:.2f}% of the base NPV)")
    return results


def main() -> int:
    published, totals = load()
    d = drivers(published)
    ref = run(d)

    years = [FIRST_YEAR + t for t in range(PERIODS)]
    columns = ([(f"mine.revenue.{k}", f"revenue.{k}", 1) for k, _ in METALS]
               + [(f"mine.opex.{k}", f"opex.{k}", -1) for k, _ in COST_LINES]
               + [("mine.capital.capex", "capex", -1),
                  ("mine.capital.closure", "closure", -1),
                  ("mine.capital.working_capital", "working_capital", -1)])
    # The fiscal charges are asserted as the cash streams themselves; the one
    # genuine state, the loss carryforward, is asserted as the mine's field.
    columns += [("mine.fiscal.royalty", "royalty", -1),
                ("mine.fiscal.ptu", "ptu", -1),
                ("mine.fiscal.income_tax", "total_taxes", -1),
                ("mine.noncash.accretion_addback", "accretion_addback", 1)]
    fields = ["asset.mine.shelter"]
    with open(HERE / "expected.csv", "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["year"] + [c for c, _, _ in columns] + fields + ["net_cash_flow"])
        for t in range(PERIODS):
            row = [years[t]]
            for _, key, sign in columns:
                src = ref[key] if key in ref else d[key]
                row.append(f"{sign * src[t]:.6f}")
            for f in fields:
                row.append(f"{ref['shelter'][t]:.6f}")
            row.append(f"{ref['after_tax_cash_flow'][t]:.6f}")
            w.writerow(row)

    metrics = {
        "model.npv": {"value": round(npv(ref["after_tax_cash_flow"]), 6), "tolerance": 1e-4,
                      "source": "reference after-tax NPV at the filing's 10%; filing prints 3,405"},
        "stream.mine.revenue.copper.total": {
            "value": round(sum(ref["revenue.copper"]), 6), "tolerance": 1e-4,
            "source": "reference LOM copper revenue; filing prints 71,439"},
        "stream.mine.capital.capex.total": {
            "value": round(-sum(d["capex"]), 6), "tolerance": 1e-4,
            "source": "reference LOM capital; filing prints 8,317"},
    }
    (HERE / "expected_metrics.json").write_text(json.dumps(metrics, indent=2) + "\n")

    ok = check_table(ref, published, totals)
    sens = check_sensitivities(d)
    # The scenario expectations, named as run.json names them.
    label = {"Operating Cost": "opex", "Capital Cost": "capex",
             "Commodity Prices": "commodity", "Copper Price": "copper",
             "Molybdenum Price": "molybdenum", "Zinc Price": "zinc"}
    scenarios = {}
    for var, row in sens.items():
        for step, value in row.items():
            if step == "+0%":
                continue          # the base run, already asserted as model.npv
            name = f"{label[var]}_{step[:-1]}".replace("+", "p").replace("-", "m")
            scenarios[name] = {"model.npv": {"value": value, "tolerance": 1e-4}}
    (HERE / "expected_scenarios.json").write_text(json.dumps(scenarios, indent=2) + "\n")
    print(f"\nwrote expected.csv, expected_metrics.json, "
          f"expected_scenarios.json ({len(scenarios)} scenarios)")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
