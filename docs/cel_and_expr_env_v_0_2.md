# CEL and Expression Environment (v0.2)

This document defines the v0.2 standard for expression parsing/evaluation in CFDL.

It supersedes v0.1 ExprEnv guidance and captures what must carry forward:
- keep `expr = "cel" string_lit` unchanged
- keep evaluation deterministic and pure
- make dependency extraction a compiler responsibility

---

## 1) Normative scope

### 1.1 Expression surface syntax
- CFDL grammar remains unchanged:
  - `expr = "cel" string_lit`
- IR expression form remains:
  - `{ "lang": "cel", "src": "..." }`

### 1.2 Determinism and safety
- Evaluation MUST be deterministic: same IR + run config + seed => same result.
- CEL runtime MUST be side-effect free.
- Do not expose nondeterministic or external-effect functions:
  - `now()`, `random()`, I/O, network, filesystem access.

### 1.3 Evaluation contexts
- **Compile-time context**: parse/type checks and static validation.
- **Run-time context**: evaluation per timestep for stream amounts and predicates.

---

## 2) ExprEnv model (v0.2)

ExprEnv is a typed host-provided data object. Namespaces are required to avoid collisions and to support tooling.

### 2.1 Required namespaces
- `model.*`
- `time.*`
- `entity.*`
- `cfg.*`
- `obs.*`

`contract.*` is optional in v0.2 core, and used when evaluating in contract/lowering contexts.

### 2.2 Core values and types
- Scalar types:
  - `Bool`, `Int`, `Decimal`, `String`
- Domain/time types:
  - `Date`, `Currency`, `Money`, `Optional<T>`
- Typical bindings:
  - `model.id: String`
  - `model.base_currency: Currency`
  - `time.t: Int`
  - `time.date: Date`
  - `time.phase: Optional<String>`
  - `entity.id`, `entity.name`, `entity.state`

### 2.3 Observables and optionality
- Observable accessors return optional values:
  - `obs.rate(name) -> Optional<Decimal>`
  - `obs.index(name) -> Optional<Decimal>`
  - `obs.fx(from, to) -> Optional<Decimal>`
- Missing observable values MUST NOT be silently defaulted.
- Whether a missing optional becomes an error is policy-driven (compiler/runtime host policy).

---

## 3) CEL capability and exclusions

### 3.1 Supported CEL features (minimum)
- arithmetic: `+ - * /`
- comparisons: `== != < <= > >=`
- boolean logic: `&& || !`
- ternary conditionals: `cond ? a : b`

### 3.2 Money helpers (minimum)
- `money(amount, currency) -> Money`
- `money_amount(x) -> Decimal`
- `money_currency(x) -> Currency`

### 3.3 Explicitly out of scope in v0.2 CEL
These are engine/results concerns, not CEL concerns:
- `npv(...)`
- `irr(...)`
- Monte Carlo summary helpers like `prob(...)`

---

## 4) Dependency extraction

Dependency extraction is a compiler-stage responsibility and is required for planning, validation, and connector hydration.

### 4.1 What MUST be extracted
- Observable dependencies:
  - `obs.rate("SOFR")`
  - `obs.index("CPI")`
  - `obs.fx("USD", "EUR")`
- Ontology/reference dependencies:
  - `ref("...")` when supported

### 4.2 Why this is required
- enables pre-run checks for missing inputs
- enables EVS connector planning/pipelines
- enables authoring UX to show required inputs

### 4.3 Allowed implementation approaches
- **Option A (preferred): AST-based extraction**
  - parse CEL, walk calls, extract structured dependencies
- **Option B (interim): normalized pattern scan**
  - normalize source (trim + canonical newlines), scan recognized call patterns

### 4.4 IR representation
Compiler output MUST include extracted dependencies in deterministic IR fields:
- `required_observables`
- `required_refs`

Ordering MUST be deterministic.

---

## 5) Ontology bindings

### 5.1 Entity and contract bindings
- Ontology-backed attributes should be projected into namespace bindings:
  - `entity.*`
  - `contract.*` (when available in context)

### 5.2 `ref()` policy
- If `ref()` is retained, treat it as an explicit external dependency and extract it into `required_refs`.
- If `ref()` is not exposed in a deployment, compiler/runtime should reject usage with expression diagnostics.

---

## 6) Diagnostics

Expression diagnostics use standard CFDL diagnostics object format and include source file + span.

v0.2 expression diagnostics:
- `E3001_EXPR_PARSE_ERROR`
- `E3002_EXPR_UNKNOWN_IDENT`
- `E3003_EXPR_TYPE_ERROR`
- `E3004_EXPR_ILLEGAL_OP`

Missing observable escalation (error vs optional) is policy controlled.

---

## 7) Implementation checklist

### Compiler / IR
- [ ] Add expression dependency extraction stage after expression parsing.
- [ ] Emit extracted dependencies into IR `required_observables` / `required_refs`.
- [ ] Add validation diagnostics for missing required dependencies based on policy.

### Engine
- [x] Supply typed ExprEnv namespaces (`model`, `time`, `entity`, `cfg`, `obs`).
- [x] Evaluate stream `amount` and `active_when` per timestep deterministically.

### Fixtures / Gold
- [ ] Add at least one valid fixture using two dependencies:
  - `obs.rate("SOFR")`
  - `obs.index("CPI")`
- [ ] Verify IR includes extracted dependency lists.
- [x] Verify deterministic repeated-run output.

---

## 8) Minimal v0.2 example

Expression:

`cel "money(cfg.base_rent * (1 + obs.index(\"CPI\")).value_or(0.0), \"USD\")"`

Extracted dependencies:
- observable: `index:CPI`

---

## 9) Status

v0.2 implementation is underway. The non-negotiable carry-forward from v0.1 is dependency extraction. Finance/valuation functions stay in engines/results layers by design.
