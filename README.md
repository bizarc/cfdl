# CFDL — the Cash Flow Domain Language

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

> **Status:** pre-1.0, under active development toward the CFDL.dev launch. The current
> language/IR spec is v0.1; interfaces may change until 1.0 freezes the IR and Results
> schemas. See `LAUNCH_PLAN.md` for the roadmap.

## Use CFDL from…

- **Files + CLI** — `cfdl compile`, `cfdl run`, `cfdl validate` (this repo, works today)
- **Python / Jupyter** — `cfdl_sdk` bindings under `python/` with pandas result accessors
  (`results.cashflows()/.metrics()/.scenarios()`) and example notebooks
- **VS Code** — extension with LSP diagnostics under `editors/vscode`
- **API server** — `crates/cfdl-server` (axum): `POST /v1/compile|validate|run`
- **Playground** — in-browser compile + run (`crates/cfdl-wasm`, Monaco docs-site page)

## Quick start

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

Learn the language:

- Language tour and user guide: `docs/09_user_guide.md`
- Tutorial examples: `examples/language_tutorial/`
- Worked examples: `examples/` (CRE development, lease-up, operating businesses)

## Public contracts (stable interfaces)

- Language spec: `docs/01_language_spec.md` · grammar: `docs/02_grammar.md`,
  `docs/schemas/CFDL_v0_1_Grammar.ebnf`
- Expression environment: `docs/03_expression_environment.md`
- Compiler: `docs/04_compiler_spec.md` · diagnostics codes: `docs/08_diagnostics.md`
- IR schema: `docs/schemas/ir.schema.json` (`docs/05_ir_schema.md`)
- Results schema: `docs/schemas/results.schema.json` (`docs/06_results_schema.md`)
- Domain pack interface: `docs/07_pack_interface.md`

Determinism is a contract: deterministic IDs, canonical ordering, stable diagnostic
codes, all enforced by the golden suite (`fixtures/` + `gold/`, run via
`./tools/golden-runner run`).

## Repository layout

| Path | Contents |
|---|---|
| `crates/` | Rust workspace: `cfdl-cli`, `cfdl-compile`, `cfdl-engine`, `cfdl-lsp`, compiler stages (`cfdl-lexer`, `cfdl-parser`, `cfdl-resolver`, `cfdl-validate`), `cfdl-expr`, `cfdl-pack`, `cfdl-metrics`, `cfdl-py` |
| `packs/` | Domain packs (`cre`, `opco`) — contract types, defaults, lowering rules (TOML) |
| `docs/` | Numbered spec set + JSON schemas + grammar |
| `docs-site/` | Docusaurus documentation site |
| `python/` | Python SDK (`cfdl_sdk`, maturin/pyo3) |
| `editors/vscode/` | VS Code extension (syntax, snippets, LSP client) |
| `fixtures/`, `gold/` | Golden test fixtures and expected outputs |
| `examples/` | Worked example models |

Rust embedders: the intended surface is `cfdl-compile` and `cfdl-engine`.

## Build & test

```bash
make ci      # fmt + clippy (-D warnings) + tests + golden suite
make gold    # golden suite only
```

Gold updates are intentional-only: `CFDL_GOLD_UPDATE=1 ./tools/golden-runner run`.

## License & contributions

CFDL is **source available** under the [Business Source License 1.1](LICENSE): you may
read, copy, modify, and make non-production and permitted production use of the code;
offering CFDL to third parties as a hosted or embedded commercial product or service
requires a commercial license. Each released version converts to Apache-2.0 four years
after release. See `LICENSE` and `NOTICE`.

CFDL is maintained by a small internal team at EVS. **External pull requests are not
accepted at this time.** Bug reports via GitHub issues are welcome — see
`CONTRIBUTING.md`.
