# CFDL SDK (Cash Flow Domain Language)

CFDL is a proprietary domain language and SDK for defining **cash-flow models** (time, structure, behavior), compiling them into a deterministic **Intermediate Representation (IR)**, and executing valuation runs to produce deterministic **Results** (DCF, scenarios, Monte Carlo).

This repository is the **CFDL SDK**: language spec + compiler + schemas + engine harness + CLI + golden fixtures.

> EVS (Enterprise Valuation SaaS) lives in a separate repository: `evs-platform`, which depends on this SDK.

---

## What’s in this repo

### Language + contracts

* Human-authored CFDL source files (`.cfdl`)
* Language specification and grammar (see `docs/`)

### Compiler toolchain

* Lexer → Parser → Resolver → Symbol checks → Validation → IR emission
* Deterministic IDs and canonical ordering
* Diagnostics with stable codes (golden-tested)

### Engine harness

* Deterministic execution over IR
* Results emission (schema-governed)
* Scenario + Monte Carlo run configs (seeded, reproducible)

### Golden fixtures

* `fixtures/invalid/*` → expected diagnostics (`gold/diag/*`)
* `fixtures/valid/*` → expected IR (`gold/ir/*`) and results (`gold/results/*`)

---

## Quick start

### Build

```bash
cargo build -p cfdl-cli
```

### Compile a model to IR

```bash
./target/debug/cfdl compile fixtures/valid/minimal_model --out /tmp/model.ir.json
```

### Run a model IR to results

```bash
./target/debug/cfdl run /tmp/model.ir.json --out /tmp/model.results.json --rate 0.10
```

### Run with a config (scenarios / Monte Carlo)

```bash
./target/debug/cfdl run /tmp/model.ir.json --out /tmp/model.results.json --config fixtures/valid/monte_carlo_smoke/run.json
```

### Verify golden fixtures (authoritative behavior)

```bash
./tools/golden-runner run
```

To update gold (intentional behavior changes only):

```bash
CFDL_GOLD_UPDATE=1 ./tools/golden-runner run
```

---

## Public contracts (stable interfaces)

The following are the contracts EVS and other consumers integrate against:

* CFDL source format: `docs/CFDL_v0_1_Language_Spec.md`
* IR schema: `docs/CFDL_v0_1_IR.schema.json`
* Results schema: `docs/CFDL_v0_1_Results.schema.json`
* Diagnostics codes: `docs/diagnostics_spec.md`
* Deterministic ID generation: `docs/id_generation.md`

---

## Crates

This is a Rust workspace. Key crates include:

* `cfdl-cli` — the CLI tool (`cfdl`)
* `cfdl-compile` — compiler pipeline (CFDL → IR)
* `cfdl-engine` — execution engine harness (IR → Results)
* `cfdl-validate`, `cfdl-resolver`, etc. — internal compiler stages

> The intended embedding surface for other repos is `cfdl-compile` and `cfdl-engine`.

---

## VSCode extension

The CFDL VSCode extension lives in `editors/vscode`.

- Development and smoke-test guide: `editors/vscode/README.md`
- End-user install and configuration: `distribution/install-configure.md`
- Language server binary used by the extension: `cfdl-lsp`

---

## Versioning and releases

This repo is versioned as an SDK.

* Tags (e.g., `v0.1.0`) identify released SDK snapshots.
* EVS pins to a tag or commit SHA.

To create and push a tag:

```bash
git tag -a v0.1.0 -m "CFDL SDK v0.1.0"
git push origin v0.1.0
```

---

## Using this SDK from `evs-platform`

You can depend on this repo via Git (no crates.io required).

In `evs-platform/Cargo.toml`:

```toml
[dependencies]
cfdl-compile = { git = "ssh://git@github.com/bizarc/cfdl.git", tag = "v0.1.0", package = "cfdl-compile" }
cfdl-engine  = { git = "ssh://git@github.com/<ORG>/cfdl.git", tag = "v0.1.0", package = "cfdl-engine" }
```

Notes:

* Use `tag = "v0.1.0"` to pin to a release.
* Or use `rev = "<commit sha>"` to pin precisely.

---

## Relationship to EVS

`evs-platform` provides:

* multi-tenant SaaS (projects, artifacts, jobs)
* connectors + pipelines (Excel/CSV, PDF, APIs)
* ontology / digital twin
* authoring UIs (wizard) and review/comment workflows
* domain packs (CRE, Operating Business, Private Credit)

`evs-platform` depends on the CFDL SDK to compile and run models deterministically.

---

## License

Proprietary. All rights reserved.
