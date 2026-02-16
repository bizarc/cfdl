---
id: example-cre-development-with-financing
title: "cre development with financing"
slug: "/examples/cre_development_with_financing"
---

This example models a development lifecycle with **construction-phase financing** (interest-only) and **permanent financing** (debt service). The transition is **hardcoded by stream dates**: no events are used.

- **Construction interest:** `loan.construction` — stream runs from 2026-01 to 2027-06 (18 months).
- **Permanent debt service:** `loan.permanent` — stream runs from 2027-07 to 2031-12 (after conversion).

Property-side cash flows use the CRE pack (construction stub, lease, ops, exit). Loan-side streams are core streams with fixed schedule ranges.

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
contract cre_construction_stub {
  term 2026-01..2026-06
}

contract cre_lease {
  term 2026-07..2031-12
  terms {
    base_rent = 25000
  }
}

contract cre_ops_revenue {
  term 2028-01..2031-12
  terms {
    amount = 30000
  }
}

contract cre_ops_expense {
  term 2028-01..2031-12
  terms {
    amount = 12000
  }
}

contract cre_exit_cap {
  term 2031-12..2031-12
  terms {
    exit_cap = 0.06
    noi_ref = ops.noi
  }
}

// Construction-phase financing: interest-only stream (fixed dates, no events)
stream construction_interest on entity loan.construction {
  schedule every monthly from 2026-01 to 2027-06
  amount cel "40000"
}

// Permanent financing: debt service stream starts when construction period ends (hardcoded transition)
stream permanent_debt_service on entity loan.permanent {
  schedule every monthly from 2027-07 to 2031-12
  amount cel "55000"
}
```
