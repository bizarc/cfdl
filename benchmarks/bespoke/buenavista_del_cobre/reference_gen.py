#!/usr/bin/env python3
"""Independent reference for buenavista_del_cobre.

The case derives the mine's production schedule from its reserve statement and
then values it. This script is the second implementation of both halves, and it
writes the per-period expectations the CFDL model is checked against.

It consumes about twenty published numbers: four reserve tonnages and their
contained metal (Table 12.8), two mill capacities, the strip ratio, the
head-grade policy (Table 12.5), unit costs (section 18), and prices and fiscal
rates (section 19). It never reads the operator's production schedule, and it
never reads the operator's cash flow. Both are comparisons, printed at the end.

Regenerate: python3 reference_gen.py
"""
from __future__ import annotations
import csv, json, pathlib

HERE = pathlib.Path(__file__).resolve().parent
LB, PERIODS, FIRST_YEAR = 2204.6, 41, 2025

STOCKS = {
    "cu_mill": dict(tonnes=2117.0, metals={"cu": 8774.0, "mo": 181.0}),
    "zn_mill": dict(tonnes=296.0,  metals={"cu": 1798.0, "zn": 1705.0}),
    "crushed": dict(tonnes=1077.0, metals={"cu": 2543.0}),
    "rom":     dict(tonnes=1041.0, metals={"cu": 3076.0}),
}
RESERVE_ORE = sum(s["tonnes"] for s in STOCKS.values())
CAP_CU_FULL, CAP_CU_REDUCED, CAP_ZN = 74.0, 43.0, 7.0
RATE_CRUSHED, RATE_ROM = 1077.0 / 41, 1041.0 / 41
GRADE_POLICY = {0: 0.50, 1: 0.48, 2: 0.43}
# The deterministic run of a stochastic assumption takes the distribution
# mean, so the reference must use the same value to be comparable.
STRIP_RATIO = (0.31 + 0.83 + 2.08) / 3.0
PRICE = {"cu": 3.30, "mo": 10.00, "zn": 1.15}
REC = dict(cu_mill=0.836, cu_leach=0.260, mo=0.660, zn=0.629)
MINING, MILL, ZINC_PLANT = 2.71, 5.83, 10.63
CRUSH, LEACH, GNA = 0.84, 0.40, 0.76
SELL = {"cu": 0.54, "mo": 1.84, "zn": 0.40}
ACCRETION, CLOSURE, CAPITAL_LOM = 34.0, 544.0, 8317.0
DUTY, PTU, TAX, DISCOUNT = 0.075, 0.10, 0.30, 0.10


def capacity(name, period):
    if name == "cu_mill":
        return CAP_CU_FULL if period <= 10 else CAP_CU_REDUCED
    return {"zn_mill": CAP_ZN, "crushed": RATE_CRUSHED, "rom": RATE_ROM}[name]


def schedule():
    """Draw each stock at capacity until it is gone. Grade is a consequence."""
    state = {k: dict(tonnes=v["tonnes"], metals=dict(v["metals"]))
             for k, v in STOCKS.items()}
    rows = []
    for t in range(PERIODS):
        row = {"year": FIRST_YEAR + t, "draw": {}, "metal": {}}
        for name, st in state.items():
            drawn = min(capacity(name, t), st["tonnes"])
            row["draw"][name] = drawn
            row["metal"][name] = {}
            for metal, remaining in list(st["metals"].items()):
                balance = (remaining / st["tonnes"]) / 10.0 if st["tonnes"] > 0 else 0.0
                grade = (GRADE_POLICY.get(t, balance)
                         if (name == "cu_mill" and metal == "cu") else balance)
                if drawn > 0:
                    grade = min(grade, (remaining / drawn) / 10.0)
                taken = drawn * grade * 10.0
                row["metal"][name][metal] = taken
                st["metals"][metal] = max(0.0, remaining - taken)
            st["tonnes"] = max(0.0, st["tonnes"] - drawn)
        rows.append(row)
    return rows


def value(rows, strip=STRIP_RATIO):
    out, shelter = [], 0.0
    for r in rows:
        d, m = r["draw"], r["metal"]
        ore = sum(d.values())
        pay_cu = (REC["cu_mill"] * (m["cu_mill"]["cu"] + m["zn_mill"]["cu"])
                  + REC["cu_leach"] * (m["crushed"]["cu"] + m["rom"]["cu"]))
        pay_mo, pay_zn = REC["mo"] * m["cu_mill"]["mo"], REC["zn"] * m["zn_mill"]["zn"]
        k = LB / 1000.0
        rev = {"copper": PRICE["cu"] * k * pay_cu,
               "molybdenum": PRICE["mo"] * k * pay_mo,
               "zinc": PRICE["zn"] * k * pay_zn}
        opex = {"mining": MINING * (1.0 + strip) * ore,
                "processing": (MILL * d["cu_mill"] + ZINC_PLANT * d["zn_mill"]
                               + CRUSH * d["crushed"] + LEACH * (d["crushed"] + d["rom"])),
                "selling": k * (SELL["cu"] * pay_cu + SELL["mo"] * pay_mo
                                + SELL["zn"] * pay_zn),
                "gna": GNA * (d["cu_mill"] + d["zn_mill"]),
                "accretion": ACCRETION}
        ebitda = sum(rev.values()) - sum(opex.values())
        dep = CAPITAL_LOM * ore / RESERVE_ORE
        duty = DUTY * ebitda
        share = PTU * max(0.0, (1.0 - DUTY) * ebitda - dep)
        gross = (1.0 - DUTY) * ebitda - dep - share
        available = gross + shelter
        shelter = min(0.0, available)
        tax = max(0.0, TAX * available - TAX * duty)
        capital = CAPITAL_LOM * ore / RESERVE_ORE
        closure = CLOSURE / 5.0 if r["year"] >= 2061 else 0.0
        out.append(dict(year=r["year"], ore=ore,
                        **{f"revenue.{a}": b for a, b in rev.items()},
                        **{f"opex.{a}": b for a, b in opex.items()},
                        ebitda=ebitda, duty=duty, profit_share=share,
                        income_tax=tax, capital=capital, closure=closure,
                        addback=ACCRETION,
                        net=ebitda + ACCRETION - duty - share - tax - capital - closure))
    return out


