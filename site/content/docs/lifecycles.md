---
id: lifecycles
title: "Lifecycles and state"
slug: "/docs/lifecycles"
description: "The declared state machine an asset walks — its edges and guards, the actions an arrival carries, the events that move it, and when a write becomes visible."
generated: none
---

# Lifecycles and state

An asset is rarely in one condition for the whole of a model. A unit is vacant,
then leased, then vacant again; a loan is current, then delinquent, then cured
or defaulted. CFDL declares that condition as a machine rather than leaving it
as a string somebody remembers to set, which is what makes the states checkable,
the moves between them permitted or refused, and the whole walk observable in
the results.

## A lifecycle is a machine

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
it; re-entering the state re-arms it. Edge availability is the memory, so a
unit that goes delinquent, cures and goes delinquent again is the topology
walked twice. Nothing anywhere latches: an event fires on each occurrence too,
and once-ness is declared rather than enforced by the engine.
A guard reads state as the period opened and the model's own **settled cash**
strictly backward — at or before the previous period — which is how "the
rent stopped arriving" becomes a condition rather than a narration.

A pack declares the same machine for its domain types in `types.toml`, with
the same optional guards:

```toml
[[lifecycles]]
lifecycle_id = "cre.unit"
initial = "vacant"
states = ["vacant", "leased", "holdover", "month_to_month"]

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

## Arrival actions

A state can carry behavior. Arriving somewhere is an occurrence, and the
bookkeeping that belongs to it — resetting a counter, recording a shortfall,
striking a new rate — belongs on the arrival rather than in a separate event
that restates the condition.

```cfdl
lifecycle servicing {
  initial current
  state current, delinquent, defaulted

  on enter defaulted { set months_in_state = 0 }

  current    -> delinquent when asset.loan.paying < 0.5
  delinquent -> defaulted  when asset.loan.paying < 0.5 and asset.loan.months_in_state >= 2

  // The cure is not a payment. It is a payment plus a probation period.
  defaulted  -> current    when asset.loan.paying >= 0.5 and asset.loan.months_in_state >= 3
}
```

The counter is what makes "three months in this condition" sayable: a plain
recurrence climbs forever, so a duration measured from the last *arrival*
needs the arrival to reset it. That gap between a borrower starting to pay
again and the loan being performing again is not an artefact — it is what
supervisors mean when they write a probation period into the definition of
default.

There are two grains, and both are real:

- **`on enter <state>`** carries what is true of the STATE however it was
  reached, so it holds for every edge that arrives, including one added later.
  This is what a pack declares once for its domain types.
- **An action on an edge** carries what is true of the PATH taken. A renewal
  and a re-let both land in `leased` and strike rent differently; an entry
  action cannot say that, because it does not know which edge fired.

Entry actions run first, then the taken edge's — the state's own setup, then
the path's refinement. A same-field write journals the earlier value
`overridden`, naming its author, so a pack's default and a model's override
are distinguishable in the record rather than silently merged. Both run on
every traversal, including one an event causes by writing `status` across a
permission edge.

An action writes **fields, never `status`**: a status write would fire a
second transition inside the same period, and a transition that should cause
another transition is an edge out of the target state, taken next period. An
action reads the same world its guard does — state as the period opened,
settled cash strictly backward — which is what keeps the whole thing acyclic.

A schedule can anchor to a state entry —
`schedule every month from state_enter(asset.suite, building) for 18 periods`
— opening a fresh window at each entry, which is what "eighteen months of
construction from whenever construction starts" needs.

## Events move assets between states

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

## Where to go next

- [The object model](/docs/object-model) — the families a lifecycle is declared on.
- [Events and options](/docs/examples/options_events) — a model that runs.
- [Schedules and calendars](/docs/guides/schedules-and-calendars) — the grid a
  guard is asked on.
- [Domain packs](/docs/packs) — the lifecycles each pack declares for its types.
