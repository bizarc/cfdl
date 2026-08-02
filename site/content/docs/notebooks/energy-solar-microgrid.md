---
id: notebook-energy
title: "Solar PPA microgrid"
slug: "/docs/notebooks/energy-solar-microgrid"
source: examples/notebooks/01_energy_solar_microgrid.ipynb
---

# Solar PPA microgrid

> Outputs below are real: the notebook runs against the `energy` pack's benchmark model, which CFDL validates against an independent reference. To run it yourself, see [the Python SDK guide](/docs/python-sdk).

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/bizarc/cfdl/blob/main/examples/notebooks/01_energy_solar_microgrid.ipynb)

A solar-plus-storage microgrid with a PPA revenue contract, degradation, O&M escalation, ITC/PTC tax attributes and MACRS depreciation, financed with sculpted debt.

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
    target = Path("/content/cfdl")
    if not target.exists():
        subprocess.run(["git", "clone", "--depth", "1", "-q", REPO, str(target)], check=True)
    return target


ROOT = repo_root()
PACKS = ROOT / "packs"

import cfdl_sdk
```

## Compile

Compile the model directory to IR.

```python
model_dir = ROOT / "benchmarks/energy/solar_ppa_microgrid"
model = cfdl_sdk.compile(model_dir, packs_dir=PACKS)
print("streams:", len(model.ir["streams"]))
```

```
streams: 7
```

## Run

Run with the benchmark's configuration and apply the `energy` pack's domain metrics.

```python
results = model.run(
    config=str(model_dir / "run.json"),
    pack="energy",
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
shape: (300, 8)
```

```
         model.net_cash_flow  stream.energy.capacity.revenue  \
period                                                         
2026-01        -2.381296e+06                          5000.0   
2026-02         1.870377e+04                          5000.0   
2026-03         1.870377e+04                          5000.0   
2026-04         1.870377e+04                          5000.0   
2026-05         1.870377e+04                          5000.0   

         stream.energy.capex.outlay  stream.energy.debt.service  \
period                                                            
2026-01                  -2400000.0               -11462.896936   
2026-02                         0.0               -11462.896936   
2026-03                         0.0               -11462.896936   
2026-04                         0.0               -11462.896936   
2026-05                         0.0               -11462.896936   

         stream.energy.itc.credit  stream.energy.om.expense  \
period                                                        
2026-01                       0.0              -5833.333333   
2026-02                       0.0              -5833.333333   
2026-03                       0.0              -5833.333333   
2026-04                       0.0              -5833.333333   
2026-05                       0.0              -5833.333333   

         stream.energy.ppa.revenue  stream.energy.storage.margin  
period                                                            
2026-01                    29750.0                        1250.0  
2026-02                    29750.0                        1250.0  
2026-03                    29750.0                        1250.0  
2026-04                    29750.0                        1250.0  
2026-05                    29750.0                        1250.0  
```

```python
# Requires the [viz] extra (pip install cfdl-sdk[viz]).
results.plot.cumulative()
```

```
<Axes: xlabel='period', ylabel='cumulative amount'>
```

![Chart produced by the preceding cell](/notebooks/energy-solar-microgrid/cell-08-1.png)

## Metrics

Core metrics (NPV/IRR/MOIC/...) plus the pack's domain metrics, with their source labelled.

```python
results.metrics_frame()
```

```
                                  metric         value currency         source
0             domain.energy.debt_service  2.751095e+06      USD  domain:energy
1                     domain.energy.dscr  3.708691e+00     None  domain:energy
2                   domain.energy.ebitda  1.020296e+07      USD  domain:energy
3                     domain.energy.opex  2.391043e+06      USD  domain:energy
4                  domain.energy.revenue  1.259400e+07      USD  domain:energy
5             domain.energy.tax_benefits  7.200000e+05      USD  domain:energy
6         entity.project.microgrid.total  5.771866e+06      USD           core
7                             model.moic  3.423834e+00     None           core
8                              model.npv  1.220669e+06      USD           core
9                  model.payback_periods  8.500000e+01     None           core
10                   model.payback_years  7.166667e+00     None           core
11                           model.total  5.771866e+06      USD           core
12                       model.wal_years  1.299224e+01     None           core
13              run.annual_discount_rate  8.000000e-02     None           core
14                  run.periods_per_year  1.200000e+01     None           core
15  stream.energy.capacity.revenue.total  1.500000e+06      USD           core
16      stream.energy.capex.outlay.total -2.400000e+06      USD           core
17      stream.energy.debt.service.total -2.751095e+06      USD           core
18        stream.energy.itc.credit.total  7.200000e+05      USD           core
19        stream.energy.om.expense.total -2.391043e+06      USD           core
20       stream.energy.ppa.revenue.total  1.071900e+07      USD           core
21    stream.energy.storage.margin.total  3.750000e+05      USD           core
```

## What-if

Re-run at a higher discount rate and compare NPV.

```python
base = results.metrics()["model.npv"]
stressed = model.run(config={"deterministic": {"annual_discount_rate": 0.10}}, pack="energy")
print(f"NPV @ base: {base:,.0f}")
print(f"NPV @ 10%: {stressed.metrics()['model.npv']:,.0f}")
```

```
NPV @ base: 1,220,669
NPV @ 10%: 737,280
```
