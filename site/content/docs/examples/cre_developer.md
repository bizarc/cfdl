---
id: example-cre-developer
title: "CRE: developer lifecycle"
slug: "/docs/examples/cre_developer"
description: "This example uses the cre pack (0.1.0) for the formal lease and construction stub and exit; standalone streams for ops revenue and ops expense (per guidance: individual revenue/expense items → stream)."
---

This example uses the `cre` pack (`0.1.0`) for the formal lease and construction stub and exit; **standalone streams** for ops revenue and ops expense (per guidance: individual revenue/expense items → stream).

## Run

Compile:

`./target/debug/cfdl compile examples/cre_developer --out /tmp/cre.ir.json --packs packs`

Run base case:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.base.results.json --config examples/cre_developer/run.base.json --packs packs`

Run stress case:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.stress.results.json --config examples/cre_developer/run.stress.json --packs packs`

## Scenario knobs

The provided run configurations demonstrate deterministic override testing with:

- `stream.cre.lease.base_rent.amount`
- `stream.ops_expense.amount`
- `stream.cre.exit.sale.amount`

---

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000,"stream.real_estate.ops_expense:amount":12000,"stream.cre.exit.sale:amount":3000000}}}
version 0.1
model "cre-developer-example"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset property : CRE.Asset.RealProperty

contract cre.construction_stub {
  term 2026-01..2026-06
  terms {
    amount = 45000
  }
}

contract cre.lease {
  term 2026-07..2031-12
  terms {
    base_rent = 25000
  }
}

// Ops income and expense as pack contracts, so the pack classifies them and
// the operating statement can claim them. Hand-written streams carry no
// category, and a statement folds categories rather than stream names.
contract cre.ops_revenue {
  term 2028-01..2031-12
  terms {
    amount = 30000
  }
}

contract cre.ops_expense {
  term 2028-01..2031-12
  terms {
    amount = 12000
  }
}

contract cre.exit_cap {
  term 2031-12..2031-12
  terms {
    exit_cap = 0.06
    noi_value = 180000
  }
}
```
