# CFDL SDK User Guide (v0.1)

This guide documents the current SDK command-line behavior and example workflows.

For first-time language onboarding (syntax, minimum model, packs, and examples), start with `docs/LANGUAGE_GUIDE.md`.

## Build CLI

From repo root:

```bash
cargo build -p cfdl-cli
```

CLI binary:

```bash
./target/debug/cfdl
```

## Compile a Model

```bash
./target/debug/cfdl compile <model_dir> --out <ir.json> [--packs <packs_dir>]
```

Notes:
- `--packs` is optional.
- If `--packs` is omitted and a local `packs/` directory exists, compile uses that directory automatically.
- Compile uses the same deterministic compiler pipeline used by tests and golden validation.

## Run IR

```bash
./target/debug/cfdl run <ir.json> --out <results.json> [--config <run.json>] [--rate <f64>] [--as-of <YYYY-MM-DD>] [--packs <packs_dir>]
```

Notes:
- `--packs` is optional.
- If `--packs` is omitted and a local `packs/` directory exists, run validates packs from that directory.
- `--config` points to a JSON run-config file.
- `--rate` and `--as-of` are fallback values used when not set in `--config`.

## Run-Config JSON

Run-config is JSON consumed by the engine. The currently supported structure is:

```json
{
  "deterministic": {
    "discount_rate": 0.1,
    "as_of": "2026-01-01",
    "parameters": {
      "stream.cre.lease.base_rent:amount": 1000.0
    }
  },
  "scenarios": {
    "stress": {
      "discount_rate": 0.12,
      "as_of": "2026-01-01",
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
      "cfg.growth": { "kind": "uniform", "min": 0.01, "max": 0.05 },
      "cfg.fixed_amount": { "kind": "fixed", "value": 100.0 }
    }
  }
}
```

Parameter key conventions:
- Stream amount overrides use colon boundary only: `stream.<dotted_stream_name>:amount` (e.g. `stream.cre.lease.base_rent:amount`).
- Config namespace values use `cfg.<path>` keys (for CEL `cfg.*` access).

Reference configs are included in:
- `examples/cre_developer/run.base.json`
- `examples/cre_developer/run.stress.json`
- `examples/opco_basic/run.base.json`
- `examples/opco_basic/run.stress.json`

## Running Examples

### CRE Developer

```bash
./target/debug/cfdl compile examples/cre_developer --out /tmp/cre.ir.json --packs packs
./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.base.results.json --config examples/cre_developer/run.base.json --packs packs
./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.stress.results.json --config examples/cre_developer/run.stress.json --packs packs
```

### OpCo Basic

```bash
./target/debug/cfdl compile examples/opco_basic --out /tmp/opco.ir.json --packs packs
./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.base.results.json --config examples/opco_basic/run.base.json --packs packs
./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.stress.results.json --config examples/opco_basic/run.stress.json --packs packs
```

## Golden Runner (Authoritative)

Golden outputs remain the source of truth for fixture behavior:

```bash
./tools/golden-runner run
```

Update gold files only when intentionally changing behavior:

```bash
CFDL_GOLD_UPDATE=1 ./tools/golden-runner run
```
