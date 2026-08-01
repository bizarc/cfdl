# CFDL v0.1 — Expression Environment

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
- **excel_compat mode.** Available to benchmark harnesses via
  `cfdl_expr::eval_with_mode`. All arithmetic runs in IEEE-754 float64,
  reproducing Excel's representation artifacts (`0.1 + 0.2 - 0.3` yields
  ~5.55e-17, exactly as Excel does). Used to prove parity against Excel
  reference models and to explain decimal-vs-float differences.

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

The host (compiler or engine) provides values under five roots:

| Root | Contents |
|---|---|
| `model` | `model.id`, `model.base_currency` |
| `time` | `time.t` (0-based period index), `time.date`, `time.phase`, `time.ppy` (periods per year for the model's calendar) |
| `entity` | attributes of the stream's owning entity |
| `cfg` | run-config values (scenario knobs) |
| `obs` | observations (rates, curves) supplied at run time |

Unknown variables are hard errors (`EXPR_EVAL`), not nulls.

`time.ppy` is how many periods of the model's calendar make a year — 365, 12,
4 or 1 — so a model can spread an annual figure without hardcoding a divisor
and without being rewritten when the calendar changes:

```
amount = inputs.rent_year / time.ppy
```

Domain packs do **not** use it. A lowering rule resolves its own
periods-per-year at compile time (`{{model.periods_per_year}}`, see
`docs/07_pack_interface.md`), because a rule may pay on its own interval: a
monthly-paying loan carried on a daily book divides by 12, not 365, and only
the compiler can see that. `time.ppy` reads the calendar and would say 365.

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

Dates: `date(y, m, d)`, `parse_date(text)` (ISO `YYYY-MM-DD` or `YYYY-MM`),
`edate(d, months)`, `eomonth(d, months)`, `months_between(d1, d2)`,
`days_between(d1, d2)`, `year_frac(d1, d2, basis)`. Date arithmetic: `d2 - d1` yields days;
`d + n` / `d - n` shift by days.

Day-count bases for `year_frac`: `"30/360"` (aliases `"30/360 us"`, `"bond"`),
`"30e/360"` (alias `"eurobond"`), `"act/360"`, `"act/365"`, per ISDA/SIFMA
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

