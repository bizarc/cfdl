#!/usr/bin/env python3
"""Independent reference for io_bullet_loan (see level_pay_pool for methodology).

IO/bullet convention: balance declines only through prepayment and default;
the final period pays no SMM prepayment — the whole surviving balance pays
as the bullet.

PROVENANCE: generated implementation; pending practitioner (Excel) review.
Regenerate: python3 reference_gen.py
"""
import csv
import json

PERIODS = 64
BALANCE, RATE, TERM_MONTHS = 10_000_000.0, 0.0725, 60
CPR, CDR, SEVERITY, RECOVERY_LAG = 0.05, 0.015, 0.40, 4
PRICE = 10_000_000.0
ANNUAL_DISCOUNT = 0.07


def annual_to_monthly(x):
    return 1.0 - (1.0 - x) ** (1.0 / 12.0)


def main():
    r = RATE / 12.0
    smm = annual_to_monthly(CPR)
    mdr = annual_to_monthly(CDR)

    recoveries = [0.0] * PERIODS
    interest_total = principal_total = recovery_total = 0.0
    principal_flows = [0.0] * PERIODS
    net = [0.0] * PERIODS
    bal = BALANCE
    for p in range(TERM_MONTHS):
        default = bal * mdr
        performing = bal - default
        interest = performing * r
        if p < TERM_MONTHS - 1:
            # SIFMA Standard Formulas section 2a: SMM is the fraction of the balance
            # outstanding AT THE BEGINNING of the month, net of SCHEDULED
            # amortisation only — defaults are not removed from the base. This
            # reference previously used the post-default balance, the same
            # misreading as the engine, which is why the two agreed. The published
            # Cash Flow A is the tiebreaker; see benchmarks/credit/sifma_cash_flow_a.
            prepay = bal * smm
            bullet = 0.0
        else:
            prepay = 0.0
            bullet = performing
        bal = performing - prepay - bullet
        recoveries[p + RECOVERY_LAG] += default * (1.0 - SEVERITY)
        net[p] += interest + prepay + bullet
        principal_flows[p] += prepay + bullet
        interest_total += interest
        principal_total += prepay + bullet

    for p in range(PERIODS):
        net[p] += recoveries[p]
        principal_flows[p] += recoveries[p]
        recovery_total += recoveries[p]

    monthly_rate = (1.0 + ANNUAL_DISCOUNT) ** (1.0 / 12.0) - 1.0    # Recurring flows are ordinary annuities: they settle at the close of the
    # period that earned them, so they are discounted one full period — the
    # same convention as Excel's NPV. One-shot flows (purchase, advance,
    # exit proceeds) settle on their own date and are not.
    npv = -PRICE + sum(
        v / ((1.0 + monthly_rate) ** (t + 1)) for t, v in enumerate(net)
    )
    net[0] -= PRICE
    inflows = sum(v for v in net if v > 0.0)
    outflows = -sum(v for v in net if v < 0.0)
    moic = inflows / outflows
    wal_years = sum((t / 12.0) * v for t, v in enumerate(net) if v > 0.0) / inflows
    principal_wal = sum(
        (t / 12.0) * v for t, v in enumerate(principal_flows) if v > 0.0
    ) / sum(v for v in principal_flows if v > 0.0)
    collections = interest_total + principal_total + recovery_total

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
                "model.wal_years": {"value": round(wal_years, 6), "tolerance": 1e-4},
                "domain.credit.interest": {"value": round(interest_total, 2), "tolerance": 1.0},
                "domain.credit.principal": {"value": round(principal_total, 2), "tolerance": 1.0},
                "domain.credit.recoveries": {"value": round(recovery_total, 2), "tolerance": 1.0},
                "domain.credit.wal_years": {"value": round(principal_wal, 6), "tolerance": 1e-4},
                "domain.credit.collections": {"value": round(collections, 2), "tolerance": 1.0},
                "domain.credit.purchase": {"value": round(PRICE, 2), "tolerance": 1.0},
                "domain.credit.collections_multiple": {
                    "value": round(collections / PRICE, 6),
                    "tolerance": 1e-4,
                },
            },
            fh,
            indent=2,
        )
    print(f"wrote expected.csv, npv={npv:,.2f}, ending balance={bal:,.2f}")


if __name__ == "__main__":
    main()
