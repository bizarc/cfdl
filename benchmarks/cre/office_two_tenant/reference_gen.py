#!/usr/bin/env python3
"""Independent reference for office_two_tenant (institutional lease-by-lease DCF).

Conventions mirrored from the documented pack contract (packs/cre/README.md):
- lease-anniversary anchoring: escalations step at floor(months_since_lease_start/12)
- free rent burns off at lease start; recoveries = max(0, opex_y - stop) * share
- rollover: renewal-probability-blended rent and TI/LC; window start set by analyst
- exit at forward NOI / cap, net of selling costs; NPV per (1+r)^(1/12)-1

PROVENANCE: generated implementation; pending practitioner review against institutional-grade references.
Regenerate: python3 reference_gen.py
"""
import csv
import json

PERIODS = 120  # 2026-01 .. 2035-12
DISC = 0.0725

def pmt(rate, nper, pv):
    f = (1.0 + rate) ** nper
    return -(pv * f) * rate / (f - 1.0)

def rollover_rent(t):
    """Rollover window starts at expiry (t=60); 3 downtime months pay only the
    renewal scenario; escalation steps on window anniversaries."""
    if t < 60:
        return 0.0
    roll_t = t - 60
    esc = 1.03 ** (roll_t // 12)
    if roll_t < 3:
        return 0.7 * 520_000.0 / 12.0 * esc
    blended = 0.7 * 520_000.0 + 0.3 * 560_000.0
    return blended / 12.0 * esc

def opex_month(t):
    return (300_000.0 / 12.0) * (1.025 ** (t // 12))

VACANCY_MONTH = 0.02 * 900_000.0 / 12.0

def forward_noi(sale_t):
    """NOI over the 12 months after the sale date (projection columns)."""
    total = 0.0
    for t in range(sale_t + 1, sale_t + 13):
        total += rollover_rent(t) - VACANCY_MONTH - opex_month(t)
    return total

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
        roll_rent = rollover_rent(t)
        vacancy = VACANCY_MONTH
        opex = opex_month(t)

        net = a_rent + a_rec + b_rent + b_rec + roll_rent - vacancy - opex - debt_pay
        # Initial lease TI/LC is a dated, one-shot cost (cre_lease_unit_ti_lc);
        # the expiry turnover costs come from the recurring rollover rule
        # (cre_rollover_ti_lc) and so close their period like other recurring
        # flows.
        ti_lc = 0.0
        ti_lc_dated = 0.0
        shot = 0.0
        exit_shot = 0.0
        if t == 0:
            ti_lc_dated += 200_000.0
        if t == 6:
            ti_lc_dated += 150_000.0
        shot -= ti_lc_dated
        if t == 60:
            ti_lc += 0.7 * 100_000.0  # renewal-scenario turnover cost at expiry
        if t == 63:
            ti_lc += 0.3 * 350_000.0  # re-lease turnover cost after downtime
        net -= ti_lc
        if t == 119:
            exit_shot += forward_noi(119) / 0.065 * 0.98
        # The per-period subtotals. Computed here already — the loop summed
        # them into `noi_total` and threw the periods away, so a lifetime NOI
        # was asserted and the 120 figures behind it were not.
        egi_t = a_rent + a_rec + b_rent + b_rec + roll_rent - vacancy
        noi_t = egi_t - opex
        rows.append((t, net, shot, exit_shot, egi_t, noi_t))
        noi_total += noi_t
        leasing_costs += ti_lc + ti_lc_dated

    monthly_rate = (1.0 + DISC) ** (1.0 / 12.0) - 1.0    # Recurring flows are ordinary annuities: they settle at the close of the
    # period that earned them, so they are discounted one full period — the
    # same convention as Excel's NPV.
    # One-shot flows split by KIND, not by being one-shot. A purchase, a debt
    # advance or a dated TI/LC cost happens on its date and is discounted from
    # the period's open. A DISPOSAL does not: a reversion is taken at the end
    # of the holding period, so it discounts the full n periods like any
    # end-of-period flow.
    #
    # Both this reference and the engine used to treat every one-shot the same
    # way, which is the shared-misunderstanding failure analytic-checks.py
    # exists to catch — two implementations agreeing is not evidence when they
    # were written from the same assumption. The tiebreaker is external:
    # benchmarks/cre/mit_rentleg_plaza reproduces MIT's published $2,292,810
    # only when the reversion discounts a full five periods.
    npv = sum(
        net / ((1.0 + monthly_rate) ** (t + 1))
        + shot / ((1.0 + monthly_rate) ** t)
        + exit_shot / ((1.0 + monthly_rate) ** (t + 1))
        for t, net, shot, exit_shot, _egi, _noi in rows
    )
    debt_total = debt_pay * PERIODS

    with open("expected.csv", "w", newline="") as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow([
            "period",
            "net_cash_flow",
            "domain.cre.egi",
            "domain.cre.noi",
            "domain.cre.debt_service",
            "domain.cre.dscr",
        ])
        for t, net, shot, exit_shot, egi_t, noi_t in rows:
            writer.writerow([
                t,
                f"{net + shot + exit_shot:.6f}",
                f"{egi_t:.6f}",
                f"{noi_t:.6f}",
                # Level payment, so this is constant — which is exactly why it
                # cannot discriminate a recomputed annual ratio from an averaged
                # one. See tools/analytic-checks.py.
                f"{debt_pay:.6f}",
                f"{noi_t / debt_pay:.6f}",
            ])

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
