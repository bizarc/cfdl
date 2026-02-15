# NOW.md

This file is the **current work queue** for agentic development. If you are an agent, follow this file **first**.

> Rules: read and follow `CLAUDE.md`, then execute tasks here. Specs in `docs/` are authoritative.

---

## Current sprint

### Task 8 — Scenarios + Monte Carlo (Milestone 8)

**Goal:** Add run configurations for scenarios and Monte Carlo with reproducible seeds; compute basic statistics.

**Principles**

* **Reproducible**: same inputs + same config + same seed ⇒ identical outputs.
* **No correlation in language/IR**: correlation is not represented in CFDL IR.
* **Run config is separate**: scenarios/Monte Carlo are driven by run config consumed by the engine.
* **Deterministic ordering**: stable ordering of trials, scenarios, and metrics.

**Deliverables**

* Run config support in engine + CLI:

  * scenario sets: named parameter override sets
  * Monte Carlo config: trial count + fixed seed
* Scenario runner:

  * compute results per scenario (same IR)
  * output includes scenario identifiers and per-scenario summary metrics (NPV at minimum)
* Monte Carlo runner:

  * sample distributions for declared assumptions/parameters in run config
  * run N trials deterministically (seeded RNG)
  * output includes trial-by-trial summaries + aggregated statistics
* Statistics (minimum):

  * mean / median / stddev of NPV
  * probability metric: P(NPV < 0)

**Golden artifacts**

* Add fixtures and gold results:

  * `fixtures/valid/scenario_compare/` + `gold/results/scenario_compare.results.json`
  * `fixtures/valid/monte_carlo_smoke/` + `gold/results/monte_carlo_smoke.results.json`

**Recommended tooling updates**

* Extend `tools/golden-runner` to run results generation with:

  * per-fixture config file support (e.g., `run.json` inside fixture) OR
  * standardized args encoded in fixture metadata

**Acceptance criteria**

* `make fmt && make lint && make test && make gold` all pass
* Scenario and Monte Carlo results are deterministic and match gold

---

## Next up (preview)

### Task 9 — Expression evaluation (CEL or equivalent) + typed env

**Goal:** Introduce a typed expression environment for amounts/terms, leaving UX simple.

### Task 10 — Pack-lowering hooks (domain packs)

**Goal:** Formalize pack interfaces that lower contracts → streams and provide aliases.

---

## Notes / decisions

* Keep the CLI thin.
* Do not add correlation (language or IR).
* Golden tests are authoritative for behavior.
