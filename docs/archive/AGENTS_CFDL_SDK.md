# AGENTS.md

This file is the execution playbook for agentic development of **CFDL v0.1**.

It provides:
- Work breakdown by milestone
- Acceptance criteria (“definition of done”)
- Guardrails (determinism, diagnostics stability)
- Suggested crate-by-crate tasks for Rust

**Authoritative specs are in `docs/`.** Implementation must conform to:
- `docs/compiler_spec_v0_1.md`
- `docs/diagnostics_spec.md`
- `docs/CFDL_v0_1_Grammar.ebnf`
- `docs/CFDL_v0_1_IR.schema.json`
- `docs/pack_interface_v0_1.md`
- `docs/cel_and_expr_env_v_0_2.md`

---

## 0) Global rules (non-negotiable)

### 0.1 Determinism
- Same sources + same pack + same compiler version → identical canonical IR.
- Arrays must be emitted in deterministic order (see compiler spec).
- IDs are deterministic (hash-based) and stable.

### 0.2 Diagnostics stability
- Never change or reuse existing diagnostic codes.
- If wording changes, golden tests should assert message via substring match only.
- Diagnostics must include file + span for parse/validation failures.

### 0.3 Golden-first development
Every meaningful capability must add:
- at least one fixture under `fixtures/`
- a corresponding gold output under `gold/`

### 0.4 No correlation
Do not add correlation to IR or language.

### 0.5 Minimal external dependencies
- Prefer small, well-known crates.
- Avoid complex parser frameworks unless necessary.
- Keep the compiler library-first; CLI is thin.

---

## 1) Milestone plan

### Milestone 0 — Repo scaffolding
**Tasks**
1. Create Rust workspace `Cargo.toml` at repo root.
2. Create empty crates:
   - `crates/cfdl-lexer`
   - `crates/cfdl-parser`
   - `crates/cfdl-resolver`
   - `crates/cfdl-validate`
   - `crates/cfdl-compile`
   - `crates/cfdl-cli`
3. Add a basic `tools/golden-runner` (can be Rust or script).

**DoD**
- `cargo test` passes.
- `cfdl --help` runs.

---

### Milestone 1 — Lexer
**Target crate:** `cfdl-lexer`

**Tasks**
1. Implement token enum:
   - keywords, identifiers, qnames, numbers, strings, dates, punctuation
2. Implement comment handling:
   - `//` line comments
   - `/* */` block comments
3. Implement span tracking per token.

**Fixtures to add**
- `fixtures/valid/lex_smoke.cfdl`
- `gold/diag/lex_smoke.tokens.json` (optional)

**DoD**
- Lexer produces stable token stream with spans.
- Unterminated string and unterminated block comment produce correct errors:
  - `E0002_UNTERMINATED_STRING`
  - `E0003_UNTERMINATED_BLOCK_COMMENT`

---

### Milestone 2 — Parser → AST
**Target crate:** `cfdl-parser`

**Tasks**
1. Define AST structs/enums (mirror `docs/compiler_spec_v0_1.md`).
2. Implement parser for:
   - version/model/time/phase/entity
   - assume
   - stream
   - contract (terms + effects with streams)
   - event + actions
   - option
   - run + metric
3. Parse errors + recovery:
   - `E0001_UNEXPECTED_TOKEN`
   - `E0004_EXPECTED_TOKEN`
   - `E0005_INVALID_DATE_LITERAL`

**Fixtures to add**
- `fixtures/valid/minimal_model/model.cfdl`
- `fixtures/invalid/parse_error_unexpected_token/model.cfdl`
- Gold diagnostics for invalid parse fixture.

**DoD**
- Parser produces AST with spans.
- Invalid syntax yields diagnostics with file/span.

---

### Milestone 3 — Imports + module graph
**Target crate:** `cfdl-resolver`

**Tasks**
1. Implement import resolution:
   - resolve relative paths
   - prohibit escaping model root
2. Detect cycles:
   - `E1201_IMPORT_CYCLE`
3. Deterministic topo ordering:
   - lexicographically smallest order among valid topological sorts
4. Merge modules into a single `CompilationUnit`.

**Fixtures to add**
- `fixtures/valid/import_simple/`
- `fixtures/invalid/import_cycle/`

**DoD**
- Import graph is deterministic.
- Cycle and missing module errors are correct:
  - `E1201_IMPORT_CYCLE`
  - `E1202_IMPORT_NOT_FOUND`
  - `E1203_IMPORT_OUTSIDE_MODEL_ROOT`

---

### Milestone 4 — Symbol tables + reference resolution
**Target crate:** `cfdl-resolver`

**Tasks**
1. Build symbol tables:
   - entities, phases, streams, contracts, options, events, metrics, assumptions
2. Enforce uniqueness:
   - `E1001..E1008`
3. Resolve references:
   - entity refs, stream refs, contract refs, option refs, phase refs

**Fixtures to add**
- `fixtures/invalid/dup_stream/`
- `fixtures/invalid/unresolved_entity_ref/`

**DoD**
- All duplicates and unresolved refs yield correct diagnostics codes.

---

### Milestone 5 — Validation
**Target crate:** `cfdl-validate`

