# Changelog

All notable changes to this project will be documented in this file.

This project follows Semantic Versioning: https://semver.org/

---

## [0.2.7] - 2026-02-19

### Added
- **A.11 Showcase example fixtures**: Four production-quality demo fixtures with realistic deal
  economics for CRE and bespoke asset classes:
  - `cre_multifamily_100unit` — 100-unit stabilized suburban multifamily, 5-yr hold, 5.2% exit
    cap → $27.5M sale; base/downside scenarios. PASS anchors: t=0 +$119,350, t=59 +$27,661,657.
  - `cre_mixed_use_io_construction` — Ground-up mixed-use, $12M IO construction loan converting
    to permanent at month 36; three independent lease-up ramps (18/6/3 mo); 96-month model.
    PASS anchors: t=0 −$75,000, t=53 +$76,500, t=95 +$31,712,863.
  - `opco_professional_services` — B2B IT consulting; three compound-growth revenue lines
    (retainers 8%/yr, fees 12%/yr, licensing 18%/yr); five OpEx lines; PE exit 8.5× EBITDA with
    base/downside/upside scenarios. PASS anchors: t=0 +$91,000, t=12 +$132,900.
  - `bespoke_oil_gas_ep` — Upstream E&P single-well; drilling/ramp/production phases; $3.5M capex
    + $250k/mo drilling costs; 6%/mo hyperbolic production decline; 18.75% royalty; LOE and G&A.
    PASS anchors: t=0 −$3,762,000, t=12 +$106,250.
- Golden suite extended from 38 to **42 fixtures** (PASS=42 FAIL=0).

---

## [0.2.6] - 2026-02-19

### Added
- **A.8 Annual rollup series**: `DeterministicSection` now includes an `annual_rollup` field
  (omitted for already-annual models) containing calendar-year summed aggregates for every
  per-period series (`model.net_cash_flow` + all `stream.*`).
  - `AnnualRollupSection` follows the same `{ series: { ...: { index, values } } }` schema as
    the existing `series` block; `index.calendar = "annual"`.
  - Computed via `build_annual_rollup()` inside `run_deterministic`.
  - All 38 golden fixtures at the time updated to include the new section.

---

## [0.2.5] - 2026-02-19

### Added
- **A.6 OpCo `growth_rate` compound formula**: The `opco.revenue_line` lowering rule now injects
  `pow(1 + growth_rate, time.t / 12.0)` into the revenue CEL expression when `growth_rate` is
  provided in contract terms. Previously the term was validated but silently ignored. `opco_growth_smoke`
  golden fixture added to lock behavior.
- **A.7 Bespoke model showcase**: `fixtures/valid/bespoke_saas/model.cfdl` — a 36-period SaaS
  startup demonstrating phases (`ramp`/`growth`/`scale`), `phase_enter` scheduling, `pow`-based
  compound growth, linear cost expressions, `active when` guards, and multiple entities. Golden
  suite extended to 38 fixtures.

---

## [0.2.4] - 2026-02-19

### Added
- **A.1 Phase schedule execution**: `PhaseEnter` and `EveryPhase` AST variants now resolve to
  concrete `OnDate`/`Every` schedule entries at compile time, using the phase date ranges declared
  in the model. Previously phase-keyed streams were silently dropped.
- **A.2 Observable binding**: The engine `obs` map is now populated from `run.json` before CEL
  evaluation; observable expressions in streams can reference `obs.<key>` correctly.
- **A.3 Exit cap / terminal value**: `cre.exit_cap` lowering is fully data-driven — `noi_value`
  and `exit_cap` are read from contract terms and the sale amount `noi_value / exit_cap` is
  substituted as the stream amount at the exit date. `exit_cap_smoke` golden fixture added.
- **A.4 NOI aggregation path**: Compiler validation confirms that CRE models declare exactly one
  ops-revenue and one ops-expense stream; `noi_smoke` golden fixture locks the gross-revenue −
  vacancy − opex = NOI aggregation path end-to-end.
- **A.5 IRR bisection solver**: `cfdl-engine` emits `model.irr` (annualized, via Newton/bisection)
  in the deterministic results block. Undefined when all cash flows are same-sign. `model.irr`
  locked in gold for IRR-bearing fixtures.
- **A.10 Annual discount rate convention**: `annual_discount_rate` in `run.json` is now the sole
  discount rate field. For monthly models the engine converts via `(1+r)^(1/12) - 1` before
  discounting. All fixture `run.json` files updated from the old per-period `discount_rate` field.
- **`cfdl-metrics` crate** — post-engine domain metrics service that reads
  engine `Results` stream totals and emits pack-specific KPIs with full lineage
  into a new `domain_metrics` field on `Results`.
  - CRE metrics: `domain.cre.noi`, `domain.cre.debt_service`,
    `domain.cre.dscr` (DSCR emitted only when debt service > 0)
  - OpCo metrics: `domain.opco.revenue`, `domain.opco.ebitda`,
    `domain.opco.ebitda_margin`
  - Each metric carries a `MetricLineage` record (numerator/denominator
    streams + formula string) for auditability
- **`DomainMetrics` / `MetricLineage` structs** added to `cfdl-engine` and
  exported; `domain_metrics: Option<DomainMetrics>` field on `Results` uses
  `#[serde(skip_serializing_if = "Option::is_none")]` so models without pack
  context produce identical output to before.
- **`--pack <name>` CLI arg** on `cfdl run` — triggers metrics computation
  and attaches the result to the serialized output.
- **`dscr_smoke` fixture** — minimal 24-month CRE model with revenue (30k/mo),
  expense (10k/mo), and debt service (15k/mo) streams; gold locks
  `domain.cre.noi = 480,000`, `domain.cre.debt_service = 360,000`,
  `domain.cre.dscr = 1.333333`.
- **Golden runner `pack` sentinel** — fixtures may include a `pack` file;
  the runner reads it and passes `--pack` to `cfdl run` automatically.

### Changed
- `results_version` bumped from `"0.1"` to `"0.2"` in all gold results files.

---

## [0.2.3] - 2026-02-16

### Added
- Contract subject parsing support for `contract ... on entity <EntityRef>`.
- Contract subject entity resolution checks with unresolved entity diagnostics.
- New fixtures and goldens:
  - `fixtures/valid/cre_subject_non_first_entity`
  - `fixtures/valid/cre_subject_implicit_fallback`
  - `fixtures/invalid/unresolved_contract_subject_ref`

### Changed
- IR contract `subject` now uses contract subject when present.
- Pack-lowered stream owner selection now supports subject-driven ownership (`owner_entity = "${subject}"`) with backward-compatible fallback for omitted subject.
- CRE and OpCo pack lowering rules were migrated from hardcoded owners to subject-based ownership defaults.
- CRE/OpCo pack documentation and examples were updated to align with subject-based lowering behavior.

### Fixed
- Deterministic compatibility path for existing fixtures that omit `on entity`.
- Golden coverage for subject-based lowering behavior and unresolved contract subject diagnostics.

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