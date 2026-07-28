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

```cfdl
version 0.1
model "tutorial-first-stream"
time calendar monthly from 2026-01 for 24

entity legal tenant

stream legal.subscription_revenue on entity legal.tenant inflow currency USD {
  schedule every monthly on day 15 from 2026-01 to 2027-12
  amount = 1200
}

stream legal.support_expense on entity legal.tenant outflow currency USD {
  schedule every monthly from 2026-01 to 2027-12
  amount = 250
}
```
