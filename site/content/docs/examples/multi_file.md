---
id: example-multi_file
title: "Multi-file model"
slug: "/docs/examples/multi_file"
description: "This example demonstrates splitting model content by concern"
---

This example demonstrates splitting model content by concern:

- `model.cfdl` for header + imports
- `structure.cfdl` for entities
- `contracts.cfdl` for streams/contracts/events

This example uses a **standalone stream** for rent (same pattern as minimal_model).

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/multi_file --out /tmp/tutorial_multi_file.ir.json
```

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
version 0.1
model "tutorial-multi-file"
time calendar monthly from 2026-01 for 24

import "structure.cfdl"
import "contracts.cfdl"
```
