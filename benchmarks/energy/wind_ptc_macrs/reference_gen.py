#!/usr/bin/env python3
"""Independent reference for wind_ptc_macrs (see solar case for methodology).

PROVENANCE: generated implementation; pending practitioner (Excel) review.
Regenerate: python3 reference_gen.py
"""
import csv
import decimal
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
        # The published credit rate is a STAIRCASE: the inflation-adjusted figure
        # is rounded to the nearest 0.1 cent per kWh, which on this $/MWh basis is
        # a WHOLE DOLLAR ($0.001/kWh x 1000), and holds
        # for the year. This reference carried the continuous ramp, the same
        # omission the engine had, which is exactly why the two agreed. Half
        # away from zero, matching the statutory convention.
        rate = decimal.Decimal(str(PTC_RATE)) * (decimal.Decimal("1") + decimal.Decimal(str(PTC_ESC))) ** y
        rate = float((rate / decimal.Decimal("1.00")).quantize(
            decimal.Decimal("1"), rounding=decimal.ROUND_HALF_UP) * decimal.Decimal("1.00"))
        ptc = production * rate if y < PTC_YEARS else 0.0
        macrs = BASIS * (MACRS_5[y] if y < len(MACRS_5) else 0.0) / 12.0 * TAX_RATE
        om = (OM_YEAR / 12.0) * ((1.0 + OM_ESC) ** y)
        shot = 0.0
        net = merchant + ptc + macrs - om
        if t < TERM_MONTHS:
            net -= monthly_debt
        if t == 0:
            shot -= CAPEX
        rows.append((t, net, shot))
        revenue_total += merchant
        opex_total += om
        tax_benefits_total += ptc + macrs

    monthly_rate = (1.0 + ANNUAL_DISCOUNT) ** (1.0 / 12.0) - 1.0    # Recurring flows are ordinary annuities: they settle at the close of the
    # period that earned them, so they are discounted one full period — the
    # same convention as Excel's NPV. One-shot flows (purchase, advance,
    # exit proceeds) settle on their own date and are not.
    npv = sum(
        net / ((1.0 + monthly_rate) ** (t + 1)) + shot / ((1.0 + monthly_rate) ** t)
        for t, net, shot in rows
    )
    ebitda = revenue_total - opex_total
    debt_total = monthly_debt * TERM_MONTHS

    with open("expected.csv", "w", newline="") as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow(["period", "net_cash_flow"])
        for t, net, shot in rows:
            writer.writerow([t, f"{net + shot:.6f}"])

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
