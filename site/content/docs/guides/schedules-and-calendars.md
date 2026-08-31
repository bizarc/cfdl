---
id: guide-schedules
title: Schedules and calendars
slug: /docs/guides/schedules-and-calendars
description: "One master timeline, and how every stream occurrence lands on it: calendars, schedules, stubs, and day rules."
generated: none
---

# Schedules and calendars

Every model has one master timeline; every stream occurrence lands on it.

## The timeline

```cfdl
time calendar monthly from 2026-01 for 72
```

Add a projection tail when a valuation needs periods beyond the hold
(e.g. exit on forward NOI):

```cfdl
time calendar monthly from 2026-01 for 120 project 12
```

Phases name sub-ranges for organization and scoping:

```cfdl
phase lease_up from 2026-01 to 2027-06
```

### The calendar is an assumption when events are involved

An [event](/docs/lifecycles) fires in the first period where its condition
holds, and never between periods. So the calendar sets how precisely a
condition can be met — which matters as soon as an event decides who gets paid.

An annual grid asks a covenant test, a flip test or a cash-trap test twelve
times less often than a monthly one, and each miss delays the consequence to
the next year end. On the [tax-equity flip](/docs/examples/energy-tax-equity-flip)
the same deal flips ten months later on an annual calendar than a monthly one,
moving about $3.5m.

Choose the grid the test is actually performed on, not the grid the reporting
happens to use.

## Schedule patterns

One-time:

```cfdl
schedule on 2026-06
```

Recurring:

```cfdl
schedule every month from 2026-01 to 2026-12
```

Day rules:

```cfdl
schedule every month on day 15 from 2026-01 to 2026-12
schedule every month on eom from 2026-01 to 2026-12
```

`YYYY-MM` dates normalize to first-of-month; `from` must be ≤ `to`, and
occurrences must land inside the model timeline.

State-anchored (the third anchor, beside dates and phases):

```cfdl
schedule every month from state_enter(asset.site, building) for 18 periods
```

Each entry of the entity into the state opens its own window of n grid
periods; a re-entered state re-anchors with a fresh window. This is what
"eighteen months of construction from whenever construction starts" needs —
the machine enters the state whenever its edge fires, and the schedule hangs
off the entry. See [lifecycles and state](/docs/lifecycles).

## Day counts

Accrual-style math uses `year_frac(d1, d2, basis)` in expressions, with
`"30/360"` (US/bond), `"act/360"`, `"act/365"`, and actual/actual bases —
decimal-exact. Date helpers: `eomonth`, `edate`, `parse_date`,
`months_between`.

## Reference links

- [Language spec — time & schedules](/docs/specification/language-spec)
- [Expression environment — date functions](/docs/specification/expression-environment)
