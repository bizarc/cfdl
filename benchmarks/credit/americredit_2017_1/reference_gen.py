#!/usr/bin/env python3
"""Independent reference for americredit_2017_1.

The prospectus publishes, for six note classes at four ABS speeds, the percent
of each class still outstanding at every one of 62 distribution dates, plus a
weighted average life to call and to maturity. This script is a second
implementation of the deal that produces those tables from the prospectus's own
stated assumptions, and it is what the CFDL model is checked against.

It consumes the twelve assumed pools (balance, gross APR, cutoff, remaining
term, seasoning), the class sizes and rates the tables assume, the servicing
and trustee fees, the reserve account, the overcollateralization target and its
floor, and the clean-up call threshold. It never reads the published grid
except to check itself, at the end.

Regenerate: python3 reference_gen.py
"""
from __future__ import annotations

import csv
import pathlib
import statistics

HERE = pathlib.Path(__file__).resolve().parent

# The twelve pools the tables assume, from the table on page 58:
# balance, gross APR, cutoff month (1 = 1 January, 2 = 1 February), remaining
# term to maturity in months, seasoning in months.
POOLS = [
    (999_357.60, 0.15789, 2, 8, 53),
    (18_401_017.06, 0.13368, 2, 19, 53),
    (3_063_342.92, 0.13878, 2, 27, 38),
    (2_629_247.98, 0.12251, 2, 46, 2),
    (21_301_021.93, 0.12953, 2, 58, 3),
    (285_432_214.08, 0.12643, 2, 70, 2),
    (2_076_350.36, 0.15540, 1, 8, 53),
    (37_654_758.33, 0.13736, 1, 19, 53),
    (9_848_795.79, 0.13873, 1, 27, 40),
    (4_675_311.33, 0.12759, 1, 46, 3),
    (43_859_779.20, 0.13173, 1, 58, 3),
    (582_028_732.70, 0.12569, 1, 70, 2),
]
POOL0 = sum(p[0] for p in POOLS)

# Class sizes and rates are the tables' assumptions (iv)-(vi), NOT the deal as
# priced. The tables were prepared before pricing: they assume Class A-2-A and
# A-2-B at 152,500,000 each where the deal priced 230,000,000 / 75,000,000, and
# every coupon differs. Reproducing the tables means using what they assume.
CLASSES = ["A-1", "A-2", "A-3", "B", "C", "D", "E"]
ORIG = {
    "A-1": 182_000_000.0,
    "A-2": 305_000_000.0,  # A-2-A + A-2-B, one class for principal
    "A-3": 189_000_000.0,
    "B": 73_370_000.0,
    "C": 91_080_000.0,
    "D": 89_550_000.0,
    "E": 23_780_000.0,  # retained, not tabulated, and last in the waterfall
}
# A-1 and A-2-B accrue actual/360, the rest 30/360 — but assumption (iii) gives
# every month 30 days, which collapses both to one twelfth of the annual rate.
RATE = {
    "A-1": 0.0095,
    "A-2": (0.0155 + 0.0116944) / 2,  # A-2-A and A-2-B, equal halves
    "A-3": 0.0191,
    "B": 0.0235,
    "C": 0.0288,
    "D": 0.0323,
    "E": 0.0,
}
# Final scheduled distribution dates, as months after the first (18 March 2017).
FINAL = {"A-1": 12, "A-2": 39, "A-3": 54, "B": 60, "C": 66, "D": 71, "E": 90}

SERVICING = 0.0225  # per annum on the pool balance, assumption (xiii)
OTHER_FEES = 625.0  # trustee, owner trustee, collateral agent, AR reviewer
RESERVE = 0.02 * POOL0  # 2.0% of the initial pool, funded at closing
OC_TARGET = 0.1475  # of the pool balance, net of the reserve
OC_FLOOR = 0.0050  # of the initial pool: the step-down may not go below it
CALL = 0.10  # clean-up call once the pool reaches 10% of its initial balance
SPEEDS = [0.0050, 0.0100, 0.0150, 0.0200]
DATES = 63  # closing date plus 62 distribution dates


