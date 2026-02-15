# OpCo Basic Example

Compile:

`./target/debug/cfdl compile examples/opco_basic --out /tmp/opco.ir.json --packs packs`

Run base:

`./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.base.results.json --config examples/opco_basic/run.base.json --packs packs`

Run stress:

`./target/debug/cfdl run /tmp/opco.ir.json --out /tmp/opco.stress.results.json --config examples/opco_basic/run.stress.json --packs packs`
