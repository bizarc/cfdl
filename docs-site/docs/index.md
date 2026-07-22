---
id: index
title: CFDL Docs
slug: /
---

# CFDL Documentation

CFDL (Cash Flow Domain Language) is a deterministic, source-available
language for modeling cash flows across asset classes — energy and
infrastructure, commercial real estate, credit, and operating businesses.
The same model file gives you the point-estimate answer **and** the
distribution around it: models are natively stochastic, deterministically
seeded, and byte-reproducible.

## Use CFDL from…

- **Files + CLI** — `cfdl compile`, `cfdl run`, `cfdl validate`
- **Python / Jupyter** — the [`cfdl_sdk` package](python-sdk) with pandas
  result accessors
- **Playground** — [compile and run in the browser](/playground), nothing to
  install
- **API server** — a [self-hostable HTTP API](api-server) over the compiler
  and engine
- **VS Code** — extension with LSP diagnostics, hover, and completion

## Recommended path

1. Read [Getting Started](getting-started)
2. Walk through the [Language Guide](language-guide)
3. Learn [Packs](packs) and the per-domain [Cookbooks](/cookbooks)
4. See how parity is proven in [Benchmarks](benchmarks)
5. Model uncertainty with [Stochastic Modeling](stochastic-modeling)
6. Use the [Language Reference](language-reference) for authoritative spec
   links

## Source links

- Repository: [https://github.com/bizarc/cfdl](https://github.com/bizarc/cfdl)
- Onboarding guide source: `docs/09_user_guide.md`
- Tutorial models: `examples/language_tutorial/`

## Licensing

CFDL is **source available** under the Business Source License 1.1 — free to
read, run, and use in production, with commercial hosting/embedding rights
reserved. See [Licensing](licensing) for details.
