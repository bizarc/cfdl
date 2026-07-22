#!/usr/bin/env python3
"""Independent reference for office_two_tenant (Argus-style lease-by-lease DCF).

Conventions mirrored from the documented pack contract (packs/cre/README.md):
- lease-anniversary anchoring: escalations step at floor(months_since_lease_start/12)
- free rent burns off at lease start; recoveries = max(0, opex_y - stop) * share
- rollover: renewal-probability-blended rent and TI/LC; window start set by analyst
- exit at forward NOI / cap, net of selling costs; NPV per (1+r)^(1/12)-1

PROVENANCE: generated implementation; pending practitioner Argus/Excel review.
Regenerate: python3 reference_gen.py
"""
import csv
import json

PERIODS = 120  # 2026-01 .. 2035-12
DISC = 0.0725

def pmt(rate, nper, pv):
    f = (1.0 + rate) ** nper
    return -(pv * f) * rate / (f - 1.0)

def lease_rent(t, start, end, rent_year, free_months, esc):
    if t < start or t > end:
        return 0.0
    lt = t - start
    if lt < free_months:
        return 0.0
    return (rent_year / 12.0) * ((1.0 + esc) ** (lt // 12))

def recoveries(t, start, end, opex_year, opex_esc, stop, share):
    if t < start or t > end:
        return 0.0
    lt = t - start
    opex = opex_year * ((1.0 + opex_esc) ** (lt // 12))
    return max(0.0, opex - stop) * share / 12.0

def main():
    debt_pay = -pmt(0.055 / 12.0, 300, 6_000_000.0)
    rows = []
    noi_total = leasing_costs = 0.0
    for t in range(PERIODS):
        # Tenant A: t 0..59; Tenant B: t 6..89; Rollover A: t 63..119.
        a_rent = lease_rent(t, 0, 59, 480_000.0, 3, 0.03)
        a_rec = recoveries(t, 0, 59, 300_000.0, 0.025, 300_000.0, 0.40)
        b_rent = lease_rent(t, 6, 89, 360_000.0, 0, 0.025)
        b_rec = recoveries(t, 6, 89, 300_000.0, 0.025, 180_000.0, 0.30)
        roll_rent = 0.0
        if 63 <= t <= 119:
            blended = 0.7 * 520_000.0 + 0.3 * 560_000.0
            roll_rent = (blended / 12.0) * (1.03 ** ((t - 63) // 12))
        vacancy = 0.02 * 900_000.0 / 12.0
        opex = (300_000.0 / 12.0) * (1.025 ** (t // 12))

        net = a_rent + a_rec + b_rent + b_rec + roll_rent - vacancy - opex - debt_pay
        ti_lc = 0.0
        if t == 0:
            ti_lc += 200_000.0
        if t == 6:
            ti_lc += 150_000.0
        if t == 63:
            ti_lc += 0.7 * 100_000.0 + 0.3 * 350_000.0
        net -= ti_lc
        if t == 119:
            net += 800_000.0 / 0.065 * 0.98
        rows.append((t, net))
        noi_total += a_rent + a_rec + b_rent + b_rec + roll_rent - vacancy - opex
        leasing_costs += ti_lc

    monthly_rate = (1.0 + DISC) ** (1.0 / 12.0) - 1.0
    npv = sum(net / ((1.0 + monthly_rate) ** t) for t, net in rows)
    debt_total = debt_pay * PERIODS

    with open("expected.csv", "w", newline="") as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow(["period", "net_cash_flow"])
        for t, net in rows:
            writer.writerow([t, f"{net:.6f}"])

    with open("expected_metrics.json", "w") as fh:
        json.dump(
            {
                "model.npv": {"value": round(npv, 2), "tolerance": 1.0},
                "domain.cre.noi": {"value": round(noi_total, 2), "tolerance": 1.0},
                "domain.cre.leasing_costs": {"value": round(leasing_costs, 2), "tolerance": 1.0},
                "domain.cre.debt_service": {"value": round(debt_total, 2), "tolerance": 1.0},
                "domain.cre.dscr": {"value": round(noi_total / debt_total, 6), "tolerance": 1e-4},
            },
            fh,
            indent=2,
        )
    print(f"wrote expected.csv, npv={npv:,.2f}, noi={noi_total:,.2f}")

if __name__ == "__main__":
    main()
