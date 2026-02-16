---
id: packs
title: Packs
---

# Packs

Packs extend CFDL with domain validation and lowering behavior while keeping core language deterministic.

## Enable a pack

```cfdl
use pack "cre" version "0.1.0"
```

## Compile pack models

```bash
./target/debug/cfdl compile examples/language_tutorial/with_pack --out /tmp/tutorial_pack.ir.json --packs packs
```

## Authoritative pack docs

- `docs/pack_interface_v_0_1.md`
- `docs/docs_packs_guide.md`
