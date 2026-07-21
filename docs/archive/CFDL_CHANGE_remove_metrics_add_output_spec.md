# CFDL Change Specification: Remove Metrics, Add Pack Output Spec

> **Purpose:** This document specifies all changes needed to the CFDL toolchain (language spec, grammar, compiler, IR schema, pack interface, expression environment) to remove `metric` from the language and move aggregation/calculation responsibility to the engine via pack-driven output specifications.

---

## Motivation

CFDL should define **time, structure, and behavior** (entities, contracts, streams, events, options, assumptions). Aggregation and metric computation are **engine concerns**, not language concerns.

Current problems:
- `metric` declarations in CFDL tightly couple the model to specific output calculations
- Adding a new metric requires editing CFDL and recompiling
- CEL built-in functions like `npv()`, `irr()`, `prob()` are engine operations masquerading as expressions
- Different domains need different output sets from the same IR

**New principle:** The engine reads the IR (streams, contracts, events) + a pack-provided output specification, and produces domain-appropriate results. CFDL never mentions NOI, NPV, IRR, or DSCR.

---

## Stream Categorization: Contract Type + Direction

No tags needed. The IR already provides the structure:

**Contract-owned streams** live in `contracts[].effects.streams`. The contract's `type` field (e.g., `Contract.Lease`, `Contract.Loan`) is the categorization. The pack's output spec references contract types + directions to define aggregations.

**Standalone streams** sit in the IR's top-level `streams[]` array. They have qualified names (e.g., `cre.ops_revenue`) and `direction`. For standalone streams, the **qualified name prefix** serves as the category — the pack's output spec can reference patterns like `cre.ops.*`.

The pack output spec maps contract types and stream name patterns to semantic categories:

```toml
[categories.operating_revenue]
match = [
  { contract_type = "Contract.Lease", direction = "inflow" },
  { stream_prefix = "cre.ops_revenue", direction = "inflow" }
]

[categories.operating_expense]
match = [
  { contract_type = "Contract.OperatingExpense", direction = "outflow" },
  { stream_prefix = "cre.ops_expense", direction = "outflow" }
]

[categories.debt_service]
match = [
  { contract_type = "Contract.Loan", direction = "outflow" },
  { contract_type = "Contract.ConstructionLoan", direction = "outflow" }
]
```

---

## Changes by File

### 1. CFDL v0.1 Language Spec (`cfdl_v_0_1.md`)

#### Remove §14.2 Metrics
Delete the entire §14.2 and its examples:
```diff
-### 14.2 Metrics
-```cfdl
-metric npv_base = cel "npv(model.cashflows, discount=assume('discount_rate'))"
-metric irr_equity = cel "irr(stream('equity_cashflows'))"
-metric prob_irr_gt15 = cel "prob(metric('irr_equity') > 0.15)"
-```
-
-Rules:
-- Metrics are named expressions.
-- In deterministic runs, metrics evaluate to scalars.
-- In Monte Carlo runs, metric values produce distributions and summary stats.
```

Rename §14 to "Runs" (remove "and metrics" from heading).

#### Remove metric references from §15 (Expressions)
Remove `npv()`, `irr()`, `prob()`, `metric()` from the required CEL built-ins list. These are engine functions, not expression-environment functions.

#### Update §15 built-ins list
Keep only model-authoring functions:
- `assume('name')` — assumption access
- `obs.rate("id")`, `obs.index("id")` — observable access (v0.2 syntax)
- `entity.id`, `entity.state` — entity property access (v0.2 syntax)
- `ref("id")` — reference data access
- Temporal functions: `t.period`, `t.index`, `t.year`, `t.month`
- Math: standard arithmetic, `min()`, `max()`, `abs()`, `round()`

#### Update reserved keywords
Remove `metric` from reserved keywords list.

#### Update the multi-file example (§17)
Remove `metrics.cfdl` from the example model:
```diff
 import "entities.cfdl"
 import "contracts.cfdl"
-import "metrics.cfdl"
```

Delete the `metrics.cfdl` example file section entirely.

#### Add note about engine-computed outputs
Add a brief note to §14 (Runs):

