---
id: example-opco-with-growth
title: "OpCo: growth via expressions"
slug: "/docs/examples/opco_with_growth"
---

This example uses **standalone streams** for revenue and opex (per guidance); pack **contract** for exit.

Revenue line with **growth_rate** > 0 (e.g. 3%). Demonstrates the industry lever for recurring revenue growth in DCF.

## Compile

```bash
./target/debug/cfdl compile examples/opco_with_growth --out /tmp/opco_growth.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/opco_growth.ir.json --out /tmp/opco_growth.results.json --config examples/opco_with_growth/run.json --packs packs
```

---

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
version 0.1
model "opco-with-growth-example"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset business : OpCo.Asset.Enterprise

// Growth stated as an expression rather than an opco.revenue_line contract,
// because the pack's growth term is an ANNUAL rate converted to the model's
// grain, and this line compounds per period. A hand-written stream carries no
// category, so each states its own — a statement folds categories, and cash in
// no category is cash no row can claim.
stream operating.revenue on entity asset.business inflow currency USD {
  schedule every month from 2026-01 to 2031-12
  category operating.revenue.recurring
  amount = 120000 * pow(1.03, time.t - 1)
}

stream operating.opex on entity asset.business outflow currency USD {
  schedule every month from 2026-01 to 2031-12
  category operating.expense.opex
  amount = 70000
}

contract opco.exit_multiple {
  term 2031-12..2031-12
  terms {
    exit_period = 72
    exit_multiple = 6.5
    base_value = 800000
  }
}
```
