#!/usr/bin/env python3
"""Independent reference for retail_strip (base-year gross-up + pct rent).

PROVENANCE: generated implementation; pending practitioner Argus/Excel review.
Regenerate: python3 reference_gen.py
"""
import csv
import json

PERIODS = 84  # 2026-01 .. 2032-12
DISC = 0.0775

def main():
    rows = []
    noi_total = leasing_costs = 0.0
    for t in range(PERIODS):
        # Anchor: t 0..83, lease-anniversary years since 2026-01.
        ya = t // 12
        a_rent = (540_000.0 / 12.0) * (1.02 ** ya)
        a_opex = 240_000.0 * 0.95 * (1.03 ** ya)
        a_rec = max(0.0, a_opex - 228_000.0) * 0.60 / 12.0
        a_sales = 11_500_000.0 * (1.03 ** ya)
        a_pct = max(0.0, a_sales - 12_000_000.0) * 0.02 / 12.0

        # Shops: t 6..65 (2026-07 .. 2031-06), 2 free months.
        s_rent = s_rec = 0.0
        if 6 <= t <= 65:
            ls = t - 6
            if ls >= 2:
                s_rent = (288_000.0 / 12.0) * (1.025 ** (ls // 12))
            s_opex = 240_000.0 * 0.95 * (1.03 ** (ls // 12))
            s_rec = max(0.0, s_opex - 0.0) * 0.30 / 12.0

        vacancy = 0.03 * 850_000.0 / 12.0
        opex = (240_000.0 / 12.0) * (1.03 ** (t // 12))

        net = a_rent + a_rec + a_pct + s_rent + s_rec - vacancy - opex
        ti_lc = 0.0
        if t == 0:
            ti_lc += 210_000.0
        if t == 6:
            ti_lc += 130_000.0
        net -= ti_lc
        if t == 83:
            net += 640_000.0 / 0.0675 * 0.985
        rows.append((t, net))
        noi_total += a_rent + a_rec + a_pct + s_rent + s_rec - vacancy - opex
        leasing_costs += ti_lc

    monthly_rate = (1.0 + DISC) ** (1.0 / 12.0) - 1.0
    npv = sum(net / ((1.0 + monthly_rate) ** t) for t, net in rows)

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
            },
            fh,
            indent=2,
        )
    print(f"wrote expected.csv, npv={npv:,.2f}, noi={noi_total:,.2f}")

if __name__ == "__main__":
    main()
