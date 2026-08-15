---
id: example-cre-lease-up
title: "CRE: lease-up"
slug: "/docs/examples/cre_lease_up"
description: "This example uses a pack contract for a formal lease (formal agreement with another party)."
---

This example uses a pack **contract** for a formal lease (formal agreement with another party).

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

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"stream.cre.lease.base_rent:amount":25000}}}
version 0.1
model "cre-lease-up-example"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset property : CRE.Asset.RealProperty

contract cre.lease {
  term 2026-07..2027-12
  terms {
    base_rent = 25000
    lease_up_months = 18
  }
}
```
