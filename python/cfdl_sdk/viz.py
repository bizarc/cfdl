"""Optional plotting helpers (requires the ``[viz]`` extra: matplotlib).

Import is lazy so the core SDK never hard-depends on matplotlib.
"""
from __future__ import annotations


def _require_mpl():
    try:
        import matplotlib.pyplot as plt
    except ImportError as exc:  # pragma: no cover - exercised via monkeypatch
        raise ImportError(
            "plotting requires matplotlib; install with: pip install cfdl-sdk[viz]"
        ) from exc
    return plt


def plot_cashflows(results, series=None, *, ax=None):
    """Step plot of per-period cash flows for one or more series."""
    plt = _require_mpl()
    frame = results.cashflows(wide=True)
    if series is not None:
        cols = [series] if isinstance(series, str) else list(series)
        frame = frame[cols]
    if ax is None:
        _, ax = plt.subplots()
    x = range(len(frame.index))
    for column in frame.columns:
        ax.step(x, frame[column].to_numpy(), where="mid", label=column)
    ax.set_xlabel("period")
    ax.set_ylabel("amount")
    ax.legend(loc="best", fontsize="small")
    return ax


def plot_cumulative(results, series="model.net_cash_flow", *, ax=None):
    """Cumulative sum of a single series (default net cash flow)."""
    plt = _require_mpl()
    frame = results.cashflows(wide=True)
    if series not in frame.columns:
        raise KeyError(f"series {series!r} not in results")
    if ax is None:
        _, ax = plt.subplots()
    cumulative = frame[series].cumsum().to_numpy()
    ax.plot(range(len(cumulative)), cumulative, label=f"cumulative {series}")
    ax.axhline(0.0, color="0.6", linewidth=0.8)
    ax.set_xlabel("period")
    ax.set_ylabel("cumulative amount")
    ax.legend(loc="best", fontsize="small")
    return ax


def plot_mc_distribution(results, metric, *, ax=None):
    """Summary (mean with min/max whiskers) of a Monte Carlo metric."""
    plt = _require_mpl()
    mc = results.monte_carlo()
    if metric not in mc.index:
        raise KeyError(f"metric {metric!r} not in Monte Carlo results")
    row = mc.loc[metric]
    if ax is None:
        _, ax = plt.subplots()
    mean = row.get("mean")
    lo, hi = row.get("min", mean), row.get("max", mean)
    ax.errorbar(
        [0], [mean], yerr=[[mean - lo], [hi - mean]], fmt="o", capsize=6, label=metric
    )
    ax.set_xticks([])
    ax.set_ylabel(metric)
    ax.legend(loc="best", fontsize="small")
    return ax


class ResultsPlotter:
    """Bound plotting proxy exposed as ``Results.plot``."""

    def __init__(self, results):
        self._results = results

    def cashflows(self, series=None, *, ax=None):
        return plot_cashflows(self._results, series, ax=ax)

    def cumulative(self, series="model.net_cash_flow", *, ax=None):
        return plot_cumulative(self._results, series, ax=ax)

    def mc_distribution(self, metric, *, ax=None):
        return plot_mc_distribution(self._results, metric, ax=ax)
