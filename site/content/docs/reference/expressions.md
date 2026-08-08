---
id: reference-expressions
title: "Expressions"
slug: "/docs/reference/expressions"
generated: regions
---

# Expressions

An amount is an expression, evaluated once per period:

```
stream tower.rent on entity asset.tower inflow currency USD {
  schedule every month from 2026-01 to 2035-12
  amount = 25000 * pow(1.03, time.t / 12.0)
}
```

Expressions are pure and total. There are no statements, no assignment, and no
loops, so the same inputs always produce the same number.

## What is in scope

| Binding | Holds |
|---|---|
| `time.*` | where you are — `time.t` is the period index, `time.date` its date |
| `inputs.*` | values declared with `assume`, including sampled ones |
| `model.*` | model-level facts such as the currency |
| `entity.*` | fields of the entity the stream belongs to |
| `cfg.*` | run configuration, such as the discount rate |
| `obs.*` | observed series a curve provides |
| `state.*` | declared states, at the current period |
| `prev` | this state's previous value — inside a state's `next` only |

Three more are bound inside a [waterfall](/docs/guides/waterfalls) step:

| Binding | Holds |
|---|---|
| `remaining` | what is left in the pot at this step |
| `paid.<step>` | what an earlier step actually paid |
| `owed.<step>` | what an earlier step would have paid, unbounded |

Inside a pack's lowering rule, `{{contract.*}}` placeholders are substituted
before the expression is parsed — so by the time it evaluates, a contract term
is a literal.

## Operators

Arithmetic `+ - * / %`, comparison `== != < <= > >=`, and boolean `and or not`.
`if(condition, then, else)` chooses between two values and is an expression, not
a branch — both arms are the same type.

## Numbers

All arithmetic is floating point. Where a source rounds — a workbook that
computes on already-rounded figures — reach for `round_to` and match the
source's *method* rather than restating its answer, so the model reproduces the
published number by doing what the publisher did.

## Functions

Generated from the engine's own dispatch table, so this is exactly what the
current build accepts.

<!-- cfdl:generated expression-builtins -->
**Arithmetic** — `abs`, `min`, `max`, `clamp`, `exp`, `ln`, `pow`, `sum`, `avg`

**Rounding** — `round`, `round_up`, `round_down`, `round_to`

**Dates** — `date`, `parse_date`, `edate`, `eomonth`, `days_between`, `months_between`, `year_frac`, `roll`, `is_business_day`, `add_business_days`

**Time value of money** — `pv`, `fv`, `pmt`, `ipmt`, `ppmt`, `nper`, `rate`

**Domain** — `macrs_rate`, `cpr_to_smm`, `cpr_to_periodic`

**Choice** — `if`

**Curves** — `curve_value`

**Series folds** — `series_sum`, `series_avg`

*37 functions.*
<!-- /cfdl:generated expression-builtins -->

## Related

- [Curves](/docs/guides/curves) — feeding observed series into `obs.*`.
- [Scenarios and run configs](/docs/guides/scenarios-and-run-configs) — where
  `inputs.*` and `cfg.*` come from.
- [Expression environment](/docs/specification/expression-environment) — the
  normative definition: types, coercion, and evaluation order.
