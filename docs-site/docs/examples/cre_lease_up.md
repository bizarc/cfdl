---
id: example-cre-lease-up
title: "cre lease up"
slug: "/examples/cre_lease_up"
---

Single lease with explicit **lease-up ramp** terms: `lease_up.start_period`, `lease_up.months`, `lease_up.start_occupancy`, `lease_up.end_occupancy`. Industry-standard occupancy ramp from lease commencement to stabilized.

## Compile

```bash
./target/debug/cfdl compile examples/cre_lease_up --out /tmp/cre_lease_up.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/cre_lease_up.ir.json --out /tmp/cre_lease_up.results.json --config examples/cre_lease_up/run.json --packs packs
```

---

> Generated from `examples/cre_lease_up/`. Code is shown below so you can see structure and elements without repo access.

## model.cfdl

```cfdl
version 0.1
model "cre-lease-up-example"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity real_estate property

contract cre_lease {
  term 2026-07..2027-12
  terms {
    base_rent = 25000
    lease_up.start_period = 6
    lease_up.months = 18
    lease_up.start_occupancy = 0.0
    lease_up.end_occupancy = 1.0
  }
}
```
