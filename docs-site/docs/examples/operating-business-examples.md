---
id: operating-business-examples
title: "Operating Business Examples"
slug: "/examples/operating-business-examples"
---

Operating Business (OpCo) examples show revenue, opex, working capital, and exit multiple for DCF-style valuation. All use the OpCo pack (`use pack "opco"`).

## Example ladder

| Example | Purpose |
|---------|---------|
| [with_pack](/examples/with_pack) | Revenue + opex only (tutorial). |
| [opco_with_growth](https://github.com/bizarc/cfdl/blob/main/examples/opco_with_growth/) | Revenue line with **growth_rate** > 0. |
| [opco_basic](https://github.com/bizarc/cfdl/blob/main/examples/opco_basic/) | Full OpCo: revenue, opex, **working capital**, exit multiple. |
| [opco_multi_file](https://github.com/bizarc/cfdl/blob/main/examples/opco_multi_file/) | Full OpCo split across `structure.cfdl`, `contracts.cfdl`. |

## Run configs

Each example directory includes a `run.json` for deterministic runs. Example:

```bash
./target/debug/cfdl compile examples/opco_basic --out /tmp/opco.ir.json --packs packs
./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.results.json --config examples/opco_basic/run.json --packs packs
```

## Structure

- **Entities:** Examples use `operating business`; multi-entity (e.g. company + LOBs) can be added when needed.
- **Pack contracts:** `opco_revenue_line`, `opco_opex_line`, `opco_working_capital`, `opco_exit_multiple` — see [Packs](/packs) and [OpCo pack README](https://github.com/bizarc/cfdl/blob/main/packs/opco/README.md).
