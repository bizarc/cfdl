---
id: example-cre-multi-file
title: "CRE: multi-file model"
slug: "/docs/examples/cre_multi_file"
description: "This example uses pack contracts in contracts.cfdl for lease, construction stub, and exit; standalone streams for ops revenue and ops expense (per guidance)."
---

This example uses pack **contracts** in `contracts.cfdl` for lease, construction stub, and exit; **standalone streams** for ops revenue and ops expense (per guidance).

Full developer lifecycle split across files: `time.cfdl` (phases), `structure.cfdl` (entities), `contracts.cfdl` (CRE pack contracts). Entry is `model.cfdl` with version, model, use pack, time, and imports.

## Compile

```bash
./target/debug/cfdl compile examples/cre_multi_file --out /tmp/cre_multi_file.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/cre_multi_file.ir.json --out /tmp/cre_multi_file.results.json --config examples/cre_multi_file/run.json --packs packs
```

---

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000,"stream.real_estate.ops_expense:amount":12000,"stream.cre.exit.sale:amount":3000000}}}
version 0.1
model "cre-multi-file-example"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

import "time.cfdl"
import "structure.cfdl"
import "contracts.cfdl"
```

## time.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000,"stream.real_estate.ops_expense:amount":12000,"stream.cre.exit.sale:amount":3000000}}}
phase construction from 2026-01 to 2026-12
phase lease_up from 2027-01 to 2027-12
phase perm from 2028-01 to 2031-12
```

## structure.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000,"stream.real_estate.ops_expense:amount":12000,"stream.cre.exit.sale:amount":3000000}}}
entity asset property : CRE.Asset.RealProperty
```

## contracts.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000,"stream.real_estate.ops_expense:amount":12000,"stream.cre.exit.sale:amount":3000000}}}
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
