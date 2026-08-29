---
id: notebook-cre
title: "CRE office acquisition (institutional lease-by-lease DCF)"
slug: "/docs/notebooks/cre-office-acquisition"
description: "A commercial real estate acquisition modeled end to end in Python, from assumptions through cash flows, metrics, and charts."
source: examples/notebooks/02_cre_office_acquisition.ipynb
generated: full
---

# CRE office acquisition (institutional lease-by-lease DCF)

> Outputs below are real: the notebook runs against the `cre` pack's benchmark model, which CFDL validates against an independent reference. To run it yourself, see [the Python SDK guide](/docs/python-sdk).

<video controls preload="metadata" style="width:100%" src="/videos/cre-office-acquisition-walkthrough.mp4"></video>

*A two-minute agent-driven walkthrough: an AI agent executes this notebook cell by cell, every output computing live in the take.*

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/bizarc/cfdl/blob/main/examples/notebooks/02_cre_office_acquisition.ipynb)

A two-tenant office acquisition modeled lease-by-lease: free rent, anniversary escalations, expense recoveries over stops, TI/LC, probability-weighted rollover, and an exit on forward NOI.

This notebook uses one of the benchmark models, which CFDL validates against an independent reference to the penny.

```python
# On Colab, install the SDK and fetch the models this notebook reads.
# Inside a checkout both are already present and this cell does nothing.
import subprocess, sys
from pathlib import Path

REPO = "https://github.com/bizarc/cfdl"


def repo_root() -> Path:
    """The checkout holding benchmarks/ and packs/, cloning it if need be.

    Searching a bounded set of ancestors means a plain `python` run outside a
    checkout fails with an explanation rather than walking to the filesystem
    root. On a hosted runtime there is no checkout to find, so fetch one.
    """
    here = Path.cwd().resolve()
    for candidate in (here, *here.parents):
        if (candidate / "Cargo.toml").exists() and (candidate / "packs").is_dir():
            return candidate

    if "google.colab" not in sys.modules:
        raise RuntimeError(
            f"No CFDL checkout found above {here}. This notebook reads a model "
            f"from benchmarks/ and pack definitions from packs/, so run it "
            f"inside a clone of {REPO}."
        )

    subprocess.run([sys.executable, "-m", "pip", "install", "-q", "cfdl-sdk[viz]"], check=True)

    # Packs and benchmark models track the engine, so take the checkout at the
    # tag matching the wheel pip just resolved. `main` runs ahead of the last
    # release and its packs may use metric ops the released engine rejects.
    import importlib
    from importlib.metadata import PackageNotFoundError, version

    importlib.invalidate_caches()
    try:
        tag = f"v{version('cfdl-sdk')}"
    except PackageNotFoundError:
        tag = None

    clone = ["git", "clone", "--depth", "1", "-q", REPO]
    target = Path("/content/cfdl")
    if not target.exists():
        pinned = tag is not None and not subprocess.run(
            clone + ["--branch", tag, str(target)]
        ).returncode
        if not pinned:
            # A dev or pre-release wheel has no matching tag; main is the best
            # available, and the notebook may fail if the two have diverged.
            print(f"warning: no {tag} tag for this SDK build; falling back to main.")
            subprocess.run(clone + [str(target)], check=True)
    return target


ROOT = repo_root()
PACKS = ROOT / "packs"

import cfdl_sdk
```

## Compile

Compile the model directory to IR.

```python
model_dir = ROOT / "benchmarks/cre/office_two_tenant"
model = cfdl_sdk.compile(model_dir, packs_dir=PACKS)
print("streams:", len(model.ir["streams"]))
```

```
streams: 17
```

## Run

Run with the benchmark's configuration and apply the `cre` pack's domain metrics.

```python
results = model.run(
    config=str(model_dir / "run.json"),
    pack="cre",
)
print("status:", results.status, "| warnings:", len(results.warnings))
```

```
status: ok | warnings: 0
```

## Cash flows

The engine returns per-period signed cash flows; `cashflows()` gives a wide DataFrame indexed by period.

```python
cf = results.cashflows()
print('shape:', cf.shape)
cf.head()
```

```
shape: (120, 26)
```

