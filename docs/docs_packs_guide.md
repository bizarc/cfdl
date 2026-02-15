# Packs Guide (v0.2)

This document explains **CFDL Packs**: how domain packs extend the CFDL SDK to provide templates, aliases, lowering rules (contracts → streams/events/options), validations, and defaults.

Packs are the mechanism that enables a **single core language** to support multiple industries (similar to Salesforce “clouds”) while keeping domain logic versioned and modular.

---

## 1. What is a Pack?

A **pack** is a versioned module that can:

- Provide **aliases** (domain names → canonical core concepts)
- Provide **templates** (standard contract/entity patterns)
- Provide **lowering rules** that translate:
  - contracts + terms → streams / events / options
- Provide **validations** (domain constraints)
- Provide **defaults**:
  - required observables
  - common metrics to compute
  - reporting conventions

### Non-goals
- Packs do not change core syntax.
- Packs do not add nondeterministic behavior.
- Packs do not embed external network calls.

---

## 2. Why Packs (and why not bake domains into core)

CFDL core must remain:
- simple
- strongly typed
- stable and versionable

Domain logic changes frequently:
- contract forms
- regulatory constraints
- industry assumptions

Packs isolate that volatility.

---

## 3. Salesforce / Palantir analogy

### Salesforce “Clouds” analogy
- Core platform + objects
- Clouds add managed packages: objects, workflows, validations, UI forms

CFDL mapping:
- Core CFDL language + IR
- Packs add: templates, lowering, validations, default observables/metrics

### Palantir analogy
- Core platform + ontology + pipelines
- Domain apps configure transformations and constraints

CFDL mapping:
- Packs are domain modules used by engines and by EVS platform authoring/UI.

---

## 4. Pack interface (conceptual)

Packs interact with the SDK via a stable host interface.

### 4.1 Capabilities
A pack may implement any subset of:

- Alias registry
- Template registry
- Lowering hooks
- Validation rules
- Defaults provider

### 4.2 Determinism rules
Packs must be deterministic:
- same inputs ⇒ same lowered outputs
- stable ordering in any emitted lists
- no random/time/network access

---

## 5. Pack structure on disk (filesystem loader)

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
    README.md
  opco/
    pack.toml
    ...
```

### 5.1 `pack.toml`
Declares identity and entrypoints.

Example:

```toml
name = "cre"
version = "0.1.0"
description = "Commercial Real Estate pack"

[entrypoints]
aliases = "aliases.toml"
templates = "templates.toml"
lowering = "lowering/rules.toml"
```

Notes:
- Use `version` for pack evolution independent of SDK version.
- Packs may declare compatibility constraints later.

### 5.2 Pack formats (TOML)

For the current SDK implementation, pack artifacts are TOML-based:

- `pack.toml` (required manifest + entrypoints)
- `aliases.toml` (alias map)
- `templates.toml` (template definitions, may be stubbed)
- `lowering/rules.toml` (lowering rule list)
- optional pack data files like `validations.toml` and `defaults.toml`

Keep pack files deterministic and avoid mixing YAML/JSON variants in the same pack.

---

## 6. Aliases

Aliases allow domain-friendly names to map to canonical names.

Examples:
- `Lease` → canonical contract kind `contract.lease`
- `RentRoll` → canonical entity/stream pattern

Aliases live in packs but can be referenced by users.

Rules:
- alias resolution must be deterministic
- packs must not create ambiguous alias collisions within a single loaded environment

---

## 7. Templates

Templates are reusable definitions that expand into canonical objects.

Types of templates:

- **Contract templates**: lease, loan, revenue line
- **Entity templates**: property, unit mix, business unit
- **Stream templates**: rent, expense, debt service

Templates can be parameterized.

Parameter sources:
- user-provided values
- ontology bindings (EVS platform)

v0.2 template format is implementation-defined, but must compile into canonical CFDL/IR structures.

---

## 8. Lowering (contracts → streams/events/options)

Lowering is the core pack power: turning contract terms into executable cash flows.

### 8.1 Inputs
- canonical contract objects and terms
- related entities (lessor/lessee, borrower/lender)
- time model and phases
- assumptions and observables requirements

### 8.2 Outputs
- streams
- events
- options
- required observables

### 8.3 Provenance requirements
Lowered outputs must include provenance:
- source contract ID/name
- pack name/version
- rule identifier

This enables “explainability” and auditability.

### 8.4 Ordering
Lowering output order must be deterministic:
- sort by stable key (symbol/name)

### 8.5 Term payloads (current host behavior)
Contract `terms { ... }` values are captured as a lightweight map and exposed to
pack lowering logic.

Current contract for packs:
- terms are key/value pairs with string payloads plus source span
- packs are responsible for explicit parsing/coercion (for example Int/Decimal/Date)
- packs should not rely on implicit casts; invalid values must emit diagnostics
- if term-level spans are unavailable for a rule, use contract span consistently

This supports deterministic pack-origin validation during lowering (for example
`E6xxx_*` in CRE) without introducing a separate compile stage.

---

## 9. Pack validations

Packs can add domain-specific validations, e.g.:
- Lease must have start/end
- Construction loan must have draw period
- Exit cap must be within bounds

Validation must:
- produce diagnostics with stable codes
- include file/span when possible
- never crash

Diagnostics codes for pack validations should be reserved per-pack (recommended convention):
- `E6xxx_*` for CRE
- `E7xxx_*` for OpCo

---

## 10. Defaults

Packs can provide defaults for:

- required observables
- default reporting currency rules
- default metrics
- recommended scenarios (e.g., cap rate up/down)

Defaults should be treated as suggestions by hosts.

---

## 11. Loading packs

### 11.1 CLI
Recommended CLI behaviors:

- `cfdl pack list --path packs/`
- `cfdl pack validate --path packs/`
- `cfdl compile <model> --packs packs/`
- `cfdl run <ir> --packs packs/ --config run.json`

The CLI should allow:
- choosing pack search path
- selecting a subset of packs

### 11.2 Embedded usage (Rust)
Hosts can load packs by providing:
- filesystem root
- enabled pack list

---

## 12. Pack versioning and compatibility

Recommended rules:
- Packs have their own semantic versions.
- SDK v0.2 supports packs with declared compatibility range.

Example:
- SDK v0.2 loads packs compatible with `sdk >=0.2,<0.3`

This can be introduced incrementally.

---

## 13. CRE pack (initial scope)

The CRE pack should support the developer workflow:

- Construction
- Lease-up
- Stabilized operations
- Exit

Initial templates:
- Lease (simplified)
- Construction loan (simplified)
- Operating revenue/expense scaffolding
- Exit cap-rate

Required observables (examples):
- discount rate
- CPI growth (optional)
- exit cap distribution (for MC)

---

## 14. Operating Business pack (initial scope)

Initial templates:
- revenue line item
- COGS / opex
- working capital (simple)
- exit multiple

Required observables (examples):
- discount rate
- inflation (optional)
- terminal multiple

---

## 15. Testing packs

Packs must be tested via golden fixtures.

Recommended test types:

- pack load tests
- alias resolution tests
- template expansion tests
- lowering tests (contract → streams)
- end-to-end example fixtures (compile + run results)

For each pack, include:
- `examples/<pack>/...` models
- `fixtures/valid/...` that use the pack
- gold IR and results

---

## 16. Migration to EVS platform

EVS platform will:
- mount packs into the runtime
- use pack defaults to generate UI forms
- use pack templates to generate CFDL from the wizard
- use lowering to hydrate models from ontology contracts

The pack interface and determinism rules are what make this possible.

