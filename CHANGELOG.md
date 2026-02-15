# Changelog

All notable changes to this project will be documented in this file.

This project follows Semantic Versioning: https://semver.org/

---

## [0.2.0] - 2026-02-15

### Overview
CFDL SDK v0.2.0 delivers an end-to-end, deterministic, pack-enabled cash flow modeling toolchain:
- A human-readable CFDL authoring format and deterministic compiler
- Deterministic IR emission with provenance
- Deterministic results engine with discounting and scenario support
- Filesystem “domain packs” with lowering-time validation and diagnostics
- Golden fixtures harness and CI validation
- A PyO3-based Python package for direct compile/run integration (no CLI shell-outs)

### Highlights
- **Domain Packs (Filesystem “Clouds”)**
  - Pack registry loader and `use pack "<name>" version "<semver>"` support
  - Deterministic lowering and provenance for generated streams
  - CRE pack (`packs/cre`) and Operating Business pack (`packs/opco`) included

- **Deterministic toolchain**
  - Stable ordering of outputs and diagnostics
  - Stable, hash-based IDs designed for reproducibility across runs

- **SDK-ready distribution**
  - PyO3 + maturin Python package providing `compile_model()` and `run_ir()` APIs
  - CI runs Rust checks, golden runner, and Python smoke tests against examples

### Added
- **Compiler pipeline**
  - Lexer + parser scaffolding with structured diagnostics
  - Import resolution with module graph, outside-root protection, and cycle detection
  - Symbol resolution (duplicate entities/streams, unresolved refs)
  - Validation pass for required global blocks and core stream/contract requirements
  - Deterministic IR emission aligned with the IR schema

- **CLI**
  - `cfdl compile <model_dir> --out <ir.json> [--packs <dir>]`
  - `cfdl run <ir.json> --out <results.json> [--packs <dir>] [--rate <decimal>] [--as-of <YYYY-MM-DD>]`
  - `cfdl pack list --path <packs_dir>`

- **Domain packs**
  - `packs/cre` v0.1.0
    - Deterministic lowering for construction/lease/ops/exit patterns
    - Lease-up behavior support
    - Lowering-time CRE validations (E6xxx)
    - Onboarding example: `examples/cre_developer`
  - `packs/opco` v0.1.0
    - Deterministic lowering for revenue/opex/working capital/exit multiple
    - Lowering-time OpCo validations (E7xxx)
    - Onboarding example: `examples/opco_basic`

- **Lowering-time validations**
  - Packs may emit diagnostics during lowering; compilation fails deterministically on lowering errors (no IR emitted)
  - CRE: E6xxx pack-origin diagnostics
  - OpCo: E7xxx pack-origin diagnostics

- **Python SDK**
  - `crates/cfdl-py` PyO3 extension + `python/` package using maturin
  - Public API:
    - `compile_model(model_dir, packs_dir=None) -> str` (IR JSON)
    - `run_ir(ir_json, packs_dir=None, config_json=None) -> str` (Results JSON)
  - Python smoke tests compile/run `examples/cre_developer` and `examples/opco_basic`

- **Docs**
  - Canonical user guide aligned to current CLI and examples: `docs/USER_GUIDE.md`
  - Diagnostics spec expanded for pack-origin codes: `docs/diagnostics_spec.md`
  - Canonical results schema file: `docs/CFDL_v0_1_Results.schema.json`
  - Pack guidance and interfaces documented in `docs/`

- **Testing & CI**
  - Golden runner supports:
    - compile → IR gold comparisons
    - run → results gold comparisons
    - invalid fixtures → diagnostics gold comparisons
  - CI executes Rust fmt/lint/test, golden runner, and Python install/tests

### Changed
- **Diagnostics semantics**
  - Validation vs emit failures are separated:
    - Missing-entity is treated as validation (not schema failure)
    - Emit/write failures use a dedicated emit-failure diagnostic
- **Deterministic IDs**
  - IDs are derived from stable keys (including source-relative path + name/symbol) and SHA-256 truncation
  - ID scheme documented in `docs/`

### Fixed
- Deterministic behavior across compile/run paths (stable ordering and rounding policy at output boundaries)
- Hard-fail compilation on lowering-time pack validation errors to prevent “plausible” but invalid outputs

### Known limitations / Notes
- Packs currently rely on deterministic lowering rules and validated term capture; future expansions may add richer typing, contract algebra primitives, and deeper ontology hydration.
- Correlation configuration is intentionally excluded from the language/IR at this stage (computed downstream as needed).

---

## [0.1.0] - (unreleased)
Initial scaffolding and specification work leading into the v0.2.0 SDK release.