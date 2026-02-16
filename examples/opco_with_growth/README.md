# OpCo With Growth Example

This example uses pack **contracts** for revenue, opex, and exit (per guidance, individual revenue/opex items could be streams).

Revenue line with **growth_rate** > 0 (e.g. 3%). Demonstrates the industry lever for recurring revenue growth in DCF.

## Compile

```bash
./target/debug/cfdl compile examples/opco_with_growth --out /tmp/opco_growth.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/opco_growth.ir.json --out /tmp/opco_growth.results.json --config examples/opco_with_growth/run.json --packs packs
```
