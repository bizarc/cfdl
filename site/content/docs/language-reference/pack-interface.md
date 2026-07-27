---
id: pack-interface
title: "Pack Interface (v0.1)"
slug: "/docs/language-reference/pack-interface"
source: docs/07_pack_interface.md
---

**CFDL Domain Pack Interface v0.1**

Domain Packs provide *additions and overrides* on top of a single core language: each pack adds contract types, lowering rules, metrics, and validations without forking the language itself.

Core principle: **Packs may extend validation and provide defaults/templates, but MUST NOT change core language semantics.**

---

## 1) Overview

A **pack** is a versioned module that can:

- Provide **type registries** (domain entity/contract/option types)
- Provide **aliases** (domain names → canonical core concepts)
- Provide **contract term schemas** and **lowering rules** (contracts → streams/events/options)
- Provide **validations** (domain constraints with stable diagnostic codes)
- Provide **defaults** (required observables, output specification, reporting conventions)

### Non-goals
- Packs do not change core syntax.
- Packs do not add nondeterministic behavior.
- Packs do not embed external network calls.

### Why packs (and why not bake domains into core)

CFDL core must remain simple, strongly typed, and stable. Domain logic — contract forms, regulatory constraints, industry assumptions — changes frequently. Packs isolate that volatility.

### Determinism rules
Packs must be deterministic:
- same inputs ⇒ same lowered outputs
- stable ordering in any emitted lists
- no random/time/network access

---

## 2) Pack selection in CFDL

CFDL models MAY select a pack:

```cfdl
use pack "cre" version "0.1.0"
```

Compiler rules:
- `use pack` MAY appear **only** in `model.cfdl`.
- At most one pack MAY be active in v0.1.

If no pack is selected:
- Compilation still works using core rules.
- Unknown type IDs are permitted (with warnings) except where the compiler is configured to require a pack.

---

## 3) Pack identity and versioning

### 3.1 Pack ID
A pack MUST have a stable ID string:
- The ID is the bare pack name matching `name` in `pack.toml`
  (e.g., `cre`, `energy`, `credit`, `opco`); `use pack "<id>"` resolves
  against it. Namespaced ids (`publisher/name`) are reserved for a future
  multi-publisher registry.

### 3.2 Pack version
A pack MUST have a semver-like version string:
- `MAJOR.MINOR[.PATCH]`

### 3.3 Compatibility
- Compiler version and pack version are **independently versioned**.
- A pack MUST declare supported compiler IR versions.

---

## 4) Pack distribution formats

A pack MAY be distributed as:
- a local directory (dev mode)
- a signed bundle file (zip/tar)
- a registry artifact (future)

v0.1 minimum: **local directory packs**.

---

## 5) Pack structure on disk

Packs are loaded from the filesystem (v0.2 default). Example structure:

```
packs/
  cre/
    pack.toml
    aliases.toml
    templates.toml
    lowering/
      rules.toml
    validations.toml
    defaults.toml
    outputs.toml
    README.md
  opco/
    pack.toml
    ...
```

### 5.1 Required manifest (`pack.toml`)

A pack directory MUST include `pack.toml`:

```toml
pack_id = "cre"
version = "0.1"
description = "Commercial Real Estate domain pack"
ir_versions = ["0.1"]

[entrypoints]
types = "ontology/types.toml"
aliases = "aliases.toml"
contract_schemas = "contracts/schemas.toml"
lowering_rules = "contracts/lowering.toml"
outputs = "outputs.toml"
docs = "docs/index.toml"
```

Rules:
- `pack_id`, `version`, `ir_versions`, and `entrypoints.types` are REQUIRED.
- `entrypoints.outputs` is REQUIRED for packs that define domain-specific metrics and aggregations.
- Other entrypoints are optional.

### 5.2 Pack formats
- All pack artifacts are TOML-based.
- Keep pack files deterministic and avoid mixing YAML/JSON variants in the same pack.

---

## 6) Pack capabilities (what a pack can provide)

### 6.1 Type registry (ontology types)
A pack MAY define types used by:
- `entity ... : <TypeId>`
- `contract <TypeId> ...`
- `option ... type <TypeId>`

**Required:**
- A pack MUST provide a type registry file for at least the types it claims.

Minimum shape:
```json
{
  "types": [
    {
      "type_id": "CRE.Asset",
      "kind": "entity",
      "fields": {
        "city": {"type": "String", "required": false},
        "units": {"type": "Int", "required": false}
      }
    }
  ]
}
```

Type registry semantics:
- Packs MUST NOT remove core types.
- Packs MAY extend fields.
- Packs MAY provide documentation strings and examples.

### 6.2 Alias registry
Aliases map domain-friendly names to canonical TypeIds or contract templates.