> Output metrics (NPV, IRR, DSCR, NOI, etc.) are computed by the engine based on the domain pack's output specification. CFDL models do not declare output metrics. See the Pack Interface specification for details.

---

### 2. CFDL Grammar (`CFDL_v0_1_Grammar.ebnf.md`)

Remove `metric_stmt` from the grammar:

```diff
 top_level_stmt  = use_stmt
                 | entity_stmt
                 | contract_stmt
                 | stream_stmt
                 | event_stmt
                 | option_stmt
                 | assume_stmt
                 | run_stmt
                 | phase_stmt
-                | metric_stmt
                 ;

-// --- runs & metrics ---
+// --- runs ---
 run_stmt        = "run" ( "deterministic" | "monte_carlo" "trials" INTEGER "seed" INTEGER ) ;
-metric_stmt     = "metric" qname "=" expr ;
```

---

### 3. IR Schema (`cfdl_v_0_1_ir_schema.md`)

Remove `metrics` from the required fields and remove the `Metric` definition:

```diff
   "required": [
     "ir_version",
     "model",
     "time",
     "phases",
     "entities",
     "assumptions",
     "contracts",
     "streams",
     "events",
     "options",
     "runs",
-    "metrics",
     "required_observables",
     "required_refs",
     "provenance"
   ],
```

Remove the `metrics` property:
```diff
-    "metrics": {
-      "type": "array",
-      "minItems": 0,
-      "items": { "$ref": "#/$defs/Metric" }
-    },
```

Remove the `Metric` definition from `$defs`:
```diff
-    "Metric": {
-      "type": "object",
-      "additionalProperties": false,
-      "required": ["id", "name", "expr", "provenance"],
-      "properties": {
-        "id": { "$ref": "#/$defs/Id" },
-        "name": { "$ref": "#/$defs/Id" },
-        "expr": { "$ref": "#/$defs/Expr" },
-        "provenance": { "$ref": "#/$defs/NodeProvenance" }
-      }
-    },
```

---

### 4. Compiler Spec (`compiler_spec_v_0_1.md`)

#### Remove metric-related AST nodes
In the AST section, remove any `MetricDecl` node type.

#### Remove metric lowering
In the lowering section, remove the rule that transforms metric AST nodes to IR objects.

#### Remove metric validation rules
Remove any validation rule related to metric expressions.

#### Update acceptance tests
Remove or update any test that exercises metric parsing/compilation.

---

### 5. CEL Expression Environment (`cel_and_expr_env_v_0_2.md`)

#### Remove financial functions from scope
Confirm these are NOT in the expression environment (they should already be removed per v0.2):
- `npv()`
- `irr()`
- `prob()`
- `metric()`
- `stream()` as a function call (keep `stream.<name>.amount` as a value access)

#### Keep model-authoring accessors only
The expression environment should provide only what CFDL model authors need:
- Observable access: `obs.rate("id")`, `obs.index("id")`
- Entity access: `entity.id`, `entity.state`, `entity.<attr>`
- Reference data: `ref("id")`
- Assumption access: `inputs.<name>` or `assume("name")`
- Time: `t.period`, `t.index`, `t.year`, `t.month`
- Stream value access (for cross-stream references, e.g., escalation): `stream.<name>.amount`

---

### 6. Pack Interface (`pack_interface_v_0_1.md`)

#### Add `outputs` capability
Add a new capability section for pack-provided output specifications:

