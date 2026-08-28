---
id: guide-waterfalls
title: Waterfalls
slug: /docs/guides/waterfalls
description: "Declare a priority of payments: ordered steps sharing out a pot of cash, with hurdles, catch-ups, and a promote."
generated: none
---

# Waterfalls

A waterfall is a priority of payments: an ordered list of steps sharing out a
pot of cash. Each step takes what it is owed, up to what is left, and the
remainder passes down.

It is how a securitization pays its tranches, how a fund pays a preferred return
before carry, and how a project pays lenders before equity.

## The shape

```cfdl
waterfall deal.distribution on entity asset.trust {
  schedule every month from 2026-01 to 2030-12
  from available

  pay servicing to party.servicer    = 12500.0
  pay senior    to asset.class_a     = 6250.0
  pay residual  to party.certificate = remaining
}
```

`available` is the trust's own netted cash for the period — its streams' signed
values, with its children rolled up by `part of` — handed to the waterfall by
the engine the way `remaining` is. Write a narrower `from` expression only when
the deal distributes a narrower amount, such as a principal-only distribution.

Three parts:

- **`on entity`** — whose cash this is.
- **`schedule`** — when it runs. The same construct a stream takes.
- **`from`** — the pot.

Then the steps, in the order they are paid.

## One form for every step

A step is `pay <name> to <payee> = <expr>`. There is no separate syntax per kind
of payment, because every kind is arithmetic:

| what you want | how to write it |
|---|---|
| a fixed amount | `= 12500.0` |
| capped at a limit | `= min(fee, cap)` |
| pay a balance down to a target | `= asset.class_a.balance - asset.trust.pool_balance` |
| fund a reserve to a target | `= max(0.0, inputs.reserve_target - prev.reserve)` — see the account, below |
| only on a certain date | `= if(time.t >= 24, balance, 0.0)` |
| an earlier step's shortfall | `= owed.trustee_fee - paid.trustee_fee` |
| everything left | `= remaining` |

## Three names a step can read

On top of everything an ordinary expression sees:

| | |
|---|---|
| `remaining` | what is still in the pot at this step |
| `paid.<step>` | what an earlier step actually paid |
| `owed.<step>` | what an earlier step would have paid, unbounded |

A step also reaches any other entity's declared properties by naming it —
`asset.class_a.original_balance` — which is how one tranche's step reads
another's balance. `entity.asset.class_a.original_balance` is the same read
spelled the long way.

`owed` and `paid` differ exactly when a step could not be paid in full, so their
difference is the shortfall. That is how a capped fee gets its overflow paid
later, and how a step measures a balance "after giving effect to" the payments
above it.

A step may only read steps **declared before it**. Reading a later one is a
compile error, because a priority of payments is an order, not a system of
equations.

## The pot never goes negative

Every step pays `min(max(0, your expression), remaining)`.

You do not have to write `min(..., remaining)` yourself, and a step that asks
for more than is left simply takes what is left. A negative expression pays
nothing rather than clawing cash back.

That also makes `= remaining` mean exactly what it says.

## Say where the remainder goes

At least one step must read `remaining`, or the model does not compile.

Without it, whatever survives the last step would vanish with nothing to show
for it. Naming the residual is the difference between a model that says the
sponsor keeps the excess and one that quietly loses it.

## When it runs

A waterfall is a post-cash-flow distribution: it runs after the period's streams
and states are known, so it allocates money that already exists. It never feeds
a stream in the same period.

Two shapes cover most deals, and a model can use both at once:

```cfdl
// Every period — a distribution date, a debt service cascade.
schedule every month from 2026-01 to 2030-12

// Once — an exit, a liquidation, a final recoupment.
schedule on 2030-12
```

## Cash that accumulates: the account

`available` is this period's cash, and a waterfall distributes only at the
periods its schedule names. What accumulates in between — a reserve building
toward a target, proceeds waiting for a quarterly date, trapped cash held
across a breach — lives in a declared **account**:

```cfdl
account reserve { }

waterfall dist on entity asset.suite {
  schedule every month from 2026-01 to 2026-12
  from available
  pay top_up   to account reserve = max(0.0, 300.0 - prev.reserve)
  pay residual to party.sponsor   = remaining
}

waterfall release on entity asset.suite {
  schedule on 2026-06
  from reserve
  pay released to party.sponsor = remaining
}
```

Three uses, all in those lines. A **step pays into an account** — the
reserve pattern (fund to target, top up when short) as one step form, with
`prev.<account>` reading the settled balance. A **waterfall draws from an
account** — `from reserve` hands it the accumulated balance in place of a
hand-written cumulative window, on its own schedule, and residue after the
last step stays for the next date. **Logic reads a balance** — a guard or
rule reads `prev.<account>` the way it reads any settled history, and the
read is unavailable (not zero) at period 0.