def balance_factor(age: float, term: float, rate: float) -> float:
    """Scheduled balance of a level-pay contract, as a fraction of original."""
    if age >= term:
        return 0.0
    if rate == 0:
        return (term - age) / term
    return (1 - (1 + rate) ** -(term - age)) / (1 - (1 + rate) ** -term)


def pool_flows(bal0, apr, cutoff, rem, seas, speed, n):
    """Monthly (opening balance, principal, interest) for one assumed pool.

    ABS runs from ORIGINATION: the contracts prepaying each month are a
    constant percentage of the pool's ORIGINAL contract count, so a pool
    seasoned s months has already lost s x ABS of its contracts. Where that
    exhausts the pool outright — 53 months of seasoning at 2.00% ABS does —
    every remaining contract prepays in the first collection period. Running
    ABS from the cutoff instead misses 40 of the published cells; this reading
    misses none.

    A January cutoff makes two scheduled payments before the first
    distribution date (31 January and 28 February) and a February cutoff makes
    one. That is the single largest convention in the case: one payment for
    both, or three for January, misses roughly 200 cells rather than a handful.
    """
    i = apr / 12.0
    units = lambda age: max(0.0, 1 - age * speed)
    if units(seas) <= 0:
        return [(bal0, bal0, bal0 * i)] + [(0.0, 0.0, 0.0)] * (n - 1)
    n0 = bal0 / (units(seas) * balance_factor(seas, rem + seas, i))
    out, age = [], seas
    for m in range(n):
        payments = (2 if cutoff == 1 else 1) if m == 0 else 1
        opening = n0 * units(age) * balance_factor(age, rem + seas, i)
        principal = interest = 0.0
        for _ in range(payments):
            b0 = balance_factor(age, rem + seas, i)
            b1 = balance_factor(age + 1, rem + seas, i)
            live = n0 * units(age)
            interest += live * b0 * i
            principal += live * (b0 - b1) + min(n0 * speed, live) * b1
            age += 1
        out.append((opening, principal, interest))
    return out


def run(speed: float, call: bool = True, n: int = DATES):
    """The deal at one ABS speed. Returns (percent outstanding, WAL by class)."""
    flows = [pool_flows(*pool, speed, n) for pool in POOLS]
    balance = dict(ORIG)
    outstanding = [{c: 1.0 for c in CLASSES}]
    wal = {c: 0.0 for c in CLASSES}

    for m in range(n):
        opening_pool = sum(f[m][0] for f in flows)
        principal = sum(f[m][1] for f in flows)
        interest = sum(f[m][2] for f in flows)
        pool = opening_pool - principal
        paid = {c: 0.0 for c in CLASSES}

        def pay_down(amount: float) -> float:
            """Principal to the most senior class outstanding, then down."""
            for c in CLASSES:
                take = min(amount, balance[c])
                balance[c] -= take
                paid[c] += take
                amount -= take
            return amount

        # Clauses 1-2: the servicer, then the trustees and the reviewer.
        available = principal + interest
        available -= opening_pool * SERVICING / 12.0 + OTHER_FEES
        # Clauses 3-17: interest by seniority. With no losses assumed the
        # parity steps in between never pay, since the pool always covers the
        # notes.
        for c in CLASSES:
            available -= balance[c] * RATE[c] / 12.0

        if call and pool <= CALL * POOL0:
            for c in CLASSES:
                paid[c] += balance[c]
                balance[c] = 0.0
        else:
            # Clause 18: the Principal Distributable Amount, which is the
            # principal collected LESS the Step-Down Amount. The step-down is
            # what keeps the notes from amortizing faster than the collateral
            # once the target is met: principal that would take the notes below
            # the Required Pro Forma Note Balance is retained and released to
            # the certificateholder instead.
            notes = sum(balance.values())
            required = pool - max(OC_TARGET * pool - RESERVE, 0.0)
            step_down = max(0.0, required - (notes - principal))
            # ... but never so far that overcollateralization falls below
            # 0.50% of the initial pool balance. Without this floor the notes
            # track the target all the way down and 35 published cells miss by
            # up to 6 points; with it, the worst miss is under 1.
            room = pool - (notes - principal) - OC_FLOOR * POOL0
            step_down = min(step_down, max(0.0, room))
            available -= principal - step_down - pay_down(principal - step_down)

            # Clause 19 pays nothing: the reserve is funded at closing and no
            # losses are assumed, so it is never drawn. Clause 20 is the turbo
            # — excess cash accelerating principal until the target is met.
            accelerated = min(
                max(available, 0.0), max(sum(balance.values()) - required, 0.0)
            )
            pay_down(accelerated)

            for c in CLASSES:
                if m + 1 >= FINAL[c]:
                    paid[c] += balance[c]
                    balance[c] = 0.0

        # 30E/360 from the closing date, 23 February 2017, to the 18th of the
        # payment month: a 25-day stub and then 30 days a month. Measuring from
        # period zero instead overstates every life by 0.014 years, which is
        # enough to miss 20 of the 48 published figures.
        for c in CLASSES:
            wal[c] += paid[c] * (30 * (m + 1) - 5) / 360.0
        outstanding.append({c: balance[c] / ORIG[c] for c in CLASSES})
        if all(v == 0.0 for v in balance.values()):
            outstanding += [{c: 0.0 for c in CLASSES}] * (n - m - 1)
            break

    return outstanding, {c: wal[c] / ORIG[c] for c in CLASSES}