```toml
# In pack.toml
[entrypoints]
aliases = "aliases.toml"
templates = "templates.toml"
lowering = "lowering/rules.toml"
outputs = "outputs.toml"           # NEW

# outputs.toml
[categories.operating_revenue]
description = "Operating revenue streams"
match = [
  { contract_type = "Contract.Lease", direction = "inflow" },
  { stream_prefix = "cre.ops_revenue", direction = "inflow" }
]

[categories.operating_expense]
description = "Operating expenses"
match = [
  { contract_type = "Contract.OperatingExpense", direction = "outflow" },
  { stream_prefix = "cre.ops_expense", direction = "outflow" }
]

[categories.debt_service]
description = "Debt service payments"
match = [
  { contract_type = "Contract.Loan", direction = "outflow" },
  { contract_type = "Contract.ConstructionLoan", direction = "outflow" }
]

[categories.exit]
description = "Exit/disposition proceeds"
match = [
  { contract_type = "Contract.ExitCap", direction = "inflow" }
]

# Aggregations (computed by engine from categories)
[aggregations.noi]
formula = "categories.operating_revenue - categories.operating_expense"
frequency = "period"     # per-period time series

[aggregations.cfads]
formula = "aggregations.noi - categories.debt_service"
frequency = "period"

[aggregations.net_cashflow]
formula = "sum(all_streams)"
frequency = "period"

# Ratios
[ratios.dscr]
numerator = "aggregations.noi"
denominator = "categories.debt_service"
frequency = "period"

# Terminal value (exit calculation)
[derived.terminal_value]
kind = "exit_cap"
noi_source = "aggregations.noi"
cap_rate_term = "exit_cap"      # references contract term key
schedule = "on_exit"

# Summary metrics (computed from aggregated series)
[metrics.npv]
source = "aggregations.net_cashflow"
method = "discount"
discount = "assume.discount_rate"

[metrics.irr]
source = "aggregations.net_cashflow"
method = "solve_irr"
```

#### Update capability list
Add `outputs` to the list of pack capabilities:
- Type registry
- Aliases
- Contract schemas
- Lowering rules
- Observable/ref IDs
- CEL extensions
- **Output specification** (NEW)

---

### 7. Packs Guide (`docs_packs_guide.md`)

Add a section (after §8 Lowering) for "Output Specification":
- Explain that packs define what metrics/aggregations the engine computes
- Show the `outputs.toml` structure
- Explain categories, aggregations, ratios, derived values, and summary metrics
- Note that different packs produce different output sets from the same IR

---

### 8. Results Schema (`cfdl_v_0_1_results_schema.md`)

**No schema change needed.** The results schema already supports arbitrary metric names and series. The difference is that metric names now come from the pack's output spec rather than CFDL source.

Add a note clarifying this:

> Metric names and series names in results are defined by the domain pack's output specification, not by the CFDL model. The engine evaluates the pack's output spec against the IR to produce results.

---

### 9. Language Guide (`LANGUAGE_GUIDE.md`)

Remove references to `metric` from the "quick map of language elements" and any examples.

Update the "When to use streams vs contracts" section if it references metrics.

---

### 10. User Guide (`USER_GUIDE.md`)

No changes expected — the user guide covers CLI usage (`compile`, `run`) and doesn't reference metrics directly. Confirm and update if needed.

---

## Implementation Checklist (Rust codebase)

For the working CFDL SDK implementation:

- [ ] **Parser:** Remove `metric_stmt` rule and `MetricDecl` AST node
- [ ] **Lexer:** Remove `metric` as a keyword token (if it's a dedicated token)
- [ ] **AST types:** Remove `MetricDecl` struct
- [ ] **Resolver:** Remove metric symbol resolution
- [ ] **Validator:** Remove metric validation rules
- [ ] **Lowering:** Remove metric → IR Metric transformation
- [ ] **IR types:** Remove `Metric` struct from IR model
- [ ] **IR serialization:** Remove `metrics` field from JSON output
- [ ] **Engine:** Add output spec loading from pack
- [ ] **Engine:** Implement category matching (contract type + direction, stream prefix)
- [ ] **Engine:** Implement aggregation computation
- [ ] **Engine:** Implement ratio/derived/summary metric computation
- [ ] **Pack loader:** Load `outputs.toml` from pack directory
- [ ] **Pack types:** Add output spec types (Category, Aggregation, Ratio, Derived, Metric)
- [ ] **Golden fixtures:** Update all golden IR files to remove `metrics` array
- [ ] **Golden fixtures:** Update all golden result files (metrics now engine-produced)
- [ ] **Tests:** Remove metric parsing tests
- [ ] **Tests:** Add output spec loading tests
- [ ] **Tests:** Add category matching tests
- [ ] **Tests:** Add aggregation/metric computation tests
- [ ] **Examples:** Update `cre_developer` and `opco_basic` examples (remove metrics.cfdl)
