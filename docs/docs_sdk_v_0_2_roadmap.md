# CFDL SDK v0.2 Roadmap

This document defines what it means for the **CFDL SDK** to be “complete enough” at **v0.2.0** while EVS platform work begins in parallel.

v0.2.0 focuses on four upgrades:

1) **Expressions (CEL) + Typed Environment** (so real models can be authored ergonomically)
2) **Pack System (interfaces + loader + lowering)** (so domain logic lives outside core)
3) **First Domain Packs:** **CRE** and **Operating Business** (so contracts hydrate into streams)
4) **Usability:** CLI + examples + Python package + user guide (so others can use it)

---

## Why v0.2.0 exists

v0.1 proved the thin waist:

> CFDL → Compiler → IR → Engine → Results

v0.2 makes the SDK useful for real modeling and early adopters by adding:
- expression-driven amounts/terms
- pack-driven contract hydration
- domain templates
- a consumable developer experience

---

## Scope decisions

### A) Finance functions (NPV/IRR/stats): CEL vs Engine

**Default position for v0.2**
- Keep **NPV/IRR/stats** in the **engine / results layer**.
- Use CEL for **model definition**: amounts, conditional logic, schedules, term activation, basic transforms.

**Pros of keeping finance functions in the Engine**
- Clear separation of concerns: CEL defines *inputs*, engines produce *outputs*.
- Easier to optimize and ensure determinism for heavy compute (e.g., 50k trials).
- Avoids making CEL a “kitchen sink” language with valuation semantics.
- Metrics can evolve without changing the language surface.

**Cons**
- Users can’t express custom derived measures inside the model (without adding a results/metrics layer).

**When finance functions in CEL help**
- If you want users to define **derived signals** (e.g., DSCR, debt yield) inline.
- If you want contracts/templates to compute values with reusable helpers.

**Recommended compromise**
- v0.2: **do not add NPV/IRR** to CEL.
- Add a small set of **safe utility functions** to CEL only when needed for usability:
  - date math (`add_months`, `days_between`, `yearfrac`)
  - numeric helpers (`min`, `max`, `clamp`, `round`)
  - collection helpers (lightweight)
- Define a separate **metrics DSL** later (or pack-provided metrics) if needed.

---

### B) Money type: where currency lives

**Recommendation for v0.2**
- Introduce a first-class **Money** type in the expression environment and IR:
  - `Money { amount: Decimal, currency: CurrencyCode }`
- Treat raw numbers as **dimensionless**; streams/amounts resolve to Money.

**Impact / why it matters**
- Multi-currency is a core enterprise requirement.
- It prevents silent category errors (adding USD to EUR).
- It enables consistent FX conversion policy (engine-level) without rewriting models.

**Design implications**
- CEL type system must support Money (either via host types or wrapping).
- Engine must implement FX conversion using an `FxProvider` abstraction.
- Results must specify:
  - reporting currency
  - conversion policy (spot vs average vs curve) (even if only one is implemented initially)

**Alternative (simpler, less safe)**
- Keep currency as a stream attribute and treat CEL outputs as numbers.
- Pros: faster to implement.
- Cons: pushes unit safety into conventions, increases enterprise risk.

**v0.2 stance**
- Implement Money now (minimal). Use one FX policy initially.

---

### C) Pack distribution model: filesystem vs compiled

**Best practice (long-term)**
- Packs are **separately versioned modules** loaded by the host (filesystem or registry), not hard-coded.

**Most like Salesforce “Clouds”**
- A shared core model with **managed packages** that add:
  - objects / aliases
  - validations
  - workflows/actions
  - UI schemas
- This maps to: packs are **versioned artifacts** loaded into an environment.

**Most like Palantir**
- Core platform + **ontology + pipelines** + “applications” configured per domain.
- Packs resemble: domain modules that define schemas, transformations, constraints, and derived objects.

**Recommendation for CFDL SDK v0.2**
- Implement a **filesystem pack loader** first:
  - packs live under `packs/<pack_name>/`
  - a `pack.toml` (or `pack.json`) declares:
    - pack name/version
    - provided aliases/templates
    - lowering rules entrypoint
    - default observables/metrics
- Support a “compiled/bundled” mode later for distribution.

**Implications**
- Filesystem packs are easiest for iteration and testing.
- EVS platform can later sync packs from a registry and mount them into the runtime.

