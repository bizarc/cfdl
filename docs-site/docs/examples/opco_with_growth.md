---
id: example-opco-with-growth
title: "opco with growth"
slug: "/examples/opco_with_growth"
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

> Generated from `examples/opco_with_growth/`. Code is shown below so you can see structure and elements without repo access.

## model.cfdl

```cfdl
version 0.1
model "opco-with-growth-example"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity operating business

// Revenue and opex as standalone streams (individual items per guidance). Growth via CEL.
stream revenue on entity operating.business inflow currency USD {
  schedule every monthly from 2026-01 to 2031-12
  amount cel "120000 * pow(1.03, time.t - 1)"
}

stream opex on entity operating.business outflow currency USD {
  schedule every monthly from 2026-01 to 2031-12
  amount cel "70000"
}

contract opco_exit_multiple {
  term 2031-12..2031-12
  terms {
    exit_period = 72
    exit_multiple = 6.5
    base_value = 800000
  }
}
```
