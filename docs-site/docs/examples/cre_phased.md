---
id: example-cre-phased
title: "cre phased"
slug: "/examples/cre_phased"
---

This example uses pack **contracts** for lease, construction stub, and exit; **standalone streams** for ops revenue and ops expense (per guidance).

Full developer lifecycle with **phases** aligning to industry stages: `construction`, `lease_up`, `perm` (stabilized). Same pack contracts as cre_developer; phases document the timeline and enable phase-relative schedules in the spec.

## Compile

```bash
./target/debug/cfdl compile examples/cre_phased --out /tmp/cre_phased.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/cre_phased.ir.json --out /tmp/cre_phased.results.json --config examples/cre_phased/run.json --packs packs
```

---

> Generated from `examples/cre_phased/`. Code is shown below so you can see structure and elements without repo access.

## model.cfdl

```cfdl
version 0.1
model "cre-phased-example"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

phase construction from 2026-01 to 2026-12
phase lease_up from 2027-01 to 2027-12
phase perm from 2028-01 to 2031-12

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
  amount cel "30000"
}

stream real_estate.ops_expense on entity real_estate.property outflow currency USD {
  schedule every monthly from 2028-01 to 2031-12
  amount cel "12000"
}

contract cre.exit_cap {
  term 2031-12..2031-12
  terms {
    exit_cap = 0.06
    noi_ref = ops.noi
  }
}
```
