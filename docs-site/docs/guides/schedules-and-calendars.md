---
id: guide-schedules
title: Schedules & Calendars
slug: /guides/schedules-and-calendars
---

# Schedules & Calendars

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

## Schedule patterns

One-time:

```cfdl
schedule on 2026-06
```

Recurring:

```cfdl
schedule every monthly from 2026-01 to 2026-12
```

Day rules:

```cfdl
schedule every monthly on day 15 from 2026-01 to 2026-12
schedule every month on eom
```

`YYYY-MM` dates normalize to first-of-month; `from` must be ≤ `to`, and
occurrences must land inside the model timeline.

## Day counts

Accrual-style math uses `year_frac(d1, d2, basis)` in expressions, with
`"30/360"` (US/bond), `"act/360"`, `"act/365"`, and actual/actual bases —
decimal-exact. Date helpers: `eomonth`, `edate`, `parse_date`,
`months_between`.

## Reference links

- [Language spec — time & schedules](/language-reference/language-spec)
- [Expression environment — date functions](/language-reference/expression-environment)
