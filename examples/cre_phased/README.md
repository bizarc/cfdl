# CRE Phased Example

Full developer lifecycle with **phases** aligning to industry stages: `construction`, `lease_up`, `perm` (stabilized). Same pack contracts as cre_developer; phases document the timeline and enable phase-relative schedules in the spec.

## Compile

```bash
./target/debug/cfdl compile examples/cre_phased --out /tmp/cre_phased.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/cre_phased.ir.json --out /tmp/cre_phased.results.json --config examples/cre_phased/run.json --packs packs
```
