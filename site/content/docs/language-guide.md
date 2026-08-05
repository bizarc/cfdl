---
id: language-guide
title: "Language Guide"
slug: "/docs/language-guide"
source: docs/09_user_guide.md
generated: full
---

This guide covers CFDL language basics, CLI usage, and common workflows. For exact grammar and semantics, see the spec docs (`01_language_spec.md` – `08_diagnostics.md`).

---

## 1) What CFDL models

CFDL models cash-flow behavior with a deterministic language:

- **Time**: model timeline and optional phases
- **Structure**: entities (what things exist)
- **Behavior**: streams, contracts, events, options
- **Analysis**: assumptions and run mode (output metrics are engine-computed per pack)

---

## 2) Minimum valid model

At minimum, a practical model should include `version`, `model`, `time`, one `entity`, and one behavior block:

```cfdl
version 0.1
model "minimal-model"
time calendar monthly from 2026-01 for 12

entity legal borrower

stream legal.rent on entity legal.borrower {
  schedule every month from 2026-01 to 2026-12
  amount = 1000
}
```

See `examples/language_tutorial/minimal_model/model.cfdl`.

---

## 3) Language elements

### Header and modules
- `version 0.1`
- `model "name"`
- `use pack "<pack-id>" version "<pack-version>"`
- `import "relative_file.cfdl"`

### Time
- `time calendar monthly from 2026-01 for 72`
- `phase lease_up from 2026-01 to 2027-06`

### Structure
- `entity <namespace> <name>` — e.g. `entity real_estate property`
- Entity references use qualified names: `real_estate.property`

### Behavior
- `stream` — direct cash-flow definitions
- `contract` — domain contracts (especially when using packs)
- `event` — conditional actions
- `option` — optional exercise behavior

### Analysis
- `assume` for assumptions
- `run deterministic` or `run monte_carlo trials <n> seed <s>`
- Output metrics (NPV, IRR, etc.) are computed by the engine per the pack's output specification

### When to use streams vs contracts

| Situation | Use |
|-----------|-----|
| Formal agreement with another party (lease, loan) | **Contract** |
| Informal agreement (handshake, memo, internal forecast) | Contract or Stream |
| Individual expense/revenue items (line-item opex, one-off) | **Stream** |
| If in doubt | **Start with a stream** |

---

## 4) Expressions and literals

CFDL uses bare, Excel-familiar expressions for executable logic:

```cfdl
amount = inputs.base_rent * 1.02
```

Common literals:
- Strings: `"hello"`
- Numbers: `1000`, `0.05`
- Dates: `2026-01`, `2026-01-15`
- Booleans: `true`, `false`

Notes:
- `YYYY-MM` dates are normalized to first-of-month.
- Expressions must be deterministic (no random/time/network behavior).

---

## 5) Assumptions and uncertainty

`assume` declares a named input; expressions read it as `inputs.<name>`.

### Fixed assumption

```cfdl
assume discount_rate = 0.10
```

### Stochastic assumption

Replace the constant with a distribution and the model becomes stochastic —
no other change required:

```cfdl
assume rent_growth ~ Normal(mean=0.03, stdev=0.01, clip=[-0.02, 0.08])
assume vacancy ~ Triangular(min=0.0, mode=0.05, max=0.15)
```

Supported distributions: `Normal(mean, stdev, clip?)`,
`LogNormal(mu, sigma, clip?)`, `Uniform(min, max)`,
`Triangular(min, mode, max)`.

### Running Monte Carlo

Enable Monte Carlo in the run-config JSON (see the run-config section):

```json
{ "monte_carlo": { "trial_count": 1000, "seed": 42 } }
```

Every run declares an explicit seed, and each assumption gets its own
deterministic draw stream — adding an assumption never reshuffles another
assumption's draws, and runs are byte-reproducible across machines. The
results' `monte_carlo` section summarizes every metric with mean, stdev,
min/max, and percentiles (p01–p99).

Because draws are ordinary values, expressions can branch on them to model
coherent per-trial outcomes (e.g. binary renew-vs-rollover decisions)
instead of expected-value blends:

```cfdl
amount = if(inputs.renewal_draw < 0.70, renewal_rent, market_rent)
```

Run-config `distributions` can add or override distributions without
touching the model file.

---

## 6) Curves (date-indexed inputs)

`curve` declares a named, date-indexed series — rate curves, price decks,
escalation paths — looked up from expressions with `curve_value`:

```cfdl
curve sofr linear {
  2026-01: 0.050
  2027-01: 0.038
}

stream loan.interest on entity fund.buyer inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 1000000 * (curve_value("sofr", time.date) + 0.0275) / 12
}
```

- `step` holds each value flat-forward until the next point; `linear`
  interpolates by calendar day.
- Lookups outside the declared range clamp to the first/last point.
- Duplicate names, duplicate point dates, or malformed points fail
  compilation with `E5008_INVALID_CURVE`.

Curves are deterministic inputs; the credit pack's floating-rate contracts
(`credit.pool_float_io_bullet`) read coupon indices from them.

---

## 7) Schedule patterns

### One-time payment
```cfdl
schedule on 2026-06
```

### Monthly recurring
```cfdl
schedule every month from 2026-01 to 2026-12
```

