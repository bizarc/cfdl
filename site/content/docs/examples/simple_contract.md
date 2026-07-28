---
id: example-simple_contract
title: "A Simple Contract"
slug: "/docs/examples/simple_contract"
---

> Generated from `examples/language_tutorial/simple_contract/`.

This example uses the CRE pack with a single **contract** for a formal lease agreement (lease = formal agreement with another party).

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/simple_contract --out /tmp/tutorial_simple_contract.ir.json --packs packs
```

## model.cfdl

```cfdl
version 0.1
model "tutorial-simple-contract"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 48

entity real_estate property

contract cre.lease {
  term 2026-01..2029-12
  terms {
    base_rent = 25000
  }
}
```
