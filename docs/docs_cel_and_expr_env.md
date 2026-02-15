# CEL and Expression Environment (v0.2)

This document specifies how **expressions** work in the CFDL SDK v0.2, including:

- Where expressions can appear in CFDL
- Supported value types (strong typing)
- The **Expression Environment (ExprEnv)**: variables, functions, and host-provided bindings
- CEL integration guidelines (syntax, determinism, diagnostics)

The goal is to keep authoring simple while enabling real modeling.

---

## 1. Design goals

1. **Strong typing**: expression evaluation must be type-safe and reject invalid operations.
2. **Deterministic**: same expression + same environment + same seed/time index ⇒ same result.
3. **Safe by default**: no I/O, no network access, no nondeterministic host functions.
4. **Portable**: works consistently across Rust, Python bindings, and TypeScript tooling.
5. **Finance outputs stay in engines**: NPV/IRR/stats are computed in the engine, not inside CEL.

---

## 2. Where expressions can appear in CFDL

Expressions evaluate to typed values. They may appear in:

- **Stream amount expressions**
  - e.g., amount depends on occupancy, rent, CPI, or a schedule index
- **Contract terms activation conditions**
  - e.g., a term becomes active when entity state changes or a phase starts
- **Schedule parameters**
  - e.g., a step-up date computed from another date
- **Assumption values** (when authored inline; distributions remain in run config)
- **Event predicates**
  - e.g., trigger refinance event when DSCR threshold crossed (threshold computed from observed values)

Non-goals for v0.2:
- expressions that declare distributions (these belong in run config)
- expressions that run valuation functions like NPV/IRR

---

## 3. Evaluation contexts

Expressions are evaluated in well-defined contexts.

### 3.1 Model compile-time evaluation
Used for:
- validating constant expressions
- deriving static parameters

Compile-time evaluation must not depend on time index.

### 3.2 Run-time evaluation (per time step)
Used for:
- stream amounts
- predicates that depend on time-series observables

Run-time evaluation is called with a context:
- current time index `t`
- current date `date`
- phase name (if any)

---

## 4. Types

All expressions evaluate to one of these types.

### 4.1 Scalar types
- `Bool`
- `Int` (signed 64-bit)
- `Decimal` (fixed precision; canonical representation)
- `String`

### 4.2 Time types
- `Date` (ISO 8601 date)
- `Period` (duration; v0.2 minimum: months and days)

### 4.3 Finance types
- `Currency` (ISO 4217 code)
- `Money`
  - `{ amount: Decimal, currency: Currency }`

### 4.4 Optional / nullability
- `Optional<T>`

Rules:
- optional values must be explicitly unwrapped with safe helpers
- missing observables return `Optional<T>` (not defaulted silently)

### 4.5 Collections
For v0.2, collections are limited:
- `List<T>` (read-only)
- `Map<String, T>` (read-only)

Collections are mainly used for:
- structured entity attributes
- observable payloads

---

## 5. Expression Environment (ExprEnv)

ExprEnv is the set of variables and functions available to an expression.

ExprEnv is assembled by the host at evaluation time and is a pure data structure.

### 5.1 Namespaces

To avoid collisions, variables are namespaced:

- `model.*` — model-level values
- `time.*` — time index/date/phase
- `entity.*` — current entity attributes/state
- `contract.*` — current contract terms (if evaluating within contract lowering)
- `obs.*` — observables (rates, FX, CPI, indices)
- `cfg.*` — run configuration values (scenario overrides)

### 5.2 Core variables

Required:

- `model.id: String`
- `model.base_currency: Currency`

- `time.t: Int`
- `time.date: Date`
- `time.phase: Optional<String>`

- `entity.id: String`
- `entity.name: String`
- `entity.state: Map<String, String>` (minimal v0.2 state representation)

Optional (host may provide):
- `entity.attrs: Map<String, Any>` (typed values only; avoid untyped blobs)

### 5.3 Observables API

Observables are accessed via functions (preferred) or via structured maps.

