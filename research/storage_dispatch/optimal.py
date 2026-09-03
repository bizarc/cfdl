#!/usr/bin/env python3
"""An INDEPENDENT, PROVABLY OPTIMAL dispatch reference.

SAM is not a valid target for this comparison. NREL/TP-6A20-68614 §1 states its
dispatch is "automated but SUBOPTIMAL", §5 that "these heuristic algorithms do
not do any optimization around the cost of energy and power", and §4.4 is titled
Controller Limitations. A model computing the per-day optimum should EXCEED it,
so agreement would be evidence of nothing.

This solves the actual problem instead: given an hourly price series and a
battery's physics, what is the most an operator could earn? That is a linear
program, and its optimum is provable rather than asserted.

    maximise   sum_t price_t * (d_t - c_t)
    subject to 0 <= c_t <= P                     charge power
               0 <= d_t <= P                     discharge power
               soc_t = soc_{t-1} + eta*c_t - d_t state of charge
               E_min <= soc_t <= E_max           usable window

Loss is taken entirely on charge, so discharging D requires charging D/eta —
the convention the CFDL model uses, stated here so the two agree by
construction rather than by luck.

TWO HORIZONS, and the difference between them is the whole chronology question:

  `daily`  — each day solved independently, opening and closing empty. This is
             the theoretical best a DAILY-GRAIN model can do, and is what the
             CFDL model should be measured against.
  `annual` — one program over all 8760 hours with state of charge carried
             across midnight. Always >= the daily figure, and the excess is
             exactly what a daily grain gives up.
"""
import numpy as np
from scipy.optimize import linprog
from scipy.sparse import csr_matrix, vstack


def optimise(price, power_mw, usable_mwh, eta, carry=False, cycles_per_day=None):
    """Return (schedule, margin). `schedule` is discharge-minus-charge, MW.

    State of charge is carried as its own variable so the constraint matrix is
    SPARSE — three non-zeros per row. Writing it as cumulative sums instead
    builds a dense n x n triangle, which is 76 million entries over a year and
    does not finish.
    """
    n = len(price)
    # variables: [c_0..c_{n-1}, d_0..d_{n-1}, soc_0..soc_{n-1}]
    c_obj = np.concatenate([price, -price, np.zeros(n)])   # linprog MINIMISES
    # soc_t - soc_{t-1} - eta*c_t + d_t = 0, with soc_{-1} = 0
    rows, cols, vals = [], [], []
    for t in range(n):
        rows += [t, t, t];            cols += [t, n + t, 2 * n + t]
        vals += [-eta, 1.0, 1.0]
        if t > 0:
            rows.append(t); cols.append(2 * n + t - 1); vals.append(-1.0)
    A_eq = csr_matrix((vals, (rows, cols)), shape=(n, 3 * n))
    b_eq = np.zeros(n)
    if not carry:                       # each block opens and closes empty
        A_eq = vstack([A_eq, csr_matrix(([1.0], ([0], [3 * n - 1])), shape=(1, 3 * n))])
        b_eq = np.concatenate([b_eq, [0.0]])
    # A WARRANTY CYCLE LIMIT, without which the optimum runs ~1.9 cycles a day
    # and is not comparable to a model that assumes one. Total discharge in each
    # day is capped at `cycles_per_day` equivalent full cycles.
    A_ub = b_ub = None
    if cycles_per_day is not None:
        ndays = n // 24
        rows, cols, vals = [], [], []
        for day in range(ndays):
            for h in range(day * 24, (day + 1) * 24):
                rows.append(day); cols.append(n + h); vals.append(1.0)
        A_ub = csr_matrix((vals, (rows, cols)), shape=(ndays, 3 * n))
        b_ub = np.full(ndays, cycles_per_day * usable_mwh)
    bounds = [(0, power_mw)] * (2 * n) + [(0, usable_mwh)] * n
    r = linprog(c_obj, A_ub=A_ub, b_ub=b_ub, A_eq=A_eq, b_eq=b_eq,
                bounds=bounds, method="highs")
    if not r.success:
        raise RuntimeError(r.message)
    return r.x[n:2 * n] - r.x[:n], float(-r.fun)


def daily(price, power_mw, usable_mwh, eta, cycles_per_day=1.0):
    """Each day solved on its own — the daily-grain optimum."""
    sched = np.zeros(len(price)); margin = 0.0
    for day in range(len(price) // 24):
        s = slice(day * 24, (day + 1) * 24)
        sched[s], m = optimise(price[s], power_mw, usable_mwh, eta, carry=False,
                               cycles_per_day=cycles_per_day)
        margin += m
    return sched, margin


def annual(price, power_mw, usable_mwh, eta, cycles_per_day=1.0):
    """One program over the year, state of charge carried across midnight."""
    return optimise(price, power_mw, usable_mwh, eta, carry=True,
                    cycles_per_day=cycles_per_day)


if __name__ == "__main__":
    import json, importlib.util, pathlib
    here = pathlib.Path(__file__).parent
    spec = importlib.util.spec_from_file_location("ref", here / "reference.py")
    ref = importlib.util.module_from_spec(spec); spec.loader.exec_module(ref)
    price = ref.hourly_prices()
    P, U = 20.0, 80.0 * 0.80
    eta = 0.96 * 0.96 * 0.9757
    out = {}
    for name, fn in (("daily", daily), ("annual", annual)):
        sched, margin = fn(price, P, U, eta)
        dis, chg = np.maximum(sched, 0), np.maximum(-sched, 0)
        e = dis.reshape(365, 24).sum(axis=1)
        out[name] = {
            "mwh_out": float(dis.sum()), "mwh_in": float(chg.sum()),
            "revenue": float((dis * price).sum()), "cost": float((chg * price).sum()),
            "margin": margin, "active_days": int((e > 0.1).sum()),
            "mean_depth": float(e[e > 0.1].mean() / U),
        }
        print(f"{name:>7}: MWh {dis.sum():>9,.0f}  margin {margin:>11,.0f}  "
              f"days {int((e>0.1).sum()):>4}  mean depth {e[e>0.1].mean()/U:>5.1%}")
    json.dump(out, open(here / "optimal.json", "w"), indent=1)
