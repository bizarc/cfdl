---
id: expression-environment
title: "Expression Environment (v0.1)"
slug: "/language-reference/expression-environment"
---

> This page is generated from `docs/03_expression_environment.md`.
> Source: https://github.com/bizarc/cfdl/blob/main/docs/03_expression_environment.md

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
| `time` | `time.t` (0-based period index), `time.date`, `time.phase` |
| `entity` | attributes of the stream's owning entity |
| `cfg` | run-config values (scenario knobs) |
| `obs` | observations (rates, curves) supplied at run time |

Unknown variables are hard errors (`EXPR_EVAL`), not nulls.

## 4. Builtin functions

Conditionals & aggregates: `if(cond, a, b)` (lazy — only the taken branch is
evaluated), `min`, `max`, `sum`, `avg`, `abs`.

Rounding: `round(x, [digits])`, `round_down(x, [digits])`,
`round_up(x, [digits])`.

Math: `pow(base, exp)` (function form of `^`), `clamp(x, lo, hi)`.

Time value of money (Excel sign conventions, decimal-exact for whole-period
terms): `pmt(rate, nper, pv, [fv], [due])`, `pv(rate, nper, pmt, [fv], [due])`,
`fv(rate, nper, pmt, [pv], [due])`, `nper(rate, pmt, pv, [fv], [due])`,
`rate(nper, pmt, pv, [fv], [due], [guess])` (Newton solver, f64, tolerance
1e-12), `ipmt(rate, per, nper, pv, [fv])` / `ppmt(rate, per, nper, pv, [fv])`
(interest/principal split of payment `per`, 1-based; ordinary annuities).

Depreciation: `macrs_rate(year, life)` — IRS Pub 946 GDS half-year convention
percentages for 5/7/15/20-year property (`year` is 0-based; 0 beyond the
recovery period).

Credit: `cpr_to_smm(cpr)`.

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

Dates: `date(y, m, d)`, `edate(d, months)`, `eomonth(d, months)`,
`year_frac(d1, d2, basis)` with basis `"30/360"` (US/bond), `"act/360"`,
`"act/365"` per ISDA/SIFMA definitions. Date arithmetic: `d2 - d1` yields
days; `d + n` / `d - n` shift by days.

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
