# CRE Multi-File Example

This example uses pack **contracts** in `contracts.cfdl` for lease, construction stub, ops revenue/expense, and exit (per guidance, individual ops items could be streams).

Full developer lifecycle split across files: `time.cfdl` (phases), `structure.cfdl` (entities), `contracts.cfdl` (CRE pack contracts). Entry is `model.cfdl` with version, model, use pack, time, and imports.

## Compile

```bash
./target/debug/cfdl compile examples/cre_multi_file --out /tmp/cre_multi_file.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/cre_multi_file.ir.json --out /tmp/cre_multi_file.results.json --config examples/cre_multi_file/run.json --packs packs
```
