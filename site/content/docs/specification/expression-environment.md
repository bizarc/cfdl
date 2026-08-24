---
id: expression-environment
title: "Expression environment (v0.1)"
slug: "/docs/specification/expression-environment"
description: "What an expression may read — time, inputs, cfg, obs, and entity fields — and the rules that constrain each binding."
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
| `entity` | fields of the stream's owning entity, and every entity's fields under its family — `entity.asset.tlb.balance` |
| `asset`, `party`, `contract`, `reference` | an entity's fields, spelled bare: `asset.tlb.balance` is the same read as `entity.asset.tlb.balance` |
| `cfg` | run-config values (scenario knobs) |
| `obs` | observations (rates, curves) supplied at run time |
| `inputs` | assumption values (`assume` statements), including ones derived from other assumptions (§2.1) |
| `prev` | a field's own previous value, bare — present inside that field's `next` only |
| `prev.<entity>.<field>` | a field one period back — `prev.asset.tlb.balance`, inside a rule |
| `remaining` | what is left in the pot — present in waterfall step expressions only (§3.2) |
| `paid` | `paid.<step>`, what an earlier waterfall step actually paid — steps only |
| `owed` | `owed.<step>`, what an earlier step would have paid, unbounded — steps only |

Unknown variables are hard errors (`EXPR_EVAL`), not nulls.

### 2.1 Derived assumptions

An `assume` may read another through `inputs.<name>`:

```
assume gross_sf   = 10000.0
assume efficiency = 0.85
assume net_sf     = inputs.gross_sf * inputs.efficiency
```

Assumptions resolve in dependency order, so each one is evaluated after
everything it reads — declaration order and name order are both irrelevant.
Random assumptions (`assume ... ~ <dist>`) resolve first: a distribution's
central value reads nothing, so a derived assumption may be built on one.

A circular derivation is refused with the cycle named, the same way a circular
series read is (§3.1): no order satisfies it, and the engine does not iterate
toward a fixed point. A name that is not an assumption is not a dependency —
it comes from the run configuration, or from nowhere, and an unresolved name
is a hard error by the rule above.

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

### 3.1 Fields that move: `<family>.<entity>.<field>` and `prev`

A field with a rule is a named number per period defined by a recurrence — the
one shape `pow(1 + r, t)` cannot express, since that applies a single period's
rate as though it had held from the start. It belongs to the entity it describes:

```cfdl
entity asset firm : Asset.Financial {
  revenue_index init 1.0
                next prev * (1 + curve_value("growth", time.date))
}

stream firm.revenue on entity asset.firm inflow currency USD {
  schedule every year from 2026-01 to 2035-01
  amount = 21765.4 * asset.firm.revenue_index
}
```

`init` is the value at period 0 and is **mandatory** — an unstated base case
would otherwise evaluate as a silent zero for every period, since an unmatched
lookup returns 0. `next` is the value at every later period.

Inside `next`, bare `prev` is this field's own previous value and
`prev.<family>.<entity>.<field>` is another field's. A rule may not read any
field at the current period, which is what keeps a cycle unexpressible:

| form | resolves to | present in |
|---|---|---|
| `<family>.<entity>.<field>` | that field at the **current** period | streams, waterfall steps, event guards |
| `prev` | this field at the **previous** period | `next` expressions |
| `prev.<family>.<entity>.<field>` | another field at the **previous** period | `next` expressions, streams |

`prev` accepts the entity-root spelling too — `prev.entity.asset.tlb.balance`
is the same read as `prev.asset.tlb.balance`, exactly as the two current-period
spellings are one read. It is the form a pack lowering rule produces, since
`field.<name>` resolves through the entity root (`docs/07`).

A stream environment carries no bare `prev`, so a stream cannot ask for "the
previous value" of something it does not own — the entry is not there to be
found. The same mechanism as `series` being empty when a wave-0 stream
evaluates.

Because everything a rule can read is already finished, no reference can close a
cycle. Fields may therefore reference each other freely, including mutually, and
**declaration order carries no meaning**:

```cfdl
entity asset pair : Asset.Financial {
  a init 1.0 next prev + prev.asset.pair.b
  b init 1.0 next prev + prev.asset.pair.a
}
```

### A field steps on the clock of whatever brought it

