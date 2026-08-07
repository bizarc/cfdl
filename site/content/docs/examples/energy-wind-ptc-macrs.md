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

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | -5,452,881.52 | ±1 |
| `domain.energy.revenue` | 90,382,400.52 | ±1 |
| `domain.energy.ebitda` | 58,795,819.78 | ±1 |
| `domain.energy.tax_benefits` | 37,981,187.94 | ±1 |
| `domain.energy.debt_service` | 36,768,755.46 | ±1 |
| `domain.energy.dscr` | 1.59907 | ±0.0001 |
