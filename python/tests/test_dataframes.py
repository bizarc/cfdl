"""Shape and structure checks for the pandas accessors."""
from __future__ import annotations

import cfdl_sdk
import pandas as pd
from conftest import PACKS_DIR, VALID_DIR, fixture_spec

import pytest


def _run(name):
    d = VALID_DIR / name
    pack, config, rate = fixture_spec(d)
    return cfdl_sdk.run(d, packs_dir=str(PACKS_DIR), config=config, rate=rate, pack=pack)


def test_cashflows_wide_and_long():
    res = _run("credit_pool_smoke")
    wide = res.cashflows(wide=True)
    assert isinstance(wide.index, pd.PeriodIndex)
    periods = wide.shape[0]
    n_series = wide.shape[1]
    assert n_series >= 1
    assert isinstance(wide.attrs["currency"], dict)

    long = res.cashflows(wide=False)
    assert list(long.columns) == ["series", "period", "amount", "currency"]
    assert len(long) == periods * n_series


def test_metrics_series_and_frame():
    res = _run("credit_pool_smoke")
    metrics = res.metrics()
    assert isinstance(metrics, pd.Series)
    assert "model.irr" in metrics.index
    assert "model.npv" in metrics.index
    # credit pack contributes domain metrics
    assert any(m.startswith("domain.credit.") for m in metrics.index)

    frame = res.metrics_frame()
    assert set(frame.columns) == {"metric", "value", "currency", "source"}
    sources = set(frame["source"])
    assert "core" in sources
    assert any(s.startswith("domain:") for s in sources)

    # A unitless metric carries a BLANK currency, never a missing value. The
    # repr of a missing value in an object column is a pandas display internal
    # — `None` in pandas 2, `NaN` in pandas 3 — and these frames are rendered
    # into committed documentation pages.
    assert not frame["currency"].isna().any()
    assert all(isinstance(c, str) for c in frame["currency"])
    assert "" in set(frame["currency"])  # ratios: irr, moic, margins


def test_scenarios_frame():
    res = _run("cre_developer_scenarios")
    sc = res.scenarios()
    assert not sc.empty
    assert "scenario" in sc.columns
    assert set(sc["scenario"]) >= {"base"}


def test_monte_carlo_frame():
    res = _run("assume_monte_carlo")
    mc = res.monte_carlo()
    assert not mc.empty
    assert mc.attrs["status"] == "ok"
    assert mc.attrs["trials"] >= 1
    assert "mean" in mc.columns


def test_annual_rollup():
    res = _run("opco_growth_smoke")
    annual = res.annual()
    # opco_growth_smoke is monthly with a rollup; frame is non-empty
    assert isinstance(annual, pd.DataFrame)