A field has no `schedule` clause of its own. An entity is not a temporal thing:
it does not start, stop or recur, so there is no cadence for it to carry. A
field declared directly on an entity therefore steps every model period.

A field a CONTRACT brings inherits that contract's schedule, because the
contract is the thing with a term and a payment frequency. The recurrence
**steps** on that cadence and **holds** between ticks, which is what lets a pool
carried on a daily calendar but paying monthly compound its hazard twelve times
a year rather than three hundred and sixty-five.

Two details that are off-by-one traps:

- The recurrence steps on **accrual** periods, not settlement periods. A
  quarterly schedule accrues at periods 0, 3, 6 and settles at 2, 5, 8; a
  stream's amount is evaluated at the accrual, so that is where the field must
  align.
- `init` is the value **at the first tick**, not at model period 0. Otherwise
  the first payment would read the second value of the recurrence.

Three further properties, each the opposite of a defensible alternative:

- **Holding is not being inactive.** Between ticks a field keeps its value. An
  inactive *stream* yields 0; a field does not, which is why `active when` has
  no meaning here — a cadence says *when the recurrence advances*, not *whether
  the quantity exists*.
- **A field is not cash.** It has no direction or currency. It is published in
  results under its own path as bare numbers, and never enters `model.total`,
  `model.npv`, the annual rollup or any domain metric.
- **`next` has no series access** in v0.1. It sees `prev`,
  `prev.<family>.<entity>.<field>`, `time.*`, `inputs.*`, `cfg`, `obs` and
  curves. Reading a *stream's* history from a recurrence is not expressible;
  `series_sum` remains the route to a stream's window, from a stream.

See `docs/14_state_and_recurrence.md` for the design and the prior art it
follows.

### 3.2 Waterfall steps: `remaining`, `paid` and `owed`

A waterfall step is an expression like any other, with four extra names.

`available` is the cash the waterfall's entity produced this period: its
streams' signed values, netted, with its children rolled up by `part of`.
Streams only — no distribution feeds it, so a waterfall can never read its own
output through it. The engine supplies it before the waterfall runs; no model
declares a field for it. `from available` is therefore the ordinary spelling of
a pot, and the `from` expression remains free for the deals that draw on
something narrower.

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

A step also sees everything a stream sees, including entity fields at the
current period — a waterfall runs after fields are evaluated, so the balances
it tests are period-close values.

Steps publish as series `<waterfall>.<step>`, so `series_sum` reaches an earlier
waterfall's output from a later one's `from` expression. That is how one
waterfall's payment becomes another's pot:

```
from series_sum("senior.residual", time.t, time.t)
```

Results publish the same series one namespace down, as
`stream.<waterfall>.<step>`, because in results every stream carries that
prefix. **The results name is not the name an expression reads.** This section
gave the results name until August 2026, and a model that followed it got an
empty pot rather than a diagnostic — a name nothing matches is not an error,
because a selector that matches nothing must be able to sum to zero. That
asymmetry is now the difference between a `.*` selector and a literal name; see
`E5022_UNKNOWN_SERIES_REFERENCE`.

A step's series is visible to a later waterfall's `from` and to nothing else.
Neither a stream nor a field's `next` can read it.

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

Cross-stream series: `series_sum(name, from_t, to_t)` /
`series_avg(name, from_t, to_t)` aggregate another stream's signed per-period
amounts over an inclusive period window (`prefix.*` wildcards supported).
Streams evaluate in dependency order — waves. A stream that reads no series is
wave 0; a reader evaluates one wave past the deepest stream it reads, against
a store in which everything it names is already finished, to any depth. A
circular read is the one thing no order can satisfy, and the engine refuses it
with the named cycle rather than iterating toward a fixed point (`docs/14`
§5). A read whose series name is computed at runtime evaluates after every
literally-named stream and cannot itself be read. Windows may extend into the
projection tail (`time ... project <n>`), which is computed for valuation
lookups but excluded from cash results and NPV.

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
`"30e/360"` (alias `"eurobond"`), `"act/360"`, `"act/365"`, `"act/act"` (ISDA;
aliases `"actual/actual"`, `"act/act isda"`), per the standard market conventions
definitions.

`act/act` splits the span at calendar-year boundaries and measures each part
against its own year's length, so a period crossing a leap year is not charged
365 days for a 366-day year: 2024-07-01 to 2025-07-01 is 184/366 + 181/365,
not 365/365.

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
