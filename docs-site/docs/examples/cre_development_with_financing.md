---
id: example-cre-development-with-financing
title: "cre development with financing"
slug: "/examples/cre_development_with_financing"
---

This example models a development lifecycle with **construction-phase financing** (interest-only) and **permanent financing** (debt service). The transition is **hardcoded by stream dates**: no events are used.

- **Construction interest:** `loan.construction` — stream runs from 2026-01 to 2027-06 (18 months).
- **Permanent debt service:** `loan.permanent` — stream runs from 2027-07 to 2031-12 (after conversion).

Property-side: CRE pack for construction stub, lease, and exit; **standalone streams** for ops revenue and ops expense (per guidance). Loan-side: **standalone streams** for construction interest and permanent debt service.

## Compile

```bash
./target/debug/cfdl compile examples/cre_development_with_financing --out /tmp/cre_fin.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/cre_fin.ir.json --out /tmp/cre_fin.results.json --config examples/cre_development_with_financing/run.json --packs packs
```

---

> Generated from `examples/cre_development_with_financing/`. Code is shown below so you can see structure and elements without repo access.

## model.cfdl

```cfdl
version 0.1
model "cre-development-with-financing"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity real_estate property
entity loan construction
entity loan permanent

// CRE pack: property-level lifecycle (construction stub, lease, ops, exit)
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

// Construction-phase financing: interest-only stream (fixed dates, no events)
stream loan.construction_interest on entity loan.construction {
  schedule every monthly from 2026-01 to 2027-06
  amount = 40000
}

// Permanent financing: debt service stream starts when construction period ends (hardcoded transition)
stream loan.permanent_debt_service on entity loan.permanent {
  schedule every monthly from 2027-07 to 2031-12
  amount = 55000
}
```
