# OpCo Multi-File Example

This example uses **standalone streams** in `contracts.cfdl` for revenue and opex (per guidance); pack **contracts** for working capital and exit.

Full OpCo valuation (revenue, opex, working capital, exit multiple) split across files: `structure.cfdl` (entities), `contracts.cfdl` (pack contracts). Entry is `model.cfdl` with version, model, use pack, time, and imports.

## Compile

```bash
./target/debug/cfdl compile examples/opco_multi_file --out /tmp/opco_multi_file.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/opco_multi_file.ir.json --out /tmp/opco_multi_file.results.json --config examples/opco_multi_file/run.json --packs packs
```
