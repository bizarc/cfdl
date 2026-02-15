# CFDL SDK User Guide (v0.2)

This guide shows how to **install**, **author**, **compile**, and **run** CFDL models using the CFDL SDK, including packs (CRE and Operating Business), scenarios, and Monte Carlo.

> EVS platform (`evs-platform`) builds on this SDK. This guide focuses on the SDK workflow.

---

## 1. What you can do with the SDK

With the CFDL SDK you can:

- Author a model in CFDL (`.cfdl`)
- Compile CFDL to deterministic IR (`.json`)
- Run the engine to produce deterministic Results (`.json`)
- Run scenarios and Monte Carlo using run configs (seeded and reproducible)
- Use packs to hydrate contracts into streams

---

## 2. Concepts (quick)

- **Entity**: an object that owns streams (e.g., Asset, Property, BusinessUnit)
- **Stream**: a cash flow vector attached to one entity
- **Contract**: first-class primitive; can lower into streams (via packs)
- **Timeline**: analysis horizon and cadence
- **Phase**: named date range; boundaries can be used as events/triggers
- **IR**: deterministic JSON representation used by engines
- **Results**: deterministic JSON output from engines
- **Run config**: scenarios/distributions/seed/trials (separate from CFDL)

---

## 3. Install

### 3.1 Prerequisites
- Rust toolchain (stable)

### 3.2 Build from source
From the repo root:

```bash
cargo build -p cfdl-cli
```

The binary will be at:

```bash
./target/debug/cfdl
```

---

## 4. Quickstart (no packs)

### 4.1 Create a minimal model
Create a folder:

```
my_model/
  model.cfdl
```

Example `model.cfdl` (minimal; adjust to your current grammar):

```cfdl
version 0.1

model "MyModel"

time {
  cadence month
  from 2026-01-01
  periods 60
}

entity asset "Asset"

stream rent on entity asset {
  schedule { from 0 to 59 }
  amount money(10000.00, "USD")
}
```

### 4.2 Compile to IR

```bash
./target/debug/cfdl compile my_model --out /tmp/my_model.ir.json
```

### 4.3 Run to Results

```bash
./target/debug/cfdl run /tmp/my_model.ir.json --out /tmp/my_model.results.json --rate 0.10
```

---

## 5. Using run config (scenarios + Monte Carlo)

Run config is a JSON file that the engine consumes. It is separate from CFDL.

### 5.1 Scenario example
Create `run.json`:

```json
{
  "scenarios": {
    "base": { "overrides": {} },
    "rent_down": { "overrides": { "cfg.rent_multiplier": 0.90 } },
    "rent_up": { "overrides": { "cfg.rent_multiplier": 1.10 } }
  }
}
```

Run:

```bash
./target/debug/cfdl run /tmp/my_model.ir.json --out /tmp/my_model.results.json --config run.json
```

### 5.2 Monte Carlo example
Create `run.json`:

```json
{
  "monte_carlo": {
    "seed": 12345,
    "trials": 1000
  },
  "distributions": {
    "cfg.exit_cap": { "type": "normal", "mean": 0.055, "stddev": 0.005 },
    "cfg.vacancy": { "type": "uniform", "min": 0.03, "max": 0.12 }
  }
}
```

Run:

```bash
./target/debug/cfdl run /tmp/my_model.ir.json --out /tmp/my_model.results.json --config run.json
```

Notes:
- Runs are reproducible for a fixed seed.
- For large trials (up to 50k), prefer server-side execution in EVS.

---

## 6. Using Packs

Packs add domain templates, aliases, lowering rules, validations, and defaults.

### 6.1 Pack location
Packs live under:

```
packs/
  cre/
  opco/
```

### 6.2 Compile with packs

```bash
./target/debug/cfdl compile my_model --out /tmp/my_model.ir.json --packs packs/
```

### 6.3 Run with packs

```bash
./target/debug/cfdl run /tmp/my_model.ir.json --out /tmp/my_model.results.json --packs packs/ --rate 0.10
```

### 6.4 Example: CRE developer workflow
The CRE pack supports:
- construction
- lease-up
- stabilized operations
- exit

Recommended next step:
- start from `examples/cre_developer/` (once available)

---

## 7. Expressions (CEL)

Expressions can appear in amounts, schedules, and predicates.

Example:

```cfdl
stream rent on entity asset {
  schedule { from 0 to 59 }
  amount money( cfg.base_rent * cfg.rent_multiplier, "USD")
}
```

See:
- `docs/cel_and_expr_env_v_0_2.md`

---

## 8. Diagnostics and troubleshooting

### 8.1 Diagnostics JSON
If a command fails, you can request diagnostics JSON:

```bash
./target/debug/cfdl compile my_model --out /tmp/my_model.ir.json --json
```

Diagnostics include:
- code
- message
- file
- span

See:
- `docs/diagnostics_spec.md`

### 8.2 Golden runner
To verify the repo state:

```bash
./tools/golden-runner run
```

To update gold (only for intentional changes):

```bash
CFDL_GOLD_UPDATE=1 ./tools/golden-runner run
```

---

## 9. Embedding in Rust (evs-platform)

EVS platform depends on the SDK via Git.

Example dependency:

```toml
[dependencies]
cfdl-compile = { git = "ssh://git@github.com/<ORG>/cfdl.git", tag = "v0.2.0", package = "cfdl-compile" }
cfdl-engine  = { git = "ssh://git@github.com/<ORG>/cfdl.git", tag = "v0.2.0", package = "cfdl-engine" }
```

---

## 10. What’s next

- Follow the v0.2 roadmap: `docs/SDK_V0_2_ROADMAP.md`
- Read packs guide: `docs/PACKS_GUIDE.md`
- Explore examples: `examples/` (CRE + OpCo)

---

## Appendix: Common CLI commands

```bash
# Compile
cfdl compile <model_dir> --out <ir.json>

# Run (rate)
cfdl run <ir.json> --out <results.json> --rate 0.10

# Run (config)
cfdl run <ir.json> --out <results.json> --config run.json

# Gold verification
./tools/golden-runner run
```