**Tasks**
1. Enforce required global statements:
   - missing/multiple `version/model/time`
2. Contract rules:
   - `term` mandatory (`E2001_CONTRACT_MISSING_TERM`)
   - `effects` required unless pack lowering exists (`E2002_CONTRACT_MISSING_EFFECTS`)
3. Stream rules:
   - schedule and amount required (`E2101`, `E2102`)
4. Schedule validation:
   - bounds within model timeline (`E2103`)
   - from<=to (`E2104`)
   - day-of-month validity (`E2105`)
   - phase existence (`E2106`)
5. Boolean slot typing (minimal):
   - empty expr is error
   - allow unknown types with `W3001_EXPR_TYPE_UNKNOWN`

**Fixtures to add**
- `fixtures/invalid/missing_time/`
- `fixtures/invalid/bad_schedule_out_of_bounds/`
- `fixtures/invalid/bad_missing_term/`

**DoD**
- Validation produces stable diagnostics.
- Compiler does not emit IR if any errors exist.

---

### Milestone 6 — Lowering + IR emission
**Target crate:** `cfdl-compile`

**Tasks**
1. Normalize dates:
   - `YYYY-MM` → `YYYY-MM-01`
2. Defaulting:
   - `active_when` defaults to `cel "true"`
3. Deterministic ID generation (hash-based).
4. Provenance propagation:
   - NodeProvenance for all nodes
   - top-level provenance sources sorted
5. Extract `obs()` / `ref()` IDs from CEL strings:
   - populate `required_observables`, `required_refs`
6. Emit canonical IR JSON matching schema.
7. Optionally validate IR JSON against schema in tests.

**Fixtures to add**
- `fixtures/valid/contract_with_effect_stream/`
- `fixtures/valid/obs_ref_extraction/`

**DoD**
- `cfdl compile` outputs deterministic JSON.
- Gold IR matches exactly.

---

### Milestone 7 — CLI
**Target crate:** `cfdl-cli`

**Commands**
- `cfdl parse <model_root>`
- `cfdl validate <model_root>`
- `cfdl compile <model_root> --out <file>`

**DoD**
- CLI returns non-zero exit code on error.
- Can output diagnostics as JSON (`--json`) and as human text by default.

---

### Milestone 8 — Pack loader (v0.1)
**Target crate:** `cfdl-resolver` or `cfdl-compile` (whichever owns configuration)

**Tasks**
1. Parse `use pack` statement.
2. Load `packs/<packdir>/pack.json`.
3. Load type registry + optional schemas/lowering rules.
4. Validate:
   - unknown type IDs (`E4001_UNKNOWN_TYPE_ID`)
   - invalid terms (`E4003_INVALID_CONTRACT_TERMS`)

**DoD**
- Pack not found → `E4004_MISSING_PACK`.
- Pack participates in determinism seed.

---

## 2) Golden runner requirements

Golden runner must support:
- running compile on each `fixtures/valid/*` directory and comparing IR to `gold/ir/<name>.json`
- running compile on each `fixtures/invalid/*` directory and comparing diagnostics to `gold/diag/<name>.diag.json`

Comparison rules:
- IR JSON: strict equality after canonicalization
- Diagnostics:
  - match `code`, `severity`, `file`, `span`
  - message match by substring list (optional)

---

## 3) Coding standards

### 3.1 Rust
- Prefer `thiserror` for errors.
- Diagnostics are data; do not panic.
- Avoid global state; pass config explicitly.

### 3.2 Logging
- Use `tracing` (optional).
- Do not log secrets or full model content.

### 3.3 Public API
Library crates should expose:
- `compile(model_root, options) -> Result<Ir, Vec<Diagnostic>>`
- `validate(model_root, options) -> Result<(), Vec<Diagnostic>>`

---

## 4) Suggested initial fixtures (copy/paste plan)

Create these fixture directories:
- `fixtures/valid/minimal_model/`
- `fixtures/valid/contract_with_effect_stream/`
- `fixtures/valid/event_sets_entity_state/`
- `fixtures/valid/phase_enter_schedule/`
- `fixtures/valid/obs_ref_extraction/`

And invalid:
- `fixtures/invalid/parse_error_unexpected_token/`
- `fixtures/invalid/import_cycle/`
- `fixtures/invalid/dup_stream/`
- `fixtures/invalid/bad_missing_term/`
- `fixtures/invalid/bad_schedule_out_of_bounds/`
- `fixtures/invalid/unresolved_entity_ref/`

Each fixture must contain a `model.cfdl` and any imported `.cfdl` files.

---

## 5) Agent instructions (how to work)

1. Pick the next smallest task from the milestones.
2. Implement only what is required for that task.
3. Add or update fixtures and gold outputs.
4. Run tests and golden runner.
5. Commit with a message like:
   - `feat(parser): parse contract effects streams`
   - `fix(validate): enforce schedule bounds`

---

## 6) Definition of done (global)

The project is “v0.1 complete” when:
- All milestones through **Milestone 7** are complete.
- Golden suite passes.
- IR and diagnostics match the specs.
- Pack loader works for local directory packs.

