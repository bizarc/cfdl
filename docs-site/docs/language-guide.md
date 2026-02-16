---
id: language-guide
title: "Language Guide"
slug: "/language-guide"
---

> This page is generated from `docs/LANGUAGE_GUIDE.md`.
> Source: https://github.com/bizarc/cfdl/blob/main/docs/LANGUAGE_GUIDE.md

This guide is the fastest path to writing valid CFDL models.

Use it as an onboarding guide. Use the specs in `docs/` as the final source of truth for exact grammar and semantics.

## Who this is for

- First-time CFDL authors
- Engineers onboarding to pack-based models
- Users moving from examples to production model structure

## What CFDL models

CFDL models cash-flow behavior with a deterministic language:

- **Time**: model timeline and optional phases
- **Structure**: entities (what things exist)
- **Behavior**: streams, contracts, events, options
- **Analysis**: assumptions, run mode, and metrics

## Minimum valid model

At minimum, a practical model should include:

- `version`
- `model`
- `time`
- one `entity`
- one behavior block (usually a `stream` or a pack-lowered `contract`)

Minimal example:

```cfdl
version 0.1
model "minimal-model"
time calendar monthly from 2026-01 for 12

entity legal borrower

stream legal.rent on entity legal.borrower {
  schedule every monthly from 2026-01 to 2026-12
  amount cel "1000"
}
```

See `examples/language_tutorial/minimal_model/model.cfdl`.

## Language elements (quick map)

### Header and modules

- `version 0.1`
- `model "name"`
- `use pack "<pack-id>" version "<pack-version>"`
- `import "relative_file.cfdl"`

### Time

- `time calendar monthly from 2026-01 for 72`
- `phase lease_up from 2026-01 to 2027-06`

### Structure

- `entity <namespace> <name>`
- Example: `entity real_estate property`
- Entity references use qualified names with at least two segments: `real_estate.property`, `org.real_estate.property`

### Behavior

- `stream` for direct cash-flow definitions
- `contract` for domain contracts (especially when using packs)
- `event` for conditional actions
- `option` for optional exercise behavior

### Naming conventions (recommended)

- Use dot notation for hierarchy and ownership boundaries.
  - Preferred: `cre.lease.base_rent`, `opco.working_capital.adjustment`
  - Allowed but less expressive: `ops_revenue`
- Use underscore only within a segment when needed (`working_capital`).
- Prefer qualified names for stream and contract instances in domain models.
- Keep entity symbols qualified and stable so ontology/data-source mappings can hydrate deterministically.

### When to use streams vs contracts

Use this when choosing streams or contracts:

| Situation | Use |
|-----------|-----|
| **Formal agreement with another party** (lease, loan, signed revenue agreement) | **Contract** |
| **Informal agreement** (handshake, memo, internal forecast) | **Contract or Stream** (either is acceptable) |
| **Individual expense (or revenue) items** (line-item opex, revenue line, one-off items) | **Stream** |
| **If in doubt** | **Start with a stream** |

See also: [Language Spec](/language-reference/language-spec) (Contracts §8 and Streams §9).

### Analysis

- `assume` for assumptions
- `run deterministic` or `run monte_carlo trials <n> seed <s>`
- `metric` for computed outputs

## Expressions and literals

CFDL uses CEL string expressions for executable expressions:

```cfdl
amount cel "inputs.base_rent * 1.02"
```

Common literals:

- Strings: `"hello"`
- Numbers: `1000`, `0.05`
- Dates: `2026-01`, `2026-01-15`
- Booleans: `true`, `false`

Notes:

- `YYYY-MM` dates are normalized to first day of month by compiler behavior.
- Keep expressions deterministic (no random/time/network behavior).

## Schedules (most common patterns)

### One-time payment

```cfdl
schedule on 2026-06
```

### Monthly recurring

```cfdl
schedule every monthly from 2026-01 to 2026-12
```

### Day rule example

```cfdl
schedule every monthly on day 15 from 2026-01 to 2026-12
```

## Packs: when and how to use them

Packs add domain behavior and validation while keeping core language stable.

Use a pack at the top of `model.cfdl`:

```cfdl
use pack "cre" version "0.1.0"
```

What packs commonly provide:

- Type and alias registries
- Contract `terms` validation
- Lowering rules (`contract` -> streams/events/options)
- Domain diagnostics

When to use packs:

- You want domain templates and validated `terms`
- You want contracts lowered automatically into executable effects

When not required:

- You are building a simple model with direct streams only

### Migrating from no-pack to pack

Use this sequence to migrate safely:

1. Keep existing timeline and entities unchanged.
2. Add `use pack "<id>" version "<ver>"` in `model.cfdl`.
3. Replace manual stream logic with pack-supported `contract` blocks gradually.
4. Run compile with `--packs packs` and resolve pack diagnostics.
5. Keep model behavior deterministic and verify expected IR/results deltas intentionally.

## Multi-file models

As models grow, split by concern:

- `model.cfdl` -> header and imports
- `time.cfdl` -> phases and timeline helpers
- `structure.cfdl` -> entities
- `contracts.cfdl` -> contracts/streams/events

Example import in `model.cfdl`:

```cfdl
import "time.cfdl"
import "structure.cfdl"
import "contracts.cfdl"
```

Rules to remember:

- Imports are relative to importing file
- Avoid cycles
- Do not import outside model root

## Common errors and fixes

- **Missing required model header fields**
  - Add `version`, `model`, and `time` once each.
- **Unresolved entity refs**
  - Confirm `entity` exists and reference uses a qualified name (`namespace.name` or deeper).
- **Schedule range issues**
  - Ensure `from <= to` and dates align with timeline.
- **Unterminated string/comment**
  - Close all quotes and block comments.
- **Pack validation errors**
  - Re-check `use pack` statement and contract `terms` shape.

For authoritative diagnostic codes, see [Diagnostics](/language-reference/diagnostics).

## Progressive tutorial examples

- `examples/language_tutorial/minimal_model/`
- `examples/language_tutorial/first_stream/`
- `examples/language_tutorial/simple_contract/`
- `examples/language_tutorial/with_pack/`
- `examples/language_tutorial/multi_file/`

## Recommended workflow

1. Start with `minimal_model`
2. Add a second stream and schedule variants (`first_stream`)
3. Move to contract-driven modeling with packs (`simple_contract` and `with_pack`)
4. Split into imports (`multi_file`)
5. Run compile/run and iterate diagnostics

## Authoritative references

- Core language: [Language Spec](/language-reference/language-spec)
- Grammar: [Grammar](/language-reference/grammar)
- Compiler behavior: [Compiler Spec](/language-reference/compiler-spec)
- Packs: [Pack Interface](/language-reference/pack-interface)
- Diagnostics: [Diagnostics](/language-reference/diagnostics)
- CLI usage: [SDK User Guide](https://github.com/bizarc/cfdl/blob/main/docs/USER_GUIDE.md)
