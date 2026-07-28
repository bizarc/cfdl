# CFDL — the Cash Flow Domain Language

[![CI](https://github.com/bizarc/cfdl/actions/workflows/ci.yml/badge.svg)](https://github.com/bizarc/cfdl/actions/workflows/ci.yml)
[![site](https://github.com/bizarc/cfdl/actions/workflows/site.yml/badge.svg)](https://github.com/bizarc/cfdl/actions/workflows/site.yml)
[![License: BUSL-1.1](https://img.shields.io/badge/license-BUSL--1.1-blue)](LICENSE)

**Documentation, tutorials, and an in-browser playground: [cfdl.dev](https://cfdl.dev)**

CFDL is a **source-available domain language for modeling cash-flowing assets**: real
estate, energy projects and microgrids, loans and credit portfolios, operating
businesses — anything that produces or consumes cash over time.

Models are written as human-readable `.cfdl` files that separate **Time** (calendars,
phases), **Structure** (entities, contracts), and **Behavior** (streams, events,
assumptions). The compiler turns them into a deterministic, canonical **IR** (JSON), and
the engine executes valuation runs — DCF, scenarios, seeded Monte Carlo — into
schema-governed **Results** (JSON). Same inputs, same version → byte-identical outputs.

```text
model.cfdl ──► cfdl compile ──► model.ir.json ──► cfdl run ──► results.json
              (lexer → parser → resolver →        (NPV, IRR, scenarios,
               validate → IR emission)             Monte Carlo, metrics)
```

```cfdl
version 0.1
model "solar-ppa"
time calendar monthly from 2027-01 for 300
entity project plant

stream plant.ppa_revenue on entity project.plant inflow currency USD {
  schedule every monthly from 2027-01 to 2051-12
  amount = inputs.mwh_p50 * inputs.ppa_price * pow(1 - inputs.degradation, time.t / 12.0)
}
```

> **Status:** pre-1.0, under active development toward the cfdl.dev launch. The current
> language/IR spec is v0.1; interfaces may change until 1.0 freezes the IR and Results
> schemas. The roadmap is maintained internally.

## Learning CFDL

This README is the engineering front for people working *on* CFDL. If you want to
*use* it, everything below is covered better, with runnable examples, at
[cfdl.dev](https://cfdl.dev):

| | |
|---|---|
| Try it without installing | [cfdl.dev/playground](https://cfdl.dev/playground) |
| Write your first model | [Getting started](https://cfdl.dev/docs/getting-started) |
| Language tour | [Language guide](https://cfdl.dev/docs/language-guide) |
| Install a surface | [CLI, Python, VS Code, API server](https://cfdl.dev/docs/install) |
| Domain packs | [Energy, CRE, credit, OpCo](https://cfdl.dev/docs/packs) |

Surfaces, in this repo: the CLI (`crates/cfdl-cli`), the Python SDK
(`python/`, pandas result accessors), a VS Code extension with LSP diagnostics
(`editors/vscode`), an axum API server (`crates/cfdl-server`), and the
WebAssembly build behind the playground (`crates/cfdl-wasm`).

## Build & run from source

```bash
cargo build -p cfdl-cli

# compile a model to IR
./target/debug/cfdl compile fixtures/valid/minimal_model --out /tmp/model.ir.json

# run the IR to results (10% discount rate)
./target/debug/cfdl run /tmp/model.ir.json --out /tmp/model.results.json --rate 0.10

# scenarios / Monte Carlo via a run config
./target/debug/cfdl run /tmp/model.ir.json --out /tmp/model.results.json \
  --config fixtures/valid/monte_carlo_smoke/run.json
```

```bash
make ci      # fmt + clippy (-D warnings) + tests + golden suite + benchmarks
make gold    # golden suite only
```

Gold updates are intentional-only: `CFDL_GOLD_UPDATE=1 ./tools/golden-runner run`.

## Public contracts (stable interfaces)

- Language spec: `docs/01_language_spec.md` · grammar: `docs/02_grammar.md`,
  `docs/schemas/CFDL_v0_1_Grammar.ebnf`
- Expression environment: `docs/03_expression_environment.md`
- Compiler: `docs/04_compiler_spec.md` · diagnostics codes: `docs/08_diagnostics.md`
- IR schema: `docs/schemas/ir.schema.json` (`docs/05_ir_schema.md`)
- Results schema: `docs/schemas/results.schema.json` (`docs/06_results_schema.md`)
- Domain pack interface: `docs/07_pack_interface.md`

These are also published, rendered, at
[cfdl.dev/docs/language-reference](https://cfdl.dev/docs/language-reference).

Determinism is a contract: deterministic IDs, canonical ordering, stable diagnostic
codes, all enforced by the golden suite (`fixtures/` + `gold/`, run via
`./tools/golden-runner run`).

## Repository layout

| Path | Contents |
|---|---|
| `crates/` | Rust workspace: `cfdl-cli`, `cfdl-compile`, `cfdl-engine`, `cfdl-server`, `cfdl-wasm`, `cfdl-lsp`, compiler stages (`cfdl-lexer`, `cfdl-parser`, `cfdl-resolver`, `cfdl-validate`), `cfdl-calc`, `cfdl-expr`, `cfdl-pack`, `cfdl-metrics`, `cfdl-py` |
| `packs/` | Domain packs (`energy`, `cre`, `credit`, `opco`) — contract types, lowering rules, metrics, validations (TOML) |
| `docs/` | Numbered spec set + JSON schemas + grammar |
| `site/` | cfdl.dev — Next.js product site, docs, and playground |
| `python/` | Python SDK (`cfdl_sdk`, maturin/pyo3) |
| `editors/vscode/` | VS Code extension (syntax, snippets, LSP client) |
| `fixtures/`, `gold/` | Golden test fixtures and expected outputs |
| `benchmarks/` | Models validated against independent references |
| `examples/` | Worked example models and Jupyter notebooks |

Rust embedders: the intended surface is `cfdl-compile` and `cfdl-engine`.

## License & contributions

CFDL is **source available** under the [Business Source License 1.1](LICENSE): you may
read, copy, modify, and make non-production and permitted production use of the code;
offering CFDL to third parties as a hosted or embedded commercial product or service
requires a commercial license. Each released version converts to Apache-2.0 four years
after release. See `LICENSE` and `NOTICE`.

CFDL is maintained by a small internal team. **External pull requests are not
accepted at this time.** Bug reports via GitHub issues are welcome — see
`CONTRIBUTING.md`.
