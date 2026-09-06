#!/usr/bin/env python3
"""Independent reference for office_renewal_option.

The office_two_tenant reference with one function replaced: Tenant A's expiry
is a renewal option rather than a probability-weighted rollover. Every other
convention is the original's, unchanged (packs/cre/README.md):
- lease-anniversary anchoring: escalations step at floor(months_since_lease_start/12)
- free rent burns off at lease start; recoveries = max(0, opex_y - stop) * share
- exit at forward NOI / cap, net of selling costs; NPV per (1+r)^(1/12)-1

The option: tested the month after expiry (t=60). The tenant renews when the
market rent exceeds the option rent ($520k). Both outcomes are leases: the
renewal from t=60 with $100k of leasing cost at commencement, or, after three
months of downtime, a market lease from t=63 with $350k. Each escalates 3% on
its own anniversaries and pays its leasing cost as a dated one-shot at its
start, as every lease in the original does.

Base: market $560k, the tenant renews. Scenario `soft_market`: market $480k,
the option lapses.

Regenerate: python3 reference_gen.py
"""
import csv
import json

PERIODS = 120  # 2026-01 .. 2035-12
DISC = 0.0725
RENEWAL_RENT_YEAR = 520_000.0
RENEWAL_TI_LC = 100_000.0
NEW_TI_LC = 350_000.0


def pmt(rate, nper, pv):
    f = (1.0 + rate) ** nper
    return -(pv * f) * rate / (f - 1.0)


def opex_month(t):
    return (300_000.0 / 12.0) * (1.025 ** (t // 12))


VACANCY_MONTH = 0.02 * 900_000.0 / 12.0


def lease_rent(t, start, end, rent_year, free_months, esc):
    if t < start or t > end:
        return 0.0
    lt = t - start
    if lt < free_months:
        return 0.0
    return (rent_year / 12.0) * ((1.0 + esc) ** (lt // 12))


def renewal_rent(t):
    """The renewal lease: the option rent from t=60, escalating on its anniversaries."""
    return lease_rent(t, 60, 131, RENEWAL_RENT_YEAR, 0, 0.03)


def market_rent(t, market_rent_year):
    """The market lease from t=63, after three months of downtime, escalating on its anniversaries."""
    return lease_rent(t, 63, 131, market_rent_year, 0, 0.03)


def recoveries(t, start, end, opex_year, opex_esc, stop, share):
    if t < start or t > end:
        return 0.0
    lt = t - start
    opex = opex_year * ((1.0 + opex_esc) ** (lt // 12))
    return max(0.0, opex - stop) * share / 12.0


def run(market_rent_year):
    renews = market_rent_year > RENEWAL_RENT_YEAR

    def a_after(t):
        return renewal_rent(t) if renews else market_rent(t, market_rent_year)

    def forward_noi(sale_t):
        total = 0.0
        for t in range(sale_t + 1, sale_t + 13):
            total += a_after(t) - VACANCY_MONTH - opex_month(t)
        return total

    debt_pay = -pmt(0.055 / 12.0, 300, 6_000_000.0)
    rows = []
    noi_total = leasing_costs = 0.0
    for t in range(PERIODS):
        a_rent = lease_rent(t, 0, 59, 480_000.0, 3, 0.03)
        a_rec = recoveries(t, 0, 59, 300_000.0, 0.025, 300_000.0, 0.40)
        b_rent = lease_rent(t, 6, 89, 360_000.0, 0, 0.025)
        b_rec = recoveries(t, 6, 89, 300_000.0, 0.025, 180_000.0, 0.30)
        after = a_after(t)
        vacancy = VACANCY_MONTH
        opex = opex_month(t)

        net = a_rent + a_rec + b_rent + b_rec + after - vacancy - opex - debt_pay
        # Dated one-shots: the commencement TI/LC of each lease that begins —
        # the two original leases, and whichever of the renewal and the re-let
        # the election leaves standing. The option itself pays nothing: the
        # exercise's cash is the renewal lease's leasing cost.
        shot = 0.0
        option = 0.0
        exit_shot = 0.0
        if t == 0:
            shot -= 200_000.0
        if t == 6:
            shot -= 150_000.0
        if renews and t == 60:
            shot -= RENEWAL_TI_LC
        if not renews and t == 63:
            shot -= NEW_TI_LC
        if t == 119:
            exit_shot += forward_noi(119) / 0.065 * 0.98
        egi_t = a_rent + a_rec + b_rent + b_rec + after - vacancy
        noi_t = egi_t - opex
        rows.append((t, net, shot, option, exit_shot, egi_t, noi_t, after if renews else 0.0, 0.0 if renews else after))
        noi_total += noi_t
        leasing_costs += -shot

    monthly_rate = (1.0 + DISC) ** (1.0 / 12.0) - 1.0
    # Recurring flows settle at the close of the period that earned them and
    # discount one full period. A dated one-shot — a leasing cost, the
    # option's exercise — discounts from the period's open. A disposal is taken
    # at the end of the holding period and discounts the full n periods.
    npv = sum(
        net / ((1.0 + monthly_rate) ** (t + 1))
        + (shot + option) / ((1.0 + monthly_rate) ** t)
        + exit_shot / ((1.0 + monthly_rate) ** (t + 1))
        for t, net, shot, option, exit_shot, _e, _n, _r, _m in rows
    )
    total = sum(net + shot + option + exit_shot for t, net, shot, option, exit_shot, _e, _n, _r, _m in rows)
    debt_total = debt_pay * PERIODS
    return rows, debt_pay, npv, total, noi_total, leasing_costs, debt_total


def main():
    rows, debt_pay, npv, total, noi_total, leasing_costs, debt_total = run(560_000.0)
    with open("expected.csv", "w", newline="") as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow([
            "period",
            "net_cash_flow",
            "cre.unit.base_rent.tenant_a_renewal",
            "cre.unit.base_rent.tenant_a_market",
            "domain.cre.egi",
            "domain.cre.noi",
            "domain.cre.debt_service",
            "domain.cre.dscr",
        ])
        for t, net, shot, option, exit_shot, egi_t, noi_t, renew, relet in rows:
            writer.writerow([
                t,
                f"{net + shot + option + exit_shot:.6f}",
                f"{renew:.6f}",
                f"{relet:.6f}",
                f"{egi_t:.6f}",
                f"{noi_t:.6f}",
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
        fh.write("\n")
    _r, _d, s_npv, s_total, s_noi, _lc, _dt = run(480_000.0)
    with open("expected_scenarios.json", "w") as fh:
        json.dump(
            {
                "soft_market": {
                    "model.npv": {"value": round(s_npv, 2), "tolerance": 1.0},
                    "model.total": {"value": round(s_total, 2), "tolerance": 1.0},
                }
            },
            fh,
            indent=2,
        )
        fh.write("\n")
    print(f"base: npv={npv:,.2f} total={total:,.2f} noi={noi_total:,.2f} leasing={leasing_costs:,.2f}")
    print(f"soft_market: npv={s_npv:,.2f} total={s_total:,.2f} noi={s_noi:,.2f}")


if __name__ == "__main__":
    main()
