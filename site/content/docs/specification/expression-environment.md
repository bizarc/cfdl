---
id: expression-environment
title: "Expression environment (v0.1)"
slug: "/docs/specification/expression-environment"
source: docs/03_expression_environment.md
generated: full
layer: specification
---

Status: Normative for the CFDL expression language (implemented by `cfdl-calc`,
exposed through `cfdl-expr`).

CFDL expressions are bare, Excel-familiar formulas written directly in model
source:

```
amount = base_rent * (1 + escalation) ^ (time.t / 12)
active when time.t >= 6
```

They are deterministic and terminating by construction: no loops, no recursion,
no I/O, no user-defined functions. Every expression, on the same inputs, always
produces the same value.

## 1. Numeric semantics

Two evaluation modes exist; models always run in **decimal mode**.

- **Decimal mode (default).** All arithmetic is exact 128-bit decimal
  (`rust_decimal`, 28 significant digits). `0.1 + 0.2 == 0.3` is `true`.
  Float64 is used ONLY as a documented escape for transcendental operations:
  fractional exponents (`x ^ 0.5`), and iterative solvers (`rate`,
  `cpr_to_smm`). Integer exponents are decimal-exact.
- **excel_compat mode.** All arithmetic runs in IEEE-754 float64, reproducing
  Excel's representation artifacts (`0.1 + 0.2 - 0.3` yields ~5.55e-17, exactly
  as Excel does), for proving parity against Excel reference models and
  explaining decimal-vs-float differences.

  It is reachable **only from Rust**, via `cfdl_expr::eval_with_mode`. There is
  no CLI flag and no run-config key, so a *model* cannot be run in it — the
  engine always evaluates in decimal. Nothing in the repo calls
  `eval_with_mode` today. See `docs/13_feature_backlog.md`.

  Whether that matters is measured rather than assumed:
  `excel_compat_stability` in `crates/cfdl-calc/src/lib.rs` runs the credit
  pack's arithmetic both ways and pins the divergence below 1e-12 — about ten
  orders of magnitude inside the tolerance of the benchmark it feeds. Decimal
  mode already routes fractional exponents through the f64 escape, so the two
  modes differ only where a model accumulates long sums or compares for
  equality.

Rounding: `round()` follows Excel semantics (half away from zero), not
banker's rounding. `round_down`/`round_up` truncate toward/away from zero.

## 2. Syntax

- Operators, by precedence (loosest to tightest):
  `or` < `and` < `not` < comparisons (`==`, `!=`, `<`, `<=`, `>`, `>=`) <
  `+ -` < `* / %` < unary `-` < `^` (right-associative).
- `=` and `<>` are accepted as Excel-style aliases for `==` and `!=` inside
  expressions.
- Literals: decimal numbers (`1200`, `0.05`, `1_000_000`), `true`/`false`,
  double-quoted strings.
- Variables are dotted paths resolved from the host environment (see §3).
- Function calls are lowercase snake_case: `pmt(0.005, 360, 100000)`.

## 3. Namespaces

The host (compiler or engine) provides values under these roots:

