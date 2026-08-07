---
id: example-minimal_model
title: "Minimal Model"
slug: "/docs/examples/minimal_model"
---

> Generated from `examples/language_tutorial/minimal_model/`.

This is the smallest practical CFDL model:

- required header statements
- one entity
- one stream

This example uses a **standalone stream** for rent (guidance: if in doubt, start with a stream).

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/minimal_model --out /tmp/tutorial_minimal.ir.json
```

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
version 0.1
model "tutorial-minimal-model"
time calendar monthly from 2026-01 for 12

entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 1000
}
```