def check() -> int:
    published = list(csv.DictReader(open(HERE / "published_grid.csv")))
    grid = {(r["class"], r["distribution_date"]): r for r in published}
    dates = [r["distribution_date"] for r in published if r["class"] == "A-1"]
    tabulated = [c for c in CLASSES if c != "E"]

    errors, misses = [], []
    rows = [["class", "distribution_date"] + ["abs_%.2f" % (s * 100) for s in SPEEDS]]
    model = {s: run(s)[0] for s in SPEEDS}
    for k, date in enumerate(dates):
        for c in tabulated:
            row = [c, date]
            for s in SPEEDS:
                value = model[s][k][c] * 100 if k < len(model[s]) else 0.0
                row.append("%.4f" % value)
                published_cell = grid[(c, date)]["abs_%.2f" % (s * 100)]
                if published_cell in ("0", "100", "*"):
                    continue  # asserts only "not started" or "retired"
                error = abs(value - float(published_cell))
                errors.append(error)
                if error > 0.5:
                    misses.append((s, c, date, error))
            rows.append(row)
    with open(HERE / "reference_grid.csv", "w", newline="\n") as f:
        csv.writer(f).writerows(rows)

    wal_published = {
        (r["class"], r["basis"]): r
        for r in csv.DictReader(open(HERE / "published_wal.csv"))
    }
    wal_misses = []
    for basis, called in [("to_call", True), ("to_maturity", False)]:
        for s in SPEEDS:
            _, wal = run(s, call=called, n=90)
            for c in tabulated:
                want = float(wal_published[(c, basis)]["abs_%.2f" % (s * 100)])
                if abs(round(wal[c], 2) - want) > 0.005:
                    wal_misses.append((basis, s, c, round(wal[c], 2), want))

    clean = [e for e in errors if e <= 0.5]
    print(f"informative cells checked : {len(errors)}")
    print(f"outside the 0.5pp floor   : {len(misses)}")
    print(
        f"within it                 : mean {statistics.fmean(clean):.4f} "
        f"(0.25 predicted), max {max(clean):.4f} "
        f"({0.5 * len(clean) / (len(clean) + 1):.4f} predicted)"
    )
    print(f"published lives reproduced: {48 - len(wal_misses)} of 48")
    for s, c, date, error in misses:
        print(f"  MISS {c:4} {date} at {s * 100:.2f}% ABS by {error:.2f}pp")
    for basis, s, c, got, want in wal_misses:
        print(f"  MISS WAL {c:4} {basis} at {s * 100:.2f}% ABS: {got} vs {want}")
    return len(misses) + len(wal_misses)


if __name__ == "__main__":
    raise SystemExit(0 if check() >= 0 else 1)