| Root | Contents |
|---|---|
| `model` | `model.id`, `model.base_currency` |
| `time` | `time.t` (0-based period index), `time.date`, `time.phase`, `time.ppy` (periods per year for the model's calendar), `time.days_in_period` |
| `entity` | attributes of the stream's owning entity, and every entity's properties under its family — `entity.asset.tlb.balance` |
| `asset`, `party`, `contract`, `reference` | an entity's properties, spelled bare: `asset.tlb.balance` is the same read as `entity.asset.tlb.balance` |
| `cfg` | run-config values (scenario knobs) |
| `obs` | observations (rates, curves) supplied at run time |
| `inputs` | assumption values (`assume` statements) |
| `state` | declared `state` values **at the current period** — present in stream expressions only |
| `prev` | declared `state` values **at the previous period** — present inside a state's `next` only |
| `remaining` | what is left in the pot — present in waterfall step expressions only (§3.2) |
| `paid` | `paid.<step>`, what an earlier waterfall step actually paid — steps only |
| `owed` | `owed.<step>`, what an earlier step would have paid, unbounded — steps only |

Unknown variables are hard errors (`EXPR_EVAL`), not nulls.

`time.ppy` is how many periods of the model's calendar make a year — 365, 12,
4 or 1 — so a model can spread an annual figure without hardcoding a divisor
and without being rewritten when the calendar changes:

```
amount = inputs.rent_year / time.ppy
```

Domain packs do **not** use it. A lowering rule resolves its own
periods-per-year at compile time (`{{model.periods_per_year}}`, see
[Pack Interface](/docs/specification/pack-interface)), because a rule may pay on its own interval: a
monthly-paying loan carried on a daily book divides by 12, not 365, and only
the compiler can see that. `time.ppy` reads the calendar and would say 365.

`time.days_in_period` is the actual calendar days the current period spans —
31 in January, 28 in a non-leap February, 1 on a daily grid. It is what makes
an Actual/360 or Actual/365 accrual expressible: `rate * time.days_in_period /
360`. Packs reach it through `{{model.accrual_divisor}}` rather than directly.

### 3.1 States: `state.<name>` and `prev`

A `state` is a named number per period defined by a recurrence — the one shape
`pow(1 + r, t)` cannot express, since that applies a single period's rate as
though it had held from the start.

```cfdl
state revenue_index {
  init  1.0
  next  prev * (1 + curve_value("growth", time.date))
}

stream firm.revenue on entity asset.firm inflow currency USD {
  schedule every year from 2026-01 to 2035-01
  amount = 21765.4 * state.revenue_index
}
```

`init` is the value at period 0 and is **mandatory** — an unstated base case
would otherwise evaluate as a silent zero for every period, since an unmatched
lookup returns 0. `next` is the value at every later period.

Inside `next`, bare `prev` is this state's own previous value and `prev.<name>`
is another state's. The two prefixes never overlap, and each exists in exactly
one place:

| prefix | resolves to | present in |
|---|---|---|
| `state.<name>` | that state at the **current** period | stream expressions |
| `prev.<name>` | that state at the **previous** period | `next` expressions |

This is separation **by absence**, not by check — a `next` environment carries
no `state` map and a stream environment carries no `prev` map, so the entry is
not there to be found. The same mechanism as `series` being empty when a
phase-1 stream evaluates.

Because everything a state can read is already finished, no reference can close
a cycle. States may therefore reference each other freely, including mutually,
and **declaration order carries no meaning**:

```cfdl
state a { init 1  next prev + prev.b }
state b { init 1  next prev + prev.a }
```

### A state has its own clock

A state may carry the same `schedule` clause a stream does:

```cfdl
state pool_survival {
  schedule every quarter from 2026-01 to 2031-01
  init 1.0
  next prev * (1 - hazard)
}
```

The recurrence **steps** on that cadence and **holds** between ticks. Absent, it
steps every model period, which is what a state without the clause has always
meant.

This matters because the model's clock is not the instrument's. A pool carried
on a daily calendar but paying monthly must compound its hazard twelve times a
year, not three hundred and sixty-five — the same separation a stream's
`schedule every quarter` already expresses on a monthly book.

Two details that are off-by-one traps, both found by building the fixture:

- The recurrence steps on **accrual** periods, not settlement periods. A
  quarterly schedule accrues at periods 0, 3, 6 and settles at 2, 5, 8; a
  stream's amount is evaluated at the accrual, so that is where the state must
  align.
- `init` is the value **at the first tick**, not at model period 0. Otherwise
  the first payment would read the second value of the recurrence.

An interval finer than the model calendar is
`E2108_SCHEDULE_FINER_THAN_CALENDAR`, exactly as for a stream.

Three further properties, each the opposite of a defensible alternative:

- **Holding is not being inactive.** Outside its window, and between ticks, a
  state keeps its value. An inactive *stream* yields 0; a state does not, which
  is why `active when` is deliberately absent — a schedule says *when the
  recurrence advances*, not *whether the quantity exists*.
- **A state is not cash.** It has no entity, direction or currency. It is
  published in results as `state.<name>` with bare numbers, and never enters
  `model.total`, `model.npv`, the annual rollup or any domain metric.
- **`next` has no series access** in v0.1. It sees `prev`, `prev.<name>`,
  `time.*`, `inputs.*`, `cfg`, `obs` and curves. Reading another *stream's*
  history from a recurrence is not expressible yet; `series_sum` remains the
  route to a stream's window, from a stream.

See `docs/14_state_and_recurrence.md` for the design and the prior art it
follows.

### 3.2 Waterfall steps: `remaining`, `paid` and `owed`

A waterfall step is an expression like any other, with three extra names.

`remaining` is what survives the steps above it. A step pays
`min(max(0, expr), remaining)`, so `= remaining` means exactly what it says,
a step asking for more than is left takes what is left, and a negative
expression pays nothing rather than clawing cash back.

`paid.<step>` and `owed.<step>` read a step declared **earlier in the same
waterfall** — what it actually paid, and what it would have paid had the pot
been deep enough. They differ exactly when a step could not be paid in full,
so their difference is that step's shortfall:

```
amount = owed.trustee_fee - paid.trustee_fee
```

That is how a capped fee gets its overflow paid at a later priority, and how a
step measures a balance "after giving effect to" the payments above it. Reading
a step declared later is a compile error.

A step also sees everything a stream sees, including `state.<name>` at the
current period — a waterfall runs after states are evaluated, so the balances
it tests are period-close values.

Steps publish as series `stream.<waterfall>.<step>`, so `series_sum` reaches an
earlier waterfall's output from a later one's `from` expression. That is how one
waterfall's payment becomes another's pot.

## 4. Builtin functions

Conditionals & aggregates: `if(cond, a, b)` (lazy — only the taken branch is
evaluated), `min`, `max`, `sum`, `avg`, `abs`.

Rounding: `round(x, [digits])`, `round_down(x, [digits])`,
`round_up(x, [digits])`.

Math: `pow(base, exp)` (function form of `^`), `clamp(x, lo, hi)`.

Time value of money (Excel sign conventions, decimal-exact for whole-period
terms). **Excel sign conventions mean `pmt` returns a negative number for a
positive `pv`** — a loan payment is money leaving. On a stream already declared
`outflow`, negate it (`amount = -pmt(...)`), or the two negatives cancel and the
payment registers as income: `pmt(rate, nper, pv, [fv], [due])`, `pv(rate, nper, pmt, [fv], [due])`,
`fv(rate, nper, pmt, [pv], [due])`, `nper(rate, pmt, pv, [fv], [due])`,
`rate(nper, pmt, pv, [fv], [due], [guess])` (Newton solver, f64, tolerance
1e-12), `ipmt(rate, per, nper, pv, [fv])` / `ppmt(rate, per, nper, pv, [fv])`
(interest/principal split of payment `per`, 1-based; ordinary annuities).

Depreciation: `macrs_rate(year, life)` — IRS Pub 946 GDS half-year convention
percentages for 5/7/15/20-year property (`year` is 0-based; 0 beyond the
recovery period).

Credit: `cpr_to_smm(cpr)`, `cpr_to_periodic(cpr, ppy)`.

`cpr_to_smm(x)` is `1 - (1-x)^(1/12)` and always means *monthly*.
`cpr_to_periodic(x, ppy)` is the same conversion on a grid of `ppy` periods per
year, and `cpr_to_periodic(x, 12) == cpr_to_smm(x)` exactly. Note this is a
**root**, not a division: CPR and CDR are effective annual rates, so they
convert by taking a root, while note rates are nominal and convert by dividing.
Using one convention for the other is a silent factor-level error.

Curves: `curve_value(name, date)` looks up a model-declared `curve`
statement at a date. `step` curves (the default) are flat-forward: the last
point at or before the query date (the first value before the first point).
`linear` curves interpolate linearly in calendar days between bracketing
points and clamp flat outside the declared range. Referencing an undeclared
curve is an evaluation error.

Cross-stream series (phase-2 streams): `series_sum(name, from_t, to_t)` /
`series_avg(name, from_t, to_t)` aggregate another stream's signed per-period
amounts over an inclusive period window (`prefix.*` wildcards supported).
Streams calling these evaluate in a second phase against finished phase-1
series — phase-2 streams cannot reference each other, so cycles are
impossible by construction. Windows may extend into the projection tail
(`time ... project <n>`), which is computed for valuation lookups but
excluded from cash results and NPV.

Logs: `ln(x)` (natural logarithm, `x > 0`) and `exp(x)`. These exist to turn a
cumulative **product** into a cumulative **sum**: a survival factor or growth
path under a *varying* rate is `PROD(1 + r_i)`, which has no closed form and is
not `pow(1 + r, t)` — that applies one period's rate as though it had held
throughout. Since `series_sum` aggregates a stream over a window,
`exp(series_sum(helper, 0, t))` recovers the product from a stream carrying
`ln(1 + r_t)`. Both escape to float64, as `pow` already does for fractional
exponents, so they are **not decimal-exact**; prefer a closed form where one
exists.

Dates: `date(y, m, d)`, `parse_date(text)` (ISO `YYYY-MM-DD` or `YYYY-MM`),
`edate(d, months)`, `eomonth(d, months)`, `months_between(d1, d2)`,
`days_between(d1, d2)`, `year_frac(d1, d2, basis)`. Date arithmetic: `d2 - d1` yields days;
`d + n` / `d - n` shift by days.

Day-count bases for `year_frac`: `"30/360"` (aliases `"30/360 us"`, `"bond"`),
`"30e/360"` (alias `"eurobond"`), `"act/360"`, `"act/365"`, per the standard market conventions
definitions.

Business days: `is_business_day(d, calendar)`, `roll(d, convention, calendar)`,
`add_business_days(d, n, calendar)`.

- Calendars: `"weekend"` / `"none"` (weekends only), `"us"` / `"us_federal"` /
  `"sifma"`, `"target"` / `"target2"` / `"eur"`, `"uk"` / `"uk_bank"` /
  `"london"`.
- Roll conventions: `"none"`, `"following"`, `"modified_following"`,
  `"preceding"`, `"modified_preceding"`.

```
roll(parse_date("2027-01-01"), "following", "us")   -- next US business day
add_business_days(time.date, 2, "london")           -- T+2 on the UK calendar
```

## 5. Errors and diagnostics

Every parse and evaluation error carries a byte-offset span into the
expression source. The compiler surfaces them as diagnostics with code
`EXPR_PARSE`; runtime failures surface as `EXPR_EVAL` warnings in Results
(the engine substitutes 0 / false and records the warning).

## 6. IR representation

Expressions are stored in IR as their raw source text with
`"lang": "cfdl"`:

```json
{ "lang": "cfdl", "src": "50000 * pow(1.15, time.t / 12.0)" }
```
