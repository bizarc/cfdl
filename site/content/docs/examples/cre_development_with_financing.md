---
id: example-cre-development-with-financing
title: "CRE: development with financing"
slug: "/docs/examples/cre_development_with_financing"
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

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000,"stream.real_estate.ops_expense:amount":12000,"stream.cre.exit.sale:amount":3000000,"stream.loan.construction_interest:amount":40000,"stream.loan.permanent_debt_service:amount":55000}}}
version 0.1
model "cre-development-with-financing"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset property : CRE.Asset.RealProperty
entity asset construction : Asset.Financial
entity asset permanent : Asset.Financial

// CRE pack: property-level lifecycle (construction stub, lease, ops, exit)
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

// Construction-phase financing: interest-only stream (fixed dates, no events).
// The CRE pack has no construction-facility contract, so this stays a stream
// and states its own category — without one, no statement row can claim it.
stream loan.construction_interest on entity asset.construction outflow currency USD {
  schedule every month from 2026-01 to 2027-06
  category financing.debt_service
  amount = 40000
}

// Permanent financing: a flat placeholder service, so it stays a stream —
// cre.permanent_debt would strike the payment from principal, rate and
// amortisation and change this example's numbers.
stream loan.permanent_debt_service on entity asset.permanent outflow currency USD {
  schedule every month from 2027-07 to 2031-12
  category financing.debt_service
  amount = 55000
}
```
