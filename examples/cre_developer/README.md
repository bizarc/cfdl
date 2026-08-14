# CRE Developer Example

This example uses the `cre` pack (`0.1.0`) for the formal lease and construction stub and exit; **standalone streams** for ops revenue and ops expense (per guidance: individual revenue/expense items → stream).

## Run

Compile:

`./target/debug/cfdl compile examples/cre_developer --out /tmp/cre.ir.json --packs packs`

Run base case:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.base.results.json --config examples/cre_developer/run.base.json --packs packs`

Run stress case:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.stress.results.json --config examples/cre_developer/run.stress.json --packs packs`

## Scenario knobs

The provided run configurations demonstrate deterministic override testing with:

- `stream.cre.lease.base_rent.amount`
- `stream.ops_expense.amount`
- `stream.cre.exit.sale.amount`
