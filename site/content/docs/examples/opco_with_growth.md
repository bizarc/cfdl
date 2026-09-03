---
id: example-opco-with-growth
title: "OpCo: growth via expressions"
slug: "/docs/examples/opco_with_growth"
description: "This example uses pack contracts throughout: opco.revenue_line and opco.opex_line for operations, opco.exit_multiple for the exit."
---

This example uses pack **contracts** throughout: `opco.revenue_line` and `opco.opex_line` for operations, `opco.exit_multiple` for the exit.

Revenue line with **growth_rate** > 0 (3% here). Demonstrates the industry lever for recurring revenue growth in DCF.

`growth_rate` is an **annual** rate. The pack converts it to the model's grain geometrically, so revenue compounds to 3% a year on a monthly calendar rather than 3% a month — the same conversion a discount rate gets.

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

// The deal's numbers, stated once. Growth is an ANNUAL rate: the pack
// converts it to the model's grain geometrically, so 3% a year is 3% a year
// on any calendar.
assume base_revenue = 120000
assume base_opex = 70000
assume exit_base_value = 800000
assume exit_multiple = 6.5

// The growth PLAN, as the operator states it: 5% while the ramp lasts, 3%
// mature. A step curve holds the last point at or before the date.
curve growth_plan step {
  2026-01: 0.05
  2029-01: 0.03
}

contract opco.revenue_line {
  term 2026-01..2031-12
  terms {
    amount = inputs.base_revenue
    growth_rate = curve_value("growth_plan", time.date)
  }
}

contract opco.opex_line {
  term 2026-01..2031-12
  terms {
    amount = inputs.base_opex
  }
}

contract opco.exit_multiple {
  term 2031-12..2031-12
  terms {
    exit_period = 72
    multiple = inputs.exit_multiple
    base = inputs.exit_base_value
  }
}
```
