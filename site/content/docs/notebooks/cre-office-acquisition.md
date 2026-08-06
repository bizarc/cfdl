---
id: notebook-cre
title: "CRE office acquisition (institutional lease-by-lease DCF)"
slug: "/docs/notebooks/cre-office-acquisition"
source: examples/notebooks/02_cre_office_acquisition.ipynb
generated: full
---

# CRE office acquisition (institutional lease-by-lease DCF)

> Outputs below are real: the notebook runs against the `cre` pack's benchmark model, which CFDL validates against an independent reference. To run it yourself, see [the Python SDK guide](/docs/python-sdk).

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/bizarc/cfdl/blob/main/examples/notebooks/02_cre_office_acquisition.ipynb)

A two-tenant office acquisition modeled lease-by-lease: free rent, anniversary escalations, expense recoveries over stops, TI/LC, probability-weighted rollover, and an exit on forward NOI.

This notebook uses the benchmark model that CFDL validates against an independent reference to the penny (see `benchmarks/`).

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
streams: 14
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
shape: (120, 22)
```

```
         domain.cre.debt_service  domain.cre.dscr  domain.cre.egi  \
period                                                              
2026-01             36845.249537        -0.719224         -1500.0   
2026-02             36845.249537        -0.719224         -1500.0   
2026-03             36845.249537        -0.719224         -1500.0   
2026-04             36845.249537         0.366397         38500.0   
2026-05             36845.249537         0.366397         38500.0   

         domain.cre.leasing_costs  domain.cre.noi  domain.cre.opex_total  \
period                                                                     
2026-01                  200000.0        -26500.0               -25000.0   
2026-02                       0.0        -26500.0               -25000.0   
2026-03                       0.0        -26500.0               -25000.0   
2026-04                       0.0         13500.0               -25000.0   
2026-05                       0.0         13500.0               -25000.0   

         domain.cre.pgr  model.net_cash_flow  stream.cre.exit.proceeds  \
period                                                                   
2026-01         40000.0       -263345.249537                       0.0   
2026-02         40000.0        -63345.249537                       0.0   
2026-03         40000.0        -63345.249537                       0.0   
2026-04         40000.0        -23345.249537                       0.0   
2026-05         40000.0        -23345.249537                       0.0   

         stream.cre.property.opex  ...  stream.cre.unit.abatement.tenant_a  \
period                             ...                                       
2026-01                  -25000.0  ...                            -40000.0   
2026-02                  -25000.0  ...                            -40000.0   
2026-03                  -25000.0  ...                            -40000.0   
2026-04                  -25000.0  ...                                 0.0   
2026-05                  -25000.0  ...                                 0.0   

         stream.cre.unit.abatement.tenant_b  \
period                                        
2026-01                                 0.0   
2026-02                                 0.0   
2026-03                                 0.0   
2026-04                                 0.0   
2026-05                                 0.0   

         stream.cre.unit.base_rent.tenant_a  \
period                                        
2026-01                             40000.0   
2026-02                             40000.0   
2026-03                             40000.0   
2026-04                             40000.0   
2026-05                             40000.0   

         stream.cre.unit.base_rent.tenant_b  \
period                                        
2026-01                                 0.0   
2026-02                                 0.0   
2026-03                                 0.0   
2026-04                                 0.0   
2026-05                                 0.0   

         stream.cre.unit.recoveries.tenant_a  \
period                                         
2026-01                                  0.0   
2026-02                                  0.0   
2026-03                                  0.0   
2026-04                                  0.0   
2026-05                                  0.0   

         stream.cre.unit.recoveries.tenant_b  stream.cre.unit.ti_lc.tenant_a  \
period                                                                         
2026-01                                  0.0                       -200000.0   
2026-02                                  0.0                             0.0   
2026-03                                  0.0                             0.0   
2026-04                                  0.0                             0.0   
2026-05                                  0.0                             0.0   

         stream.cre.unit.ti_lc.tenant_b  stream.cre.vacancy.loss  \
period                                                             
2026-01                             0.0                  -1500.0   
2026-02                             0.0                  -1500.0   
2026-03                             0.0                  -1500.0   
2026-04                             0.0                  -1500.0   
2026-05                             0.0                  -1500.0   

         stream.loan.permanent_debt_service  
period                                       
2026-01                       -36845.249537  
2026-02                       -36845.249537  
2026-03                       -36845.249537  
2026-04                       -36845.249537  
2026-05                       -36845.249537  

[5 rows x 22 columns]
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

Core metrics (NPV/IRR/MOIC/...) plus the pack's domain metrics, with their source labelled.

```python
results.metrics_frame()
```

```
                                       metric         value currency  \
0                     domain.cre.debt_service  4.421430e+06      USD   
1                             domain.cre.dscr  1.067287e+00     None   
2                    domain.cre.leasing_costs  5.250000e+05      USD   
3                              domain.cre.noi  4.718934e+06      USD   
4                    entity.asset.tower.total  3.009647e+06      USD   
5                                  model.moic  3.234175e+00     None   
6                                   model.npv  1.424274e+06      USD   
7                       model.payback_periods  5.300000e+01     None   
8                         model.payback_years  4.500000e+00     None   
9                                 model.total  3.009647e+06      USD   
10                            model.wal_years  8.508512e+00     None   
11                   run.annual_discount_rate  7.250000e-02     None   
12                       run.periods_per_year  1.200000e+01     None   
13             stream.cre.exit.proceeds.total  3.237143e+06      USD   
14             stream.cre.property.opex.total -3.361015e+06      USD   
15    stream.cre.rollover.rent.tenant_a.total  2.782460e+06      USD   
16   stream.cre.rollover.ti_lc.tenant_a.total -1.750000e+05      USD   
17   stream.cre.unit.abatement.tenant_a.total -1.200000e+05      USD   
18   stream.cre.unit.abatement.tenant_b.total  0.000000e+00      USD   
19   stream.cre.unit.base_rent.tenant_a.total  2.548385e+06      USD   
20   stream.cre.unit.base_rent.tenant_b.total  2.717075e+06      USD   
21  stream.cre.unit.recoveries.tenant_a.total  3.075942e+04      USD   
22  stream.cre.unit.recoveries.tenant_b.total  3.012687e+05      USD   
23       stream.cre.unit.ti_lc.tenant_a.total -2.000000e+05      USD   
24       stream.cre.unit.ti_lc.tenant_b.total -1.500000e+05      USD   
25              stream.cre.vacancy.loss.total -1.800000e+05      USD   
26   stream.loan.permanent_debt_service.total -4.421430e+06      USD   

        source  
0   domain:cre  
1   domain:cre  
2   domain:cre  
3   domain:cre  
4         core  
5         core  
6         core  
7         core  
8         core  
9         core  
10        core  
11        core  
12        core  
13        core  
14        core  
15        core  
16        core  
17        core  
18        core  
19        core  
20        core  
21        core  
22        core  
23        core  
24        core  
25        core  
26        core  
```

## What-if

Inspect the derived forward-NOI exit value and the DSCR domain metric.

```python
mf = results.metrics_frame()
mf[mf["metric"].str.contains("dscr|noi|exit", case=False)]
```

```
                            metric         value currency      source
1                  domain.cre.dscr  1.067287e+00     None  domain:cre
3                   domain.cre.noi  4.718934e+06      USD  domain:cre
13  stream.cre.exit.proceeds.total  3.237143e+06      USD        core
```