---

### D) Python: true pip package vs in-repo wrapper

**Difference in implementation**

1) **In-repo wrapper (fastest)**
- A Python module in `python/` that shells out to `cfdl` or uses pyo3 bindings later.
- Pros: minimal packaging overhead.
- Cons: not a real installable dependency, harder for external use.

2) **True pip package (recommended for adoption)**
- Publish a `cfdl-sdk` package that provides:
  - `compile()` / `run()` / `validate()` functions
  - loads packs from filesystem
  - optionally bundles a platform-specific `cfdl` binary or uses Rust bindings

**Approach options**
- **Option A (fast + robust): ship the Rust binary and call it from Python**
  - Python package includes a small wrapper that downloads/locates `cfdl`.
  - Pros: simplest, keeps Rust as source of truth.
  - Cons: binary distribution complexity per OS.

- **Option B (best UX): PyO3/maturin bindings**
  - Build Python extension modules from Rust (`maturin`), expose compile/run APIs.
  - Pros: very nice dev UX, no subprocess.
  - Cons: more build complexity; still very doable.

**Recommendation for v0.2**
- Deliver a real pip package using **PyO3 + maturin** if possible.
- If time is tight, do Option A first, then migrate.

---

## v0.2.0 Milestones

### Milestone 9 — CEL + Typed Expression Environment
**Goal:** expressions are usable for stream amounts, terms activation, schedule params, and simple conditionals.

Deliverables:
- `cfdl-expr` crate
- `ExprEnv` spec + implementation
- CEL integration (parse/eval)
- Diagnostics for expr errors (parse/type/missing symbol)
- Golden fixtures for expr evaluation cases

### Milestone 10 — Pack Host + Lowering Pipeline
**Goal:** core can load packs and apply lowering rules deterministically.

Deliverables:
- `cfdl-pack` interface crate (traits + schemas)
- Filesystem pack loader
- Lowering stage in compile/run pipeline:
  - contracts + terms → streams/events/options
- Provenance preserved:
  - “this stream was generated by lowering contract X via pack Y”
- Golden fixtures: pack lowering deterministic outputs

### Milestone 11 — CRE Pack (v0.1)
**Goal:** CRE developer workflow primitives exist (construction → lease-up → ops → exit).

Deliverables:
- Pack aliases + templates:
  - Lease (simplified)
  - Construction period + draws
  - Operating revenue/expense scaffolding
  - Exit (cap-rate)
- Lowering rules that generate streams and required observables
- Example models + gold results (scenario + MC)

### Milestone 12 — Operating Business Pack (v0.1)
**Goal:** OpCo templates for revenue/opex/working capital + exit.

Deliverables:
- Templates:
  - Revenue line items
  - COGS/Opex
  - Working capital simple schedule
  - Exit multiple / terminal value
- Lowering rules and defaults
- Example models + gold results

### Milestone 13 — User Guide + Python Package + Examples
**Goal:** someone can install and use the SDK and packs without reading source.

Deliverables:
- `docs/USER_GUIDE.md`
- `docs/PACKS_GUIDE.md`
- `examples/` for CRE and OpCo
- Python package:
  - build/install instructions
  - minimal API surface

---

## Proposed Directory Additions (SDK repo)

- `crates/cfdl-expr/`
- `crates/cfdl-pack/`
- `packs/cre/`
- `packs/opco/`
- `examples/cre_developer/`
- `examples/opco_basic/`
- `python/` (or `bindings/python/`)
- `docs/USER_GUIDE.md`
- `docs/PACKS_GUIDE.md`

---

## Definition of Done for v0.2.0

v0.2.0 is “SDK complete enough” when:

- CEL expressions are supported and golden-tested.
- Packs can be loaded from filesystem and lowering is deterministic.
- CRE and OpCo packs exist with at least one example each.
- CLI supports running with packs and run configs.
- A user can install the Python package and run an example end-to-end.

---

## Relationship to EVS Platform

The EVS platform (`evs-platform` repo) depends on the SDK:
- compile CFDL to IR
- run engines deterministically
- load packs (domain logic)

The platform adds:
- multi-tenant artifact store
- job runner
- ontology/digital twin + data connectors
- UI wizard and review/commenting

The SDK must remain cleanly separable and versionable.

