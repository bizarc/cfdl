---
id: example-minimal_model
title: "minimal model"
slug: "/examples/minimal_model"
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

```cfdl
version 0.1
model "tutorial-minimal-model"
time calendar monthly from 2026-01 for 12

entity legal borrower

stream rent on entity legal.borrower {
  schedule every monthly from 2026-01 to 2026-12
  amount cel "1000"
}
```
