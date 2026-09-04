---
id: guide-contracts-packs
title: Contracts and packs
slug: /docs/guides/contracts-and-packs
description: "When to declare a contract instead of building streams by hand, the abstract masters a contract type refines, and what a domain pack expands that contract into."
generated: none
---

# Contracts and packs

Streams are the raw building block; contracts are how models stay readable.

## Streams vs contracts

| Situation | Use |
|---|---|
| Formal agreement with another party (lease, loan, PPA) | **Contract** |
| Individual expense/revenue line items | **Stream** |
| If in doubt | Start with a stream |

A stream is explicit about everything:

```cfdl
stream ops.revenue on entity asset.company inflow currency USD {
  schedule every month from 2026-01 to 2027-12
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
    rent = 25000
  }
}
```

## The contract vocabulary

Concrete contract types come from packs, but the language itself defines the
**masters** those types specialize — eleven kinds of agreement, each stating
what a kind of agreement *is*:

| Master | Parties | What it is |
|---|---|---|
| `Contract.Debt` | lender, borrower | Borrowed money and its service — a loan, a facility, a note, a pool of them. |
| `Contract.Lease` | lessor, lessee | Use of an asset in exchange for rent, and the rent's own mechanics. |
| `Contract.Purchase` | buyer, seller | Acquiring the asset itself. |
| `Contract.Sale` | seller, buyer | Disposing of the asset itself — an exit, a disposition, a takeout. |
| `Contract.Supply` | supplier, buyer | Goods or output delivered over a term for a price, seen from either side — a PPA, a merchant sale, a capacity payment. |
| `Contract.Service` | provider, recipient | Work done on or for the asset — management, operations and maintenance, servicing. |
| `Contract.Tax` | taxpayer, authority | A tax obligation or attribute — cash taxes, a credit, a depreciation shield. |
| `Contract.Option` | grantor, holder | An election — cash the holder chooses to take. Every pack's elections refine this. |
| `Contract.Derivative` | party, counterparty | A hedge or exchange of exposures — a swap, a rate cap, a collar. |
| `Contract.Insurance` | insurer, insured | Premiums against losses — property, title, business interruption. |

A master is **abstract**. It binds no lowering rule — a pack that gives one a
rule is refused at load — and because a model reaches a contract type only
through its rule, a master cannot be instantiated. What a master is for is
being refined, and being selected by.

The roster is extensible; the absence of a refinement in today's packs is not
evidence that a master is unneeded.

## Refinement

A pack type declares the type it specializes with `refines` — the language
base's master, or another type in the same pack:

```toml
[[entities]]
type_id = "CRE.Asset.RealProperty"
family = "asset"
class = "real"
refines = "Asset.Real"

[[entities]]
type_id = "CRE.Asset.Unit"
family = "asset"
class = "real"
refines = "CRE.Asset.RealProperty"   # chains are fine; they end at a master
```

The is-a edge is recorded rather than conventional, so it is a fact the tooling
can read: a metric or validation written against `Asset.Real` survives a new
pack unchanged. Checked when the pack loads:

- The target must exist, in this pack or the language base.
- A refinement stays in its **family**, and an asset refinement keeps its
  master's **class** — what a thing is does not change by specializing it.
- Single parent, no cycles. A chain ends at a type that refines nothing: a
  master.
- **Fields inherit down the chain.** A refinement carries every master field
  without restating it, and a model is checked against the effective roster. A
  refinement may *strengthen* a redeclared field — an optional master field
  becoming required, as `CRE.Asset.Unit` does with `rentable_area` — and may
  not retype, re-unit or weaken it.

Packs may extend the core types and may not remove them.

## What the pack does with it

At compile time the pack's lowering rules fill in schedule and amount
expressions from the terms, with validated defaults, producing ordinary streams
in the IR. Contracts are vocabulary; streams are the cash. Nothing is deferred
to run time: the `cfdl compile` output shows the streams that were generated,
and missing or malformed terms fail compilation with named diagnostics rather
than producing a quietly different model.

Terms can carry units — `27.50 "USD/MWh"` — and the unit is checked against
what the rule expects, so a figure passed in the wrong measure is a compile
error instead of a result wrong by a constant factor.

## Instances

Templates support instances via a dotted id — each gets its own streams:

```cfdl
contract cre.lease_unit.tenant_a on entity asset.tower { ... }
contract cre.lease_unit.tenant_b on entity asset.tower { ... }
```

## Selecting contracts by type

Because refinement is recorded, a [slice](/docs/specification/language-spec) can
select by master type and reach every refinement transitively:

```cfdl
slice debt {
  type Contract.Debt
}
```

`type Contract.Debt` selects every stream lowered from a contract whose type
is-a `Contract.Debt` — in any pack, including packs that do not exist yet — and
streams owned by entities of a conforming type. An unknown type is refused with
the known types named (`E1363_SLICE_UNKNOWN_TYPE`).

## Migrating a hand-built model onto a pack

1. Keep the timeline and entities unchanged.
2. Add `use pack "<id>" version "<ver>"`.
3. Replace stream groups with the equivalent contract, one at a time.
4. Compile with `--packs` and resolve diagnostics.
5. Diff the IR/results and confirm the deltas are intended.

## Reference links

- [Domain packs overview](/docs/packs) and the four pack guides
- [The object model](/docs/object-model) — the families a contract is written between
- [Pack interface](/docs/specification/pack-interface) — the normative format
- [Language spec](/docs/specification/language-spec) — contracts, slices, and selection
