---
id: example-cre-developer
title: "CRE: Developer Lifecycle"
slug: "/docs/examples/cre_developer"
---

This example uses the `cre` pack (`0.1.0`) for the formal lease and construction stub and exit; **standalone streams** for ops revenue and ops expense (per guidance: individual revenue/expense items → stream). It mirrors `fixtures/valid/cre_developer_smoke`.

## Run

Compile:

`./target/debug/cfdl compile examples/cre_developer --out /tmp/cre.ir.json --packs packs`

Run base case:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.base.results.json --config examples/cre_developer/run.base.json --packs packs`

Run stress case:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.stress.results.json --config examples/cre_developer/run.stress.json --packs packs`

## Scenario knobs

The provided run configs demonstrate deterministic override testing with:

- `stream.cre.lease.base_rent.amount`
- `stream.ops_expense.amount`
- `stream.cre.exit.sale.amount`

---

> Generated from `examples/cre_developer/`. Code is shown below so you can see structure and elements without repo access.

## model.cfdl

```cfdl
version 0.1
model "cre-developer-example"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity real_estate property

contract cre.construction_stub {
  term 2026-01..2026-06
}

contract cre.lease {
  term 2026-07..2031-12
  terms {
    base_rent = 25000
  }
}

// Ops as standalone streams (individual revenue/expense items per guidance)
stream real_estate.ops_revenue on entity real_estate.property inflow currency USD {
  schedule every monthly from 2028-01 to 2031-12
  amount = 30000
}

stream real_estate.ops_expense on entity real_estate.property outflow currency USD {
  schedule every monthly from 2028-01 to 2031-12
  amount = 12000
}

contract cre.exit_cap {
  term 2031-12..2031-12
  terms {
    exit_cap = 0.06
    noi_ref = ops.noi
  }
}
```
