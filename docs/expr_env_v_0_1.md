# expr_env_v0_1.md

**CFDL Expression Environment Specification v0.1**

**Status:** Draft

This document defines the expression environment contract for CFDL v0.1. CFDL embeds expressions using:

```cfdl
cel "<expression>"
```

The compiler validates expression *presence*, provides *slot typing* guarantees for required-boolean contexts, extracts `obs()` / `ref()` dependencies, and emits expressions into IR unchanged.

This specification is designed so that:
- The Rust compiler can validate and emit expressions deterministically.
- Engines (Rust/Python/TS) can evaluate expressions consistently.
- Domain packs can add functions without changing core semantics.

---

## 1) Normative requirements

1. **Language tag**: expressions MUST be tagged `lang: "cel"` in IR.
2. **Side-effect free**: expressions MUST be pure and deterministic given the same inputs.
3. **Termination**: engines SHOULD impose execution limits to prevent runaway expressions.
4. **No implicit correlation**: distributions are sampled independently unless a downstream engine chooses otherwise.
5. **Boolean slots are strict**: `event.when` and `stream.active_when` MUST evaluate to `Bool`.

---

## 2) Expression representation in IR

An expression in IR is:

```json
{ "lang": "cel", "src": "..." }
```

Engines MUST treat `src` as canonical expression source.

---

## 3) Evaluation model

### 3.1 Discrete time context
Expressions are evaluated at discrete time steps defined by the master timeline.

The environment MUST provide a time binding `t` with at least:
- `t.index` (Int): 0-based time index
- `t.date` (Date as ISO string `YYYY-MM-DD`)
- `t.year_index` (Int): number of whole years elapsed since timeline start (floor), or engine-defined year index consistent within a model

### 3.2 Value typing
Engines MUST operate with the following value kinds:
- `Bool`
- `Int`
- `Decimal`
- `String`
- `Date` (ISO string)
- `Money` (object `{amount, currency}`)
- `Rate` (object `{value, basis?}`)
- `Series<T>` (engine internal; may be represented in helper calls)
- `EntityView` (object-like access)

### 3.3 Money arithmetic rules (normative)
1. `Money + Money` requires same currency (otherwise error unless engine auto-converts via `convert`).
2. `Money - Money` same as above.
3. `Money * Decimal` and `Decimal * Money` → Money.
4. `Money / Decimal` → Money.
5. `Decimal / Money` is invalid.
6. `Rate` is represented as a Decimal-like in arithmetic only when explicitly accessing `.value` or using helper functions.

---

## 4) Required namespaces and functions (core)

All engines that execute CFDL expressions MUST implement the following bindings.

### 4.1 Inputs and assumptions

#### `inputs.<name>`
- Provides stochastic samples (for Monte Carlo) or deterministic values (for deterministic runs if configured).
- Type is determined by the assumption declaration.

#### `assume(name: String) -> Any`
- Returns the declared assumption value.
- For deterministic assumptions: returns scalar.
- For random assumptions in Monte Carlo: returns the sampled scalar at this trial.

**Compiler guidance:**
- The compiler MAY validate that `name` exists as an assumption when it can lexically infer it.

### 4.2 Entities

#### `entity(symbol: String) -> EntityView`
- Returns a view of the entity’s attributes and state.
- Access: `entity('asset.Sunset').units`

**EntityView semantics:**
- Must support field access for declared `attrs` and dynamic `state`.
- Unknown fields return null or error depending on engine strictness.

### 4.3 Observables and references (ontology linkage)

#### `obs(id: String) -> Decimal | Money | Rate | String | Bool`
- Returns the observable value for `id` at `t.date`.

#### `ref(id: String) -> Any`
- Returns a static reference object (table row, config object, curve descriptor, etc.)

**Compiler requirement:**
- The compiler MUST extract all `obs('...')` and `ref('...')` usages into `required_observables` and `required_refs`.

### 4.4 Streams

#### `stream(name: String) -> Series<Money>`
- Returns the named stream’s computed series.
- In deterministic runs, this is a concrete series.
- In MC runs, this may represent a series per trial (engine-defined).

