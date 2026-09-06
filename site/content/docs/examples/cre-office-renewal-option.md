---
id: benchmark-cre-office-renewal-option
title: "CRE: a renewal option on a lease"
slug: "/docs/examples/cre-office-renewal-option"
description: "The two-tenant office DCF with Tenant A's expiry as a renewal option on the lease: exercised at expiry when the market rent exceeds the option rent, and a re-let at market when it does not."
source: benchmarks/cre/office_renewal_option
---

# CRE: a renewal option on a lease

The two-tenant office DCF with Tenant A's expiry as a renewal option on the lease: exercised at expiry when the market rent exceeds the option rent, and a re-let at market when it does not.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

The two-tenant office building of `office_two_tenant`, held for ten years,
with one change. Tenant A's five-year lease carries a renewal option: five more
years at $520k a year, escalating 3% on the renewal term's anniversaries, with
$100k of tenant improvements and leasing commissions at renewal. At expiry the
tenant renews if the market rent then exceeds the option rent. If the option
lapses, the space is re-let at market after three months of downtime, on a new
lease with $350k of leasing costs. Tenant B, the vacancy allowance, the
operating expenses, the permanent mortgage and the sale on forward net
operating income are the original's, unchanged.

## The reference

Institutional lease-by-lease office DCF conventions, as practiced by the
commercial valuation software this kind of model is built in. The original case
handles Tenant A's expiry as a probability-weighted rollover. This case handles
it as the right the lease grants: a renewal at stated terms, exercised or not,
with each outcome a lease of its own.

