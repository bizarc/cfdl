---
id: guide-curves
title: Curves
slug: /docs/guides/curves
description: "Declare named, date-indexed series — rate curves, price decks, escalation paths — and read them from expressions."
generated: none
---

# Curves

`curve` declares a named, date-indexed series — rate curves, price decks,
escalation paths — read from expressions with `curve_value(name, date)`.

```cfdl
curve sofr linear {
  2026-01: 0.050
  2027-01: 0.038
}

stream loan.interest on entity asset.loan inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 1000000 * (curve_value("sofr", time.date) + 0.0275) / 12
}
```

## Interpolation

- `step` — flat-forward: each value holds until the next point.
- `linear` — calendar-day interpolation between points.
- Lookups outside the range clamp to the first/last point.

Duplicate curve names, duplicate point dates, or malformed points fail
compilation with `E5008_INVALID_CURVE`.

## Where curves shine

The credit pack's floating-rate pools read their coupon index from a named
curve with margin, floor, and cap:

```cfdl
contract credit.pool_float_io_bullet.bridge on entity asset.buyer {
  term 2026-01..2029-05
  terms {
    balance = 15000000
    index_curve = "sofr"
    margin = 0.0275
    rate_floor = 0.07
    rate_cap = 0.09
    term_months = 36
  }
}
```

Curves are deterministic inputs today; per-trial stochastic rate paths are
on the roadmap.

## Reference links

- [Credit pack guide](/docs/packs/credit)
- [Expression environment — curve_value](/docs/specification/expression-environment)
