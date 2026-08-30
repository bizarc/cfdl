---
id: benchmark-cre-basic-acquisition-exit-cap
title: "CRE: a stabilized acquisition and a terminal cap rate"
slug: "/docs/examples/cre-basic-acquisition-exit-cap"
description: "A stabilized commercial property bought on a going-in cap rate, held ten years with fully reimbursed expenses, and sold by capping the final year's net operating income."
source: benchmarks/cre/basic_acquisition_exit_cap
---

# CRE: a stabilized acquisition and a terminal cap rate

A stabilized commercial property bought on a going-in cap rate, held ten years with fully reimbursed expenses, and sold by capping the final year's net operating income.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A stabilized 10,000 square foot commercial building, bought for $1,417,958 —
year-one net operating income capped at 6.0% — held for ten years and sold by
capping the tenth year's net operating income at 6.5%, net of 2% selling costs.

The building is fully leased for the whole hold with no rollover and no
lease-up, so its income is a single stabilized stream rather than a rent roll:
potential gross income of $100,000, full reimbursement of the three recoverable
expenses, and a 10% vacancy and credit deduction taken against the two
together. The leases reimburse 100% of real estate taxes, insurance and common
area maintenance; management is 3% of effective gross revenue; a capital
reserve runs at $0.20 per square foot. Every line escalates at 3% a year.

The interest of it is the shape at the two ends. The purchase settles at the
open of the first year and earns no discount; the sale settles at the close of
the tenth, alongside that year's operating cash, and is struck on the net
operating income of the year that has just finished rather than a forward year.

## The reference

A published teaching model for a basic real estate acquisition, distributed as
a spreadsheet. It states every driver — year-one income, each expense, the
management fee, the reserve, the escalator, both cap rates, the selling cost —
and publishes the resulting cash flow line by line together with its own
internal rate of return, equity multiple and net present value.

