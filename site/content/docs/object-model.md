---
id: object-model
title: "The object model"
slug: "/docs/object-model"
description: "The four kinds of thing a model declares — time, entities, streams, and contracts — and how everything else is built from them."
generated: none
---

# The object model

A CFDL model is a set of declarations about things that exist and what they do
with cash. There are four kinds of thing, and everything in the language is one
of them.

| | |
|---|---|
| **Asset** | Something that produces or consumes cash — a building, a plant, a loan pool, a going concern. |
| **Party** | Someone who contracts, owns, lends or occupies. |
| **Container** | A grouping that scopes cash — a fund, a portfolio, an SPV, a transaction. It holds cash-producers; cash attached directly to it is deal-level cash. |
| **Contract** | An agreement, written on an asset, between parties. It emits cash flows. |
| **Reference** | Something observed rather than owned — a rate curve, an index, a price path. |

These are called *families*. A family is the broad answer; a *type* is the
specific one, and types come from the pack a model uses.

```cfdl
entity asset tower : CRE.Asset.RealProperty {
  asset_class   = "office"
  rentable_area = 30000
}

entity party acme : CRE.Party.Tenant {
  name = "Acme Corp"
}
```

The word after `entity` is the family. The name follows. What comes after the
colon is the type, which is checked against the active pack's vocabulary — a
misspelled type is a compile error, not a silently different model.

Fields inherit down the type chain. A type that `refines` another carries
every master field without restating it, and a model is checked against the
effective roster — required fields and near-miss detection include the
masters'. A refinement may strengthen a field (an optional master field
becomes required) but never retype, re-unit, or weaken it: a reader who
learned a field from the master is not lied to by the refinement.

## Every model has a vocabulary

The type after the colon is optional, and a model with no pack still has one to
choose from. The language itself defines:

- `Asset.Real` — a physical thing: land, a building, plant, equipment, a reserve.
- `Asset.Financial` — a claim on cash: a loan, a pool, a security, an equity
  interest, a going concern.
- `Asset.Intangible` — a right without physical form: a royalty, a license, a
  patent.
- `Party` — anyone a contract is with.
- `Container.Fund`, `Container.Portfolio`, `Container.SPV`,
  `Container.Transaction` — groupings that scope cash without producing it.
- Eleven abstract contract masters — `Contract.Debt`, `Contract.Lease` and the
  rest. A master states what a kind of agreement is; a pack's concrete contract
  types refine it.

A pack adds its own types on top and cannot remove these, and each pack type
records what it specializes, so "is a" is a fact the tooling can read. See
[Contracts and packs](/docs/guides/contracts-and-packs) for the masters, the
refinement chain, and what a pack expands a contract into.

## Assets can nest, and never have to

A building can be modeled as one asset with a blended rent roll, as a set of
unit types, or suite by suite. All three are correct; which is right depends on
what the model is for. The same is true of a loan pool modeled as a pool or as
its loans, and a field modeled as a field or as its wells.

So hierarchy exists and nothing requires it. Declare it with `part of`:

```cfdl
entity asset tower   : CRE.Asset.RealProperty
entity asset suite_a : CRE.Asset.Unit { rentable_area = 10000  part of asset.tower }
entity asset suite_b : CRE.Asset.Unit { rentable_area =  5000  part of asset.tower }
```

Streams attach at whatever grain their asset sits at: suite rent on the suite, a
building-wide expense on the building. The building's totals include its suites'
totals because they are its suites — the relation is what aggregates, not a
shared name prefix.

## Assets have lifecycles

An asset may carry a **lifecycle**: a declared finite state machine, with a
closed set of states, the state it opens in, and guarded edges that move it.

```cfdl
entity asset suite {
  lifecycle unit
  state leased
}
```

Because the states are enumerated, an asset is in a known state from period
zero and a misspelled state name is a compile error. Events move an entity
across the machine's edges, streams switch on where it is with
`active in state`, and every transition is in the results.
See [Lifecycles and state](/docs/lifecycles).

## Options are contracts with an election

An option is written on an asset and is between parties, exactly as any other
contract is. What it adds is an exercise condition and a payoff.

```cfdl
option mgmt_pool on entity asset.target type OpCo.Contract.EquityOption {
  parties { grantor = party.sponsor, holder = party.mgmt }
  exercise when time.t >= 1
  payoff 250.0
}
```

The exercise condition reads the same environment an
[event condition](/docs/lifecycles) does, including entity state, so an option
can be conditioned on where an asset is in its lifecycle.

## Waterfalls pay entities

A priority of payments shares a pot out in order, and every step names who is
paid — a party, or an asset such as a note class.

```cfdl
waterfall deal.distribution on entity asset.trust {
  schedule every month from 2026-01 to 2030-12
  from available

  pay servicing to party.servicer    = 12500.0
  pay senior    to asset.class_a     = 6250.0
  pay residual  to party.certificate = remaining
}
```

Each step's cash counts toward its payee's total, so a waterfall is how money
reaches the parties and tranches an ontology already names. Cash that should
**accumulate** between distribution dates — a reserve, proceeds waiting for a
quarterly date — lives in a declared `account`: a step pays into one, a
waterfall draws one with `from <account>`, and logic reads its settled
balance as `prev.<account>`. See [Waterfalls](/docs/guides/waterfalls).

## Quantities can carry units

A number in a contract's terms can state what it measures.

```cfdl
contract energy.ptc on entity asset.plant {
  term 2026-01..2028-01
  terms {
    mwh_year       = 250000 "MWh/yr"
    credit_per_mwh = 27.50 "USD/MWh"
  }
}
```

The unit is checked against what the lowering rule expects. A megawatt-hour
figure passed where megawatts are wanted becomes a compile error instead of a
result that is wrong by a factor of the hours in a year.

## Where to go next

- [Language guide](/docs/language-guide) — the full syntax.
- [Lifecycles and state](/docs/lifecycles) — the state machine an asset walks.
- [Events and options](/docs/examples/options_events) — a model that runs.
- [Domain packs](/docs/packs) — the types, lifecycles and contracts each pack
  defines.
