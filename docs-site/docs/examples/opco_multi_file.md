---
id: example-opco-multi-file
title: "OpCo: Multi-File Model"
slug: "/examples/opco_multi_file"
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

> Generated from `examples/opco_multi_file/`. Code is shown below so you can see structure and elements without repo access.

## model.cfdl

```cfdl
version 0.1
model "opco-multi-file-example"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 72

import "structure.cfdl"
import "contracts.cfdl"
```

## structure.cfdl

```cfdl
entity operating business
```

## contracts.cfdl

```cfdl
// Revenue and opex as standalone streams (individual items per guidance)
stream operating.revenue on entity operating.business inflow currency USD {
  schedule every monthly from 2026-01 to 2031-12
  amount = 120000
}

stream operating.opex on entity operating.business outflow currency USD {
  schedule every monthly from 2026-01 to 2031-12
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
