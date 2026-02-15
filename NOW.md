# NOW (CFDL SDK) — v0.2.0 Milestones 9–13

This NOW.md defines the active roadmap for **CFDL SDK v0.2.0**. It is designed for agentic execution.

Source-of-truth contracts:
- Surface grammar: `docs/CFDL_v0_1_Grammar.ebnf.md`
- IR contract: `docs/CFDL_v0_1_IR.schema.json` (mirrored in `docs/cfdl_v_0_1_ir_schema.md`)
- Results contract: `docs/CFDL_v0_1_Results.schema.json` (mirrored in `docs/cfdl_v_0_1_results_schema.md`)
- Diagnostics codes: `docs/diagnostics_spec.md`

Rules:
1) Do not invent new syntax; the parser must converge to the EBNF.
2) Preserve determinism: stable ordering, stable IDs, reproducible runs.
3) All new behavior must come with fixtures + gold updates.

---

## Milestone 9 — CEL Expressions + Typed ExprEnv

### Goal
Add real CEL parsing/evaluation with a typed environment so expressions can be used for stream amounts and predicates.

### Scope
- Expressions remain the v0.1 grammar form: `cel "..."` (string literal).
- Implement `Money`, `Date`, `Decimal`, `Optional` and enforce type safety.
- Provide host functions for basic date math and numeric helpers.
- Observables access via deterministic host functions.

### Deliverables
- `crates/cfdl-expr/` crate
  - typed value model
  - CEL compile/eval API
  - `ExprEnv` namespaces: `model.*`, `time.*`, `entity.*`, `cfg.*`, `obs.*`
- Engine integration
  - evaluate stream `amount` per timestep
  - evaluate `active when` predicates
- Diagnostics
  - Add: `E3001`–`E3004` (parse/unknown/type/illegal op) to `docs/diagnostics_spec.md`
- Fixtures + gold
  - `fixtures/valid/expr_smoke/` + `gold/ir/*` + `gold/results/*`
  - `fixtures/invalid/expr_type_error/` + `gold/diag/*`

### Acceptance criteria
- `make fmt && make lint && make test && make gold` all pass.
- Deterministic results across repeated runs.

---

## Milestone 10 — Pack Host + Loader + Lowering Stage (Done)

### Goal
Load packs from filesystem and apply deterministic lowering (contracts/templates → streams/events/options) with provenance.

### Scope
- Support `use pack "<name>" version "<semver>"` per grammar.
- Implement a filesystem pack loader.
- Implement a minimal lowering stage used during compile/run.

### Deliverables
- `crates/cfdl-pack/`
  - pack manifest model (`pack.toml`)
  - registry + filesystem loader
  - stable pack interface (aliases, lowering hooks; templates can be stubbed)
- Compiler integration
  - apply lowering after symbols/validation
  - preserve provenance: pack name/version + rule id + source contract reference
- CLI integration
  - `--packs <dir>` for compile/run (or default `packs/`)
  - `cfdl pack list --path <dir>`
- Fixtures + gold
  - `packs/testpack/` (minimal)
  - `fixtures/valid/use_pack_smoke/` + IR/results gold

### Acceptance criteria
- Pack loading deterministic ordering.
- Model compiles/runs with packs enabled.
- Golden runner covers lowering outputs.

---

## Milestone 11 — CRE Pack v0.1 (Developer workflow) (Current)

### Goal
Implement `packs/cre` to support a CRE developer lifecycle: construction → lease-up → stabilized ops → exit.

### Scope
- Minimal templates/aliases and lowering rules:
  - Lease → rent streams
  - Exit cap / sale event → terminal inflow stream
  - Construction interest-only or draw stream scaffolding
- CRE validations (E6xxx)

### Deliverables
- `packs/cre/`
  - `pack.toml`, `aliases.toml`, `templates.toml`, `lowering/rules.toml`, `validations.toml`, `defaults.toml`, `README.md`
- Examples + gold
  - `examples/cre_developer/` with `model.cfdl` and run configs
  - gold IR + results
- Fixtures
  - at least one `fixtures/valid/cre_developer_smoke/` wired into gold runner

### Acceptance criteria
- Example compiles and runs deterministically.
- Lowered streams carry provenance from contract + pack.

---

## Milestone 12 — Operating Business Pack v0.1

### Goal
Implement `packs/opco` for operating business valuation: revenue/opex/WC + exit.

### Scope
- Minimal templates/aliases and lowering rules:
  - revenue line inflows
  - opex outflows
  - optional working capital adjustment stream
  - exit multiple → terminal inflow stream
- OpCo validations (E7xxx)

### Deliverables
- `packs/opco/`
  - `pack.toml`, `aliases.toml`, `templates.toml`, `lowering/rules.toml`, `validations.toml`, `defaults.toml`, `README.md`
- Examples + gold
  - `examples/opco_basic/` with `model.cfdl` and run configs
  - gold IR + results
- Fixtures
  - at least one `fixtures/valid/opco_smoke/` wired into gold runner

### Acceptance criteria
- Example compiles and runs deterministically.
- Pack validations produce stable diagnostics.

---

## Milestone 13 — Python Package + Docs + Examples in CI

### Goal
Make the SDK usable by others and ready for scenario testing while EVS platform work begins.

### Scope
- Provide an installable Python package.
- Ensure docs match real CLI/grammar.
- Ensure examples are validated in CI.

### Deliverables
- Python
  - Preferred: PyO3 + maturin bindings exposing compile/run
  - Fallback: Python wrapper calling `cfdl` binary
- Documentation
  - Update `docs/USER_GUIDE.md` to match actual CLI flags and grammar
  - Ensure `docs/PACKS_GUIDE.md` is consistent with pack loader implementation
- CI
  - Add workflow steps to build/test Python integration
  - Add smoke tests that run `examples/cre_developer` and `examples/opco_basic`

### Acceptance criteria
- A user can install and run an example from Python.
- CI passes; golden runner remains authoritative.

---

## Work sequencing (recommended)

1) Milestone 9 (CEL) — unlock expressive models and scenario testing.
2) Milestone 10 (Packs host) — create stable boundary for domains.
3) Milestone 11 (CRE pack) — first “real deal” example.
4) Milestone 12 (OpCo pack) — second “cloud.”
5) Milestone 13 (Python + docs + CI) — usability and adoption.

---

## Definition of Done for v0.2.0

v0.2.0 is complete when:
- CEL expressions are supported and golden-tested.
- Packs load from filesystem and lowering is deterministic with provenance.
- CRE and OpCo packs exist with at least one example each.
- CLI supports packs + run configs.
- Python package can compile/run examples.
- All tests + gold pass in CI.

