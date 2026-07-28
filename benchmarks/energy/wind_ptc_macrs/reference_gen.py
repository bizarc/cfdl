#!/usr/bin/env python3
"""Independent reference for wind_ptc_macrs (see solar case for methodology).

PROVENANCE: generated implementation; pending practitioner (Excel) review.
Regenerate: python3 reference_gen.py
"""
import csv
import json

PERIODS = 240
CAPEX = 42_000_000.0
MWH, PRICE, PRICE_ESC, DEG, AVAIL = 105_000.0, 42.0, 0.015, 0.007, 0.95
PTC_RATE, PTC_ESC, PTC_YEARS = 27.5, 0.02, 10
BASIS, TAX_RATE = 42_000_000.0, 0.21
MACRS_5 = [0.20, 0.32, 0.192, 0.1152, 0.1152, 0.0576]
OM_YEAR, OM_ESC = 1_300_000.0, 0.02
RATE, TERM_MONTHS, PRINCIPAL = 0.055, 180, 25_000_000.0
ANNUAL_DISCOUNT = 0.075

def pmt(rate, nper, pv):
    f = (1.0 + rate) ** nper
    return -(pv * f) * rate / (f - 1.0)

def main():
    monthly_debt = -pmt(RATE / 12.0, TERM_MONTHS, PRINCIPAL)
    rows = []
    revenue_total = opex_total = tax_benefits_total = 0.0
    for t in range(PERIODS):
        y = t // 12
        production = (MWH / 12.0) * AVAIL * ((1.0 - DEG) ** y)
        merchant = production * PRICE * ((1.0 + PRICE_ESC) ** y)
        ptc = production * PTC_RATE * ((1.0 + PTC_ESC) ** y) if y < PTC_YEARS else 0.0
        macrs = BASIS * (MACRS_5[y] if y < len(MACRS_5) else 0.0) / 12.0 * TAX_RATE
        om = (OM_YEAR / 12.0) * ((1.0 + OM_ESC) ** y)
        net = merchant + ptc + macrs - om
        if t < TERM_MONTHS:
            net -= monthly_debt
        if t == 0:
            net -= CAPEX
        rows.append((t, net))
        revenue_total += merchant
        opex_total += om
        tax_benefits_total += ptc + macrs

    monthly_rate = (1.0 + ANNUAL_DISCOUNT) ** (1.0 / 12.0) - 1.0
    npv = sum(net / ((1.0 + monthly_rate) ** t) for t, net in rows)
    ebitda = revenue_total - opex_total
    debt_total = monthly_debt * TERM_MONTHS

    with open("expected.csv", "w", newline="") as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow(["period", "net_cash_flow"])
        for t, net in rows:
            writer.writerow([t, f"{net:.6f}"])

    with open("expected_metrics.json", "w") as fh:
        json.dump(
            {
                "model.npv": {"value": round(npv, 2), "tolerance": 1.0},
                "domain.energy.revenue": {"value": round(revenue_total, 2), "tolerance": 1.0},
                "domain.energy.ebitda": {"value": round(ebitda, 2), "tolerance": 1.0},
                "domain.energy.tax_benefits": {"value": round(tax_benefits_total, 2), "tolerance": 1.0},
                "domain.energy.debt_service": {"value": round(debt_total, 2), "tolerance": 1.0},
                "domain.energy.dscr": {"value": round(ebitda / debt_total, 6), "tolerance": 1e-4},
            },
            fh,
            indent=2,
        )
    print(f"wrote expected.csv, npv={npv:,.2f}")

if __name__ == "__main__":
    main()
