# CRE Lease-Up Example

Single lease with explicit **lease-up ramp** terms: `lease_up.start_period`, `lease_up.months`, `lease_up.start_occupancy`, `lease_up.end_occupancy`. Industry-standard occupancy ramp from lease commencement to stabilized.

## Compile

```bash
./target/debug/cfdl compile examples/cre_lease_up --out /tmp/cre_lease_up.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/cre_lease_up.ir.json --out /tmp/cre_lease_up.results.json --config examples/cre_lease_up/run.json --packs packs
```
