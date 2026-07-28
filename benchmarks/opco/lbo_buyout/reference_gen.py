#!/usr/bin/env python3
"""Independent reference for lbo_buyout.

Month-by-month recursion of the documented opco conventions
(packs/opco/lowering/rules.toml): continuous annual-compound growth on the
model clock; DSO/DPO/DIO working capital booked as initial build +
period-over-period change + release at exit; scheduled term debt (IO, then
level-pay amortization, balloon of remaining balance at exit); cash taxes
at rate * max(0, EBITDA - D&A - interest); exit at a multiple of
trailing-12 EBITDA net of selling costs.

PROVENANCE: generated implementation; pending practitioner (Excel) review.
Regenerate: python3 reference_gen.py
"""
import csv
import json

PERIODS = 60
REV0, REV_G = 1_000_000.0, 0.06
OPEX0, OPEX_G = 650_000.0, 0.04
AR_DAYS, AP_DAYS, INV_DAYS = 45.0, 30.0, 10.0
CAPEX_PCT = 0.03
PRINCIPAL, RATE, IO_MONTHS, AMORT_MONTHS = 21_000_000.0, 0.085, 12, 84
TAX_RATE, DA_MONTHLY = 0.26, 150_000.0
PRICE = 33_600_000.0
EXIT_MULTIPLE, SELLING_COSTS = 8.5, 0.015
ANNUAL_DISCOUNT = 0.12


def rev(t):
    return REV0 * (1.0 + REV_G) ** (t / 12.0)


def opex(t):
    return OPEX0 * (1.0 + OPEX_G) ** (t / 12.0)


def wc_balance(t):
    # WC = AR + INV - AP = rev*DSO/365 + cost*(DIO - DPO)/365 on annualized
    # run rates (cost base = opex).
    return (12.0 / 365.0) * (rev(t) * AR_DAYS + opex(t) * (INV_DAYS - AP_DAYS))


def main():
    r = RATE / 12.0
    f = (1.0 + r) ** AMORT_MONTHS
    pay = PRINCIPAL * r * f / (f - 1.0)

    def balance_after(k):
        g = (1.0 + r) ** k
        return PRINCIPAL * g - pay * (g - 1.0) / r

    totals = {
        "revenue": 0.0, "ebitda": 0.0, "capex": 0.0, "wc": 0.0,
        "taxes": 0.0, "debt_service": 0.0, "fcf": 0.0,
    }
    net = [0.0] * PERIODS
    shots = [0.0] * PERIODS
    for t in range(PERIODS):
        revenue, ox = rev(t), opex(t)
        ebitda = revenue - ox
        capex = CAPEX_PCT * revenue
        if t == 0:
            wc_flow = wc_balance(0)
        else:
            wc_flow = wc_balance(t) - wc_balance(t - 1)
        if t == PERIODS - 1:
            wc_flow -= wc_balance(t)  # release at exit
        if t < IO_MONTHS:
            interest, principal = PRINCIPAL * r, 0.0
        else:
            k = t - IO_MONTHS + 1
            interest = balance_after(k - 1) * r
            principal = pay - interest
        if t == PERIODS - 1:
            principal += balance_after(PERIODS - 1 - IO_MONTHS + 1)  # balloon
        taxes = max(0.0, TAX_RATE * (ebitda - interest - DA_MONTHLY))
        net[t] += ebitda - capex - wc_flow - interest - principal - taxes
        if t == 0:
            shots[t] += PRINCIPAL - PRICE
        if t == PERIODS - 1:
            trailing = sum(rev(s) - opex(s) for s in range(PERIODS - 12, PERIODS))
            shots[t] += EXIT_MULTIPLE * trailing * (1.0 - SELLING_COSTS)
        totals["revenue"] += revenue
        totals["ebitda"] += ebitda
        totals["capex"] += capex
        totals["wc"] += wc_flow
        totals["taxes"] += taxes
        totals["debt_service"] += interest + principal
        totals["fcf"] += ebitda - capex - wc_flow - taxes

    monthly_rate = (1.0 + ANNUAL_DISCOUNT) ** (1.0 / 12.0) - 1.0    # Recurring flows are ordinary annuities: they settle at the close of the
    # period that earned them, so they are discounted one full period — the
    # same convention as Excel's NPV. One-shot flows (purchase, advance,
    # exit proceeds) settle on their own date and are not.
    npv = sum(
        v / ((1.0 + monthly_rate) ** (t + 1)) for t, v in enumerate(net)
    ) + sum(v / ((1.0 + monthly_rate) ** t) for t, v in enumerate(shots))
    for t in range(PERIODS):
        net[t] += shots[t]
    inflows = sum(v for v in net if v > 0.0)
    outflows = -sum(v for v in net if v < 0.0)
    moic = inflows / outflows

    with open("expected.csv", "w", newline="") as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow(["period", "net_cash_flow"])
        for t, v in enumerate(net):
            writer.writerow([t, f"{v:.6f}"])

    with open("expected_metrics.json", "w") as fh:
        json.dump(
            {
                "model.npv": {"value": round(npv, 2), "tolerance": 1.0},
                "model.moic": {"value": round(moic, 6), "tolerance": 1e-4},
                "domain.opco.revenue": {"value": round(totals["revenue"], 2), "tolerance": 1.0},
                "domain.opco.ebitda": {"value": round(totals["ebitda"], 2), "tolerance": 1.0},
                "domain.opco.ebitda_margin": {
                    "value": round(totals["ebitda"] / totals["revenue"], 6),
                    "tolerance": 1e-4,
                },
                "domain.opco.capex": {"value": round(totals["capex"], 2), "tolerance": 1.0},
                "domain.opco.working_capital": {"value": round(totals["wc"], 2), "tolerance": 1.0},
                "domain.opco.taxes": {"value": round(totals["taxes"], 2), "tolerance": 1.0},
                "domain.opco.debt_service": {
                    "value": round(totals["debt_service"], 2),
                    "tolerance": 1.0,
                },
                "domain.opco.fcf": {"value": round(totals["fcf"], 2), "tolerance": 1.0},
                "domain.opco.fcf_to_debt_service": {
                    "value": round(totals["fcf"] / totals["debt_service"], 6),
                    "tolerance": 1e-4,
                },
            },
            fh,
            indent=2,
        )
    print(f"wrote expected.csv, npv={npv:,.2f}, moic={moic:.4f}")


if __name__ == "__main__":
    main()
