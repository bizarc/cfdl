---
id: example-with_pack
title: "with pack"
slug: "/examples/with_pack"
---

> Generated from `examples/language_tutorial/with_pack/`.

This example shows a slightly larger pack-enabled model using pack **contracts** for revenue and opex. In real models, individual revenue/opex items are often modeled as **streams**; see the Language Guide "When to use streams vs contracts."

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/with_pack --out /tmp/tutorial_with_pack.ir.json --packs packs
```

## model.cfdl

```cfdl
version 0.1
model "tutorial-with-pack"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity operating business

contract opco_revenue_line {
  term 2026-01..2030-12
  terms {
    amount = 120000
    growth_rate = 0.0
  }
}

contract opco_opex_line {
  term 2026-01..2030-12
  terms {
    amount = 70000
  }
}
```