Minimum v0.2 functions:

- `obs.rate(name: String) -> Optional<Decimal>`
- `obs.fx(from: Currency, to: Currency) -> Optional<Decimal>`
- `obs.index(name: String) -> Optional<Decimal>`

Rules:
- observables must be deterministic for a given run + seed + date.
- missing observables return `Optional`.

### 5.4 Money helpers

- `money(amount: Decimal, currency: Currency) -> Money`
- `money_amount(x: Money) -> Decimal`
- `money_currency(x: Money) -> Currency`

Arithmetic rules:
- `Money + Money` requires same currency.
- `Money * Decimal` is allowed.
- `Money / Decimal` is allowed.
- Currency conversion is NOT done inside CEL by default.
  - Conversion happens in engine/policy layer.

### 5.5 Time helpers

Minimum v0.2 functions:

- `add_months(d: Date, n: Int) -> Date`
- `add_days(d: Date, n: Int) -> Date`
- `days_between(a: Date, b: Date) -> Int`
- `yearfrac(a: Date, b: Date) -> Decimal` (convention specified below)

`yearfrac` convention for v0.2:
- Actual/365 (Act/365)

### 5.6 Numeric helpers

- `min(a: Decimal, b: Decimal) -> Decimal`
- `max(a: Decimal, b: Decimal) -> Decimal`
- `clamp(x: Decimal, lo: Decimal, hi: Decimal) -> Decimal`
- `round(x: Decimal, dp: Int) -> Decimal`

---

## 6. CEL integration rules

### 6.1 Syntax
Use standard CEL syntax for:
- arithmetic: `+ - * /`
- comparisons: `== != < <= > >=`
- booleans: `&& || !`
- conditionals: `cond ? a : b`

### 6.2 Determinism constraints
Disallow or do not expose:
- time-of-day / now()
- random()
- any host function with side effects

### 6.3 Type checking
Expressions must be type-checked before evaluation.

Failure modes:
- parse error
- unknown identifier
- type mismatch
- illegal operation (e.g., Money + Decimal)

### 6.4 Normalization
Before hashing into IR (if expressions are stored in IR), expression strings are normalized:
- trim whitespace
- canonicalize line endings
- preserve original for display, but store normalized for deterministic identity if needed

---

## 7. Diagnostics

Expression diagnostics use the standard diagnostics format.

Recommended codes for v0.2:

- `E3001_EXPR_PARSE_ERROR`
- `E3002_EXPR_UNKNOWN_IDENT`
- `E3003_EXPR_TYPE_ERROR`
- `E3004_EXPR_ILLEGAL_OP`
- `E3005_EXPR_MISSING_OBSERVABLE`

Rules:
- diagnostics include file + span pointing at the expression location in the CFDL source.
- missing observables should only be an error when the value is required (host decides policy).

---

## 8. Authoring guidelines

### 8.1 Keep expressions small
Prefer:
- readable, local logic
- pack templates for larger logic

Avoid:
- multi-screen expressions
- embedded valuation logic

### 8.2 Prefer named observables
Use:
- `obs.rate("SOFR")`
- `obs.fx("USD", "EUR")`

Instead of hardcoding.

### 8.3 Money as the stream output
Stream amounts should resolve to Money at the end.

Examples:
- `money( rent_psf * sqft * occupancy, "USD")`
- `money( base_rent * pow(1 + cpi_growth, time.t/12), "USD")` (if pow is exposed; otherwise compute in engine)

---

## 9. Implementation notes (non-normative)

- Rust: implement `cfdl-expr` with CEL parsing/eval and a typed value enum.
- Python: expose compile/run and expression evaluation via bindings (PyO3/maturin preferred).
- TypeScript: tooling can reuse the same type model and offer editor diagnostics.

---

## 10. Open items (v0.3+)

- richer time conventions (30/360, Act/Act)
- unit types beyond Money (area, percentages)
- expression caching / memoization
- pack-defined function extensions

