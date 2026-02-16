---
id: pack-interface
title: "Pack Interface (v0.1)"
slug: "/language-reference/pack-interface"
---

> This page is generated from `docs/pack_interface_v_0_1.md`.
> Source: https://github.com/bizarc/cfdl/blob/main/docs/pack_interface_v_0_1.md

**CFDL Domain Pack Interface v0.1**

**Status:** Draft

This document defines the contract between:
- the CFDL compiler/runtime toolchain, and
- **Domain Packs** (industry overlays)

Domain Packs provide *additions and overrides* similar to “industry clouds” (e.g., Financial Services Cloud) while preserving a single core language.

Core principle: **Packs may extend validation and provide defaults/templates, but MUST NOT change core language semantics.**

---

## 1) Goals

1. **Industry overlays**: Add types, aliases, validators, and lowering rules for a domain.
2. **Ontology linkage**: Packs expose type registries and canonical IDs for `obs()` and `ref()`.
3. **Deterministic compilation**: Pack version participates in determinism and reproducibility.
4. **Composable**: Multiple packs are not supported in v0.1; the interface should not block future multi-pack layering.
5. **Tooling-friendly**: Editors can query a pack for type/term help, docs, and autocomplete.

---

## 2) Pack selection in CFDL

CFDL models MAY select a pack:

```cfdl
use pack "evs/cre" version "0.1"
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
- Format recommendation: `publisher/name` (e.g., `evs/cre`, `evs/operating_business`)

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

## 5) Required pack manifest

A pack directory MUST include `pack.json`:

```json
{
  "pack_id": "evs/cre",
  "version": "0.1",
  "description": "Commercial Real Estate domain pack",
  "ir_versions": ["0.1"],
  "entrypoints": {
    "types": "ontology/types.json",
    "aliases": "aliases.json",
    "contract_schemas": "contracts/schemas.json",
    "lowering_rules": "contracts/lowering.json",
    "cel_extensions": "cel/extensions.json",
    "docs": "docs/index.json"
  }
}
```

Rules:
- `pack_id`, `version`, `ir_versions`, and `entrypoints.types` are REQUIRED.
- Other entrypoints are optional.

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
          "amount_expr": {"lang": "cel", "src": "terms.base_rent"}
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
- If lowering fails, emit `E500x` and fail compilation.

### 6.5 Ontology observable and reference IDs
Packs define canonical IDs for:
- `obs('<OntologyId>')`
- `ref('<OntologyId>')`

Packs MAY provide registries:
- `observables.json`
- `refs.json`

Compiler behavior:
- If pack provides registries, the compiler MAY validate that referenced IDs exist.
- Missing observable IDs SHOULD be warnings in v0.1 (allow offline modeling).

### 6.6 CEL extensions
A pack MAY add CEL functions or macros.

Rules:
- Pack functions MUST NOT override core function names.
- Pack functions MUST declare signatures.

Example:
```json
{
  "functions": [
    {"name": "pmt", "args": ["Money", "Rate", "Int"], "returns": "Money"}
  ]
}
```

### 6.7 Documentation metadata
Packs SHOULD provide docs for:
- types and fields
- contract templates
- examples

Editors may use this for hover hints and snippet insertion.

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
- `Pack.cel_extensions() -> Option<CelExtensions>`

### 7.2 Error behavior
- Pack not found: `E4004_MISSING_PACK`
- Pack manifest invalid: `E4004_MISSING_PACK` with details
- Unsupported IR version: `E4004_MISSING_PACK` with details

---

## 8) Determinism and provenance with packs

### 8.1 Determinism
Pack identity MUST participate in determinism:
- ID generation seed includes `pack_id@version` if present.

### 8.2 Provenance
The compiler SHOULD record pack info in top-level provenance notes.

---

## 9) Future-proofing (non-normative)

v0.2+ may add:
- multi-pack layering (base + overlays)
- signed pack artifacts
- executable lowering plugins (WASM)
- richer ontology reasoning

This v0.1 interface is designed to evolve without breaking core models.
