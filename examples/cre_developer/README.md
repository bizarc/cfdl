# CRE Developer Example

This example mirrors the `fixtures/valid/cre_developer_smoke` model and uses
the `cre` pack (`0.1.0`) for deterministic contract lowering.

## Run

Compile:

`./target/debug/cfdl compile examples/cre_developer --out /tmp/cre.ir.json --packs packs`

Run base case:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.base.results.json --config examples/cre_developer/run.base.json --packs packs`

Run stress case:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.stress.results.json --config examples/cre_developer/run.stress.json --packs packs`

## Scenario knobs

The provided run configs demonstrate deterministic override testing with:

- `stream.cre.lease.base_rent.amount`
- `stream.cre.ops.expense.amount`
- `stream.cre.exit.sale.amount`
