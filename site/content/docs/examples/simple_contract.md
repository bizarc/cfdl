---
id: example-simple_contract
title: "A simple contract"
slug: "/docs/examples/simple_contract"
description: "This example uses the CRE pack with a single contract for a formal lease agreement (lease = formal agreement with another party)."
---

This example uses the CRE pack with a single **contract** for a formal lease agreement (lease = formal agreement with another party).

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/simple_contract --out /tmp/tutorial_simple_contract.ir.json --packs packs
```

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000}}}
version 0.1
model "tutorial-simple-contract"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 48

entity asset property : CRE.Asset.RealProperty

contract cre.lease {
  term 2026-01..2029-12
  terms {
    rent = 25000
  }
}
```
