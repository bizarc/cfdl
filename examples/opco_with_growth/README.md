# OpCo With Growth Example

This example uses pack **contracts** throughout: `opco.revenue_line` and `opco.opex_line` for operations, `opco.exit_multiple` for the exit.

Revenue line with **growth_rate** > 0 (3% here). Demonstrates the industry lever for recurring revenue growth in DCF.

`growth_rate` is an **annual** rate. The pack converts it to the model's grain geometrically, so revenue compounds to 3% a year on a monthly calendar rather than 3% a month — the same conversion a discount rate gets.

## Compile

```bash
./target/debug/cfdl compile examples/opco_with_growth --out /tmp/opco_growth.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/opco_growth.ir.json --out /tmp/opco_growth.results.json --config examples/opco_with_growth/run.json --packs packs
```
