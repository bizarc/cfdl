---
id: benchmark-opco-lbo-buyout
title: "OpCo: leveraged buyout"
slug: "/docs/examples/opco-lbo-buyout"
source: benchmarks/opco/lbo_buyout
---

# OpCo: leveraged buyout

A leveraged buyout: entry at a stated multiple, debt paid down out of operating cash flow, and an exit that returns the sponsor's equity.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The model

```cfdl
version 0.1
model "lbo-buyout"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity operating target

// $12M-revenue services business bought at 8.0x run-rate EBITDA with a
// 5.0x term loan; sold after 5 years at 8.5x trailing-12 EBITDA.

contract opco.revenue_line on entity operating.target {
  term 2026-01..2030-12
  terms {
    amount = 1000000
    growth_rate = 0.06
  }
}

contract opco.opex_line on entity operating.target {
  term 2026-01..2030-12
  terms {
    amount = 650000
    growth_rate = 0.04
  }
}

// DSO 45 / DPO 30 / DIO 10; ending balance released at exit.
contract opco.working_capital_policy on entity operating.target {
  term 2026-01..2030-12
  terms {
    ar_days = 45
    ap_days = 30
    inv_days = 10
    release_at_end = 1
  }
}

// Maintenance capex at 3% of revenue.
contract opco.capex_line on entity operating.target {
  term 2026-01..2030-12
  terms {
    pct_of_revenue = 0.03
  }
}

// 5.0x leverage: $21mm term loan, 8.5%, 12 months IO then 7-year
// amortization, balloon at exit.
contract opco.term_debt on entity operating.target {
  term 2026-01..2030-12
  terms {
    principal = 21000000
    rate = 0.085
    io_months = 12
    amort_months = 84
  }
}

// Cash taxes at 26% on EBITDA - D&A - interest (no NOL carryforward).
contract opco.cash_taxes on entity operating.target {
  term 2026-01..2030-12
  terms {
    tax_rate = 0.26
    da_monthly = 150000
  }
}

// Entry at 8.0x annualized run-rate EBITDA = 8.0 * (350k * 12) = $33.6mm.
contract opco.acquisition on entity operating.target {
  term 2026-01..2026-01
  terms {
    price = 33600000
  }
}

// Exit at 8.5x trailing-12 EBITDA net of 1.5% selling costs.
contract opco.exit_ebitda on entity operating.target {
  term 2030-12..2030-12
  terms {
    exit_multiple = 8.5
    selling_costs = 0.015
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.12
  }
}
```

## Verified results

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 13,883,137.75 | ±1 |
| `model.moic` | 3.467004 | ±0.0001 |
| `domain.opco.revenue` | 69,485,786.14 | ±1 |
| `domain.opco.ebitda` | 26,469,420.78 | ±1 |
| `domain.opco.ebitda_margin` | 0.380933 | ±0.0001 |
| `domain.opco.capex` | 2,084,573.58 | ±1 |
| `domain.opco.working_capital` | 0 | ±1 |
| `domain.opco.taxes` | 2,648,405.28 | ±1 |
| `domain.opco.debt_service` | 28,283,246.61 | ±1 |
| `domain.opco.fcf` | 21,736,441.91 | ±1 |
| `domain.opco.fcf_to_debt_service` | 0.768527 | ±0.0001 |
