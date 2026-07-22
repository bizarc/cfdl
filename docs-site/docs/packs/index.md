---
id: packs-overview
title: Domain Packs
slug: /packs
---

# Domain Packs

A **pack** is a domain library for CFDL: it ships contract templates,
validations, and industry metrics so a model declares *business terms*
(rent, PPA price, CPR) instead of hand-building every stream. Every pack is
gated by a [benchmark parity suite](/benchmarks) against independent
Excel-grade reference models.

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
| `energy` | Renewables & project finance: PPA/merchant revenue, storage, ITC/PTC, MACRS, debt service | [Energy pack guide](/cookbooks/energy) |
| `cre` | Commercial real estate at institutional DCF fidelity: lease-by-lease, rollover, recoveries, exit on forward NOI | [CRE pack guide](/cookbooks/cre) |
| `credit` | Loan pools: CPR/CDR, severity, recovery lag, floaters off rate curves, purchase pricing | [Credit pack guide](/cookbooks/credit) |
| `opco` | Operating businesses / LBO: working capital, capex, term debt, cash taxes, trailing-EBITDA exit | [OpCo pack guide](/cookbooks/opco) |

## Using a pack

```cfdl
version 0.1
model "office_deal"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 120

entity real_estate property

contract cre.lease {
  term 2026-01..2035-12
  terms {
    base_rent = 25000
  }
}
```

Compile with the packs directory available (`--packs packs/` from a
checkout, or the packs bundle from a release), and pass `--pack cre` at run
time to compute the pack's domain metrics.

## Going deeper

- Per-pack guides: [Energy](/cookbooks/energy) · [CRE](/cookbooks/cre) ·
  [Credit](/cookbooks/credit) · [OpCo](/cookbooks/opco)
- Runnable Jupyter notebooks, one per pack, in
  [`examples/notebooks/`](https://github.com/bizarc/cfdl/tree/main/examples/notebooks)
- The authoritative pack format spec:
  [Pack Interface](/language-reference/pack-interface)
