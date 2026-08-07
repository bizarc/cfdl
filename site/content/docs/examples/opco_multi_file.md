---
id: example-opco-multi-file
title: "OpCo: multi-file model"
slug: "/docs/examples/opco_multi_file"
---

This example uses **standalone streams** in `contracts.cfdl` for revenue and opex (per guidance); pack **contracts** for working capital and exit.

Full OpCo valuation (revenue, opex, working capital, exit multiple) split across files: `structure.cfdl` (entities), `contracts.cfdl` (pack contracts). Entry is `model.cfdl` with version, model, use pack, time, and imports.

## Compile

```bash
./target/debug/cfdl compile examples/opco_multi_file --out /tmp/opco_multi_file.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/opco_multi_file.ir.json --out /tmp/opco_multi_file.results.json --config examples/opco_multi_file/run.json --packs packs
```

---

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
version 0.1
model "opco-multi-file-example"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 72

import "structure.cfdl"
import "contracts.cfdl"
```

## structure.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
entity asset business : OpCo.Asset.Enterprise
```

## contracts.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
// Revenue and opex as standalone streams (individual items per guidance)
stream operating.revenue on entity asset.business inflow currency USD {
  schedule every month from 2026-01 to 2031-12
  amount = 120000
}

stream operating.opex on entity asset.business outflow currency USD {
  schedule every month from 2026-01 to 2031-12
  amount = 70000
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