Example:
```json
{
  "aliases": [
    {"alias": "Lease", "resolves_to": "Contract.Lease"},
    {"alias": "SeniorLoan", "resolves_to": "Contract.Loan.Senior"}
  ]
}
```

Compiler usage:
- Aliases are used by editors/CLI for suggestions.
- Aliases MAY be expanded during lowering if present in source.

Rules:
- Alias resolution must be deterministic.
- Packs must not create ambiguous alias collisions within a single loaded environment.

### 6.3 Contract term schemas
A pack MAY provide schemas for contract terms per TypeId.

Example:
```json
{
  "contracts": [
    {
      "type_id": "Contract.Lease",
      "terms": {
        "base_rent": {"type": "Money", "required": true},
        "rent_growth": {"type": "Rate", "required": false},
        "start_date": {"type": "Date", "required": false}
      }
    }
  ]
}
```

Compiler usage:
- If schema exists, the compiler MUST validate required terms and types.
- Schema validation errors are `E4003_INVALID_CONTRACT_TERMS`.

### 6.4 Lowering rules (contract → effects)
A pack MAY provide lowering rules that generate `effects` from `terms`.

**Core guarantee:**
- If a pack declares a lowering rule for a contract type, the compiler MAY allow `effects` to be omitted in source.

Lowering rule semantics:
- Inputs: contract instance (type, terms, subject, term date range)
- Output: one or more streams and/or derived term expansions
- Naming convention: generated stream names SHOULD be qualified names (dot-separated hierarchy), e.g. `cre.lease.base_rent`.
- Ownership convention: `owner` SHOULD resolve to a qualified entity symbol.

Rule interface options:
- **Declarative lowering** (recommended for v0.1)
- **Plugin function** (future)

v0.1 recommended declarative structure:
```json
{
  "lowering": [
    {
      "type_id": "Contract.Lease",
      "generates": [
        {
          "stream_name": "rent",
          "owner": "${subject}",
          "direction": "inflow",
          "currency": "${contract.currency}",
          "schedule": {"kind": "Every", "every": "monthly", "on_rule": {"kind": "EndOfMonth"}},
          "amount_expr": {"lang": "cfdl", "src": "terms.base_rent"}
        }
      ]
    }
  ]
}
```

Template rules:
- `${subject}` resolves to the contract subject entity symbol.
- `${contract.currency}` resolves to the contract currency.

Compiler behavior:
- Lowering runs after core validation.
- Generated streams MUST carry provenance notes referencing the contract.
- Generated streams SHOULD use deterministic dotted naming to preserve ontology/data-source mapping stability.
- If lowering fails, emit `E500x` and fail compilation.

### 6.5 Term payloads (current host behavior)
Contract `terms { ... }` values are captured as a lightweight map and exposed to pack lowering logic.

Current contract for packs:
- Terms are key/value pairs with string payloads plus source span.
- Packs are responsible for explicit parsing/coercion (e.g. Int/Decimal/Date).
- Packs should not rely on implicit casts; invalid values must emit diagnostics.
- If term-level spans are unavailable for a rule, use contract span consistently.

### 6.6 Ontology observable and reference IDs
Packs define canonical IDs for:
- `obs.rate(<name>)`, `obs.index(<name>)`, `obs.fx(<from>, <to>)`
- `ref.<name>`

Packs MAY provide registries:
- `observables.json`
- `refs.json`

Compiler behavior:
- If pack provides registries, the compiler MAY validate that referenced IDs exist.
- Missing observable IDs SHOULD be warnings in v0.1 (allow offline modeling).

### 6.7 Expression functions

The expression function vocabulary is fixed and built into the engine
(`cfdl-calc`) — `pmt`, `year_frac`, `cpr_to_smm`, `curve_value`, and so on
(see the [Expression Environment](/docs/language-reference/expression-environment)).
Packs do **not** define their own expression functions in v0.1.

### 6.8 Pack validations

Packs can add domain-specific validations, e.g.:
- Lease must have start/end
- Construction loan must have draw period
- Exit cap must be within bounds

Validation must:
- Produce diagnostics with stable codes
- Include file/span when possible
- Never crash

Validations are declared as data in `packs/<pack>/validations.toml` and
evaluated by the compiler; they are not implemented in the engine. Each rule
names the contract it applies to, the check, a stable diagnostic code, and a
message. Available checks: `term_present`, `any_term_present`, `term_number`
(integer or decimal, with `min`/`max`/`exclusive_min`/`exclusive_max`, and
`when`/`on_invalid` to control absent and unparseable values),
`term_range_within_timeline`, `term_enum`, and `term_compare`. The set is
closed — no expressions, recursion, or message interpolation — so evaluating
a pack's validations is bounded work that cannot crash or hang the compiler.

The file declares its pack's reserved code prefix, which the loader enforces:

- `E6xxx_*` for CRE
- `E7xxx_*` for OpCo
- `E8xxx_*` for Energy
- `E9xxx_*` for Credit

Terms that a lowering template requires are checked generically by
`E5006_MISSING_CONTRACT_TERM`; validations express the domain constraints on
top of that (bounds, enumerations, and relationships between terms).

### 6.9 Documentation metadata
Packs SHOULD provide docs for:
- types and fields
- contract templates
- output definitions
- examples

Editors may use this for hover hints and snippet insertion.

### 6.10 Output specification (engine-computed metrics and aggregations)

A pack MUST provide an output specification if it defines domain-specific metrics. Output metrics (NPV, IRR, NOI, DSCR, etc.) are NOT defined in CFDL — they are computed by the engine based on the pack's output spec.

**Core principle:** CFDL defines time, structure, and behavior (streams, events, options). The engine reads the IR + the pack output spec and produces domain-appropriate results. Different packs produce different output sets from the same IR.

#### 6.10.1 Stream categorization

The output spec defines **categories** that group streams by semantic purpose. Categories match streams using:
- **Contract type + direction:** For streams inside contract `effects` blocks
- **Stream name prefix + direction:** For standalone streams

```toml
[categories.operating_revenue]
description = "Operating revenue streams"
match = [
  { contract_type = "Contract.Lease", direction = "inflow" },
  { stream_prefix = "cre.ops_revenue", direction = "inflow" },
  { stream_prefix = "cre.projected_leaseup", direction = "inflow" }
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
```

Matching rules:
- A stream matches a category if ANY match rule is satisfied.
- A stream MAY match multiple categories (engine resolves by priority).
- Streams that match no category still appear in net cashflows.

#### 6.10.2 Aggregations

Aggregations combine categories into domain-meaningful time series:

```toml
[aggregations.noi]
description = "Net Operating Income"
formula = "categories.operating_revenue - categories.operating_expense"
frequency = "period"

[aggregations.cfads]
description = "Cash Flow Available for Debt Service"
formula = "aggregations.noi - categories.debt_service"
frequency = "period"

[aggregations.net_cashflow]
description = "Total net cashflows across all streams"
formula = "sum(all_streams)"
frequency = "period"
```

Aggregation rules:
- Aggregations produce per-period time series in the Results `series` map.
- Aggregations MAY reference other aggregations (engine resolves order).
- The engine MUST detect circular references and emit an error.

#### 6.10.3 Ratios

Ratios express relationships between aggregations:

```toml
[ratios.dscr]
description = "Debt Service Coverage Ratio"
numerator = "aggregations.noi"
denominator = "categories.debt_service"
frequency = "period"
```

Ratio rules:
- Ratios produce per-period time series.
- Division by zero SHOULD emit a warning and produce `null` for that period.

#### 6.10.4 Derived values

Derived values are domain-specific computations the engine performs:

```toml
[derived.terminal_value]
description = "Exit sale proceeds from cap rate valuation"
kind = "exit_cap"
noi_source = "aggregations.noi"
cap_rate_term = "exit_cap"
schedule = "on_exit"
```

Derived rules:
- The `kind` field tells the engine which computation to apply.
- Inputs reference aggregations, categories, or contract terms by name.
- Derived values may insert additional cashflows into the model (e.g., terminal value).

#### 6.10.5 Summary metrics

Summary metrics are computed from aggregated series or net cashflows:

```toml
[metrics.npv]
description = "Net Present Value"
source = "aggregations.net_cashflow"
method = "discount"
discount = "assume.discount_rate"

[metrics.irr]
description = "Internal Rate of Return"
source = "aggregations.net_cashflow"
method = "solve_irr"
```

Metric rules:
- Metrics produce scalar values in the Results `metrics` map.
- For Monte Carlo runs, metrics produce `MetricSummary` distributions.
- The engine MUST support at minimum: `discount` (NPV), `solve_irr` (IRR).
- Packs MAY define custom method names if the engine supports them.

---

## 7) Compiler ↔ Pack API (programmatic)

### 7.1 Pack loader interface
The compiler should expose a minimal interface:

- `load_pack(pack_id, version) -> Pack`
- `Pack.type_registry() -> TypeRegistry`
- `Pack.aliases() -> AliasRegistry`
- `Pack.contract_schema(type_id) -> Option<ContractSchema>`
- `Pack.lowering_rule(type_id) -> Option<LoweringRule>`
- `Pack.observable_registry() -> Option<ObservableRegistry>`
- `Pack.ref_registry() -> Option<RefRegistry>`
- `Pack.output_spec() -> Option<OutputSpec>`

### 7.2 Error behavior
- Pack not found: `E4004_MISSING_PACK`
- Pack manifest invalid: `E4004_MISSING_PACK` with details
- Unsupported IR version: `E4004_MISSING_PACK` with details

