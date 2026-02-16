# OpCo Basic Example

This example uses **standalone streams** for revenue and opex (per guidance); pack **contracts** for working capital and exit multiple. See the Language Guide "When to use streams vs contracts."

Compile:

`./target/debug/cfdl compile examples/opco_basic --out /tmp/opco.ir.json --packs packs`

Run base:

`./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.base.results.json --config examples/opco_basic/run.base.json --packs packs`

Run stress:

`./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.stress.results.json --config examples/opco_basic/run.stress.json --packs packs`
