---
id: benchmark-cre-office-two-tenant
title: "CRE: two-tenant office"
slug: "/docs/examples/cre-office-two-tenant"
description: "An institutional two-tenant office DCF: free rent, anniversary escalations, recoveries above expense stops, tenant improvements and leasing commissions, probability-blended rollover, and a forward-NOI exit over ten years."
source: benchmarks/cre/office_two_tenant
---

# CRE: two-tenant office

An institutional two-tenant office DCF: free rent, anniversary escalations, recoveries above expense stops, tenant improvements and leasing commissions, probability-blended rollover, and a forward-NOI exit over ten years.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

An institutional two-tenant office building held for ten years. Tenant A takes a
five-year lease with three months free and 3% anniversary escalations; Tenant B
takes seven years from mid-year one at 2.5%. Both recover operating expenses
above their own expense stop, at different pro-rata shares. Tenant A's expiry is
modeled as a probability-weighted rollover — 70% renewal at one rent, otherwise
a new tenant at market after three months of downtime, with different tenant
improvement and leasing commission costs on each branch. A permanent mortgage
runs underneath, and the building is sold on forward net operating income.

## The reference

Institutional lease-by-lease office DCF conventions, as practiced by the
commercial valuation software this kind of model is built in.

**Not redistributable.** The source cannot be published, so the reference is an
independent recreation of its conventions — built separately from the model and
compared against it period by period.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.lease_unit` (two instances), `cre.rollover`, `cre.vacancy_loss`, `cre.opex_line`, `cre.permanent_debt`, `cre.exit_forward` |
| Language features | multiple instances of one contract type, per-period subtotals |
| Conventions | free rent, anniversary escalation, recoveries above an expense stop, tenant improvements and leasing commissions, probability-blended rollover with downtime, a forward-NOI exit |

More of the CRE pack's contract surface than any other case.

## The result

Present value **1,424,273.80**, net operating income **4,718,933.90**, leasing
costs **525,000.00** and debt service **4,421,429.94**.

Asserted: five per-period series across 120 months — effective gross income, net
operating income, debt service, the coverage ratio and net cash flow — plus the
four lifetime figures.

Assertion is per period rather than on the totals: a lifetime coverage ratio of
1.4 can contain a year at 0.9.

## The delta

None: every period agrees inside a one-cent tolerance across all 120 months.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.0725}}
version 0.1
model "office-two-tenant"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 120 project 12

entity asset tower : CRE.Asset.RealProperty

// Tenant A: 5-year lease, 3 months free, 3% anniversary escalations,
// recoveries above a full stop at 40% pro-rata, $200k TI/LC.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2030-12
  terms {
    rent_year = 480000
    free_rent_months = 3
    escalation = 0.03
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 300000
    pro_rata_share = 0.40
    ti_total = 120000
    lc_total = 80000
  }
}

// Tenant A rollover: window starts AT EXPIRY; during the 3 downtime months
// only the renewal scenario (70%) pays. Runs through the projection tail so
// exit valuation sees a full forward year.
contract cre.rollover.tenant_a on entity asset.tower {
  term 2031-01..2036-12
  terms {
    renewal_probability = 0.7
    renewal_rent_year = 520000
    market_rent_year = 560000
    market_escalation = 0.03
    downtime_months = 3
    renewal_ti_lc = 100000
    new_ti_lc = 350000
  }
}

// Tenant B: 7-year lease from mid-2026, 2.5% escalations, $180k stop at 30%.
contract cre.lease_unit.tenant_b on entity asset.tower {
  term 2026-07..2033-06
  terms {
    rent_year = 360000
    escalation = 0.025
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 180000
    pro_rata_share = 0.30
    ti_total = 100000
    lc_total = 50000
  }
}

contract cre.vacancy_loss on entity asset.tower {
  term 2026-01..2036-12
  terms {
    rate = 0.02
    potential_gross_year = 900000
  }
}

contract cre.opex_line on entity asset.tower {
  term 2026-01..2036-12
  terms {
    amount_year = 300000
    escalation = 0.025
  }
}

// Sale at the end of the hold; NOI for the valuation year is DERIVED from
// the modeled streams over the 12 projection months after the sale date.
contract cre.exit_forward on entity asset.tower {
  term 2035-12..2035-12
  terms {
    exit_cap = 0.065
    selling_costs = 0.02
  }
}

// Permanent debt: $6m at 5.50%, 25-year amortization, 10-year hold.
//
// Was a hand-written stream computing its own `pmt`. `cre.permanent_debt`
// states the loan instead of its arithmetic, and reproduces the same
// 4,421,429.94 of debt service — the payment is identical because with one
// combined stream principal is the plug.
//
// The balloon stays off (the default): the unamortized $4.5m is repaid out of
// the sale, not as debt service, and folding it into the final period would
// make that period's DSCR meaningless.
// `funded_at_close = 0`: the reference model's cash flow starts
// post-financing — it nets rents against debt service and never books the
// draw — so the proceeds the contract funds by default are excluded here to
// state what the source states.
contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2035-12
  terms {
    principal = 6000000
    rate = 0.055
    amort_months = 300
    funded_at_close = 0
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.0725
  }
}
```

## Verified results

Checked period by period: **5 series** across **120 periods** — **600 values** in all, each within the tolerance shown.

- `net_cash_flow` — within ±0.01
- `domain.cre.egi` — within ±0.01
- `domain.cre.noi` — within ±0.01
- `domain.cre.debt_service` — within ±0.01
- `domain.cre.dscr` — within ±1.0e-6

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 1,424,273.8 | ±1 |
| `domain.cre.noi` | 4,718,933.9 | ±1 |
| `domain.cre.leasing_costs` | 525,000 | ±1 |
| `domain.cre.debt_service` | 4,421,429.94 | ±1 |
| `domain.cre.dscr` | 1.067287 | ±0.0001 |
