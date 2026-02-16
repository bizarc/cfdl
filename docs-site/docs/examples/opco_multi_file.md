---
id: example-opco-multi-file
title: "opco multi file"
slug: "/examples/opco_multi_file"
---

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
contract opco_revenue_line {
  term 2026-01..2031-12
  terms {
    amount = 120000
    growth_rate = 0.0
  }
}

contract opco_opex_line {
  term 2026-01..2031-12
  terms {
    amount = 70000
  }
}

contract opco_working_capital {
  term 2026-01..2031-12
  terms {
    amount = 3000
  }
}

contract opco_exit_multiple {
  term 2031-12..2031-12
  terms {
    exit_period = 72
    exit_multiple = 6.5
    base_value = 800000
  }
}
```
