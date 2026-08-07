---
id: example-first_stream
title: "Your First Stream"
slug: "/docs/examples/first_stream"
---

> Generated from `examples/language_tutorial/first_stream/`.

This example expands from minimal model by adding:

- two streams
- monthly schedule variants

This example uses **standalone streams** for individual revenue and expense items (subscription_revenue, support_expense).

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/first_stream --out /tmp/tutorial_first_stream.ir.json
```

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
version 0.1
model "tutorial-first-stream"
time calendar monthly from 2026-01 for 24

entity asset product : Asset.Financial

stream saas.subscription_revenue on entity asset.product inflow currency USD {
  schedule every month on day 15 from 2026-01 to 2027-12
  amount = 1200
}

stream saas.support_expense on entity asset.product outflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 250
}
```
