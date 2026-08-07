---
id: reference-cli
title: CLI reference
slug: /docs/reference/cli
generated: none
---

# CLI reference

<!-- Keep in sync with crates/cfdl-cli/src/main.rs (clap definitions). -->

```
cfdl [--json] <command>
```

`--json` (global) switches diagnostics and command output to structured
JSON — use it in scripts and CI.

## `cfdl compile`

```bash
cfdl compile <model_dir> --out <ir.json> [--packs <packs_dir>]
```

Compiles a model directory to canonical IR JSON. `<model_dir>` must contain
`model.cfdl` (plus any imported files). `--packs` points at a packs
directory for `use pack` resolution; a local `packs/` directory is used
automatically when present.

## `cfdl validate`

```bash
cfdl validate <model_dir> [--packs <packs_dir>]
```

Compile-checks without writing IR — it runs the same pipeline as `compile`
and discards the result, so the two can never disagree. Pass `--packs` for a
model that uses one: pack-lowered contracts have no `effects` block of their
own, and a pack's domain validations only resolve with the registry
available. Exit code is nonzero on any error diagnostic; combine with
`--json` for machine-readable diagnostics (code, message, file, span).

## `cfdl parse`

```bash
cfdl parse <model_dir>
```

Dumps the typed AST — useful for debugging grammar-level surprises and for
tooling.

## `cfdl run`

```bash
cfdl run <ir.json> --out <results.json> \
  [--config <run.json>] [--rate <f64>] [--as-of <YYYY-MM-DD>] \
  [--packs <packs_dir>] [--pack <name>]
```

Evaluates compiled IR and writes Results JSON.

- `--config` — a [run-config file](/docs/reference/run-config): discount rate,
  parameter overrides, scenarios, Monte Carlo.
- `--rate` — fallback annual discount rate when the config doesn't set one
  (default `0.0`).
- `--as-of` — valuation as-of date.
- `--pack <name>` — computes that pack's domain metrics in addition to the
  core set; requires `--packs` (or a local `packs/`) so the pack's metric
  declarations resolve.

Precedence: run-config values win over `--rate`/`--as-of` fallbacks.

## `cfdl pack list`

```bash
cfdl pack list --path <packs_dir>
```

Lists the packs (id, version) visible in a packs directory.

## Exit codes and diagnostics

Commands exit `0` on success and nonzero when errors were emitted.
Diagnostics carry stable codes (`E1202`, `E5008`, …) cataloged in the
[Diagnostics reference](/docs/specification/diagnostics); codes are never
renamed or reused.
