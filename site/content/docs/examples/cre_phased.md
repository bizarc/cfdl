---
id: example-cre-phased
title: "CRE: phased development"
slug: "/docs/examples/cre_phased"
description: "This example uses pack contracts for lease, construction stub, and exit; standalone streams for ops revenue and ops expense (per guidance)."
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

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000,"stream.real_estate.ops_expense:amount":12000,"stream.cre.exit.sale:amount":3000000}}}
version 0.1
model "cre-phased-example"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

phase construction from 2026-01 to 2026-12
phase lease_up from 2027-01 to 2027-12
phase perm from 2028-01 to 2031-12

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
contract cre.revenue_line {
  term 2028-01..2031-12
  terms {
    amount = 30000
  }
}

contract cre.opex_line {
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