```
         domain.cre.debt_service  domain.cre.dscr  domain.cre.egi  domain.cre.leasing_costs  \
period                                                                                        
2026-01             36845.249537        -0.719224         -1500.0                  200000.0   
2026-02             36845.249537        -0.719224         -1500.0                       0.0   
2026-03             36845.249537        -0.719224         -1500.0                       0.0   
2026-04             36845.249537         0.366397         38500.0                       0.0   
2026-05             36845.249537         0.366397         38500.0                       0.0   

         domain.cre.noi  domain.cre.opex_total  domain.cre.pgr  entity.asset.tower.net_cash_flow  \
period                                                                                             
2026-01        -26500.0               -25000.0         40000.0                    -263345.249537   
2026-02        -26500.0               -25000.0         40000.0                     -63345.249537   
2026-03        -26500.0               -25000.0         40000.0                     -63345.249537   
2026-04         13500.0               -25000.0         40000.0                     -23345.249537   
2026-05         13500.0               -25000.0         40000.0                     -23345.249537   

         model.net_cash_flow  stream.cre.debt.interest  ...  stream.cre.rollover.ti_lc.tenant_a  \
period                                                  ...                                       
2026-01       -263345.249537             -27500.000000  ...                                 0.0   
2026-02        -63345.249537             -27457.167606  ...                                 0.0   
2026-03        -63345.249537             -27414.138897  ...                                 0.0   
2026-04        -23345.249537             -27370.912974  ...                                 0.0   
2026-05        -23345.249537             -27327.488931  ...                                 0.0   

         stream.cre.unit.abatement.tenant_a  stream.cre.unit.abatement.tenant_b  \
period                                                                            
2026-01                            -40000.0                                 0.0   
2026-02                            -40000.0                                 0.0   
2026-03                            -40000.0                                 0.0   
2026-04                                 0.0                                 0.0   
2026-05                                 0.0                                 0.0   

         stream.cre.unit.base_rent.tenant_a  stream.cre.unit.base_rent.tenant_b  \
period                                                                            
2026-01                             40000.0                                 0.0   
2026-02                             40000.0                                 0.0   
2026-03                             40000.0                                 0.0   
2026-04                             40000.0                                 0.0   
2026-05                             40000.0                                 0.0   

         stream.cre.unit.recoveries.tenant_a  stream.cre.unit.recoveries.tenant_b  \
period                                                                              
2026-01                                  0.0                                  0.0   
2026-02                                  0.0                                  0.0   
2026-03                                  0.0                                  0.0   
2026-04                                  0.0                                  0.0   
2026-05                                  0.0                                  0.0   

         stream.cre.unit.ti_lc.tenant_a  stream.cre.unit.ti_lc.tenant_b  stream.cre.vacancy.loss  
period                                                                                            
2026-01                       -200000.0                             0.0                  -1500.0  
2026-02                             0.0                             0.0                  -1500.0  
2026-03                             0.0                             0.0                  -1500.0  
2026-04                             0.0                             0.0                  -1500.0  
2026-05                             0.0                             0.0                  -1500.0  

[5 rows x 26 columns]
```

```python
# Requires the [viz] extra (pip install cfdl-sdk[viz]).
results.plot.cumulative()
```

```
<Axes: xlabel='period', ylabel='cumulative amount'>
```

![Chart produced by the preceding cell](/notebooks/cre-office-acquisition/cell-08-1.png)

## Metrics

Core metrics (NPV/IRR/MOIC/...) plus the pack's domain metrics, with their source labeled.

```python
results.metrics_frame()
```

```
                                     metric         value currency      source
0                   domain.cre.debt_service  4.421430e+06      USD  domain:cre
1                           domain.cre.dscr  1.067287e+00           domain:cre
2                  domain.cre.leasing_costs  5.250000e+05      USD  domain:cre
3                            domain.cre.noi  4.718934e+06      USD  domain:cre
4                  entity.asset.tower.total  3.009647e+06      USD        core
5                                 model.irr  2.907830e-01                 core
6                                model.moic  3.234175e+00                 core
7                                 model.npv  1.424274e+06      USD        core
8                     model.payback_periods  5.300000e+01                 core
9                       model.payback_years  4.500000e+00                 core
10                              model.total  3.009647e+06      USD        core
11                          model.wal_years  8.508512e+00                 core
12                 run.annual_discount_rate  7.250000e-02                 core
13                     run.periods_per_year  1.200000e+01                 core
14           stream.cre.debt.interest.total -2.930792e+06      USD        core
15          stream.cre.debt.principal.total -1.490638e+06      USD        core
16           stream.cre.debt.proceeds.total  0.000000e+00      USD        core
17           stream.cre.exit.proceeds.total  3.303207e+06      USD        core
18      stream.cre.exit.selling_costs.total -6.606414e+04      USD        core
19               stream.cre.opex.line.total -3.361015e+06      USD        core
20  stream.cre.rollover.rent.tenant_a.total  2.782460e+06      USD        core
21  stream.cre.rollover.ti_lc.tenant_a.t... -1.750000e+05      USD        core
22  stream.cre.unit.abatement.tenant_a.t... -1.200000e+05      USD        core
23  stream.cre.unit.abatement.tenant_b.t...  0.000000e+00      USD        core
24  stream.cre.unit.base_rent.tenant_a.t...  2.548385e+06      USD        core
25  stream.cre.unit.base_rent.tenant_b.t...  2.717075e+06      USD        core
26  stream.cre.unit.recoveries.tenant_a....  3.075942e+04      USD        core
27  stream.cre.unit.recoveries.tenant_b....  3.012687e+05      USD        core
28     stream.cre.unit.ti_lc.tenant_a.total -2.000000e+05      USD        core
29     stream.cre.unit.ti_lc.tenant_b.total -1.500000e+05      USD        core
30            stream.cre.vacancy.loss.total -1.800000e+05      USD        core
```

