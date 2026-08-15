---
id: benchmark-energy-crest-solar-cost-based
title: "Energy: cost-based solar feed-in tariff"
slug: "/docs/examples/energy-crest-solar-cost-based"
description: "A distributed solar project paid a cost-based feed-in tariff, with an abating payment in lieu of property tax and a revenue-linked royalty."
source: benchmarks/energy/crest_solar_cost_based
---

# Energy: cost-based solar feed-in tariff

A distributed solar project paid a cost-based feed-in tariff, with an abating payment in lieu of property tax and a revenue-linked royalty.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A 2 MW-dc distributed solar project paid a cost-based feed-in tariff. It
generates 3,161,597 kWh in its first year — 2,000 kW at an 18.0456% net capacity
factor over 8,760 hours — degrading 0.5% a year across a 25-year life, and is
paid a flat 23.15 c/kWh.

Five operating expense lines run against it, and they do not share an escalator:
fixed operations and maintenance, insurance and a land lease each inflate at
1.6%; a payment in lieu of property tax **abates 10% a year** on a stated
schedule; and a royalty takes 3% of tariff revenue. $3.15m of level-pay debt
runs 18 years at 7%, maturing seven years before the asset does.

## The reference

A cost-based renewable energy tariff model published by a national laboratory as
a spreadsheet, and independently ported to Python by a third party. Both were
run; the comparison is three-way.

It publishes a complete annual cash flow, so every line is checkable period by
period.

