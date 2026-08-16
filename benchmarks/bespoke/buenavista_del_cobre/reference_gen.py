#!/usr/bin/env python3
"""Independent reference for buenavista_del_cobre.

The case models the mine from the operator's published INPUTS and compares the
statement it produces against the operator's published ANSWER. This script is
the second implementation of that model, in Python, and it writes the per-period
expectations the CFDL model is checked against.

It consumes only inputs: the production schedule (Table 13.3), published unit
costs (section 18), stated prices and fiscal rates (section 19), and the four
recovery assumptions the report does not state. It never reads Table 19.1.

Regenerate: python3 reference_gen.py
"""
from __future__ import annotations
import csv, json, pathlib

HERE = pathlib.Path(__file__).resolve().parent
LB = 2204.6
PERIODS, FIRST_YEAR = 41, 2025

# Recovery to payable — the four numbers the report does not state.
REC_CU_MILL, REC_CU_LEACH, REC_MO, REC_ZN = 0.836, 0.260, 0.660, 0.629
PRICE = {"cu": 3.30, "mo": 10.00, "zn": 1.15}          # 19.1
MINING, MILL, ZINC_PLANT = 2.71, 5.83, 10.63           # T18.7 / T18.8 / T18.9
CRUSH, LEACH, GNA = 0.84, 0.40, 0.76                   # T18.8 / 18.3.1
SELL = {"cu": 0.54, "mo": 1.84, "zn": 0.40}            # T12.4 / T12.6
ACCRETION, CLOSURE, CAPEX_LOM = 34.0, 544.0, 8317.0    # 18.3.3 / T18.1
DUTY, PTU, TAX, DISCOUNT = 0.075, 0.10, 0.30, 0.10     # 19.2 / 19.1


def load():
    rows = list(csv.DictReader(open(HERE / "published_production_schedule.csv")))
    f = lambda r, k: float(r[k]) if r[k] else 0.0
    out = []
    for r in rows:
        out.append(dict(
            year=int(float(r["year"])),
            cu_mill_kt=(f(r,"conc_mt")*f(r,"conc_cu") + f(r,"zn_mt")*f(r,"zn_cu"))/100*1e3,
            cu_leach_kt=(f(r,"rom_mt")*f(r,"rom_cu") + f(r,"crush_mt")*f(r,"crush_cu"))/100*1e3,
            mo_kt=f(r,"conc_mt")*f(r,"conc_mo")/100*1e3,
            zn_kt=f(r,"zn_mt")*f(r,"zn_zn")/100*1e3,
            moved=f(r,"total_mt"), milled=f(r,"conc_mt"), zinc_mill=f(r,"zn_mt"),
            crushed=f(r,"crush_mt"), leached=f(r,"rom_mt")+f(r,"crush_mt"),
            ore=f(r,"conc_mt")+f(r,"zn_mt")+f(r,"rom_mt")+f(r,"crush_mt")))
    return out


def build(drivers, price=None, rec=None):
    price = {**PRICE, **(price or {})}
    rec = {**dict(cu_mill=REC_CU_MILL, cu_leach=REC_CU_LEACH, mo=REC_MO, zn=REC_ZN),
           **(rec or {})}
    total_moved = sum(d["moved"] for d in drivers)
    total_ore = sum(d["ore"] for d in drivers)
    rows, shelter = [], 0.0
    for i, d in enumerate(drivers):
        pay_cu = rec["cu_mill"]*d["cu_mill_kt"] + rec["cu_leach"]*d["cu_leach_kt"]
        pay_mo, pay_zn = rec["mo"]*d["mo_kt"], rec["zn"]*d["zn_kt"]
        k = LB/1000.0
        rev = {"copper": price["cu"]*k*pay_cu, "molybdenum": price["mo"]*k*pay_mo,
               "zinc": price["zn"]*k*pay_zn}
        opex = {
            "mining": MINING*d["moved"],
            "processing": (MILL*d["milled"] + ZINC_PLANT*d["zinc_mill"]
                           + CRUSH*d["crushed"] + LEACH*d["leached"]),
            "selling": k*(SELL["cu"]*pay_cu + SELL["mo"]*pay_mo + SELL["zn"]*pay_zn),
            "gna": GNA*(d["milled"] + d["zinc_mill"]),
            "accretion": ACCRETION}
        ebitda = sum(rev.values()) - sum(opex.values())
        dep = CAPEX_LOM*d["ore"]/total_ore
        duty = DUTY*ebitda
        profit_share = PTU*max(0.0, (1.0-DUTY)*ebitda - dep)
        gross = (1.0-DUTY)*ebitda - dep - profit_share
        available = gross + shelter
        shelter = min(0.0, available)
        income_tax = max(0.0, TAX*available - TAX*duty)
        capital = CAPEX_LOM*d["moved"]/total_moved
        closure = CLOSURE/5.0 if d["year"] >= 2061 else 0.0
        rows.append(dict(year=d["year"], **{f"revenue.{m}": v for m, v in rev.items()},
                         **{f"opex.{m}": v for m, v in opex.items()},
                         ebitda=ebitda, duty=duty, profit_share=profit_share,
                         income_tax=income_tax, shelter=shelter, capital=capital,
                         closure=closure, addback=ACCRETION,
                         # accretion is charged in opex but never leaves the
                         # bank, so it is added back -- the same treatment the
                         # sibling report prints as its own line
                         net=ebitda + ACCRETION - duty - profit_share - income_tax
                             - capital - closure))
    return rows


