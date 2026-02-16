---
id: example-opco-basic
title: "opco basic"
slug: "/examples/opco_basic"
---

Compile:

`./target/debug/cfdl compile examples/opco_basic --out /tmp/opco.ir.json --packs packs`

Run base:

`./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.base.results.json --config examples/opco_basic/run.base.json --packs packs`

Run stress:

`./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.stress.results.json --config examples/opco_basic/run.stress.json --packs packs`

---

> Generated from `examples/opco_basic/`. Code is shown below so you can see structure and elements without repo access.

## model.cfdl

```cfdl
version 0.1
model "opco-basic-example"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity operating business

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