**Not redistributable.** The source cannot be published, so the reference is an
independent recreation of its conventions, built separately from the model and
compared against it period by period. It is the original case's recreation with
the rollover replaced by the option and its two outcomes.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.lease_unit` (four instances), `cre.vacancy_loss`, `cre.opex_line`, `cre.permanent_debt`, `cre.exit_forward`, and the election `CRE.Contract.RenewalOption` |
| Language features | an option written on a contract, an option's own terms, an option tested on a stated date, actions on exercise, a scheduled event, a run scenario |
| Conventions | free rent, anniversary escalation, recoveries above an expense stop, tenant improvements and leasing commissions, a renewal option at stated terms, downtime and re-letting at market, a forward-NOI exit |

The renewal is an election, not a probability. The option is written on the
lease it extends and states its own terms:

```cfdl
option renewal on contract cre.lease_unit.tenant_a type CRE.Contract.RenewalOption {
  parties { landlord = party.landlord_co, tenant = party.acme }
  terms {
    renewal_rent_year = inputs.renewal_rent_year
    renewal_term_months = 60
    renewal_ti_lc = 100000
  }
  schedule on 2031-01
  exercise when inputs.market_rent_year > contract.renewal_rent_year
  payoff 0
  set entity asset.tower.tenant_a_renewed = 1
  deactivate stream cre.unit.base_rent.tenant_a_market
  deactivate stream cre.unit.abatement.tenant_a_market
  deactivate stream cre.unit.recoveries.tenant_a_market
  deactivate stream cre.unit.ti_lc.tenant_a_market
}
```

Both outcomes are leases the model declares: the renewal lease from January
2031, and the market lease from April 2031. The schedule tests the election
once, in the month after expiry. On exercise the option records the renewal on
the building and switches off the market lease; a scheduled event with the
opposite test switches off the renewal lease when the option lapses. The exit
values the building on the lease left standing, because the forward net
operating income is derived from the leases' own streams.

The market rent is an assumption, so the outcome the option does not take is a
run scenario rather than a second model. In the base run the market rent is
$560k and the tenant renews. In the `soft_market` scenario the market rent is
$480k, the option lapses, and the space is re-let.

## The result

Base run: present value **1,362,611.39**, net operating income
**4,697,224.27**, leasing costs **450,000.00** and debt service
**4,421,429.94**. Soft-market scenario: present value **595,382.95** and total
cash **1,495,550.50**.

Asserted: seven per-period series across 120 months, the renewal lease's rent,
the market lease's rent, effective gross income, net operating income, debt
service, the coverage ratio and net cash flow, plus the five lifetime figures
and the two scenario figures. Every asserted cell agrees with the reference to
the cent.

Against the original case's present value of 1,424,273.80, the renewal
outcome is **61,662.41** lower and the re-let outcome **828,890.85** lower.
The two figures bound what the original's 70/30 blend stands for.

## The delta

None: every period agrees inside a one-cent tolerance across all 120 months,
and both scenario figures agree to the cent.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.0725},"scenarios":{"soft_market":{"parameters":{"inputs.market_rent_year":480000}}}}
version 0.1
model "office-renewal-option"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 120 project 12

// The two-tenant office of `office_two_tenant`, with one change: Tenant A's
// expiry is not a probability-weighted rollover but the renewal option the
// lease grants. Five more years at a stated rent, exercised at expiry if the
// market rent then exceeds the option rent; if the option lapses, the space is
// re-let at market after three months of downtime. The election decides which
// branch pays, and the branch it does not take is the scenario.

entity asset tower : CRE.Asset.RealProperty {
  // Written by the option's exercise: the renewal, on the record.
  tenant_a_renewed init 0 next prev
}
entity party landlord_co : Party { name = "Landlord" }
entity party acme : Party { name = "Tenant A" }

// The market at expiry. The base run renews; the scenario re-lets.
assume market_rent_year = 560000
// The stated terms of the right, read by the option and by the renewal lease.
assume renewal_rent_year = 520000

// Tenant A: 5-year lease, 3 months free, 3% anniversary escalations,
// recoveries above a full stop at 40% pro-rata, $200k TI/LC.
contract cre.lease_unit tenant_a on entity asset.tower {
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
  parties {
    landlord = party.landlord_co
    tenant = party.acme
  }
}

// THE RENEWAL OPTION, WRITTEN ON THE LEASE. Tested once, the month after
// expiry: the tenant renews if the market rent exceeds the option rent. Both
// outcomes are leases, declared below; the exercise forecloses the re-let by
// switching its streams off, and the lapse event forecloses the renewal.
option renewal on contract cre.lease_unit.tenant_a type CRE.Contract.RenewalOption {
  parties { landlord = party.landlord_co, tenant = party.acme }
  terms {
    renewal_rent_year = inputs.renewal_rent_year
    renewal_term_months = 60
    renewal_ti_lc = 100000
  }
  schedule on 2031-01
  exercise when inputs.market_rent_year > contract.renewal_rent_year
  payoff 0
  set entity asset.tower.tenant_a_renewed = 1
  deactivate stream cre.unit.base_rent.tenant_a_market
  deactivate stream cre.unit.abatement.tenant_a_market
  deactivate stream cre.unit.recoveries.tenant_a_market
  deactivate stream cre.unit.ti_lc.tenant_a_market
}

// The lapse: the same test, the other way. When the option is not exercised
// the renewal lease never begins.
event tenant_a.lapse schedule on 2031-01 when inputs.market_rent_year <= inputs.renewal_rent_year {
  deactivate stream cre.unit.base_rent.tenant_a_renewal
  deactivate stream cre.unit.abatement.tenant_a_renewal
  deactivate stream cre.unit.recoveries.tenant_a_renewal
  deactivate stream cre.unit.ti_lc.tenant_a_renewal
}

// The renewal lease: the option rent for five years, escalating 3% on its
// anniversaries, $100k of leasing costs at commencement. Runs through the
// projection tail so the exit sees a full forward year.
contract cre.lease_unit tenant_a_renewal on entity asset.tower {
  term 2031-01..2036-12
  terms {
    rent_year = inputs.renewal_rent_year
    escalation = 0.03
    ti_total = 60000
    lc_total = 40000
  }
  parties {
    landlord = party.landlord_co
    tenant = party.acme
  }
}

// The re-let: market rent after three months of downtime, escalating 3% on
// the new lease's anniversaries, $350k of new leasing costs.
contract cre.lease_unit tenant_a_market on entity asset.tower {
  term 2031-04..2036-12
  terms {
    rent_year = inputs.market_rent_year
    escalation = 0.03
    ti_total = 250000
    lc_total = 100000
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
    growth_rate = 0.025
  }
}

// Sale at the end of the hold; NOI for the valuation year is DERIVED from
// the modeled streams over the 12 projection months after the sale date.
contract cre.exit_forward on entity asset.tower {
  term 2035-12..2035-12
  terms {
    cap_rate = 0.065
    selling_costs = 0.02
  }
}

// Permanent debt: $6m at 5.50%, 25-year amortization, 10-year hold. The
// balloon stays off: the unamortized balance is repaid out of the sale.
// `funded_at_close = 0`: the reference's cash flow starts post-financing.
contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2035-12
  terms {
    principal = 6000000
    interest_rate = 0.055
    amortization_months = 300
    funded_at_close = 0
  }
}

slice leases {
  type Contract.Lease
}

slice debt_service {
  type Contract.Debt
}

run deterministic
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.0725
  },
  "scenarios": {
    "soft_market": {
      "parameters": {
        "inputs.market_rent_year": 480000.0
      }
    }
  }
}
```

## Verified results

Checked period by period: **7 series** across **120 periods** — **840 values** in all, each within the tolerance shown.

- `net_cash_flow` — within ±0.01
- `cre.unit.base_rent.tenant_a_renewal` — within ±0.01
- `cre.unit.base_rent.tenant_a_market` — within ±0.01
- `domain.cre.egi` — within ±0.01
- `domain.cre.noi` — within ±0.01
- `domain.cre.debt_service` — within ±0.01
- `domain.cre.dscr` — within ±1.0e-6

Checked per scenario, each a full run under its own parameters:

| Scenario | `model.npv` | `model.total` |
|---|---:|---:|
| `soft_market` | 595,382.95 | 1,495,550.5 |

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 1,362,611.39 | ±1 |
| `domain.cre.noi` | 4,697,224.27 | ±1 |
| `domain.cre.leasing_costs` | 450,000 | ±1 |
| `domain.cre.debt_service` | 4,421,429.94 | ±1 |
| `domain.cre.dscr` | 1.062377 | ±0.0001 |
