---
id: reference-expressions
title: "Expressions"
slug: "/docs/reference/expressions"
description: "What an expression may read and what it may do: bindings, operators, functions, and the order things evaluate in."
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
| `<family>.<entity>.<field>` | an entity's field — a value that moves is declared `init … next …` on the entity and read here. There is no `state.*` namespace (`E1125`) |
| `prev` | this field's previous value — inside a field's `next` only |

Three more are bound inside a [waterfall](/docs/guides/waterfalls) step:

| Binding | Holds |
|---|---|
| `remaining` | what is left in the pot at this step |
| `paid.<step>` | what an earlier step actually paid |
| `owed.<step>` | what an earlier step would have paid, unbounded |
| `available` | this period's netted cash of the waterfall's entity — also the default pot |
| `prev.<account>` | an account's settled balance one period back — steps, rules and guards alike; unavailable (not zero) at period 0 |

In logic — an event's guard, a field's rule, a lifecycle edge's `when` — a
series window must end at `time.t - 1` or earlier: logic reads settled
history, and a window that touches the current period is refused
(`E1134`).

Inside a pack's lowering rule, `{{contract.*}}` placeholders are substituted
before the expression is parsed. A term holding an expression is substituted
parenthesised, so it associates the way it reads; a literal or input reference
is substituted verbatim.

## Operators

Arithmetic `+ - * / %`, comparison `== != < <= > >=`, and boolean `and or not`.
`if(condition, then, else)` chooses between two values and is an expression, not
a branch — both arms are the same type.

## Numbers

Arithmetic is decimal — `0.1 + 0.2` equals `0.3`, and a cent is a cent —
with float64 only at the storage boundary. Where a source rounds — a workbook that
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

**Other** — `irr`, `moic`, `quantile_at`, `quantile_mean`, `quantile_of`

*42 functions.*
<!-- /cfdl:generated expression-builtins -->

## Related

- [Curves](/docs/guides/curves) — feeding observed series into `obs.*`.
- [Scenarios and run configurations](/docs/guides/scenarios-and-run-configs) — where
  `inputs.*` and `cfg.*` come from.
- [Expression environment](/docs/specification/expression-environment) — the
  normative definition: types, coercion, and evaluation order.
