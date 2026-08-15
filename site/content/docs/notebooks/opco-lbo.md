---
id: notebook-opco
title: "Operating company LBO"
slug: "/docs/notebooks/opco-lbo"
description: "An operating company leveraged buyout modeled end to end in Python, run through the engine with its cash flows and returns charted."
source: examples/notebooks/04_opco_lbo.ipynb
generated: full
---

# Operating company LBO

> Outputs below are real: the notebook runs against the `opco` pack's benchmark model, which CFDL validates against an independent reference. To run it yourself, see [the Python SDK guide](/docs/python-sdk).

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/bizarc/cfdl/blob/main/examples/notebooks/04_opco_lbo.ipynb)

A five-year services buyout: revenue/opex growth, DSO/DPO/DIO working capital, %-of-revenue capex, a term loan (IO then amortizing with a balloon), cash taxes, and an exit on trailing-twelve EBITDA.

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
shape: (60, 26)
```

```
         asset.target.opco_capex_growth  asset.target.opco_opex_growth  \
period                                                                   
2026-01                             1.0                       1.000000   
2026-02                             1.0                       1.003274   
2026-03                             1.0                       1.006558   
2026-04                             1.0                       1.009853   
2026-05                             1.0                       1.013159   

         asset.target.opco_revenue_growth  domain.opco.cfads  domain.opco.debt_service_coverage  \
period                                                                                            
2026-01                          1.000000     -745379.794521                          -5.010957   
2026-02                          1.004868      302754.171794                           2.035322   
2026-03                          1.009759      304616.658715                           2.047843   
2026-04                          1.014674      306490.722025                           2.060442   
2026-05                          1.019613      308376.426292                           2.073119   

         domain.opco.debt_service_periodic  domain.opco.ebitda_periodic  \
period                                                                    
2026-01                           148750.0                350000.000000   
2026-02                           148750.0                352739.619707   
2026-03                           148750.0                355495.966170   
2026-04                           148750.0                358269.131912   
2026-05                           148750.0                361059.209939   

         domain.opco.lfcf_periodic  domain.opco.net_cash_from_financing  \
period                                                                    
2026-01               2.010587e+07                           20851250.0   
2026-02               1.540042e+05                            -148750.0   
2026-03               1.558667e+05                            -148750.0   
2026-04               1.577407e+05                            -148750.0   
2026-05               1.596264e+05                            -148750.0   

         domain.opco.net_cash_from_investing  ...  stream.opco.acquisition.price  \
period                                        ...                                  
2026-01                        -3.363000e+07  ...                    -33600000.0   
2026-02                        -3.014603e+04  ...                            0.0   
2026-03                        -3.029276e+04  ...                            0.0   
2026-04                        -3.044022e+04  ...                            0.0   
2026-05                        -3.058838e+04  ...                            0.0   

         stream.opco.capex  stream.opco.debt.interest  stream.opco.debt.principal  \
period                                                                              
2026-01      -30000.000000                  -148750.0                         0.0   
2026-02      -30146.026517                  -148750.0                         0.0   
2026-03      -30292.763825                  -148750.0                         0.0   
2026-04      -30440.215385                  -148750.0                         0.0   
2026-05      -30588.384673                  -148750.0                         0.0   

         stream.opco.debt.proceeds  stream.opco.exit.value  stream.opco.opex.recurring  \
period                                                                                   
2026-01                 21000000.0                     0.0              -650000.000000   
2026-02                        0.0                     0.0              -652127.930858   
2026-03                        0.0                     0.0              -654262.828009   
2026-04                        0.0                     0.0              -656404.714257   
2026-05                        0.0                     0.0              -658553.612483   

         stream.opco.revenue.recurring  stream.opco.taxes  stream.opco.working_capital.adjustment  
period                                                                                             
2026-01                   1.000000e+06      -13325.000000                           -1.052055e+06  
2026-02                   1.004868e+06      -14037.301124                           -5.802120e+03  
2026-03                   1.009759e+06      -14753.951204                           -5.832592e+03  
2026-04                   1.014674e+06      -15474.974297                           -5.863220e+03  
2026-05                   1.019613e+06      -16200.394584                           -5.894004e+03  

[5 rows x 26 columns]
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

Core metrics (NPV/IRR/MOIC/...) plus the pack's domain metrics, with their source labeled.

```python
results.metrics_frame()
```

```
                                     metric         value currency       source
0                         domain.opco.capex  2.084574e+06      USD  domain:opco
1                  domain.opco.debt_service  2.828325e+07      USD  domain:opco
2                        domain.opco.ebitda  2.646942e+07      USD  domain:opco
3                 domain.opco.ebitda_margin  3.809330e-01           domain:opco
4                           domain.opco.fcf  2.173644e+07      USD  domain:opco
5           domain.opco.fcf_to_debt_service  7.685270e-01           domain:opco
6                       domain.opco.revenue  6.948579e+07      USD  domain:opco
7                         domain.opco.taxes  2.648405e+06      USD  domain:opco
8               domain.opco.working_capital -0.000000e+00      USD  domain:opco
9                 entity.asset.target.total  3.335678e+07      USD         core
10                                model.irr  3.076380e-01                  core
11                               model.moic  3.467004e+00                  core
12                                model.npv  1.388314e+07      USD         core
13                    model.payback_periods  5.900000e+01                  core
14                      model.payback_years  5.000000e+00                  core
15                              model.total  3.335678e+07      USD         core
16                          model.wal_years  4.791671e+00                  core
17                 run.annual_discount_rate  1.200000e-01                  core
18                     run.periods_per_year  1.200000e+01                  core
19      stream.opco.acquisition.price.total -3.360000e+07      USD         core
20                  stream.opco.capex.total -2.084574e+06      USD         core
21          stream.opco.debt.interest.total -7.283247e+06      USD         core
22         stream.opco.debt.principal.total -2.100000e+07      USD         core
23          stream.opco.debt.proceeds.total  2.100000e+07      USD         core
24             stream.opco.exit.value.total  5.250358e+07      USD         core
25         stream.opco.opex.recurring.total -4.301637e+07      USD         core
26      stream.opco.revenue.recurring.total  6.948579e+07      USD         core
27                  stream.opco.taxes.total -2.648405e+06      USD         core
28  stream.opco.working_capital.adjustme...  0.000000e+00      USD         core
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
