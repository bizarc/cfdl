# CFDL v0.1 — Language, Compiler, and Tooling

This repository contains the **CFDL v0.1** language documents (under `docs/`) and the implementation scaffolding for:

- **Rust**: lexer → parser → resolver → validator → compiler (IR emitter)
- **CLI**: `cfdl parse|validate|compile|fmt`
- **Tooling**: TypeScript + Python bindings and editor/notebook integrations
- **Packs**: Domain overlays (CRE, Operating Business, etc.) via a stable pack interface

The repository is designed for **agentic development**: small, testable steps with golden fixtures.

---

## Repository layout

You will start with an **empty root** containing only this `README.md` (and optionally `AGENTS.md` / `CLAUDE.md`) plus an `docs/` folder pre-populated with the specification documents.

### Proposed directory structure

```text
.
├─ README.md
├─ AGENTS.md                    # optional: agent instructions, task breakdown, guardrails
├─ docs/                       # provided: all specs created so far
│  ├─ CFDL_v0_1_Grammar.ebnf
│  ├─ CFDL_v0_1_IR.schema.json
│  ├─ CFDL_v0_1_Results.schema.json
│  ├─ compiler_spec_v0_1.md
│  ├─ diagnostics_spec.md
│  ├─ expr_env_v0_1.md
│  └─ pack_interface_v0_1.md
│
├─ crates/                       # Rust workspace
│  ├─ cfdl-lexer/
│  ├─ cfdl-parser/
│  ├─ cfdl-resolver/
│  ├─ cfdl-validate/
│  ├─ cfdl-compile/
│  └─ cfdl-cli/
│
├─ fixtures/
│  ├─ valid/
│  └─ invalid/
│
├─ gold/
│  ├─ ir/                         # expected canonical IR JSON outputs
│  └─ diag/                       # expected diagnostics outputs
│
├─ packs/                         # local domain packs (directory form)
│  ├─ evs-cre/
│  │  ├─ pack.json
│  │  └─ ...
│  └─ evs-operating-business/
│     ├─ pack.json
│     └─ ...
│
├─ bindings/
│  ├─ typescript/
│  └─ python/
│
└─ tools/
   ├─ golden-runner/              # compares emitted IR/diagnostics vs gold
   └─ json-canonicalize/          # ensures stable json output (if needed)
```

> Notes
> - `docs/` is treated as the source of truth for language behavior.
> - `crates/` is a Rust workspace with small crates per stage.
> - `fixtures/` and `gold/` enable deterministic golden tests.
> - `packs/` provides local pack folders for development.

---

## Build philosophy

1. **Deterministic**: identical inputs produce identical canonical IR.
2. **Separation of concerns**:
   - Language spec is not the implementation.
   - Packs extend validation and lowering; they do not change core semantics.
3. **Golden-driven development**:
   - Most compiler behavior is validated by fixture → expected IR/diagnostics.
4. **Ergonomics for analysts**:
   - Standalone `.cfdl` authoring + notebook workflows.

---

## Implementation milestones (recommended)

### Milestone 0 — Parsing
- Implement lexer with spans and comment support.
- Implement parser producing AST with spans (per `compiler_spec_v0_1.md`).
- Add `cfdl parse` CLI command.

### Milestone 1 — Imports + Symbol Resolution
- Implement import graph, cycle detection, deterministic ordering.
- Build symbol tables and resolve references.

### Milestone 2 — Validation
- Structural validation (required statements, duplicates, schedule bounds).
- Diagnostics produced per `diagnostics_spec.md`.

### Milestone 3 — Lowering + IR Emission
- Date/literal normalization, defaulting.
- Deterministic ID generation.
- `obs()` / `ref()` dependency extraction.
- Emit IR JSON conforming to `CFDL_v0_1_IR.schema.json`.

### Milestone 4 — CLI and Tooling
- `cfdl validate`, `cfdl compile`, `cfdl fmt`.
- TypeScript/Python bindings that:
  - call compiler/validator
  - parse diagnostics
  - load IR

### Milestone 5 — Domain Packs
- Implement pack loader + registries per `pack_interface_v0_1.md`.
- Create first packs:
  - `evs/cre`
  - `evs/operating_business`

---

## Canonical commands (planned)

### CLI
```bash
# Parse to AST (debug)
cfdl parse <model_root>

# Validate only (no IR emission)
cfdl validate <model_root>

# Compile to canonical IR JSON
cfdl compile <model_root> --out build/model.ir.json

# Format source (optional; future)
cfdl fmt <model_root>
```

### Golden runner
```bash
# Run all fixtures and compare output to gold/
./tools/golden-runner run
```

---

## Golden fixtures (how we test correctness)

- `fixtures/valid/*.cfdl` compile successfully and produce an IR JSON identical to `gold/ir/<name>.json`.
- `fixtures/invalid/*.cfdl` fail compilation and produce diagnostics identical to `gold/diag/<name>.diag.json`.

Minimum fixture set (from compiler spec):
- `minimal_model`
- `contract_with_effect_stream`
- `event_sets_entity_state`
- `phase_enter_schedule`
- `bad_duplicate_stream`
- `bad_missing_term`
- `bad_schedule_out_of_bounds`
- `obs_ref_extraction`

---

## Domain packs (industry overlays)

Domain packs behave like “industry clouds”:

- Core CFDL stays stable.
- Packs add:
  - ontology type registries
  - contract term schemas
  - aliases
  - declarative lowering rules (contract → streams/effects)
  - optional CEL function signatures

See `docs/pack_interface_v0_1.md`.

---

## Ontology and data integration (how CFDL links to external data)

CFDL expressions can call:
- `obs('...')` for time-varying observables
- `ref('...')` for static reference objects

The compiler extracts these IDs into:
- `required_observables[]`
- `required_refs[]`

This is the bridge to:
- data connectors
- data pipelines
- ontology registries

See `docs/expr_env_v0_1.md`.

---

## Specs index

All authoritative specifications live under `docs/`:

- `compiler_spec_v0_1.md` — stages, AST, validation, lowering, determinism
- `diagnostics_spec.md` — diagnostics shape + codes
- `expr_env_v0_1.md` — CEL environment contract
- `pack_interface_v0_1.md` — domain pack interface
- `CFDL_v0_1_Grammar.ebnf` — grammar
- `CFDL_v0_1_IR.schema.json` — canonical IR schema
- `CFDL_v0_1_Results.schema.json` — results schema

---

## Contributing / agentic workflow

When using agents:

- Work in **small PR-sized increments** (1–3 files per step).
- Every increment must add or update:
  - a fixture in `fixtures/` and
  - a gold artifact in `gold/` (IR or diagnostics)
- Never change stable error codes; add new ones if needed.
- Keep determinism: stable ordering + stable IDs.

If you include `AGENTS.md` / `CLAUDE.md`, place:
- task checklist by milestone
- coding standards
- “definition of done” for each crate

