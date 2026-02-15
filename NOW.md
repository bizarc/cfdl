# NOW.md

This file is the **current work queue** for agentic development. If you are an agent, follow this file **first**.

> Rules: read and follow `CLAUDE.md`, then execute tasks here. Specs in `@docs/` are authoritative.

---

## Current sprint

### Task 3 — Imports + module graph (Milestone 3)

**Goal:** Implement `import "..."` resolution, deterministic module ordering, and cycle detection.

**Deliverables**

* Import resolution in `crates/cfdl-resolver`
* Deterministic module graph ordering (stable topo order)
* Enforce “no escape” outside model root
* Diagnostics:

  * `E1201_IMPORT_CYCLE`
  * `E1202_IMPORT_NOT_FOUND`
  * `E1203_IMPORT_OUTSIDE_MODEL_ROOT`
* Fixtures + gold for at least:

  * `fixtures/invalid/import_cycle/` + `gold/diag/import_cycle.diag.json`
  * `fixtures/invalid/import_not_found/` + `gold/diag/import_not_found.diag.json`
  * `fixtures/invalid/import_outside_root/` + `gold/diag/import_outside_root.diag.json`

**Acceptance criteria**

* `make fmt && make lint && make test && make gold` all pass
* `cfdl --json compile ...` emits valid JSON diagnostics matching gold

---

## Next up

### Task 3 — Imports + module graph (Milestone 3)

**Goal:** Implement `import "..."` resolution, deterministic module ordering, and cycle detection.

**Deliverables**

* Import resolution in `crates/cfdl-resolver`
* Diagnostics:

  * `E1201_IMPORT_CYCLE`
  * `E1202_IMPORT_NOT_FOUND`
  * `E1203_IMPORT_OUTSIDE_MODEL_ROOT`
* Fixtures + gold for at least:

  * cycle
  * missing import

---

### Task 4 — Symbol tables + uniqueness (Milestone 4)

**Goal:** Build symbol tables and enforce uniqueness for core identifiers.

**Deliverables**

* Symbol registry + resolution
* Diagnostics for duplicates and unresolved refs per `@docs/diagnostics_spec.md`
* Fixtures + gold for:

  * duplicate stream
  * unresolved entity reference

---

## Notes / decisions

* Keep the CLI thin.
* Do not add correlation (language or IR).
* Prefer adding **invalid fixtures first** (valid fixtures only once IR emission exists).