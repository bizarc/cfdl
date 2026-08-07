---
id: benchmark-energy-wind-ptc-macrs
title: "Energy: wind with PTC and MACRS"
slug: "/docs/examples/energy-wind-ptc-macrs"
source: benchmarks/energy/wind_ptc_macrs
---

# Energy: wind with PTC and MACRS

A wind project claiming the production tax credit over ten years and depreciating on the MACRS five-year schedule.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A 30 MW wind project selling at merchant prices, claiming the production tax
credit over its first ten years, and depreciating on the five-year MACRS
schedule. Project debt runs underneath. The interaction is the point: the credit
runs for ten years, depreciation for five, and the debt for longer than either —
so the cash flow shape changes twice before the hold ends.

## The reference

Project-finance conventions for a merchant wind asset with federal tax
attributes: the production credit's inflation adjustment and statutory ten-year
window, and the MACRS half-year convention.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared period by period.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.merchant`, `energy.ptc`, `energy.om`, `energy.debt_service`, `energy.macrs_shield`, `energy.capex` |
| Language features | pack contracts with staggered terms on one asset |
| Conventions | merchant pricing, a ten-year production credit, MACRS five-year depreciation, level-pay debt |

## The result

Present value **−5,452,881.52**, lifetime revenue **90,382,400.52** and lifetime
EBITDA **58,795,819.78**.

Asserted: net cash flow per period, plus the three summary figures.

## The delta

None: every period agrees inside a one-cent tolerance, including the periods
where the credit expires and where depreciation runs out.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.075}}
version 0.1
model "wind-ptc-macrs"
use pack "energy" version "0.1.0"
time calendar monthly from 2027-01 for 240

entity asset windfarm : Energy.Asset.GenerationFacility

// 30 MW wind: merchant revenue with availability, 10-year PTC,
// 5-year MACRS shield, level-pay debt.
contract energy.capex on entity asset.windfarm {
  term 2027-01..2027-01
  terms { amount = 42000000 }
}

contract energy.merchant on entity asset.windfarm {
  term 2027-01..2046-12
  terms {
    mwh_year = 105000
    price = 42
    price_escalation = 0.015
    degradation = 0.007
    availability = 0.95
  }
}

contract energy.ptc on entity asset.windfarm {
  term 2027-01..2036-12
  terms {
    mwh_year = 105000
    credit_per_mwh = 27.5
    escalation = 0.02
    degradation = 0.007
    availability = 0.95
  }
}

contract energy.macrs_shield on entity asset.windfarm {
  term 2027-01..2046-12
  terms {
    basis = 42000000
    tax_rate = 0.21
    life = 5
  }
}

contract energy.om on entity asset.windfarm {
  term 2027-01..2046-12
  terms {
    om_year = 1300000
    escalation = 0.02
  }
}

contract energy.debt_service on entity asset.windfarm {
  term 2027-01..2041-12
  terms {
    rate = 0.055
    term_months = 180
    principal = 25000000
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.075
  }
}
```

## Verified results

Checked period by period: **1 series** across **240 periods**, each within ±0.01 of the reference.

- `net_cash_flow`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | -5,452,881.52 | ±1 |
| `domain.energy.revenue` | 90,382,400.52 | ±1 |
| `domain.energy.ebitda` | 58,795,819.78 | ±1 |
| `domain.energy.tax_benefits` | 37,981,187.94 | ±1 |
| `domain.energy.debt_service` | 36,768,755.46 | ±1 |
| `domain.energy.dscr` | 1.59907 | ±0.0001 |
