# NOW.md

This file is the **current work queue** for agentic development. If you are an agent, follow this file **first**.

> Rules: read and follow `CLAUDE.md`, then execute tasks here. Specs in `@docs/` are authoritative.

---

## Current sprint

### Task 7 — Results emission + deterministic engine harness (Milestone 7)

**Goal:** Execute a minimal valuation pipeline on emitted IR and produce deterministic results JSON.

**Deliverables**

* Implement a deterministic engine harness crate (recommended: `crates/cfdl-engine`) that:

  * loads IR (in-memory struct or JSON)
  * executes deterministic cash-flow aggregation + discounting
  * produces Results JSON matching `@docs/CFDL_v0_1_Results.schema.json`
* Minimal deterministic outputs:

  * per-stream cash flow series (as emitted/normalized)
  * entity and model totals
  * NPV using a flat discount rate (configurable)
  * IRR if present in results schema (optional if not yet defined)
* Create a `gold/results/` directory and add:

  * `gold/results/minimal_model.results.json`

**Recommended tooling updates**

* Extend `tools/golden-runner` to verify results for valid fixtures:

  * run `cfdl compile ... --out <tmp_ir>`
  * run a deterministic results command (recommended CLI subcommand `run`) to emit `<tmp_results>`
  * compare against `gold/results/<fixture>.results.json`

**Acceptance criteria**

* `make fmt && make lint && make test && make gold` all pass
* Results output is deterministic across repeated runs

---

## Next up

### Task 8 — Scenarios + Monte Carlo (Milestone 8)

**Goal:** Add run configurations for scenarios and Monte Carlo with reproducible seeds; compute basic statistics.

**Deliverables**

* Run config supports:

  * scenario sets (parameter override sets)
  * Monte Carlo (trial count, fixed seed)
* Monte Carlo runner produces reproducible trial outputs
* Basic statistics in results:

  * mean/median/stddev for NPV (and IRR if supported)
  * probability metrics (e.g., P(NPV<0))
* Golden results for at least:

  * `fixtures/valid/scenario_compare/` + `gold/results/scenario_compare.results.json`
  * `fixtures/valid/monte_carlo_smoke/` + `gold/results/monte_carlo_smoke.results.json`

---

## Notes / decisions

* Keep the CLI thin.
* Do not add correlation (language or IR).
* Prefer adding invalid fixtures first; add valid fixtures only when a milestone requires them.
* Golden tests are authoritative for behavior.
