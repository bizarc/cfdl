---
id: benchmark-cre-retail-strip
title: "CRE: retail strip with expense stops"
slug: "/docs/examples/cre-retail-strip"
source: benchmarks/cre/retail_strip
---

# CRE: retail strip with expense stops

A retail strip centre with base-year expense gross-ups, percentage rent over a breakpoint, and staggered tenant rollover across a ten-year hold.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A retail strip centre held for ten years. An anchor tenant pays base rent plus
percentage rent above a stated breakpoint; inline shops sit on net leases with
staggered expiries. Operating expense recoveries run off a base-year stop that is
grossed up to 95% occupancy, so the landlord recovers as though the centre were
nearly full even when it is not. The centre is sold on forward net operating
income.

## The reference

Retail centre valuation conventions — base-year gross-ups and percentage rent
as practised in institutional retail underwriting.

**Not redistributable.** The source cannot be published, so the reference is an
independent recreation of its conventions, built separately from the model and
compared against it period by period.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.lease_unit` (two instances), `cre.percentage_rent`, `cre.vacancy_loss`, `cre.property_opex`, `cre.exit` |
| Language features | multiple instances of one contract type |
| Conventions | a base-year expense stop with a 95% gross-up, percentage rent over a breakpoint, net leases, staggered rollover |

A gross-up implemented as a flat recovery understates income in every year the
centre sits below the gross-up threshold.

## The result

Present value **8,328,491.23**, net operating income **4,012,080.73** and leasing
costs **340,000.00**.

Asserted: effective gross income, net operating income and net cash flow per
period across 120 months, plus the three lifetime figures.

Percentage rent, the base-year gross-up and the recovery stop can each be wrong
in offsetting ways that a lifetime NOI would not show, so assertion is per
period.

## The delta

None: every period agrees inside a one-cent tolerance across all 120 months.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.0775}}
version 0.1
model "retail-strip"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 84

entity asset strip_center : CRE.Asset.RealProperty

// Anchor grocer: base-year stop (stop = year-0 grossed-up opex), 95% gross-up,
// 60% pro-rata, plus percentage rent above a $12M breakpoint.
contract cre.lease_unit.anchor on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    rent_year = 540000
    escalation = 0.02
    opex_year = 240000
    opex_escalation = 0.03
    expense_stop_year = 228000
    gross_up_factor = 0.95
    pro_rata_share = 0.60
    ti_total = 150000
    lc_total = 60000
  }
}

contract cre.percentage_rent.anchor on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    sales_year = 11500000
    sales_growth = 0.03
    breakpoint_year = 12000000
    overage_pct = 0.02
  }
}

// Inline shops: single lease with net recoveries (no stop), 30% share.
contract cre.lease_unit.shops on entity asset.strip_center {
  term 2026-07..2031-06
  terms {
    rent_year = 288000
    free_rent_months = 2
    escalation = 0.025
    opex_year = 240000
    opex_escalation = 0.03
    gross_up_factor = 0.95
    pro_rata_share = 0.30
    ti_total = 90000
    lc_total = 40000
  }
}

contract cre.vacancy_loss on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    rate = 0.03
    potential_gross_year = 850000
  }
}

contract cre.property_opex on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    opex_year = 240000
    escalation = 0.03
  }
}

contract cre.exit on entity asset.strip_center {
  term 2032-12..2032-12
  terms {
    noi_forward_year = 640000
    exit_cap = 0.0675
    selling_costs = 0.015
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.0775
  }
}
```

## Verified results

Checked period by period: **3 series** across **84 periods** — **252 values** in all, each within ±0.01 of the reference.

- `net_cash_flow`
- `domain.cre.egi`
- `domain.cre.noi`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 8,328,491.23 | ±1 |
| `domain.cre.noi` | 4,012,080.73 | ±1 |
| `domain.cre.leasing_costs` | 340,000 | ±1 |