The balance follows one law — carried balance, plus the account's declared
`from` inflow, plus what steps allocated in, minus what a drawing waterfall
took — and it has **no floor**: an account fed a deal's whole net cash is
the deal's cumulative position, negative through the J-curve. What a step
may take is floored at zero, because cash that is not there cannot be
allocated. An `owner` makes it a party's: `pay … to <party>` then
accumulates there, and the balance is allocated cash, not an obligation.

The step's series is the flow; the account's balance is the position,
published as the non-cash series `account.<name>` with every movement —
inflow, allocation in, allocation out — journaled with the balance before
and after.

## What comes out

Each step publishes as a series named `stream.<waterfall>.<step>`, and its cash
counts toward the payee's total. A waterfall is not a separate kind of output:
statements, metrics and the results document read it the way they read anything
else.

## One waterfall's output as another's pot

Because steps publish as series, a later waterfall can draw on an earlier one:

```cfdl
stream fund.sale_proceeds on entity asset.fund inflow currency USD {
  schedule on 2025-01
  amount = inputs.proceeds
}

waterfall fund.distribution on entity asset.fund {
  schedule on 2025-01
  from available
  pay carry    to party.gp = remaining * 0.20
  pay lp_share to party.lp = remaining
}

waterfall firm.carry_split on entity asset.mgmt_co {
  schedule on 2025-01
  from series_sum("fund.distribution.carry", 0, 5)
  pay team_pool  to party.team     = remaining * 0.40
  pay firm_share to party.founders = remaining
}
```

The rule is the same order that governs steps: a waterfall may read any
waterfall **declared before it**. That is what a fund carry rolling into a
management company and out again to a stakeholder needs, and it keeps the second
pot as a computed number rather than an assumption holding a stale copy of it.

## Worked structures

Seven real waterfalls are encoded in the test suite, and between them they use
only the rules above.

**An auto ABS note stack, checked against the issuer's own figures.** Six
classes of notes repaid in strict seniority out of the principal a pool of car
loans throws off. An exhibit filed by the trust states, for each class and every
monthly distribution date, the percent of that class still outstanding — and the
model reproduces all 208 published cells to within 0.0054 percentage points.
The exhibit rounds to 0.01, so that is the floor a reader can check against.

This is the one to read if you want to know whether the construct is right
rather than merely expressive: the collateral underneath it was already
reconciled to the cent by a separate case, so any disagreement could only have
been the waterfall. See [the case](/docs/examples/credit-auto-abs-tranches).

**A 22-step consumer ABS priority of payments** — servicer and trustee ahead of
the notes, five rated classes taking interest then principal in strict
seniority, a reserve topped to its specified level, an overcollateralization
target, and a certificateholder taking what survives.

**A private fund carry waterfall** — capital back, a compounding 8% preferred
return, a full GP catch-up, then 80/20. It reproduces its published figures
exactly, and one definition covers three published structures: the only thing
separating them is what the catch-up is computed on, which is an argument.

**An LBO exit split** — the sponsor's converted preferred against management's
rollover and exercised options.

**An IRR-hurdle waterfall** — three participants whose vested percentages step
up as the LP's return crosses eight hurdles.

**A GP stakes nested split** — a fund waterfall, then the management company's
split with the deal team, then the founders against a passive minority investor.
Three waterfalls, each drawing on the one above.

**A partial catch-up** — the same fund waterfall where the GP takes half of each
dollar above the preferred instead of all of it. Set the catch-up rate to 1 and
it returns the full catch-up's numbers exactly; the two structures are one
expression with a different argument.

### On hurdles and catch-ups

Both of the last two are commonly said to need an iterative solver. Neither
does, and the reason is worth knowing before you reach for one.

A **catch-up** that pays the GP 20% of everything distributed in two tiers
combined is `X / (pref + X) = 0.20`, so `X = pref / 4`. One division.

An **IRR hurdle** does not solve for a rate — the rate is an input. What is
unknown is the payment that reaches it, and present value is linear in a payment
at a fixed rate. Where a hurdle also selects among tiers, the thresholds are
computable in advance and choosing between them is an ordered comparison.

So a waterfall step stays an expression, and the language needs no solver to
carry these structures.

## Related

- [The object model](/docs/object-model) — assets, parties and the entities a
  waterfall pays.
- [Reading results](/docs/guides/reading-results) — where the per-step series
  land.
- [Auto ABS note classes](/docs/examples/credit-auto-abs-tranches) — a
  sequential-pay stack reconciled against an issuer's published grid.