### Day rule example
```cfdl
schedule every month on day 15 from 2026-01 to 2026-12
```

---

## 8) Naming conventions

- Use dot notation for hierarchy: `cre.lease.base_rent`, `opco.working_capital.adjustment`
- Use underscore only within a segment: `working_capital`
- Keep entity symbols qualified and stable for ontology/data-source mapping

---

## 9) Using packs

Packs add domain behavior and validation. Use at the top of `model.cfdl`:

```cfdl
use pack "cre" version "0.1.0"
```

When to use packs: domain templates, validated `terms`, automatic contract lowering.
When not required: simple models with direct streams only.

### Migrating from no-pack to pack

1. Keep existing timeline and entities unchanged.
2. Add `use pack "<id>" version "<ver>"` in `model.cfdl`.
3. Replace manual stream logic with pack-supported `contract` blocks gradually.
4. Run compile with `--packs packs` and resolve pack diagnostics.
5. Verify expected IR/results deltas intentionally.

---

## 10) Multi-file models

Split by concern as models grow:

- `model.cfdl` → header and imports
- `time.cfdl` → phases and timeline
- `structure.cfdl` → entities
- `contracts.cfdl` → contracts/streams/events

```cfdl
import "time.cfdl"
import "structure.cfdl"
import "contracts.cfdl"
```

Rules: imports are relative, no cycles, do not import outside model root.

---

## 11) CLI

### Build

```bash
cargo build -p cfdl-cli
```

### Compile a model

```bash
./target/debug/cfdl compile <model_dir> --out <ir.json> [--packs <packs_dir>]
```

- `--packs` is optional. The four standard packs are built into the binary, so
  `use pack` resolves without it. A local `packs/` directory is used when
  present, and takes precedence — a directory holding packs is authoritative,
  so a mistyped path fails rather than silently falling back to the built-in
  copies.

### Run IR

```bash
./target/debug/cfdl run <ir.json> --out <results.json> [--config <run.json>] [--rate <f64>] [--as-of <YYYY-MM-DD>] [--packs <packs_dir>]
```

### Run-config JSON

```json
{
  "deterministic": {
    "annual_discount_rate": 0.1,
    "as_of": "2026-01-01",
    "parameters": {
      "stream.cre.lease.base_rent:amount": 1000.0
    }
  },
  "scenarios": {
    "stress": {
      "annual_discount_rate": 0.12,
      "parameters": {
        "stream.cre.lease.base_rent:amount": 800.0
      }
    }
  },
  "monte_carlo": {
    "trial_count": 1000,
    "seed": 12345,
    "distributions": {
      "cfg.exit_multiple": { "kind": "normal", "mean": 8.0, "stddev": 1.0 },
      "cfg.growth": { "kind": "uniform", "min": 0.01, "max": 0.05 }
    }
  }
}
```

Parameter key conventions:
- Stream overrides: `stream.<dotted_stream_name>:amount`
- Config namespace: `cfg.<path>` (for `cfg.*` access in expressions)

---

## 12) Running examples

### CRE Developer

```bash
./target/debug/cfdl compile examples/cre_developer --out /tmp/cre.ir.json --packs packs
./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.results.json --config examples/cre_developer/run.base.json --packs packs
```

### OpCo Basic

```bash
./target/debug/cfdl compile examples/opco_basic --out /tmp/opco.ir.json --packs packs
./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.results.json --config examples/opco_basic/run.base.json --packs packs
```

---

## 13) Golden runner

Golden outputs are the source of truth for fixture behavior:

```bash
./tools/golden-runner run
```

Update gold files only when intentionally changing behavior:

```bash
CFDL_GOLD_UPDATE=1 ./tools/golden-runner run
```

---

## 14) Common errors and fixes

- **Missing required model header fields** — Add `version`, `model`, and `time` once each.
- **Unresolved entity refs** — Confirm `entity` exists and reference uses a qualified name.
- **Schedule range issues** — Ensure `from <= to` and dates align with timeline.
- **Unterminated string/comment** — Close all quotes and block comments.
- **Pack validation errors** — Re-check `use pack` statement and contract `terms` shape.

For diagnostic codes, see `08_diagnostics.md`.

---

## 15) Progressive tutorial examples

- `examples/language_tutorial/minimal_model/`
- `examples/language_tutorial/first_stream/`
- `examples/language_tutorial/simple_contract/`
- `examples/language_tutorial/with_pack/`
- `examples/language_tutorial/multi_file/`

Recommended workflow: start with minimal → add streams → contracts with packs → multi-file.

---

## 16) Authoritative references

| Doc | Content |
|-----|---------|
| `01_language_spec.md` | Core language spec |
| `02_grammar.md` | Formal EBNF grammar |
| `03_expression_environment.md` | Expression environment |
| `04_compiler_spec.md` | Compiler pipeline spec |
| `05_ir_schema.md` | IR schema (human-readable) |
| `06_results_schema.md` | Results schema (human-readable) |
| `07_pack_interface.md` | Pack interface and API |
| `08_diagnostics.md` | Error codes and diagnostic format |
| `schemas/ir.schema.json` | Machine-readable IR schema |
| `schemas/results.schema.json` | Machine-readable results schema |
