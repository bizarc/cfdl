---
id: install-playground
title: Playground (zero install)
slug: /install/playground
---

# Playground — zero install

The [Playground](/playground) runs the real CFDL compiler and engine in
your browser via WebAssembly. Nothing to install, nothing leaves your
machine — models compile and run locally in the page.

## What it can do

- Full compile + run with diagnostics as editor markers and live metrics.
- `use pack` against the embedded packs (energy, cre, credit, opco).
- Seeded Monte Carlo — the engine's deterministic seeding is wasm-clean, so
  results match the CLI byte for byte.

## Limits vs the CLI

- Single-buffer editing (no multi-file models or imports).
- No filesystem output — results render in the page rather than writing
  `ir.json` / `results.json`.
- Long Monte Carlo runs are constrained by the browser tab.

When you outgrow it: [install the CLI](cli) or the
[Python SDK](python).