**Not redistributable.** No terms are stated by the publisher, so the figures
are asserted against and the workbook is not vendored. `NOTES.md` records what
it is and where it came from.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.revenue_line`, `cre.vacancy_loss`, `cre.opex_line`, `cre.exit_cap` |
| Declared | 8 contracts, 3 native streams, one entity, no curves |
| Language features | a flow settled at a period's open against flows settled at its close; a contract term carrying an expression rather than a scalar |
| Conventions | annual escalation compounded from the model's start, a vacancy deduction on income plus recoveries, an exit struck on trailing rather than forward income |

The acquisition price, the capital reserve and the selling costs are declared
as streams; everything else is a pack contract.

The vacancy deduction is taken on potential gross income *plus* reimbursements,
so its base is stated as an expression that escalates with the two lines it
sits on rather than as a fixed figure.

## The result

`model.npv` at 7% = **90,853.72729** against the published 90,853.72728969366,
`model.irr` = **0.078484** against the published 0.07848381094537493, and the
equity multiple **1.851981** against the published 1.8519810968410686. All
three exact.

Every line reproduces per period: potential gross income, reimbursements, the
vacancy deduction, all four operating expenses, the capital reserve, the
purchase, the sale at 1,707,797.546881 and its selling costs.

## The delta

None. Every asserted figure is the source's own and reproduces exactly.

The equity multiple is measured on the capital invested — the acquisition —
with the ten years of operations counted as what the investment returned.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.07}}
// A stabilized 10,000 SF commercial building, bought at a 6.0% cap on year-one
// NOI, held ten years and sold by capping the tenth year's NOI at 6.5%.
//
// The building is fully leased for the whole hold with no rollover, so the
// income is a single stabilized stream rather than a rent roll: potential gross
// income, full reimbursement of the three recoverable expenses, and a vacancy
// and credit deduction taken against the total. Every line — income, expenses
// and the capital reserve — escalates at 3% a year, so each is its stated
// year-one amount compounded from the model's start.
//
// TIMING. The purchase settles at the OPEN of the first period and every
// operating flow at its close, which is what puts the acquisition at year 0 and
// the first year's cash at year 1. `schedule on <date>` settles at the period's
// open; a one-period `every` is an ordinary annuity and falls at its close.
//
// The sale is taken at the close of the tenth year, alongside that year's
// operating cash, and is struck on the tenth year's net operating income —
// the year that has just finished, not a forward year.

version 0.1
model "basic-acquisition-exit-cap"
use pack "cre" version "0.1.0"
time calendar annual from 2016-02 for 10

entity asset property : CRE.Asset.RealProperty

// ------------------------------------------------------------------ purchase
// Year-one NOI capped at 6.0%. Paid at the open of period 0, so it discounts
// nothing — the basis every return below is measured against.
stream cre.acquisition.purchase on entity asset.property outflow currency USD {
  schedule on 2016-02
  category investing.acquisition.purchase
  amount = 1417958.3333333335
}

// -------------------------------------------------------------------- income
contract cre.revenue_line.potential_gross on entity asset.property {
  term 2016-02..2025-02
  terms {
    amount_year = 100000.0
    escalation  = 0.03
  }
}

// The leases reimburse 100% of real estate taxes, insurance and CAM. The three
// recover at the same 3% their expenses grow at, so the recovery is stated as
// its own escalating line rather than derived from them.
contract cre.revenue_line.reimbursements on entity asset.property {
  term 2016-02..2025-02
  terms {
    amount_year = 17500.0
    escalation  = 0.03
  }
}

// Vacancy and credit loss at 10% of potential gross income PLUS reimbursements,
// so the base is the 117,500 the two lines make in year one, escalating with
// them.
contract cre.vacancy_loss.stabilized on entity asset.property {
  term 2016-02..2025-02
  terms {
    rate                 = 0.10
    potential_gross_year = 117500.0 * pow(1.03, time.t / time.ppy)
  }
}

// ------------------------------------------------------------------ expenses
contract cre.opex_line.real_estate_taxes on entity asset.property {
  term 2016-02..2025-02
  terms {
    amount_year = 10000.0
    escalation  = 0.03
  }
}

contract cre.opex_line.insurance on entity asset.property {
  term 2016-02..2025-02
  terms {
    amount_year = 2500.0
    escalation  = 0.03
  }
}

contract cre.opex_line.cam on entity asset.property {
  term 2016-02..2025-02
  terms {
    amount_year = 5000.0
    escalation  = 0.03
  }
}

// The management fee is 3% of effective gross revenue. Effective gross revenue
// grows at exactly 3%, so the fee does too and is stated as its year-one amount
// on the same escalator.
contract cre.opex_line.management_fee on entity asset.property {
  term 2016-02..2025-02
  terms {
    amount_year = 3172.5
    escalation  = 0.03
  }
}

// ------------------------------------------------------------- capital outlay
// A reserve at $0.20 per square foot on 10,000 SF, escalating with everything
// else. It sits below net operating income and above the return, so it is a
// capital line and not an operating expense.
stream cre.capital.reserve on entity asset.property outflow currency USD {
  schedule every year from 2016-02 to 2025-02
  category investing.capital.capex
  amount = 2000.0 * pow(1.03, time.t / time.ppy)
}

// ---------------------------------------------------------------------- exit
// The tenth year's net operating income capped at 6.5%, taken at the close of
// that year.
contract cre.exit_cap.reversion on entity asset.property {
  term 2025-02..2025-02
  terms {
    noi_value = 111006.84054723257
    exit_cap  = 0.065
  }
}

// Selling costs at 2% of the gross sale price, on the same date.
stream cre.exit.selling_costs on entity asset.property outflow currency USD {
  schedule every year from 2025-02 to 2025-02
  category investing.disposal.selling_costs
  amount = 111006.84054723257 / 0.065 * 0.02
}

// ------------------------------------------------------------------- returns
// The equity multiple, on the valuation plane: what came back over what went
// in. Invested capital is the acquisition and nothing else — the operating
// years are what the investment RETURNED, not a reduction in what it cost.
//
// Stated here rather than read from `model.moic`, which is a ratio of the
// model's net-positive periods to its net-negative ones. Those are the same
// number only when no period holds both, and period 0 here holds the purchase
// at its open and the first year's cash at its close.
metric invested        = 0.0 - series_sum("cre.acquisition.purchase", 0, 9)
metric returned        = model.total + metric.invested
metric equity_multiple = metric.returned / metric.invested
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.07
  }
}
```

## Verified results

Checked period by period: **11 series** across **10 periods** — **110 values** in all, each within ±0.001 of the reference.

- `cre.acquisition.purchase`
- `cre.revenue.line.potential_gross`
- `cre.revenue.line.reimbursements`
- `cre.vacancy.loss`
- `cre.opex.line.real_estate_taxes`
- `cre.opex.line.insurance`
- `cre.opex.line.cam`
- `cre.opex.line.management_fee`
- `cre.capital.reserve`
- `cre.exit.sale`
- `cre.exit.selling_costs`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 90,853.73 | ±0.00001 |
| `model.irr` | 0.07848381094537493 | ±0.000001 |
| `metric.invested` | 1,417,958.33 | ±0.00001 |
| `metric.returned` | 2,626,032.03 | ±0.0001 |
| `metric.equity_multiple` | 1.8519810968410686 | ±0.000001 |