### 7.3 CLI
Recommended CLI behaviors:

- `cfdl pack list --path packs/`
- `cfdl pack validate --path packs/`
- `cfdl compile <model> --packs packs/`
- `cfdl run <ir> --packs packs/ --config run.json`

---

## 8) Determinism and provenance with packs

### 8.1 Determinism
Pack identity MUST participate in determinism:
- ID generation seed includes `pack_id@version` if present.

### 8.2 Provenance
The compiler SHOULD record pack info in top-level provenance notes.

---

## 9) Testing packs

Packs must be tested via golden fixtures.

Recommended test types:
- Pack load tests
- Alias resolution tests
- Template expansion tests
- Lowering tests (contract → streams)
- End-to-end example fixtures (compile + run results)

For each pack, include:
- `examples/<pack>/...` models
- `fixtures/valid/...` that use the pack
- Gold IR and results

---

## 10) Future-proofing (non-normative)

v0.2+ may add:
- multi-pack layering (base + overlays)
- signed pack artifacts
- executable lowering plugins (WASM)
- richer ontology reasoning
- sensitivity analysis definitions in output spec
- custom engine computation plugins

This v0.1 interface is designed to evolve without breaking core models.

---

## Parameterized lowering rules (templates)

Lowering-rule fields `amount_expr`, `schedule_from`, and `schedule_to` may
contain `{{contract.<key>}}` placeholders, resolved at compile time:

1. `{{contract.term_start}}` / `{{contract.term_end}}` — the contract's
   `term A..B` range (normalized dates).
2. `{{contract.<key>}}` — the contract's `terms { <key> = <value> }` entry
   (dotted keys like `lease_up.months` are supported).
3. Rule defaults — a per-rule `[rules.defaults]` table supplies fallback
   values when the contract does not declare the term.

A placeholder with no contract value and no default is a compile error
(`E5006_MISSING_CONTRACT_TERM`), one diagnostic per missing key.

Substitution is textual: numeric terms yield valid expression fragments;
string-valued terms must be quoted inside the template. Example:

```toml
[[rules]]
id = "lease_base_rent_v2"
contract_name = "cre.lease"
stream_name = "cre.lease.base_rent"
owner_entity = "${subject}"
direction = "inflow"
currency = "USD"
amount_expr = "{{contract.base_rent}} * clamp((time.t - {{contract.lease_up.start_period}} + 1) / {{contract.lease_up.months}}, 0, 1)"
schedule_kind = "every"
schedule_from = "{{contract.term_start}}"
schedule_to = "{{contract.term_end}}"

[rules.defaults]
"lease_up.start_period" = "6"
"lease_up.months" = "18"
```

---

## Declarative domain metrics (metrics.toml)

Packs declare their metric sets via a `metrics = "metrics.toml"` entrypoint;
the engine host evaluates them after a run (`cfdl run --pack <name>`). Adding
a domain means adding a metrics.toml, not Rust. Metrics are evaluated in file
order, so ratios may reference earlier ids.

```toml
[[metrics]]
id = "domain.cre.noi"           # output key
kind = "money"                  # money | number
op = "sum"                      # sum | negated_sum | ratio | wal_years
numerator_streams = ["cre.lease.base_rent", "cre.ops.revenue"]
denominator_streams = ["cre.ops.expense"]
formula = "sum(numerator_streams) + sum(denominator_streams)"  # lineage text

[[metrics]]
id = "domain.cre.debt_service"
kind = "money"
op = "negated_sum"
numerator_streams = ["loan.permanent_debt_service"]
formula = "-sum(numerator_streams)"
require_positive = true          # omit unless value > 0

[[metrics]]
id = "domain.cre.dscr"
kind = "number"
op = "ratio"
numerator_metric = "domain.cre.noi"
denominator_metric = "domain.cre.debt_service"
formula = "domain.cre.noi / domain.cre.debt_service"
# ratios are omitted when either input metric is absent or the denominator ~ 0

[[metrics]]
id = "domain.credit.wal_years"
kind = "number"                  # wal_years requires kind = "number"
op = "wal_years"
numerator_streams = ["credit.pool.sched_principal.*", "credit.pool.prepay.*"]
formula = "wal_years(numerator_streams)"
# weighted average life in years of the matched streams' positive per-period
# amounts: sum(t/ppy * v) / sum(v), using the engine's run.periods_per_year;
# omitted when the matched streams have no positive amounts
```

Engine-universal metrics are computed for every model regardless of pack:
`model.npv`, `model.irr`, `model.moic`, `model.payback_periods` /
`model.payback_years` (first period cumulative net cash turns non-negative,
when the model starts cash-negative), and `model.wal_years` (inflow-weighted
average life). All operate on the net cash-flow series.
