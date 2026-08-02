#!/usr/bin/env python3
"""Independent reference for float_bridge_pool.

Floating IO/bullet convention (see io_bullet_loan for the balance dynamics):
the coupon for each period is clamp(index + margin, floor, cap), where the
index is a flat-forward (step) lookup on the declared curve — the last curve
point at or before the period date.

PROVENANCE: generated implementation; pending practitioner (Excel) review.
Regenerate: python3 reference_gen.py
"""
import csv
import datetime
import json

PERIODS = 41
BALANCE, TERM_MONTHS = 15_000_000.0, 36
MARGIN, RATE_FLOOR, RATE_CAP = 0.0275, 0.07, 0.09
CPR, CDR, SEVERITY, RECOVERY_LAG = 0.10, 0.025, 0.45, 5
PRICE = 15_000_000.0
ANNUAL_DISCOUNT = 0.075
START = datetime.date(2026, 1, 1)
CURVE = [
    (datetime.date(2026, 1, 1), 0.048),
    (datetime.date(2026, 7, 1), 0.045),
    (datetime.date(2027, 1, 1), 0.042),
    (datetime.date(2027, 7, 1), 0.040),
    (datetime.date(2028, 1, 1), 0.0385),
]


def annual_to_monthly(x):
    return 1.0 - (1.0 - x) ** (1.0 / 12.0)


def period_date(p):
    month0 = (START.year * 12 + START.month - 1) + p
    return datetime.date(month0 // 12, month0 % 12 + 1, 1)


def index_at(d):
    value = CURVE[0][1]
    for point_date, point_value in CURVE:
        if point_date <= d:
            value = point_value
        else:
            break
    return value


def main():
    smm = annual_to_monthly(CPR)
    mdr = annual_to_monthly(CDR)

    recoveries = [0.0] * PERIODS
    interest_total = principal_total = recovery_total = 0.0
    principal_flows = [0.0] * PERIODS
    net = [0.0] * PERIODS
    bal = BALANCE
    for p in range(TERM_MONTHS):
        coupon = min(max(index_at(period_date(p)) + MARGIN, RATE_FLOOR), RATE_CAP)
        default = bal * mdr
        performing = bal - default
        interest = performing * coupon / 12.0
        if p < TERM_MONTHS - 1:
            # By convention SMM is the fraction of the balance
            # outstanding AT THE BEGINNING of the month, net of SCHEDULED
            # amortisation only — defaults are not removed from the base. This
            # reference previously used the post-default balance, the same
            # misreading as the engine, which is why the two agreed. The published
            # The external reference is the tiebreaker; see
            # benchmarks/credit/mbs_pool_conventions.
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
    # WAL is measured on the same time axis as the discounting above, and for
    # the same reason: a collection at the close of period t sits at (t+1)/12
    # of a year, which is what the market means by "years from the closing
    # date to the distribution date". The purchase is excluded rather than
    # netted — it settles on its own date, a full period before that period's
    # collections, so the two are not the same cash at the same moment and
    # cannot cancel. Hence WAL is computed BEFORE the price is subtracted.
    wal_years = sum(
        ((t + 1) / 12.0) * v for t, v in enumerate(net) if v > 0.0
    ) / sum(v for v in net if v > 0.0)
    principal_wal = sum(
        ((t + 1) / 12.0) * v for t, v in enumerate(principal_flows) if v > 0.0
    ) / sum(v for v in principal_flows if v > 0.0)
    net[0] -= PRICE
    inflows = sum(v for v in net if v > 0.0)
    outflows = -sum(v for v in net if v < 0.0)
    moic = inflows / outflows
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
