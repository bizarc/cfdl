---
id: cre-examples
title: "CRE examples"
slug: "/docs/examples/cre-examples"
generated: none
---

Commercial Real Estate (CRE) examples show construction → lease-up → stabilized lifecycle, NOI, exit cap rate, and optional construction-to-permanent financing. All use the CRE pack (`use pack "cre"`). For when to use **streams vs contracts** (e.g. formal lease → contract; individual ops items → stream), see the [Language guide](/docs/language-guide#when-to-use-streams-vs-contracts).

## Example ladder

| Example | Purpose |
|---------|---------|
| [simple_contract](/docs/examples/simple_contract) | Single lease (tutorial). |
| [cre_lease_up](/docs/examples/cre_lease_up) | Lease with explicit lease-up ramp terms. |
| [cre_developer](/docs/examples/cre_developer) | Full lifecycle: construction stub, lease, ops revenue/expense, exit cap. |
| [cre_phased](/docs/examples/cre_phased) | Same as full lifecycle with **phases** (construction, lease_up, perm). |
| [cre_multi_file](/docs/examples/cre_multi_file) | Full lifecycle split across `time.cfdl`, `structure.cfdl`, `contracts.cfdl`. |
| [cre_development_with_financing](/docs/examples/cre_development_with_financing) | Full lifecycle plus **construction** and **permanent** loan streams (time-bounded; no events). |

## Run configurations

Each example directory includes a `run.json` (and/or `run.base.json`, `run.stress.json`) for deterministic runs. Example:

```bash
./target/debug/cfdl compile examples/cre_developer --out /tmp/cre.ir.json --packs packs
./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.results.json --config examples/cre_developer/run.json --packs packs
```

## Structure

- **Entities:** Examples use one or more entities (e.g. `real_estate property`, `loan construction`, `loan permanent`). Streams are owned by an entity.
- **Pack contracts:** `cre_construction_stub`, `cre_lease`, `cre_ops_revenue`, `cre_ops_expense`, `cre_exit_cap` — see [Packs](/docs/packs) and [the CRE pack guide](/docs/packs/cre).
