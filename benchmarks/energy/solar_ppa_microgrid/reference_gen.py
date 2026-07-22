#!/usr/bin/env python3
"""Independent reference implementation for the solar_ppa_microgrid case.

Implements the documented model conventions (packs/energy/README.md) from
scratch — NOT by calling cfdl — so the benchmark is a genuine cross-check:
- annual quantities spread evenly across months
- escalation/degradation step annually: factor ** floor(t / 12)
- level-pay debt via the standard annuity formula (Excel PMT)
- NPV: per-period rate (1 + annual) ** (1/12) - 1, cash at period start

PROVENANCE: generated implementation; pending practitioner (Excel) review —
see LAUNCH_PLAN.md risk register item 3.

Regenerate: python3 reference_gen.py  (writes expected.csv, expected_metrics.json)
"""
import csv
import json
import math

PERIODS = 300
CAPEX = 2_400_000.0            # t = 0 outflow
ITC = 720_000.0                # t = 11 inflow
MWH_YEAR, PPA_PRICE, ESC, DEG = 4_200.0, 85.0, 0.02, 0.005
STORAGE_MWH, SPREAD = 500.0, 30.0
CAPACITY_YEAR = 60_000.0
OM_YEAR, OM_ESC = 70_000.0, 0.025
RATE, TERM_MONTHS, PRINCIPAL = 0.06, 240, 1_600_000.0
ANNUAL_DISCOUNT = 0.08

def pmt(rate, nper, pv):
    f = (1.0 + rate) ** nper
    return -(pv * f) * rate / (f - 1.0)

def main():
    monthly_debt = -pmt(RATE / 12.0, TERM_MONTHS, PRINCIPAL)  # positive payment
    rows = []
    for t in range(PERIODS):
        year = t // 12
        ppa = (MWH_YEAR / 12.0) * ((1.0 - DEG) ** year) * PPA_PRICE * ((1.0 + ESC) ** year)
        storage = (STORAGE_MWH / 12.0) * SPREAD
        capacity = CAPACITY_YEAR / 12.0
        om = (OM_YEAR / 12.0) * ((1.0 + OM_ESC) ** year)
        net = ppa + storage + capacity - om
        if t < TERM_MONTHS:
            net -= monthly_debt
        if t == 0:
            net -= CAPEX
        if t == 11:
            net += ITC
        rows.append((t, net))

    monthly_rate = (1.0 + ANNUAL_DISCOUNT) ** (1.0 / 12.0) - 1.0
    npv = sum(net / ((1.0 + monthly_rate) ** t) for t, net in rows)
    total = sum(net for _, net in rows)

    revenue = sum(
        (MWH_YEAR / 12.0) * ((1.0 - DEG) ** (t // 12)) * PPA_PRICE * ((1.0 + ESC) ** (t // 12))
        + (STORAGE_MWH / 12.0) * SPREAD
        + CAPACITY_YEAR / 12.0
        for t in range(PERIODS)
    )
    opex = sum((OM_YEAR / 12.0) * ((1.0 + OM_ESC) ** (t // 12)) for t in range(PERIODS))
    ebitda = revenue - opex
    debt_total = monthly_debt * TERM_MONTHS
    dscr = ebitda / debt_total

    with open("expected.csv", "w", newline="") as fh:
        writer = csv.writer(fh)
        writer.writerow(["period", "net_cash_flow"])
        for t, net in rows:
            writer.writerow([t, f"{net:.6f}"])

    with open("expected_metrics.json", "w") as fh:
        json.dump(
            {
                "model.npv": {"value": round(npv, 2), "tolerance": 1.0},
                "model.total": {"value": round(total, 2), "tolerance": 1.0},
                "domain.energy.revenue": {"value": round(revenue, 2), "tolerance": 1.0},
                "domain.energy.ebitda": {"value": round(ebitda, 2), "tolerance": 1.0},
                "domain.energy.debt_service": {"value": round(debt_total, 2), "tolerance": 1.0},
                "domain.energy.dscr": {"value": round(dscr, 6), "tolerance": 1e-4},
            },
            fh,
            indent=2,
        )
    print(f"wrote expected.csv ({PERIODS} periods), npv={npv:,.2f}, dscr={dscr:.4f}")

if __name__ == "__main__":
    main()
