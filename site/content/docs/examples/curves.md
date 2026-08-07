---
id: example-curves
title: "Curves"
slug: "/docs/examples/curves"
---

A **curve** is a dated series — a forward price, an index, a rate path —
declared once and read by date.

This model declares a three-point power price curve and drives revenue from it,
so the price path lives in one place instead of being restated in every amount.

`linear` interpolates between the stated points; `step` holds the last point
forward. Read a curve with `curve_value(name, date)`.

Note that `obs.*` is a different thing: observations supplied at run time rather
than declared in the model.

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "tutorial-curves"
time calendar monthly from 2026-01 for 36

entity asset solar : Asset.Real

// A curve is a dated series: a forward price, an index, a rate.
// `linear` says how to find a value between two stated points.
curve power_price linear {
  2026-01: 42.10
  2027-01: 44.75
  2028-01: 46.20
}

// Read it with curve_value(name, date). The value tracks the curve period
// by period, so revenue follows the price without the model restating it.
stream solar.energy_revenue on entity asset.solar inflow currency USD {
  schedule every month from 2026-01 to 2028-12
  amount = 1200 * curve_value("power_price", time.date)
}

stream solar.om on entity asset.solar outflow currency USD {
  schedule every month from 2026-01 to 2028-12
  amount = 9000
}
```
