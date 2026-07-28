---
id: benchmark-energy-solar-ppa-microgrid
title: "Energy: solar PPA microgrid"
slug: "/docs/examples/energy-solar-ppa-microgrid"
source: benchmarks/energy/solar_ppa_microgrid
---

# Energy: solar PPA microgrid

2 MW solar + storage microgrid, 25-year PPA, level-pay debt.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The model

```cfdl
version 0.1
model "solar-ppa-microgrid"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 300

entity project microgrid

// 2 MW solar + storage microgrid, 25-year PPA.
contract energy.capex on entity project.microgrid {
  term 2026-01..2026-01
  terms { amount = 2400000 }
}

contract energy.itc on entity project.microgrid {
  term 2026-12..2026-12
  terms { credit = 720000 }
}

contract energy.ppa on entity project.microgrid {
  term 2026-01..2050-12
  terms {
    mwh_year = 4200
    ppa_price = 85
    escalation = 0.02
    degradation = 0.005
  }
}

contract energy.storage_arbitrage on entity project.microgrid {
  term 2026-01..2050-12
  terms {
    mwh_cycled_year = 500
    spread = 30
  }
}

contract energy.capacity on entity project.microgrid {
  term 2026-01..2050-12
  terms { payment_year = 60000 }
}

contract energy.om on entity project.microgrid {
  term 2026-01..2050-12
  terms {
    om_year = 70000
    escalation = 0.025
  }
}

contract energy.debt_service on entity project.microgrid {
  term 2026-01..2045-12
  terms {
    rate = 0.06
    term_months = 240
    principal = 1600000
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.08
  }
}
```

## Verified results

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 1,239,647.39 | ±1 |
| `model.total` | 5,771,865.78 | ±1 |
| `domain.energy.revenue` | 12,594,004.52 | ±1 |
| `domain.energy.ebitda` | 10,202,961.05 | ±1 |
| `domain.energy.debt_service` | 2,751,095.26 | ±1 |
| `domain.energy.dscr` | 3.708691 | ±0.0001 |
