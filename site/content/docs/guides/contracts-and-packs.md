---
id: guide-contracts-packs
title: Contracts & Packs
slug: /docs/guides/contracts-and-packs
---

# Contracts & Packs

Streams are the raw building block; contracts are how models stay readable.

## Streams vs contracts

| Situation | Use |
|---|---|
| Formal agreement with another party (lease, loan, PPA) | **Contract** |
| Individual expense/revenue line items | **Stream** |
| If in doubt | Start with a stream |

A stream is explicit about everything:

```cfdl
stream ops.revenue on entity operating.company inflow currency USD {
  schedule every monthly from 2026-01 to 2027-12
  amount = 30000
}
```

A contract declares business terms and lets the pack's templates expand
them into streams at compile time:

```cfdl
use pack "cre" version "0.1.0"

contract cre.lease on entity asset.tower {
  term 2026-07..2031-12
  terms {
    base_rent = 25000
  }
}
```

## What the pack does with it

At compile time the pack's lowering rules fill in schedule and amount
expressions from the terms (with validated defaults), producing ordinary
streams in the IR — there is no runtime magic, and `cfdl compile` output
shows exactly what was generated. Missing or malformed terms fail
compilation with named diagnostics.

## Instances

Templates support instances via a dotted id — each gets its own streams:

```cfdl
contract cre.lease_unit.tenant_a on entity asset.tower { ... }
contract cre.lease_unit.tenant_b on entity asset.tower { ... }
```

## Migrating a hand-built model onto a pack

1. Keep the timeline and entities unchanged.
2. Add `use pack "<id>" version "<ver>"`.
3. Replace stream groups with the equivalent contract, one at a time.
4. Compile with `--packs` and resolve diagnostics.
5. Diff the IR/results and confirm the deltas are intended.

## Reference links

- [Domain Packs overview](/docs/packs) and the four pack guides
- [Pack Interface spec](/docs/language-reference/pack-interface)
