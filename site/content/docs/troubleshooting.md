---
id: troubleshooting
title: Troubleshooting
generated: none
---

# Troubleshooting

Organized by surface. Diagnostic codes (`E….`) are cataloged in the
[Diagnostics reference](/docs/language-reference/diagnostics).

## CLI

- **`use pack` fails to resolve** — pass `--packs <dir>` pointing at a packs
  directory (the repo's `packs/` from a checkout, or an extracted
  `cfdl-packs-<version>.tar.gz`). Check what's visible with
  `cfdl pack list --path <dir>`.
- **Domain metrics missing from results** — `cfdl run` needs `--pack <name>`
  (e.g. `--pack cre`) to apply a pack's metric set; core metrics
  (NPV/IRR/…) are always computed.
- **NPV is zero or looks undiscounted** — no discount rate reached the run:
  pass `--rate` or set `deterministic.annual_discount_rate` in the
  run-config.
- **Machine-readable errors** — add the global `--json` flag; diagnostics
  come out structured with codes and spans.

## Compile errors (any surface)

- **Missing required model header** — every model needs `version`, `model`,
  and `time` exactly once.
- **Unresolved entity refs** — the `entity` must exist and references use
  the qualified `namespace.name` form.
- **Import failures** — import paths are relative to the importing file; no
  cycles; imports cannot escape the model root.
- **Pack term validation** — the pack's required terms must be present with
  the right shapes; see each pack guide's term table (missing-term
  diagnostics name the term).
- **Schedule range issues** — `from <= to`, and dates must land inside the
  model timeline.

## Monte Carlo

- **`monte_carlo.status: "not_run"`** — Monte Carlo is enabled by the
  run-config (`"monte_carlo": { "trial_count": N, "seed": S }`), not by the
  model alone.
- **Results differ between machines** — they shouldn't: runs are
  byte-reproducible given the same release. Confirm the same version
  (results carry `engine.version` and `model_hash`) and the same seed.

## VS Code

- **No diagnostics/hover** — verify `cfdl.serverPath` points at the right
  `cfdl-lsp` binary and that it is executable; set
  `cfdl.trace.server = "verbose"` to inspect client/server traffic.
- **Pack diagnostics wrong in the editor** — `cfdl.packsPath` must point to
  the same packs directory you compile with.
- Full setup: [VS Code and LSP](/docs/install/vscode).

## Python SDK

- **`CompileError` / `RunError`** — both carry structured details
  (`CompileError.diagnostics` has codes and spans; don't parse the string
  form).
- **Import fails after checkout install** — editable installs build the
  native module; a Rust toolchain must be available. See
  [Install for Python](/docs/install/python).

## API server

- **HTTP 413** — request body over 1 MiB. **408/timeout** — 10 s cap.
  **400 on Monte Carlo** — trial count above the server cap (1000); it is
  rejected, never silently truncated.
- **422 with a diagnostics body** — same compile/validate errors as the
  CLI, structured.

## Playground

- Single-buffer only (no imports/multi-file); long Monte Carlo runs are
  limited by the browser tab. When you hit a wall, move to the
  [CLI](/docs/install/cli).
