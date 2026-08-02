---
id: notebook-opco
title: "Operating company LBO"
slug: "/docs/notebooks/opco-lbo"
source: examples/notebooks/04_opco_lbo.ipynb
---

# Operating company LBO

> Outputs below are real: the notebook runs against the `opco` pack's benchmark model, which CFDL validates against an independent reference. To run it yourself, see [the Python SDK guide](/docs/python-sdk).

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/bizarc/cfdl/blob/main/examples/notebooks/04_opco_lbo.ipynb)

A five-year services buyout: revenue/opex growth, DSO/DPO/DIO working capital, %-of-revenue capex, a term loan (IO then amortizing with a balloon), cash taxes, and an exit on trailing-twelve EBITDA.

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
model_dir = ROOT / "benchmarks/opco/lbo_buyout"
model = cfdl_sdk.compile(model_dir, packs_dir=PACKS)
print("streams:", len(model.ir["streams"]))
```

```
streams: 10
```

## Run

Run with the benchmark's configuration and apply the `opco` pack's domain metrics.

```python
results = model.run(
    config=str(model_dir / "run.json"),
    pack="opco",
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
shape: (60, 11)
```

```
         model.net_cash_flow  stream.opco.acquisition.price  \
period                                                        
2026-01        -1.349413e+07                    -33600000.0   
2026-02         1.540042e+05                            0.0   
2026-03         1.558667e+05                            0.0   
2026-04         1.577407e+05                            0.0   
2026-05         1.596264e+05                            0.0   

         stream.opco.capex  stream.opco.debt.interest  \
period                                                  
2026-01      -30000.000000                  -148750.0   
2026-02      -30146.026517                  -148750.0   
2026-03      -30292.763825                  -148750.0   
2026-04      -30440.215385                  -148750.0   
2026-05      -30588.384673                  -148750.0   

         stream.opco.debt.principal  stream.opco.debt.proceeds  \
period                                                           
2026-01                         0.0                 21000000.0   
2026-02                         0.0                        0.0   
2026-03                         0.0                        0.0   
2026-04                         0.0                        0.0   
2026-05                         0.0                        0.0   

         stream.opco.exit.value  stream.opco.opex.recurring  \
period                                                        
2026-01                     0.0              -650000.000000   
2026-02                     0.0              -652127.930858   
2026-03                     0.0              -654262.828009   
2026-04                     0.0              -656404.714257   
2026-05                     0.0              -658553.612483   

         stream.opco.revenue.recurring  stream.opco.taxes  \
period                                                      
2026-01                   1.000000e+06      -13325.000000   
2026-02                   1.004868e+06      -14037.301124   
2026-03                   1.009759e+06      -14753.951204   
2026-04                   1.014674e+06      -15474.974297   
2026-05                   1.019613e+06      -16200.394584   

         stream.opco.working_capital.adjustment  
period                                           
2026-01                           -1.052055e+06  
2026-02                           -5.802120e+03  
2026-03                           -5.832592e+03  
2026-04                           -5.863220e+03  
2026-05                           -5.894004e+03  
```

```python
# Requires the [viz] extra (pip install cfdl-sdk[viz]).
results.plot.cumulative()
```

```
<Axes: xlabel='period', ylabel='cumulative amount'>
```

![Chart produced by the preceding cell](/notebooks/opco-lbo/cell-08-1.png)

## Metrics

Core metrics (NPV/IRR/MOIC/...) plus the pack's domain metrics, with their source labelled.

```python
results.metrics_frame()
```

```
                                          metric         value currency  \
0                              domain.opco.capex  2.084574e+06      USD   
1                       domain.opco.debt_service  2.828325e+07      USD   
2                             domain.opco.ebitda  2.646942e+07      USD   
3                      domain.opco.ebitda_margin  3.809330e-01     None   
4                                domain.opco.fcf  2.173644e+07      USD   
5                domain.opco.fcf_to_debt_service  7.685270e-01     None   
6                            domain.opco.revenue  6.948579e+07      USD   
7                              domain.opco.taxes  2.648405e+06      USD   
8                    domain.opco.working_capital -0.000000e+00      USD   
9                  entity.operating.target.total  3.335678e+07      USD   
10                                     model.irr  3.076380e-01     None   
11                                    model.moic  3.467004e+00     None   
12                                     model.npv  1.388314e+07      USD   
13                         model.payback_periods  5.900000e+01     None   
14                           model.payback_years  5.000000e+00     None   
15                                   model.total  3.335678e+07      USD   
16                               model.wal_years  4.791671e+00     None   
17                      run.annual_discount_rate  1.200000e-01     None   
18                          run.periods_per_year  1.200000e+01     None   
19           stream.opco.acquisition.price.total -3.360000e+07      USD   
20                       stream.opco.capex.total -2.084574e+06      USD   
21               stream.opco.debt.interest.total -7.283247e+06      USD   
22              stream.opco.debt.principal.total -2.100000e+07      USD   
23               stream.opco.debt.proceeds.total  2.100000e+07      USD   
24                  stream.opco.exit.value.total  5.250358e+07      USD   
25              stream.opco.opex.recurring.total -4.301637e+07      USD   
26           stream.opco.revenue.recurring.total  6.948579e+07      USD   
27                       stream.opco.taxes.total -2.648405e+06      USD   
28  stream.opco.working_capital.adjustment.total  0.000000e+00      USD   

         source  
0   domain:opco  
1   domain:opco  
2   domain:opco  
3   domain:opco  
4   domain:opco  
5   domain:opco  
6   domain:opco  
7   domain:opco  
8   domain:opco  
9          core  
10         core  
11         core  
12         core  
13         core  
14         core  
15         core  
16         core  
17         core  
18         core  
19         core  
20         core  
21         core  
22         core  
23         core  
24         core  
25         core  
26         core  
27         core  
28         core  
```

## What-if

Report the free-cash-flow-to-debt-service coverage and MOIC.

```python
m = results.metrics()
print("FCF / debt service:", round(m["domain.opco.fcf_to_debt_service"], 3))
print("MOIC:", round(m["model.moic"], 3))
```

```
FCF / debt service: 0.769
MOIC: 3.467
```
