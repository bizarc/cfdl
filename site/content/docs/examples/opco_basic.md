---
id: example-opco-basic
title: "OpCo: basic operating model"
slug: "/docs/examples/opco_basic"
description: "This example uses standalone streams for revenue and opex (per guidance); pack contracts for working capital and exit multiple."
---

This example uses **standalone streams** for revenue and opex (per guidance); pack **contracts** for working capital and exit multiple. See [When to use streams vs contracts](/docs/language-guide).

Compile:

`./target/debug/cfdl compile examples/opco_basic --out /tmp/opco.ir.json --packs packs`

Run base:

`./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.base.results.json --config examples/opco_basic/run.base.json --packs packs`

Run stress:

`./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.stress.results.json --config examples/opco_basic/run.stress.json --packs packs`

---

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
version 0.1
model "opco-basic-example"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset business : OpCo.Asset.Enterprise

// Revenue and opex as pack contracts, so the pack classifies them and the
// statements can claim them. Hand-written streams carry no category, and a
// statement folds categories rather than stream names.
contract opco.revenue_line {
  term 2026-01..2031-12
  terms {
    amount = 120000
  }
}

contract opco.opex_line {
  term 2026-01..2031-12
  terms {
    amount = 70000
  }
}

contract opco.working_capital {
  term 2026-01..2031-12
  terms {
    amount = 3000
  }
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
