# NOW.md

This file is the **current work queue** for agentic development. If you are an agent, follow this file **first**.

> Rules: read and follow `CLAUDE.md`, then execute tasks here. Specs in `@docs/` are authoritative.

---

## Current sprint

### Task 5 — Validation (Milestone 5)

**Goal:** Enforce structural and semantic constraints (required statements, mandatory contract term, stream schedule rules, schedule bounds).

**Deliverables**

* Validation pass in `crates/cfdl-validate` (or equivalent module) enforcing:

  * required global statements (`version`, `model`, `time`)
  * contracts: `term` mandatory; effects required unless pack-lowered
  * streams: schedule + amount required
  * schedule validity + bounds within model timeline
* Diagnostics per `@docs/diagnostics_spec.md`
* Fixtures + gold for at least:

  * `fixtures/invalid/missing_time/`
  * `fixtures/invalid/bad_missing_term/`
  * `fixtures/invalid/bad_schedule_out_of_bounds/`

**Acceptance criteria**

* `make fmt && make lint && make test && make gold` all pass
* `cfdl --json compile ...` emits valid JSON diagnostics matching gold

---

## Next up

### Task 6 — IR emission skeleton (Milestone 6)

**Goal:** Emit deterministic IR JSON for minimal valid models (enough to introduce the first valid fixtures).

**Deliverables**

* Canonical IR emitter in `crates/cfdl-compile` matching `@docs/CFDL_v0_1_IR.schema.json`
* Deterministic ID generation
* Provenance propagation
* Fixtures + gold:

  * `fixtures/valid/minimal_model/` + `gold/ir/minimal_model.json`

## Notes / decisions

* Keep the CLI thin.
* Do not add correlation (language or IR).
* Prefer adding **invalid fixtures first**; add valid fixtures once IR emission exists.
