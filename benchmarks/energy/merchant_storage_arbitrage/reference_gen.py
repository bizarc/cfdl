#!/usr/bin/env python3
"""Independent reference for merchant_storage_arbitrage.

Two things, from one stated price series:

  1. the model's own market inputs — the daily TBx block prices, which are the
     mean of the dearest hours a battery of this duration can reach and the
     cheapest it can draw from;
  2. the REFERENCE — a provably optimal dispatch, solved as a linear program.

The reference is an optimum rather than a heuristic on purpose. NREL's SAM was
tried first and cannot anchor this case: NREL/TP-6A20-68614 §1 states its
dispatch is "automated but SUBOPTIMAL", §5 that "these heuristic algorithms do
not do any optimization around the cost of energy and power", and §4.4 is
titled Controller Limitations. Agreement with a heuristic would be evidence of
nothing. This solves the problem the model is trying to solve:

    maximise   sum_t price_t * (discharge_t - charge_t)
    subject to 0 <= charge_t, discharge_t <= power
               soc_t = soc_{t-1} + eta * charge_t - discharge_t
               0 <= soc_t <= usable
               sum of discharge within a day <= one full cycle

Loss is taken entirely on charge, so discharging D requires charging D/eta.
The model uses the same convention, stated in both places so they agree by
construction rather than by luck.

State of charge is carried as its own LP variable to keep the constraint matrix
sparse. Written as cumulative sums it is a dense 8760 x 8760 triangle and the
solve does not finish.

Regenerate: python3 reference_gen.py     (needs numpy and scipy)
"""
import csv
import datetime as dt
import pathlib

import numpy as np
from scipy.optimize import linprog
from scipy.sparse import csr_matrix, vstack

HERE = pathlib.Path(__file__).resolve().parent
SEED = 20260902
START = dt.date(2026, 1, 1)

POWER_MW = 20.0
NAMEPLATE_MWH = 80.0
SOC_MIN, SOC_MAX = 0.15, 0.95
USABLE_MWH = NAMEPLATE_MWH * (SOC_MAX - SOC_MIN)
ETA = 0.96 * 0.96 * 0.9757          # AC/DC x DC/AC x cell
CHARGE_MWH = USABLE_MWH / ETA


def hourly_prices():
    """The stated price year. Synthetic and seeded, so both sides see the same
    series and a reader can regenerate it. Summer-peaking, evening-peaked, with
    day-to-day variation and twelve scarcity days."""
    rng = np.random.default_rng(SEED)
    h = np.arange(8760)
    doy, hod = h // 24, h % 24
    seasonal = 8.0 * np.sin(2 * np.pi * (doy - 200) / 365.0)
    diurnal = 18.0 * np.exp(-(((hod - 19) % 24)) ** 2 / 5.0) \
            - 9.0 * np.exp(-(((hod - 4) % 24)) ** 2 / 9.0)
    price = 32.0 + seasonal + diurnal \
          + rng.normal(0, 7, 365).repeat(24) + rng.normal(0, 3, 8760)
    for d in rng.choice(365, 12, replace=False):
        price[d * 24 + np.array([17, 18, 19, 20])] += rng.uniform(150, 600)
    return np.maximum(price, 3.0)


def tbx_blocks(price):
    """The day's achievable prices at THIS asset's duration: the mean of the
    dearest hours it can discharge into and the cheapest it can charge from.

    This is the market's own battery product — TB2, TB4, "top-bottom spread" —
    quoted at a duration because a one-hour and a four-hour asset see different
    spreads from identical prices. Sorting a day's hours is the definition of
    the product, not a dispatch decision: it says which hours the product
    references, never whether the battery runs.
    """
    srt = np.sort(price.reshape(365, 24), axis=1)

    def block(hours, dear):
        col = srt[:, ::-1] if dear else srt
        n, frac = int(hours), hours - int(hours)
        w = np.zeros(24)
        w[:n] = 1.0
        if frac > 0:
            w[n] = frac
        return (col * w).sum(axis=1) / w.sum()

    return (block(USABLE_MWH / POWER_MW, True),
            block(CHARGE_MWH / POWER_MW, False))


def optimal_dispatch(price, cycles_per_day=1.0):
    """The provable optimum, each day solved independently — the best a
    daily-grain model can do. Returns discharge-minus-charge in MW."""
    sched = np.zeros(len(price))
    for day in range(len(price) // 24):
        s = slice(day * 24, (day + 1) * 24)
        p = price[s]
        n = 24
        # variables: [charge, discharge, soc], n each
        obj = np.concatenate([p, -p, np.zeros(n)])       # linprog MINIMISES
        rows, cols, vals = [], [], []
        for t in range(n):
            rows += [t, t, t]
            cols += [t, n + t, 2 * n + t]
            vals += [-ETA, 1.0, 1.0]
            if t > 0:
                rows.append(t); cols.append(2 * n + t - 1); vals.append(-1.0)
        A_eq = csr_matrix((vals, (rows, cols)), shape=(n, 3 * n))
        b_eq = np.zeros(n)
        # each day opens and closes empty
        A_eq = vstack([A_eq, csr_matrix(([1.0], ([0], [3 * n - 1])), shape=(1, 3 * n))])
        b_eq = np.concatenate([b_eq, [0.0]])
        # warranty: at most one equivalent full cycle a day
        A_ub = csr_matrix((np.ones(n), (np.zeros(n, int), np.arange(n, 2 * n))),
                          shape=(1, 3 * n))
        b_ub = np.array([cycles_per_day * USABLE_MWH])
        bounds = [(0, POWER_MW)] * (2 * n) + [(0, USABLE_MWH)] * n
        r = linprog(obj, A_ub=A_ub, b_ub=b_ub, A_eq=A_eq, b_eq=b_eq,
                    bounds=bounds, method="highs")
        if not r.success:
            raise RuntimeError(f"day {day}: {r.message}")
        sched[s] = r.x[n:2 * n] - r.x[:n]
    return sched


def main():
    price = hourly_prices()
    capture, cost = tbx_blocks(price)
    sched = optimal_dispatch(price)
    pr = price.reshape(365, 24)
    dis = np.maximum(sched, 0).reshape(365, 24)
    chg = np.maximum(-sched, 0).reshape(365, 24)
    revenue, spend = (dis * pr).sum(axis=1), (chg * pr).sum(axis=1)

    with open(HERE / "expected.csv", "w", newline="\n") as fh:
        w = csv.writer(fh)
        w.writerow(["period", "market.discharge", "market.charge"])
        for i in range(365):
            w.writerow([i, f"{revenue[i]:.2f}", f"{-spend[i]:.2f}"])

    with open(HERE / "market_inputs.csv", "w", newline="\n") as fh:
        w = csv.writer(fh)
        w.writerow(["date", "capture_price", "cost_price"])
        for i in range(365):
            w.writerow([(START + dt.timedelta(days=i)).isoformat(),
                        f"{capture[i]:.4f}", f"{cost[i]:.4f}"])

    print(f"days dispatched      : {(dis.sum(axis=1) > 0.1).sum()}")
    print(f"MWh discharged       : {dis.sum():,.1f}")
    print(f"revenue              : {revenue.sum():,.2f}")
    print(f"cost                 : {spend.sum():,.2f}")
    print(f"margin               : {revenue.sum() - spend.sum():,.2f}")


if __name__ == "__main__":
    main()
