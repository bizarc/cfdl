"""JSON -> value/DataFrame helpers shared by the Results accessors."""
from __future__ import annotations

import warnings

import pandas as pd

# CFDL calendar cadence -> pandas period frequency alias.
_CALENDAR_FREQ = {
    "daily": "D",
    "monthly": "M",
    "quarterly": "Q",
    "annual": "Y",
}


def scalar(value) -> float:
    """Coerce a metric/summary value (bare number or Money object) to float."""
    if isinstance(value, dict) and "amount" in value:
        return float(value["amount"])
    return float(value)


def currency_of(value) -> str | None:
    if isinstance(value, dict):
        return value.get("currency")
    return None


def period_index(index: dict) -> pd.Index:
    """Build a PeriodIndex from a series ``index`` block.

    Falls back to a RangeIndex (with a warning) for an unknown calendar, so a
    new cadence never turns a data accessor into an exception.
    """
    periods = int(index.get("periods", 0))
    calendar = index.get("calendar", "")
    freq = _CALENDAR_FREQ.get(calendar)
    if freq is None:
        warnings.warn(
            f"unknown calendar {calendar!r}; using an integer period index",
            RuntimeWarning,
            stacklevel=2,
        )
        return pd.RangeIndex(periods, name="t")
    start = index.get("start")
    return pd.period_range(start=start, periods=periods, freq=freq)
