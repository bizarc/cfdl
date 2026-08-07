---
id: packs-overview
title: Domain packs
slug: /docs/packs
generated: none
---

# Domain packs

A **pack** is a domain library for CFDL: it ships contract templates,
validations, and industry metrics so a model declares *business terms*
(rent, PPA price, CPR) instead of hand-building every stream. Every pack is
gated by a [benchmark suite](/docs/benchmarks) checked against independent
reference models.

## What a pack gives you

- **Contract templates** — declare `contract cre.lease { ... }` with the
  pack's term vocabulary; the compiler lowers it to fully-specified streams.
- **Validations** — missing or inconsistent terms fail at compile time with
  precise diagnostics, not at run time.
- **Domain metrics** — each pack declares its own metric set (e.g. DSCR,
  NOI, collections multiple, exit proceeds) computed alongside the core
  NPV/IRR/MOIC/payback/WAL metrics.

## The four packs

| Pack | Domain | Guide |
|---|---|---|
| `energy` | Renewables & project finance: PPA/merchant revenue, storage, ITC/PTC, MACRS, debt service | [Energy pack guide](/docs/packs/energy) |
| `cre` | Commercial real estate, lease by lease: rollover, recoveries, exit on forward NOI | [CRE pack guide](/docs/packs/cre) |
| `credit` | Loan pools: CPR/CDR, severity, recovery lag, floaters off rate curves, purchase pricing | [Credit pack guide](/docs/packs/credit) |
| `opco` | Operating businesses / LBO: working capital, capex, term debt, cash taxes, trailing-EBITDA exit | [OpCo pack guide](/docs/packs/opco) |

## Using a pack

```cfdl
version 0.1
model "office_deal"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 120

entity asset property : CRE.Asset.RealProperty

contract cre.lease {
  term 2026-01..2035-12
  terms {
    base_rent = 25000
  }
}
```

Compile with a packs directory available — `--packs packs/`, pointing at the
packs bundle from a release — and pass `--pack cre` at run time to compute the
pack's domain metrics.

## Going deeper

- Per-pack guides: [Energy](/docs/packs/energy) · [CRE](/docs/packs/cre) ·
  [Credit](/docs/packs/credit) · [OpCo](/docs/packs/opco)
- Worked notebooks, one per pack, with their real outputs and charts:
  [Notebooks](/docs/notebooks)
- The normative pack format:
  [Pack interface](/docs/specification/pack-interface)
