---
id: example-options_events
title: "Events and options"
slug: "/docs/examples/options_events"
description: "An event fires when its condition first becomes true."
---

An **event** fires when its condition first becomes true. It can set entity
state, deactivate a stream, and exercise an option.

An **option** is a payoff that only lands if it is exercised — here by the
event, at month 12, for 15,000.

The debt service stream stops two ways: the event deactivates it, and its own
`active when` reads the state the event set. Either alone would be enough;
together they show both mechanisms.

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "tutorial-options-events"
time calendar monthly from 2026-01 for 24

entity asset senior : Asset.Financial

// An EVENT fires when its condition first becomes true. It can change
// entity state, switch streams off, and exercise an option.
event refinance when time.t >= 12 {
  set entity asset.senior.status = "refinanced"
  deactivate stream loan.debt_service
  exercise option refi_fee
}

// The stream stops two ways: the event deactivates it, and its own
// `active when` reads the state the event set. Either alone is enough.
stream loan.debt_service on entity asset.senior outflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 4200
  active when entity.status != "refinanced"
}

// An OPTION is a payoff that only lands if it is exercised.
option refi_fee type Option.Refinance {
  exercise when false
  payoff 15000
}
```
