---
id: notebook-credit
title: "Credit loan pool (level-pay)"
slug: "/docs/notebooks/credit-loan-pool"
source: examples/notebooks/03_credit_loan_pool.ipynb
generated: full
---

# Credit loan pool (level-pay)

> Outputs below are real: the notebook runs against the `credit` pack's benchmark model, which CFDL validates against an independent reference. To run it yourself, see [the Python SDK guide](/docs/python-sdk).

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/bizarc/cfdl/blob/main/examples/notebooks/03_credit_loan_pool.ipynb)

A homogeneous level-pay loan pool with CPR prepayments, CDR defaults, loss severity, a recovery lag, a servicing strip and prepayment penalties — priced at a discount to par.

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
shape: (126, 21)
```

```
         asset.buyer.credit_level_pay_survival_auto_a  \
period                                                  
2026-01                                 1.000000        
2026-02                                 0.991393        
2026-03                                 0.982861        
2026-04                                 0.974402        
2026-05                                 0.966016        

         asset.buyer.credit_level_pay_survival_lag_auto_a  domain.credit.balance_outstanding  \
period                                                                                         
2026-01                                      1.0                                2.442971e+07   
2026-02                                      1.0                                2.411241e+07   
2026-03                                      1.0                                2.379807e+07   
2026-04                                      1.0                                2.348667e+07   
2026-05                                      1.0                                2.317816e+07   

         domain.credit.gross_collections  domain.credit.net_collections  \
period                                                                    
2026-01                    457194.867943                  446795.723595   
2026-02                    452225.142236                  441976.718401   
2026-03                    447301.537920                  437202.339282   
2026-04                    442423.647210                  432472.191964   
2026-05                    437591.065867                  427785.885597   

         domain.credit.original_balance  domain.credit.pool_factor  \
period                                                               
2026-01                      24750000.0                   0.987059   
2026-02                      24750000.0                   0.974239   
2026-03                      24750000.0                   0.961538   
2026-04                      24750000.0                   0.948956   
2026-05                      24750000.0                   0.936491   

         domain.credit.principal_collections  domain.credit.principal_paid_negated  \
period                                                                               
2026-01                        320285.175229                         -3.202852e+05   
2026-02                        317299.872656                         -6.375850e+05   
2026-03                        314341.003541                         -9.519261e+05   
2026-04                        311408.337988                         -1.263334e+06   
2026-05                        308501.648088                         -1.571836e+06   

         domain.credit.principal_paid_to_date  ...  domain.credit.total_investment_income  \
period                                         ...                                          
2026-01                          3.202852e+05  ...                          136909.692714   
2026-02                          6.375850e+05  ...                          134925.269580   
2026-03                          9.519261e+05  ...                          132960.534379   
2026-04                          1.263334e+06  ...                          131015.309222   
2026-05                          1.571836e+06  ...                          129089.417780   

         entity.asset.buyer.net_cash_flow  model.net_cash_flow  \
period                                                           
2026-01                     -2.430320e+07        -2.430320e+07   
2026-02                      4.419767e+05         4.419767e+05   
2026-03                      4.372023e+05         4.372023e+05   
2026-04                      4.324722e+05         4.324722e+05   
2026-05                      4.277859e+05         4.277859e+05   

         stream.credit.pool.interest.auto_a  stream.credit.pool.penalty.auto_a  \
period                                                                           
2026-01                       135188.876529                        1720.816184   
2026-02                       133229.509847                        1695.759733   
2026-03                       131289.582287                        1670.952092   
2026-04                       129368.918209                        1646.391013   
2026-05                       127467.343513                        1622.074267   

         stream.credit.pool.prepay.auto_a  stream.credit.pool.recoveries.auto_a  \
period                                                                            
2026-01                     172081.618419                                   0.0   
2026-02                     169575.973277                                   0.0   
2026-03                     167095.209193                                   0.0   
2026-04                     164639.101296                                   0.0   
2026-05                     162207.426682                                   0.0   

         stream.credit.pool.sched_principal.auto_a  stream.credit.pool.servicing.auto_a  \
period                                                                                    
2026-01                            148203.556810                          -10399.144348   
2026-02                            147723.899379                          -10248.423834   
2026-03                            147245.794348                          -10099.198637   
2026-04                            146769.236692                           -9951.455247   
2026-05                            146294.221405                           -9805.180270   

         stream.credit.purchase.price.auto_a  
period                                        
2026-01                          -24750000.0  
2026-02                                  0.0  
2026-03                                  0.0  
2026-04                                  0.0  
2026-05                                  0.0  

[5 rows x 21 columns]
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

Core metrics (NPV/IRR/MOIC/...) plus the pack's domain metrics, with their source labeled.

```python
results.metrics_frame()
```

```
                                     metric         value currency         source
0                 domain.credit.collections  3.072748e+07      USD  domain:credit
1        domain.credit.collections_multiple  1.241514e+00           domain:credit
2                    domain.credit.interest  6.499895e+06      USD  domain:credit
3                   domain.credit.penalties  8.220768e+04      USD  domain:credit
4                   domain.credit.principal  2.297806e+07      USD  domain:credit
5                    domain.credit.purchase  2.475000e+07      USD  domain:credit
6                  domain.credit.recoveries  1.167320e+06      USD  domain:credit
7                   domain.credit.servicing  4.999919e+05      USD  domain:credit
8                   domain.credit.wal_years  4.056967e+00           domain:credit
9                  entity.asset.buyer.total  5.477491e+06      USD           core
10                               model.moic  1.225381e+00                    core
11                                model.npv -2.959752e+05      USD           core
12                    model.payback_periods  8.000000e+01                    core
13                      model.payback_years  6.750000e+00                    core
14                              model.total  5.477491e+06      USD           core
15                          model.wal_years  3.843940e+00                    core
16                 run.annual_discount_rate  6.000000e-02                    core
17                     run.periods_per_year  1.200000e+01                    core
18  stream.credit.pool.interest.auto_a.t...  6.499895e+06      USD           core
19  stream.credit.pool.penalty.auto_a.total  8.220768e+04      USD           core
20   stream.credit.pool.prepay.auto_a.total  8.220768e+06      USD           core
21  stream.credit.pool.recoveries.auto_a...  1.167320e+06      USD           core
22  stream.credit.pool.sched_principal.a...  1.475729e+07      USD           core
23  stream.credit.pool.servicing.auto_a.... -4.999919e+05      USD           core
24  stream.credit.purchase.price.auto_a.... -2.475000e+07      USD           core
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
