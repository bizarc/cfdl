"""The :class:`Results` wrapper and its pandas accessors."""
from __future__ import annotations

import json

import pandas as pd

from ._frames import currency_of, period_index, scalar


class Results:
    """A parsed CFDL results document (docs/06_results_schema.md).

    Wraps the raw results dict and exposes pandas views over the deterministic
    series, metrics (core + domain), scenarios, and Monte Carlo summaries.
    """

    def __init__(self, raw: dict, results_json: str | None = None):
        self.raw = raw
        self._json = results_json

    @classmethod
    def from_json(cls, results_json: str) -> "Results":
        return cls(json.loads(results_json), results_json)

    # -- scalars -----------------------------------------------------------
    @property
    def results_json(self) -> str | None:
        return self._json

    @property
    def model_hash(self) -> str:
        return self.raw.get("model_hash", "")

    @property
    def status(self) -> str:
        return self.raw.get("deterministic", {}).get("status", "")

    @property
    def warnings(self) -> list[str]:
        return list(self.raw.get("warnings", []))

    def to_dict(self) -> dict:
        return self.raw

    # -- cash flows --------------------------------------------------------
    def cashflows(self, *, wide: bool = True) -> pd.DataFrame:
        """Per-period cash flows from ``deterministic.series``.

        ``wide=True`` (default): one column per series over a PeriodIndex, with
        per-series currency in ``df.attrs["currency"]``. ``wide=False``: a long
        frame with columns ``[series, period, amount, currency]``.
        """
        series = self.raw.get("deterministic", {}).get("series", {})
        if not series:
            return pd.DataFrame()

        if wide:
            columns: dict[str, pd.Series] = {}
            currencies: dict[str, str | None] = {}
            index_ref = None
            for name, block in sorted(series.items()):
                idx = period_index(block["index"])
                index_ref = index_ref if index_ref is not None else idx
                values = [scalar(v) for v in block["values"]]
                columns[name] = pd.Series(values, index=idx)
                currencies[name] = next(
                    (currency_of(v) for v in block["values"] if currency_of(v)),
                    None,
                )
            frame = pd.DataFrame(columns)
            frame.index.name = "period"
            frame.attrs["currency"] = currencies
            return frame

        rows = []
        for name, block in sorted(series.items()):
            idx = period_index(block["index"])
            for period, value in zip(idx, block["values"]):
                rows.append(
                    {
                        "series": name,
                        "period": period,
                        "amount": scalar(value),
                        "currency": currency_of(value),
                    }
                )
        return pd.DataFrame(rows, columns=["series", "period", "amount", "currency"])

    # -- metrics -----------------------------------------------------------
    def _all_metrics(self) -> dict:
        core = dict(self.raw.get("deterministic", {}).get("metrics", {}))
        domain = (self.raw.get("domain_metrics") or {}).get("metrics", {})
        merged = dict(core)
        merged.update(domain)
        return merged

    def metrics(self) -> pd.Series:
        """Flat ``name -> float`` series of all metrics (core + domain).

        Per-metric currency (where present) is preserved in ``.attrs``.
        """
        merged = self._all_metrics()
        values = {name: scalar(v) for name, v in merged.items()}
        currencies = {
            name: currency_of(v) for name, v in merged.items() if currency_of(v)
        }
        out = pd.Series(values, dtype="float64").sort_index()
        out.attrs["currency"] = currencies
        return out

    def metrics_frame(self) -> pd.DataFrame:
        """Metrics as a frame with ``[metric, value, currency, source]``.

        ``source`` is ``core`` or ``domain:<pack>``.
        """
        core = self.raw.get("deterministic", {}).get("metrics", {})
        domain_block = self.raw.get("domain_metrics") or {}
        domain = domain_block.get("metrics", {})
        pack = domain_block.get("pack")
        rows = []
        for name, value in core.items():
            rows.append(
                {
                    "metric": name,
                    "value": scalar(value),
                    "currency": currency_of(value),
                    "source": "core",
                }
            )
        for name, value in domain.items():
            rows.append(
                {
                    "metric": name,
                    "value": scalar(value),
                    "currency": currency_of(value),
                    "source": f"domain:{pack}" if pack else "domain",
                }
            )
        return pd.DataFrame(
            rows, columns=["metric", "value", "currency", "source"]
        ).sort_values("metric", ignore_index=True)

    # -- annual rollup -----------------------------------------------------
    def annual(self) -> pd.DataFrame:
        """Annual rollup series, or an empty frame when absent."""
        rollup = self.raw.get("deterministic", {}).get("annual_rollup")
        if not rollup:
            return pd.DataFrame()
        series = rollup.get("series", {})
        columns = {
            name: [scalar(v) for v in block["values"]]
            for name, block in sorted(series.items())
        }
        return pd.DataFrame(columns)

    # -- scenarios ---------------------------------------------------------
    def scenarios(self) -> pd.DataFrame:
        """One row per scenario, metric columns (Money flattened to amount)."""
        summaries = self.raw.get("scenarios", {}).get("summaries", [])
        if not summaries:
            return pd.DataFrame()
        rows = []
        for summary in summaries:
            row = {"scenario": summary.get("name")}
            for metric, value in summary.get("metrics", {}).items():
                row[metric] = scalar(value)
            rows.append(row)
        return pd.DataFrame(rows)

    # -- monte carlo -------------------------------------------------------
    def monte_carlo(self) -> pd.DataFrame:
        """Monte Carlo per-metric summary stats (rows=metrics, cols=stats).

        Run metadata (``status``, ``trials``, ``seed``) is in ``.attrs``.
        Empty frame when Monte Carlo was not run.
        """
        mc = self.raw.get("monte_carlo", {})
        metrics = mc.get("metrics", {})
        rows = {}
        for name, summary in metrics.items():
            rows[name] = {
                stat: scalar(val)
                for stat, val in summary.items()
                if stat != "type"
            }
        frame = pd.DataFrame.from_dict(rows, orient="index")
        frame.attrs.update(
            status=mc.get("status"), trials=mc.get("trials"), seed=mc.get("seed")
        )
        return frame

    # -- plotting ----------------------------------------------------------
    @property
    def plot(self):
        """Lazy plotting proxy (requires the ``[viz]`` extra)."""
        from . import viz

        return viz.ResultsPlotter(self)

    def __repr__(self) -> str:
        return (
            f"Results(status={self.status!r}, model_hash={self.model_hash[:12]!r}, "
            f"warnings={len(self.warnings)})"
        )
