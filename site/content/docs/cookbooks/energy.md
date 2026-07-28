---
id: cookbook-energy
title: "Energy pack guide"
slug: "/docs/cookbooks/energy"
source: packs/energy/README.md
---

Energy & microgrids: solar/wind PPA and merchant revenue, battery storage
arbitrage, capacity payments, O&M, investment tax credits, capex, and level-pay
project debt. All rules are template-driven (`{{contract.*}}` + defaults) —
no hardcoded amounts.

## Conventions

- Annual quantities (`mwh_year`, `om_year`, `payment_year`) spread evenly
  across months.
- Escalation and degradation step **annually**: `factor ^ floor(t / 12)`,
  matching common project-finance Excel practice.
- `energy.debt_service` uses the engine's decimal-exact `pmt()` (Excel sign
  conventions).

## Contract types

| Contract | Required terms | Optional (default) |
|---|---|---|
| `energy.ppa` | `mwh_year`, `ppa_price` | `escalation` (0), `degradation` (0), `availability` (1) |
| `energy.merchant` | `mwh_year`, `price` | `price_escalation` (0), `degradation` (0), `availability` (1) |
| `energy.storage_arbitrage` | `mwh_cycled_year`, `spread` | `degradation` (0) |
| `energy.capacity` | `payment_year` | — |
| `energy.om` | `om_year` | `escalation` (0) |
| `energy.itc` | `credit` | — (fires on `term_start`) |
| `energy.capex` | `amount` | — (fires on `term_start`) |
| `energy.debt_service` | `rate`, `term_months`, `principal` | — |
| `energy.ptc` | `mwh_year`, `credit_per_mwh` | `escalation` (0), `degradation` (0), `availability` (1); term bounds the credit window |
| `energy.macrs_shield` | `basis`, `tax_rate` | `life` (5; also 7/15/20) — IRS Pub 946 GDS half-year tables via `macrs_rate()` |

Tax attributes (ITC, PTC, MACRS shield) report under
`domain.energy.tax_benefits` and are excluded from revenue/EBITDA.

## Not yet modeled (roadmap)

Tax equity / partnership flip structures (HLBV), DSCR-sculpted debt sizing,
full tax computation (the MACRS stream models the shield value, not taxable
income). Planned for later pack increments.

## Quick start

A 2 MW solar-plus-storage microgrid with a 25-year escalating PPA:

```cfdl
version 0.1
model "my-microgrid"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 300

entity project microgrid

contract energy.capex on entity project.microgrid {
  term 2026-01..2026-01
  terms { amount = 2400000 }
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

contract energy.om on entity project.microgrid {
  term 2026-01..2050-12
  terms { om_year = 70000 escalation = 0.025 }
}
```

Vintages are contract-anchored: each contract's escalation and degradation
clocks start at its own term start (its COD), not the model start.

## Run it

```bash
cfdl compile my-microgrid --packs packs --out my-microgrid/ir.json
cfdl run my-microgrid/ir.json --packs packs --pack energy --out my-microgrid/results.json --rate 0.08
```

`--pack energy` computes the pack's domain metrics (revenue, EBITDA,
`domain.energy.tax_benefits`, debt service) alongside NPV/IRR/MOIC.

## Recipes

**Add the investment tax credit** (fires once, at its term start; reports
under tax benefits, not EBITDA):

```cfdl
contract energy.itc on entity project.microgrid {
  term 2026-12..2026-12
  terms { credit = 720000 }
}
```

**Level-pay project debt** (decimal-exact `pmt()`, Excel sign conventions):

```cfdl
contract energy.debt_service on entity project.microgrid {
  term 2026-01..2045-12
  terms { rate = 0.06 term_months = 240 principal = 1600000 }
}
```

**Wind with PTC and MACRS**: pair `energy.ptc` (per-MWh credit over the
credit window) with `energy.macrs_shield` (IRS Pub 946 GDS tables via
`macrs_rate()`); see `benchmarks/energy/wind_ptc_macrs/`.

Full worked models: `benchmarks/energy/solar_ppa_microgrid/` and the solar
microgrid notebook in `examples/notebooks/`.

## Metrics reference

Computed automatically whenever a model runs with the `energy` pack, alongside the core metrics (NPV, IRR, MOIC, payback, WAL). Enumerated from the pack definition, so this list is always complete.

| Metric | Type | Built from |
|---|---|---|
| `domain.energy.revenue` | money | `energy.ppa.revenue`, `energy.merchant.revenue`, `energy.storage.margin`, `energy.capacity.revenue` |
| `domain.energy.opex` | money | `energy.om.expense` |
| `domain.energy.ebitda` | money | `energy.ppa.revenue`, `energy.merchant.revenue`, `energy.storage.margin`, `energy.capacity.revenue`, `energy.om.expense` |
| `domain.energy.debt_service` | money | `energy.debt.service` |
| `domain.energy.dscr` | number | `domain.energy.ebitda` ÷ `domain.energy.debt_service` |
| `domain.energy.tax_benefits` | money | `energy.itc.credit`, `energy.ptc.credit`, `energy.macrs.shield` |

## Validations reference

Checked at compile time. Each is a stable diagnostic code that is never renamed or reused; see [diagnostics](/docs/language-reference/diagnostics).

| Code | Rejects |
|---|---|
| `E8001_ENERGY_INVALID_DEGRADATION` | Energy 'degradation' must be a fraction between 0 and 1. |
| `E8002_ENERGY_INVALID_AVAILABILITY` | Energy 'availability' must be a fraction between 0 and 1. |
| `E8003_ENERGY_INVALID_ESCALATION` | Energy 'escalation' must be greater than or equal to -1. |
| `E8004_ENERGY_INVALID_PRICE_ESCALATION` | Energy 'price_escalation' must be greater than or equal to -1. |
| `E8010_ENERGY_INVALID_MACRS_LIFE` | Energy MACRS 'life' must be one of 5, 7, 15, or 20. |
| `E8011_ENERGY_INVALID_TAX_RATE` | Energy 'tax_rate' must be a fraction between 0 and 1. |
| `E8020_ENERGY_DEBT_INVALID_RATE` | Energy debt 'rate' must be non-negative. |
| `E8021_ENERGY_DEBT_INVALID_TERM_MONTHS` | Energy debt 'term_months' must be a whole number greater than 0. |
| `E8022_ENERGY_DEBT_INVALID_PRINCIPAL` | Energy debt 'principal' must be greater than 0. |

## Worked example models

Benchmark cases are validated period-by-period against an independent
reference implementation.

- [Energy: solar PPA microgrid](/docs/examples/energy-solar-ppa-microgrid)
- [Energy: wind with PTC and MACRS](/docs/examples/energy-wind-ptc-macrs)
