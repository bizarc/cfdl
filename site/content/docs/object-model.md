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

## Every model has a vocabulary

The type after the colon is optional, and a model with no pack still has one to
choose from. The language itself defines:

- `Asset.Real` — a physical thing: land, a building, plant, equipment, a reserve.
- `Asset.Financial` — a claim on cash: a loan, a pool, a security, an equity
  interest, a going concern.
- `Asset.Intangible` — a right without physical form: a royalty, a license, a
  patent.
- `Party` — anyone a contract is with.

A pack adds its own types on top and cannot remove these. Contract types are
the exception: they exist only in packs, because a contract type is bound to a
lowering rule and lowering rules are what a pack is.

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

## Assets have lifecycles — and a lifecycle is a machine

A lifecycle is a declared finite state machine: the closed set of states, the
state it opens in, and the **edges** — each an arrow with an optional guard. A
model declares one directly, and an entity binds it by name:

```cfdl
lifecycle unit {
  initial vacant
  state vacant, leased, downtime
  vacant -> leased    when time.t >= 1
  leased -> downtime  when series_sum("cre.rent", time.t - 1, time.t - 1) < 50
  downtime -> leased
}

entity asset suite {
  lifecycle unit
  state leased
}
```

A guarded edge is evaluated each period the entity is in the edge's
from-state — and only then. Taking the edge moves the machine, which disarms
it; re-entering the state re-arms it. There is no latch: a unit that goes
delinquent and cures and goes delinquent again is the topology walked twice.
A guard reads state as the period opened and the model's own **settled cash**
strictly backward — at or before the previous period — which is how "the
rent stopped arriving" becomes a condition rather than a narration.

A pack declares the same machine for its domain types in `types.toml`, with
the same optional guards:

```toml
[[lifecycles]]
lifecycle_id = "cre.unit"
initial = "vacant"
states = ["vacant", "leased", "holdover", "downtime"]

[[lifecycles.transitions]]
from = "vacant"
to = "leased"
```

Because the set is enumerated and the initial state is declared, an asset is
always in a known state from period zero, and a state name that does not
exist — in an edge, an opening `state`, or a `set` — is rejected at compile
time with the declared set in the message. Edges are declared only as used:
an edge you did not declare does not exist, and a machine that declares no
edges leaves events unconstrained.

A schedule can anchor to a state entry —
`schedule every month from state_enter(asset.suite, building) for 18 periods`
— opening a fresh window at each entry, which is what "eighteen months of
construction from whenever construction starts" needs.

## Events move assets between states too

An event is a condition and a one-time change: it fires at the first period
its condition holds, once.

```cfdl
event expiry when time.t >= 2 {
  set entity asset.suite.status = "vacant"
}
```

Conditions can read time, computed values, the state of any entity, and
settled series strictly backward. The target of a `set` is resolved: writing
to an entity or a field that does not exist is an error — and a status write
is validated against the machine's declared edges, refused with the edge
named where no edge permits the move. A guard-less edge exists exactly for
this: a permission an event's write may take, which the machine never fires
on its own. When you find yourself writing a *pair* of events that set and
un-set one status, the claim is a regime that returns, and it is two guarded
edges of one lifecycle.

### Contracts and streams can depend on state

```cfdl
stream cre.rent on entity asset.suite inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  category operating.revenue.base_rent
  amount = 100
  active in state leased, holdover
}
```

`active in state` names states from the lifecycle, so those names are checked
too. This is the reason to prefer it over comparing a status string: a typo in
`entity.status == "leasd"` is not an error, it is a condition that is
false forever and says nothing.

### When a write becomes visible

Two rules, not one:

- An **event or option guard** reads state as the period *opened*. Every guard
  in a period therefore sees the same state, and the order declarations appear
  in cannot change an answer.
- A **stream** reads state as the period *closed*. A transition takes effect in
  the period its event fires.

All transitions evaluate against the state at the start of the period, the state
commits, then outputs read the committed result.

### The grid decides when an event can fire

An event fires in the first period where its condition holds. It cannot fire
between periods, because there is nothing between them — so **the calendar
sets how precisely a condition can be met**.

That is a modeling choice with money attached whenever an event decides who
gets paid. A tax-equity partnership flips when its investor's return reaches a
target; on an annual grid the test can only be asked at year ends. In
[a worked case](/docs/examples/energy-tax-equity-flip) the investor is $445,000
short of its hurdle at the end of year two and two months of cash clear it, but
the annual grid has no period between month 24 and month 36 in which to notice.
The same deal on a monthly calendar flips **ten months earlier**, and about
$3.5m changes hands on the strength of one line.

The same applies to a covenant test, a cash trap, a rate step, or any trigger
whose date is not written into a contract. Choose the grid the test is actually
performed on. A model that states its trigger date hides the question; one that
derives it cannot.

### Transitions are in the results

Results carry a transition log: for each state change, the period, the date, the
entity, the field, the value before and after, and the event that caused it.
Entity state would otherwise be unobservable — nothing else distinguishes an
event that fired against a misspelled target from one that never fired. Entries
are recorded even when the value does not change, because the question the log
answers is whether the event fired.

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

The exercise condition reads the same environment an event condition does,
including entity state, so an option can be conditioned on where an asset is in
its lifecycle.

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
- [Events and options](/docs/examples/options_events) — a model that runs.
- [Domain packs](/docs/packs) — the types, lifecycles and contracts each pack
  defines.
