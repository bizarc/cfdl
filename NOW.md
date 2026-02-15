# NOW.md

This file is the **current work queue** for agentic development. If you are an agent, follow this file **first**.

> Rules: read and follow `CLAUDE.md`, then execute tasks here. Specs in `@docs/` are authoritative.

---

## Current sprint

### Task 6 — IR emission skeleton (Milestone 6)

**Goal:** Emit deterministic IR JSON for minimal valid models (enough to introduce the first valid fixtures).

**Deliverables**

* Canonical IR emitter in `crates/cfdl-compile` matching `@docs/CFDL_v0_1_IR.schema.json`
* Deterministic ID generation
* Provenance propagation
* Introduce the first valid fixture + gold IR:

  * `fixtures/valid/minimal_model/` + `gold/ir/minimal_model.json`

**Acceptance criteria**

* `make fmt && make lint && make test && make gold` all pass
* `cfdl compile ...` writes an IR JSON file that matches the IR schema
* IR output is deterministic and canonicalizable (key order stable in canonical form)

---

## Next up

### Task 7 — Results emission + deterministic engine harness (Milestone 7)

**Goal:** Execute the minimal valuation pipeline on emitted IR and produce deterministic results JSON.

**Deliverables**

* Results emitter matching `@docs/CFDL_v0_1_Results.schema.json`
* Deterministic “engine harness” interface (inputs: IR + run config; outputs: results)
* Minimal deterministic engine path:

  * discounting / NPV
  * IRR (if defined in results spec)
  * basic aggregations (entity/stream totals)
* Golden results for at least:

  * `fixtures/valid/minimal_model/` + `gold/results/minimal_model.results.json`

---

### Task 8 — Scenarios + Monte Carlo (Milestone 8)

**Goal:** Add run configurations for scenarios and Monte Carlo with reproducible seeds; compute basic statistics.

**Deliverables**

* Run config schema support (scenario sets, parameter overrides, seeds)
* Monte Carlo runner with reproducible seeding and trial tracking
* Basic statistics in results:

  * distributions of NPV/IRR (if supported)
  * mean/median/stddev
  * probability metrics (e.g., P(NPV<0))
* Golden results for at least:

  * scenario comparison fixture
  * monte carlo fixture (fixed seed)

## Notes / decisions

* Keep the CLI thin.
* Do not add correlation (language or IR).
* Prefer adding **invalid fixtures first**; add valid fixtures once IR emission exists.