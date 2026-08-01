#!/usr/bin/env python3
"""Independent reference for level_pay_pool.

Month-by-month recursion of the documented pool convention
(packs/credit/lowering/rules.toml): defaults leave at the start of the
period and earn no interest; interest and scheduled principal accrue on the
performing balance; SMM applies to performing balance net of scheduled
principal; recoveries return (1 - severity) of defaulted face,
recovery_lag_months later. The pack computes the same convention in closed
form — this recursion is the independent check.

PROVENANCE: generated implementation; pending practitioner (Excel) review.
Regenerate: python3 reference_gen.py
"""
import csv
import json

PERIODS = 126
BALANCE, RATE, TERM_MONTHS = 25_000_000.0, 0.065, 120
CPR, CDR, SEVERITY, RECOVERY_LAG = 0.08, 0.02, 0.35, 6
SERVICING_FEE, PREPAY_PENALTY_RATE = 0.005, 0.01
PRICE = 24_750_000.0  # 99.0 (1-point discount)
ANNUAL_DISCOUNT = 0.06


def annual_to_monthly(x):
    return 1.0 - (1.0 - x) ** (1.0 / 12.0)


def main():
    r = RATE / 12.0
    smm = annual_to_monthly(CPR)
    mdr = annual_to_monthly(CDR)

    recoveries = [0.0] * PERIODS
    interest_total = principal_total = recovery_total = 0.0
    servicing_total = penalty_total = 0.0
    principal_flows = [0.0] * PERIODS
    net = [0.0] * PERIODS
    bal = BALANCE
    for p in range(TERM_MONTHS):
        default = bal * mdr
        performing = bal - default
        interest = performing * r
        servicing = performing * SERVICING_FEE / 12.0
        c = r / ((1.0 + r) ** (TERM_MONTHS - p) - 1.0)
        sched = performing * c
        # SIFMA Standard Formulas section 2a: SMM is the fraction of the balance
        # outstanding AT THE BEGINNING of the month, net of SCHEDULED
        # amortisation only — defaults are not removed from the base. This
        # reference previously used the post-default balance, the same
        # misreading as the engine, which is why the two agreed. The published
        # Cash Flow A is the tiebreaker; see benchmarks/credit/sifma_cash_flow_a.
        prepay = (bal - bal * c) * smm
        penalty = prepay * PREPAY_PENALTY_RATE
        bal = performing - sched - prepay
        recoveries[p + RECOVERY_LAG] += default * (1.0 - SEVERITY)
        net[p] += interest + sched + prepay + penalty - servicing
        principal_flows[p] += sched + prepay
        interest_total += interest
        principal_total += sched + prepay
        servicing_total += servicing
        penalty_total += penalty

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
    collections = interest_total + principal_total + recovery_total + penalty_total

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
                "domain.credit.servicing": {"value": round(servicing_total, 2), "tolerance": 1.0},
                "domain.credit.penalties": {"value": round(penalty_total, 2), "tolerance": 1.0},
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
