---
id: example-cre-multi-file
title: "cre multi file"
slug: "/examples/cre_multi_file"
---

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

> Generated from `examples/cre_multi_file/`. Code is shown below so you can see structure and elements without repo access.

## model.cfdl

```cfdl
version 0.1
model "cre-multi-file-example"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

import "time.cfdl"
import "structure.cfdl"
import "contracts.cfdl"
```

## time.cfdl

```cfdl
phase construction from 2026-01 to 2026-12
phase lease_up from 2027-01 to 2027-12
phase perm from 2028-01 to 2031-12
```

## structure.cfdl

```cfdl
entity real_estate property
```

## contracts.cfdl

```cfdl
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
```
