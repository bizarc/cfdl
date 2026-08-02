---
id: notebook-credit
title: "Credit loan pool (level-pay)"
slug: "/docs/notebooks/credit-loan-pool"
source: examples/notebooks/03_credit_loan_pool.ipynb
---

# Credit loan pool (level-pay)

> Outputs below are real: the notebook runs against the `credit` pack's benchmark model, which CFDL validates against an independent reference. To run it yourself, see [the Python SDK guide](/docs/python-sdk).

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/bizarc/cfdl/blob/main/examples/notebooks/03_credit_loan_pool.ipynb)

A homogeneous level-pay loan pool with CPR prepayments, CDR defaults, loss severity, a recovery lag, a servicing strip and prepayment penalties — priced at a discount to par.

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
model_dir = ROOT / "benchmarks/credit/level_pay_pool"
model = cfdl_sdk.compile(model_dir, packs_dir=PACKS)
print("streams:", len(model.ir["streams"]))
```

```
streams: 7
```

## Run

Run with the benchmark's configuration and apply the `credit` pack's domain metrics.

```python
results = model.run(
    config=str(model_dir / "run.json"),
    pack="credit",
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
shape: (126, 8)
```

```
         model.net_cash_flow  stream.credit.pool.interest.auto_a  \
period                                                             
2026-01        -2.430320e+07                       135188.876529   
2026-02         4.419767e+05                       133229.509847   
2026-03         4.372023e+05                       131289.582287   
2026-04         4.324722e+05                       129368.918209   
2026-05         4.277859e+05                       127467.343513   

         stream.credit.pool.penalty.auto_a  stream.credit.pool.prepay.auto_a  \
period                                                                         
2026-01                        1720.816184                     172081.618419   
2026-02                        1695.759733                     169575.973277   
2026-03                        1670.952092                     167095.209193   
2026-04                        1646.391013                     164639.101296   
2026-05                        1622.074267                     162207.426682   

         stream.credit.pool.recoveries.auto_a  \
period                                          
2026-01                                   0.0   
2026-02                                   0.0   
2026-03                                   0.0   
2026-04                                   0.0   
2026-05                                   0.0   

         stream.credit.pool.sched_principal.auto_a  \
period                                               
2026-01                              148203.556810   
2026-02                              147723.899379   
2026-03                              147245.794348   
2026-04                              146769.236692   
2026-05                              146294.221405   

         stream.credit.pool.servicing.auto_a  \
period                                         
2026-01                        -10399.144348   
2026-02                        -10248.423834   
2026-03                        -10099.198637   
2026-04                         -9951.455247   
2026-05                         -9805.180270   

         stream.credit.purchase.price.auto_a  
period                                        
2026-01                          -24750000.0  
2026-02                                  0.0  
2026-03                                  0.0  
2026-04                                  0.0  
2026-05                                  0.0  
```

```python
# Requires the [viz] extra (pip install cfdl-sdk[viz]).
results.plot.cumulative()
```

```
<Axes: xlabel='period', ylabel='cumulative amount'>
```

![Chart produced by the preceding cell](/notebooks/credit-loan-pool/cell-08-1.png)

## Metrics

Core metrics (NPV/IRR/MOIC/...) plus the pack's domain metrics, with their source labelled.

```python
results.metrics_frame()
```

```
                                             metric         value currency  \
0                         domain.credit.collections  3.072748e+07      USD   
1                domain.credit.collections_multiple  1.241514e+00     None   
2                            domain.credit.interest  6.499895e+06      USD   
3                           domain.credit.penalties  8.220768e+04      USD   
4                           domain.credit.principal  2.297806e+07      USD   
5                            domain.credit.purchase  2.475000e+07      USD   
6                          domain.credit.recoveries  1.167320e+06      USD   
7                           domain.credit.servicing  4.999919e+05      USD   
8                           domain.credit.wal_years  4.056967e+00     None   
9                           entity.fund.buyer.total  5.477491e+06      USD   
10                                       model.moic  1.225381e+00     None   
11                                        model.npv -2.959752e+05      USD   
12                            model.payback_periods  8.000000e+01     None   
13                              model.payback_years  6.750000e+00     None   
14                                      model.total  5.477491e+06      USD   
15                                  model.wal_years  3.843940e+00     None   
16                         run.annual_discount_rate  6.000000e-02     None   
17                             run.periods_per_year  1.200000e+01     None   
18         stream.credit.pool.interest.auto_a.total  6.499895e+06      USD   
19          stream.credit.pool.penalty.auto_a.total  8.220768e+04      USD   
20           stream.credit.pool.prepay.auto_a.total  8.220768e+06      USD   
21       stream.credit.pool.recoveries.auto_a.total  1.167320e+06      USD   
22  stream.credit.pool.sched_principal.auto_a.total  1.475729e+07      USD   
23        stream.credit.pool.servicing.auto_a.total -4.999919e+05      USD   
24        stream.credit.purchase.price.auto_a.total -2.475000e+07      USD   

           source  
0   domain:credit  
1   domain:credit  
2   domain:credit  
3   domain:credit  
4   domain:credit  
5   domain:credit  
6   domain:credit  
7   domain:credit  
8   domain:credit  
9            core  
10           core  
11           core  
12           core  
13           core  
14           core  
15           core  
16           core  
17           core  
18           core  
19           core  
20           core  
21           core  
22           core  
23           core  
24           core  
```

## What-if

Show the collections multiple and the principal-weighted WAL.

```python
m = results.metrics()
print("collections multiple:", round(m["domain.credit.collections_multiple"], 4))
print("WAL (years):", round(m["domain.credit.wal_years"], 3))
```

```
collections multiple: 1.2415
WAL (years): 4.057
```