def npv(series, rate=DISCOUNT):
    return sum(c / (1.0 + rate) ** t for t, c in enumerate(series))


def main() -> int:
    rows = schedule()
    priced = value(rows)
    cols = ([("mine.revenue.copper", "revenue.copper", 1),
             ("mine.revenue.molybdenum", "revenue.molybdenum", 1),
             ("mine.revenue.zinc", "revenue.zinc", 1)]
            + [(f"mine.opex.{m}", f"opex.{m}", -1)
               for m in ("mining", "processing", "selling", "gna", "accretion")]
            + [("mine.fiscal.duty", "duty", -1),
               ("mine.fiscal.profit_share", "profit_share", -1),
               ("mine.fiscal.income_tax", "income_tax", -1),
               ("mine.noncash.accretion_addback", "addback", 1),
               ("mine.capital.sustaining", "capital", -1),
               ("mine.capital.closure", "closure", -1)])
    with open(HERE / "expected.csv", "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["year"] + [c for c, _, _ in cols] + ["net_cash_flow"])
        for r in priced:
            w.writerow([r["year"]] + [f"{s * r[k]:.6f}" for _, k, s in cols]
                       + [f"{r['net']:.6f}"])
    metrics = {
        "model.npv": {"value": round(npv([r["net"] for r in priced]), 6),
                      "tolerance": 1e-4,
                      "source": "our after-tax NPV at the filing's stated 10%"},
        "stream.mine.capital.sustaining.total": {
            "value": round(-sum(r["capital"] for r in priced), 6), "tolerance": 1e-4,
            "source": "capital drawn with the ore, Table 18.1 over the reserve"}}
    (HERE / "expected_metrics.json").write_text(json.dumps(metrics, indent=2) + "\n")

    plan = list(csv.DictReader(open(HERE / "published_production_schedule.csv")))
    f = lambda r, c: float(r[c]) if r[c] else 0.0
    print("OUR SCHEDULE vs THE OPERATOR'S (Table 13.3)\n")
    print(f"  {'line':22}{'ours Mt':>10}{'theirs Mt':>11}{'diff':>8}")
    for name, col in (("cu_mill", "conc_mt"), ("zn_mill", "zn_mt"),
                      ("crushed", "crush_mt"), ("rom", "rom_mt")):
        o = sum(r["draw"][name] for r in rows)
        t = sum(f(r, col) for r in plan)
        print(f"  {name:22}{o:10,.0f}{t:11,.0f}{(o / t - 1) * 100:+7.1f}%")
    ore_o = sum(r["ore"] for r in priced)
    wt = sum(f(r, "waste_mt") for r in plan)
    print(f"  {'waste (strip x ore)':22}{STRIP_RATIO * ore_o:10,.0f}{wt:11,.0f}"
          f"{(STRIP_RATIO * ore_o / wt - 1) * 100:+7.1f}%")
    cu_o = sum(sum(r["metal"][s]["cu"] for s in r["metal"]) for r in rows)
    ct = sum(f(r, "cont_cu_kt") for r in plan)
    print(f"  {'contained copper kt':22}{cu_o:10,.0f}{ct:11,.0f}{(cu_o / ct - 1) * 100:+7.1f}%")

    pub = {r["line_item"]: [float(c) if c.strip() else 0.0 for c in list(r.values())[3:]]
           for r in csv.DictReader(open(HERE / "published_grid.csv"))}
    rev = sum(r[f"revenue.{m}"] for r in priced for m in ("copper", "molybdenum", "zinc"))
    opex = sum(r[f"opex.{m}"] for r in priced
               for m in ("mining", "processing", "selling", "gna", "accretion"))
    print("\nOUR CASH FLOW vs THE OPERATOR'S (Table 19.1), US$ M\n")
    print(f"  {'line':22}{'ours':>10}{'theirs':>10}{'diff':>8}")
    for lbl, o, t in (("Total revenue", rev, sum(pub["TOTAL REVENUE"])),
                      ("Total operating cost", opex, sum(pub["Total Operating Cost"])),
                      ("Capital", sum(r["capital"] for r in priced), sum(pub["Total Capex"]))):
        print(f"  {lbl:22}{o:10,.0f}{t:10,.0f}{(o / t - 1) * 100:+7.1f}%")
    v = npv([r["net"] for r in priced])
    print(f"  {'After-tax NPV @10%':22}{v:10,.0f}{3405:10,}{(v / 3405 - 1) * 100:+7.1f}%")
    print(f"\nwrote expected.csv ({len(priced)} periods), expected_metrics.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