## What-if

Inspect the derived forward-NOI exit value and the DSCR domain metric.

```python
mf = results.metrics_frame()
mf[mf["metric"].str.contains("dscr|noi|exit", case=False)]
```

```
                                 metric         value currency      source
1                       domain.cre.dscr  1.067287e+00           domain:cre
3                        domain.cre.noi  4.718934e+06      USD  domain:cre
17       stream.cre.exit.proceeds.total  3.303207e+06      USD        core
18  stream.cre.exit.selling_costs.total -6.606414e+04      USD        core
```

## Extended analysis — the DataFrame is the API

Everything below is ordinary pandas over `cashflows()`. The engine guarantees the
numbers (this case is asserted against an independent reference in CI); the
analysis on top is yours.

```python
# Annual rollup with a computed coverage column.
annual = cf[["domain.cre.egi", "domain.cre.noi", "domain.cre.debt_service"]].groupby(cf.index.year).sum()
annual["dscr"] = annual["domain.cre.noi"] / annual["domain.cre.debt_service"]
annual.round(2)
```

```
        domain.cre.egi  domain.cre.noi  domain.cre.debt_service  dscr
period                                                               
2026         540000.00       240000.00                442142.99  0.54
2027         881025.00       573525.00                442142.99  1.30
2028         910322.62       595135.12                442142.99  1.35
2029         940426.85       617359.66                442142.99  1.40
2030         971360.07       640216.20                442142.99  1.45
2031         920924.75       581502.28                442142.99  1.32
2032         991457.87       643549.84                442142.99  1.46
2033         780329.82       423724.09                442142.99  0.96
2034         563330.76       197809.89                442142.99  0.45
2035         580770.69       206111.80                442142.99  0.47
```

Year one is lease-up — a 0.5x coverage year a lifetime DSCR of 1.07 would have
hidden entirely. Per-period series are what make covenant work possible.

```python
# Covenant screen and trailing-12 NOI.
tight = cf[cf["domain.cre.dscr"] < 1.0]
print(f"months below 1.00x DSCR: {len(tight)} of {len(cf)} (last: {tight.index.max()})")
cf["domain.cre.noi"].rolling(12).sum().plot(title="Trailing-12 NOI")
```

```
months below 1.00x DSCR: 36 of 120 (last: 2035-12)
```

```
<Axes: title={'center': 'Trailing-12 NOI'}, xlabel='period'>
```

![Chart produced by the preceding cell](/notebooks/cre-office-acquisition/cell-16-2.png)

```python
# Tenant-level revenue composition: the lease-by-lease grain survives into results.
rev_cols = [c for c in cf.columns if ".base_rent." in c or ".recoveries." in c or ".rollover.rent." in c]
rev = cf[rev_cols].groupby(cf.index.year).sum()
rev.rename(columns=lambda c: c.replace("stream.cre.", "")).plot.area(title="Revenue composition by year")
```

```
<Axes: title={'center': 'Revenue composition by year'}, xlabel='period'>
```

![Chart produced by the preceding cell](/notebooks/cre-office-acquisition/cell-17-1.png)

```python
# Value sensitivity: re-run the deal across a discount grid — four engine runs.
import pandas as pd
npv = pd.Series({
    rate: model.run(config={"deterministic": {"annual_discount_rate": rate}}, pack="cre").metrics()["model.npv"]
    for rate in (0.06, 0.08, 0.10, 0.12)
})
npv.round(0)
```

```
0.06    1622142.0
0.08    1316581.0
0.10    1064405.0
0.12     855277.0
dtype: float64
```
