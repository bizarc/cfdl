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

## Start here

1. [How CFDL Works](concepts) — the two-minute mental model.
2. [Getting Started](getting-started) — run your first model in the
   browser, then with the CLI.
3. [Install & Setup](install) — pick your surface and set it up.

## Pick your surface

| | Setup |
|---|---|
| **[Playground](/playground)** — compile + run in the browser, zero install | [about](install/playground) |
| **CLI** — `cfdl compile / run / validate` for files, git, and CI | [install](install/cli) |
| **[Python SDK](python-sdk)** — pandas accessors over results, notebooks | [install](install/python) |
| **[API server](api-server)** — self-hostable HTTP API | [setup](install/api-server) |
| **VS Code** — diagnostics, hover, completion via the CFDL LSP | [setup](install/vscode) |

## Learn and build

- [Language Guide](language-guide) and the progressive
  [tutorial examples](/examples)
- [Domain Packs](/packs): [Energy](/cookbooks/energy) ·
  [CRE](/cookbooks/cre) · [Credit](/cookbooks/credit) ·
  [OpCo](/cookbooks/opco)
- [Stochastic Modeling](stochastic-modeling) — assumptions, Monte Carlo,
  percentile outputs
- [Benchmarks](benchmarks) — how parity with institutional-grade references
  is proven
- [Language Reference](language-reference) — authoritative specs and
  schemas

## Licensing

CFDL is **source available** under the Business Source License 1.1 — free to
read, run, and use in production, with commercial hosting/embedding rights
reserved. See [Licensing](licensing) for details.