**Not redistributable.** The spreadsheet states no license and the port declares
none, which means default copyright. Neither is vendored or wired into the test
suite: the port was cloned outside the repository, run once, and only its output
numbers carried across.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.ppa`, `energy.om` (four instances), `energy.debt_service` |
| Language features | contracts with per-instance suffixes, one native stream, term units |
| Conventions | production degradation, three escalation rates including a **negative** one, level-pay amortization |

The four operating expense contracts are the same type at different escalators,
which is why they are asserted as separate lines rather than as one total.

## The result

**Exact on every individual line.** All seven stream columns agree with the
reference across all 25 periods with zero disagreement.

Asserted: seven stream columns across 25 periods, plus `domain.energy.opex`, the
reference's own published expense total. The reference publishes operating
expenses as a single figure, so the four decomposed lines have to sum back to it
in every period.

## The delta

One non-zero figure: **5.0e-7**, on the summed expense column at period 19.

It is not arithmetic. Results carry money to six decimal places, and the engine
rounds a subtotal it computed from *unrounded* components — which is a different
operation from summing five *already-rounded* components, and the two differ by
up to half of the last published place. 5e-7 is exactly that half, and the
floor any case here can assert to.

One thing the case does **not** validate: the reference's actual purpose is to
solve the tariff that clears a target equity return, sweeping the rate until net
present value crosses zero. CFDL has no solve-to-target construct, so the solved
rate — 23.15 c/kWh — is carried across as a constant. Everything downstream of
the tariff is checked period by period; the solve itself is not.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.12}}
// A 2 MW-dc distributed solar project under a cost-based feed-in tariff,
// reconciled against a national laboratory's cost-based-incentive model.
//
// WHY THIS CASE IS DIFFERENT FROM utility_pv_singleowner. That case checks CFDL
// against one external model. This one checks it against a model that exists
// TWICE — the laboratory publishes it as a spreadsheet, and an independent
// contributor has ported it to Python. Agreement is therefore three-way, and a
// three-way agreement rules out something a two-way one cannot: that CFDL and
// the reference share a mistake. See NOTES.md for what the port is and how it
// was run.
//
// THE TARIFF IS AN INPUT HERE, NOT AN OUTPUT. The reference model's PURPOSE is
// to solve the tariff that clears a target equity return — it sweeps the rate
// until net present value crosses zero. CFDL has no solve-to-target construct,
// so the solved rate is carried across as a constant and the CASH FLOW at that
// rate is what gets asserted. Everything downstream of the tariff is validated
// period by period; the solve itself is not. NOTES.md, "What this case does not
// validate".
//
// Period 0 is operating year 1. There is no construction period: the reference
// treats installed cost as a year-zero equity outlay outside the operating
// cash flow, so adding a period 0 for it would shift every degradation and
// escalation exponent by one and validate nothing.

version 0.1
model "crest-solar-cost-based"
use pack "energy" version "0.1.0"
time calendar annual from 2026-01 for 25

entity asset plant : Energy.Asset.GenerationFacility

// 2,000 kW-dc at an 18.0456% net capacity factor over 8,760 hours gives
// 3,161,597.15 kWh in year one — 3,161.59715 MWh. The capacity factor is the
// reference's own published figure for the chosen state, carried to full
// precision because it multiplies every period.
//
// The tariff is 23.15 c/kWh = $231.50/MWh, flat: this is a cost-based tariff
// with a 0% escalator, so the year-one rate IS the nominal levelized rate.
// 0.5%/yr module degradation.
contract energy.ppa on entity asset.plant {
  term 2026-01..2050-01
  terms {
    mwh_year    = 3161.59715 "MWh/yr"
    ppa_price   = 231.50 "USD/MWh"
    degradation = 0.005
    escalation  = 0
  }
}

// ---------------------------------------------------------------------------
// Operating expenses. Four lines, and they do NOT share an escalator — which
// is the point of asserting them separately rather than as one total. Three
// inflate at 1.6%; the payment in lieu of taxes DECLINES 10% a year on a
// stated abatement schedule. A single blended escalator would reproduce the
// year-one total and drift from every year after it.
// ---------------------------------------------------------------------------

// Fixed O&M: $6.50/kW-dc-yr on 2,000 kW.
contract energy.om.fixed on entity asset.plant {
  term 2026-01..2050-01
  terms {
    om_year    = 13000
    escalation = 0.016
  }
}

// Insurance: 0.4% of the $7,000,000 of hard cost.
contract energy.om.insurance on entity asset.plant {
  term 2026-01..2050-01
  terms {
    om_year    = 28000
    escalation = 0.016
  }
}

// Land lease.
contract energy.om.land_lease on entity asset.plant {
  term 2026-01..2050-01
  terms {
    om_year    = 5000
    escalation = 0.016
  }
}

// Payment in lieu of property tax, abating 10% a year. A NEGATIVE escalation
// term — the pack's `pow(1 + escalation, t)` carries it with no special case,
// which is worth pinning: an escalator implemented as a growth-only ratchet
// would silently hold this flat.
contract energy.om.pilot on entity asset.plant {
  term 2026-01..2050-01
  terms {
    om_year    = 50000
    escalation = -0.1
  }
}

// ---------------------------------------------------------------------------
// Royalty: 3% of tariff revenue.
//
// HAND-WRITTEN, because the energy pack cannot express it. Every pack expense
// rule takes a fixed annual amount and escalates it; none takes a percentage
// of another stream. So the royalty is restated from its drivers here — the
// same production and price the PPA contract uses, times 3%. That is a real
// pack gap, recorded in NOTES.md rather than worked around silently: the
// duplication below is the evidence for it.
//
// The category is `operating.expense.om` because it is the ONLY operating
// expense category the energy pack defines. A royalty is not operations and
// maintenance. It does not change any number here — `domain.energy.opex` globs
// `operating.expense.*` — but it does mean the pack cannot tell these lines
// apart in a statement. Second finding, also in NOTES.md.
// ---------------------------------------------------------------------------
stream energy.royalty.expense on entity asset.plant outflow currency USD {
  schedule every year from 2026-01 to 2050-01
  category operating.expense.om
  amount = 3161.59715 * 231.50 * pow(1 - 0.005, time.t) * 0.03
}

// 45% of the $7,000,000 of hard cost, 18 years at 7%, level annual payments.
// Ends in period 17 (2043), leaving seven unlevered years — the cliff is
// asserted at periods 17 and 18.
contract energy.debt_service on entity asset.plant {
  term 2026-01..2043-01
  terms {
    rate        = 0.07
    term_months = 216
    principal   = 3150000
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

Checked period by period: **8 series** across **25 periods** — **193 values** in all, each within ±1e-6 of the reference.

- `energy.ppa.revenue`
- `energy.om.expense.fixed`
- `energy.om.expense.insurance`
- `energy.om.expense.land_lease`
- `energy.om.expense.pilot`
- `energy.royalty.expense`
- `domain.energy.opex`
- `energy.debt.service`