#### `model.cashflows -> Series<Money>`
- Returns the aggregated cashflows for the model (engine-defined aggregation convention).

### 4.5 Finance functions

#### `npv(series: Series<Money>, discount: Rate|Decimal) -> Money`
- Returns net present value.
- Discount interpretation (annual vs period) is engine-defined but MUST be consistent and documented.

#### `irr(series: Series<Money>) -> Rate`
- Returns internal rate of return.

#### `convert(m: Money, to: String) -> Money`
- Converts money to target currency using `fx`.

#### `fx(from: String, to: String, date: Date) -> Decimal`
- Returns FX rate multiplier.
- OPTIONAL in v0.1 core, RECOMMENDED for multi-currency support.

### 4.6 Probability helper (Monte Carlo)

#### `prob(condition: Bool) -> Decimal`
- In Monte Carlo context, returns empirical probability across trials.
- In deterministic context, returns `1.0` if true else `0.0`.

---

## 5) Slot typing rules (compiler)

CFDL v0.1 compiler enforces slot types where required.

### 5.1 Strict boolean slots
The compiler MUST ensure that the following expressions are intended as boolean:
- `event.when`
- `stream.active_when`

**Minimum v0.1 approach (normative):**
- If expression source is empty → hard error.
- If expression source is a literal `true`/`false` → ok.
- Otherwise, the compiler emits the expression and MAY:
  - (a) attempt CEL type-check with a minimal signature environment, or
  - (b) emit a warning `W3001_EXPR_TYPE_UNKNOWN` and rely on runtime.

**If the compiler performs type-checking** and the expression is not boolean → emit:
- `E2201_EVENT_WHEN_NOT_BOOL` or `E2202_STREAM_ACTIVE_NOT_BOOL`.

### 5.2 Money amount slots
For `stream.amount`, the engine MUST produce `Money` values.

Compiler behavior:
- If the stream declares `currency` and expression is a numeric literal, compiler MAY wrap implicitly as Money at runtime.
- Otherwise, compile and allow runtime evaluation.

---

## 6) CEL compatibility guidance

### 6.1 CEL subset
CFDL expressions are a **CEL-compatible** subset.

Engines SHOULD support at least:
- arithmetic: `+ - * /` on numbers and Money (with rules)
- comparisons: `== != < <= > >=`
- boolean logic: `&& || !`
- conditionals: `cond ? a : b` (recommended)
- functions defined in §4

### 6.2 Determinism
- Engines MUST not allow non-deterministic functions (e.g., `now()`) in core.

### 6.3 Pack extensions
Packs MAY add functions:
- Must not override core names.
- Must declare signatures in pack metadata.

---

## 7) Dependency extraction (compiler)

### 7.1 Required extraction
Compiler MUST extract `obs()` and `ref()` IDs by scanning CEL source.

Accepted patterns:
- `obs('ID')`, `obs("ID")`
- `ref('ID')`, `ref("ID")`

Extraction rules:
- Ignore escaped quotes inside the argument.
- Do not attempt full CEL parsing in v0.1.
- If extraction fails for a given expression, emit `W3002_OBS_REF_EXTRACTION_FAILED`.

---

## 8) Examples

### 8.1 Boolean activation
```cfdl
active when cel "entity('loan.Senior').status != 'refinanced'"
```

### 8.2 Observable-driven trigger
```cfdl
event refi when cel "obs('Rates.SOFR.1M') < 0.035" { ... }
```

### 8.3 Money arithmetic
```cfdl
amount cel "terms.base_rent * (1 + inputs.rent_growth)"
```

### 8.4 Multi-currency conversion
```cfdl
amount cel "convert(terms.rent_eur, 'USD')"
```

---

## 9) Conformance

An implementation conforms to CFDL v0.1 expression environment if it:
1. Accepts `{lang:"cel", src:"..."}` expressions.
2. Provides required bindings and functions in §4.
3. Enforces boolean slot semantics for `event.when` and `stream.active_when`.
4. Evaluates expressions deterministically given identical inputs.

