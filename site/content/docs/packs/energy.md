---
id: pack-energy
title: "Energy"
slug: "/docs/packs/energy"
description: "The energy pack: contracted and merchant revenue, operating cost, tax benefits, and project debt for a generating asset."
generated: regions
---

# Energy

Generating assets: contracted and merchant revenue, operating cost, tax benefits and project debt.

## What it models

A project selling under a power purchase agreement or into a merchant market, with production degrading over life, availability applied, prices escalating or read from a forward curve, and tax credits claimed over their statutory window. Storage arbitrage on a cycled-spread basis.

## Contracts

Declare a contract and the pack expands it into the streams those terms imply,
each classified so it lands on the right line of a
[statement](/docs/reference/statements).

<!-- cfdl:generated contracts-energy -->
| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `energy.ppa` | `availability`, `degradation`, `escalation`, `price`, `quantity` | `energy.ppa.revenue[.suffix]` |
| `energy.merchant` | `availability`, `degradation`, `escalation`, `price`, `quantity` | `energy.merchant.revenue[.suffix]` |
| `energy.storage_arbitrage` | `degradation`, `price`, `quantity` | `energy.storage.margin[.suffix]` |
| `energy.capacity` | `price` | `energy.capacity.revenue[.suffix]` |
| `energy.om` | `escalation`, `fee_year` | `energy.om.expense[.suffix]` |
| `energy.itc` | `amount` | `energy.itc.credit[.suffix]` |
| `energy.capex` | `amount` | `energy.capex.outlay[.suffix]` |
| `energy.debt_service` | `funded_at_close`, `interest_rate`, `principal` | `energy.debt.proceeds[.suffix]`, `energy.debt.interest[.suffix]`, `energy.debt.principal[.suffix]` |
| `energy.ptc` | `amount`, `availability`, `degradation`, `escalation`, `quantity`, `round_step` | `energy.ptc.credit[.suffix]` |
| `energy.macrs_shield` | `basis`, `life`, `tax_rate` | `energy.macrs.shield[.suffix]` |
<!-- /cfdl:generated contracts-energy -->

A contract can be declared more than once by giving it a suffix, so the pieces
stay separable in the results.

## Reporting

A project operating statement down to cash flow available for debt service, with coverage tested every period, which is the covenant a project lender holds.

## Related

- [Statements](/docs/reference/statements) — the pro forma this pack produces
- [Metrics](/docs/reference/metrics) — what it reports over the whole model
- [Validation](/docs/benchmarks) — the reference models it is gated against
