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

## The case

A five-year leveraged buyout of a services business. Entry at 8.0× a $33.6mm
run-rate EBITDA, funded with term debt; the business grows, pays the debt down
out of operating cash flow, and is sold at an exit multiple. Working capital
moves on a days-based policy — receivables, payables and inventory — rather than
as a stated figure, and cash taxes are paid on the levered result.

## The reference

Sponsor buyout conventions: sources and uses at entry, a debt schedule paid from
operating cash flow, working capital driven by days outstanding, and an exit at a
multiple of trailing earnings.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared period by period.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.working_capital_policy`, `opco.term_debt`, `opco.cash_taxes`, `opco.acquisition`, `opco.exit_ebitda` |
| Language features | pack contracts across an entry, a hold and an exit |
| Conventions | entry at a multiple, days-based working capital, debt amortisation from operating cash flow, an exit on trailing EBITDA |

The widest span of the opco pack's contract surface: the case covers the whole
transaction rather than one mechanic.

## The result

Present value **13,883,137.75**, multiple on invested capital **3.467004** and
lifetime revenue **69,485,786.14**.

Asserted: net cash flow per period, plus the three summary figures.

## The delta

None: every period agrees inside a one-cent tolerance. The multiple carries a
basis-point tolerance, being computed from an iterative root.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.12}}
version 0.1
model "lbo-buyout"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity asset target : OpCo.Asset.Enterprise

// $12M-revenue services business bought at 8.0x run-rate EBITDA with a
// 5.0x term loan; sold after 5 years at 8.5x trailing-12 EBITDA.

contract opco.revenue_line on entity asset.target {
  term 2026-01..2030-12
  terms {
    amount = 1000000
    growth_rate = 0.06
  }
}

contract opco.opex_line on entity asset.target {
  term 2026-01..2030-12
  terms {
    amount = 650000
    growth_rate = 0.04
  }
}

// DSO 45 / DPO 30 / DIO 10; ending balance released at exit.
contract opco.working_capital_policy on entity asset.target {
  term 2026-01..2030-12
  terms {
    ar_days = 45
    ap_days = 30
    inv_days = 10
    release_at_end = 1
  }
}

// Maintenance capex at 3% of revenue.
contract opco.capex_line on entity asset.target {
  term 2026-01..2030-12
  terms {
    pct_of_revenue = 0.03
  }
}

// 5.0x leverage: $21mm term loan, 8.5%, 12 months IO then 7-year
// amortization, balloon at exit.
contract opco.term_debt on entity asset.target {
  term 2026-01..2030-12
  terms {
    principal = 21000000
    rate = 0.085
    io_months = 12
    amort_months = 84
  }
}

// Cash taxes at 26% on EBITDA - D&A - interest (no NOL carryforward).
contract opco.cash_taxes on entity asset.target {
  term 2026-01..2030-12
  terms {
    tax_rate = 0.26
    da_monthly = 150000
  }
}

// Entry at 8.0x annualized run-rate EBITDA = 8.0 * (350k * 12) = $33.6mm.
contract opco.acquisition on entity asset.target {
  term 2026-01..2026-01
  terms {
    price = 33600000
  }
}

// Exit at 8.5x trailing-12 EBITDA net of 1.5% selling costs.
contract opco.exit_ebitda on entity asset.target {
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

Checked period by period: **1 series** across **60 periods**, each within ±0.01 of the reference.

- `net_cash_flow`

Summary metrics for the base run:

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