def npv(series, rate=DISCOUNT):
    return sum(c/(1.0+rate)**t for t, c in enumerate(series))


def main() -> int:
    rows = build(load())
    cols = ([("mine.revenue.copper","revenue.copper",1),
             ("mine.revenue.molybdenum","revenue.molybdenum",1),
             ("mine.revenue.zinc","revenue.zinc",1)]
            + [(f"mine.opex.{m}", f"opex.{m}", -1)
               for m in ("mining","processing","selling","gna","accretion")]
            + [("mine.fiscal.duty","duty",-1), ("mine.fiscal.profit_share","profit_share",-1),
               ("mine.fiscal.income_tax","income_tax",-1),
               ("mine.noncash.accretion_addback","addback",1),
               ("mine.capital.sustaining","capital",-1),
               ("mine.capital.closure","closure",-1)])
    with open(HERE / "expected.csv", "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["year"] + [c for c,_,_ in cols] + ["asset.mine.shelter","net_cash_flow"])
        for r in rows:
            w.writerow([r["year"]] + [f"{s*r[k]:.6f}" for _,k,s in cols]
                       + [f"{r['shelter']:.6f}", f"{r['net']:.6f}"])
    metrics = {
        "model.npv": {"value": round(npv([r["net"] for r in rows]), 6), "tolerance": 1e-4,
                      "source": "our after-tax NPV at the filing's stated 10%"},
        "stream.mine.revenue.copper.total": {
            "value": round(sum(r["revenue.copper"] for r in rows), 6), "tolerance": 1e-4,
            "source": "our life-of-mine copper revenue"},
        "stream.mine.capital.sustaining.total": {
            "value": round(-sum(r["capital"] for r in rows), 6), "tolerance": 1e-4,
            "source": "capital, Table 18.1"},
    }
    (HERE / "expected_metrics.json").write_text(json.dumps(metrics, indent=2) + "\n")

    published = {r["line_item"]: [float(c) if c.strip() else 0.0
                                  for c in list(r.values())[3:]]
                 for r in csv.DictReader(open(HERE / "published_grid.csv"))}
    print("OUR STATEMENT vs THE FILING — life of mine, US$ M\n")
    print(f"  {'line':24}{'ours':>10}{'theirs':>10}{'diff':>9}")
    rev = sum(r[f"revenue.{m}"] for r in rows for m in ("copper","molybdenum","zinc"))
    opex = sum(r[f"opex.{m}"] for r in rows
               for m in ("mining","processing","selling","gna","accretion"))
    for lbl, ours, theirs in (("Total revenue", rev, sum(published["TOTAL REVENUE"])),
                              ("Total operating cost", opex, sum(published["Total Operating Cost"])),
                              ("EBITDA", rev-opex, sum(published["EBITDA"])),
                              ("Income tax", sum(r["income_tax"] for r in rows),
                               sum(published["Total Taxes"])),
                              ("Capital", sum(r["capital"] for r in rows),
                               sum(published["Total Capex"]))):
        print(f"  {lbl:24}{ours:10,.0f}{theirs:10,.0f}{(ours/theirs-1)*100:+8.1f}%")
    value = npv([r["net"] for r in rows])
    print(f"\n  {'After-tax NPV @10%':24}{value:10,.0f}{3405:10,}{(value/3405-1)*100:+8.1f}%")
    print(f"\nwrote expected.csv ({len(rows)} periods), expected_metrics.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
