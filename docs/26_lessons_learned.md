# Lessons learned — internal

Not published. `site/scripts/sync-content.mjs` publishes `docs/01`–`docs/09`;
this file, like the backlog, is repository-only.

Two things belong here, and nothing else does.

**Corrected reasoning.** A claim about the language that was investigated and
found wrong, kept so it is not raised again. The backlog holds work to do; when
an item turns out not to be work, the item goes and the reasoning comes here.

**How to achieve a behavior.** A shape the language already supports that was
not obvious enough to find, written down once so the next reader does not
rediscover it by probing.

A true capability belongs in the language documentation, not here.

---

## Corrected reasoning

### A balance drawn down by a payment does not need new syntax

**Claimed:** `balance(t) = balance(t-1) - paid(t-1)` is inexpressible; it needs
`prev.<stream>` and `prev.<waterfall>.<step>`, and the engine must be
restructured from layer-major to period-major to support them.

**Actually:** state the amount ONCE as a field. The waterfall step pays that
field and the balance subtracts it, because a step reads a rule-bearing entity
field at the current period.

```
field  pay_amt        0   250   250   250   250
field  bal         1000  1000   750   500   250
step   principal      0   250   250   250   250     <- reads the field
```

The restructure was never the work, and the field/stream boundary `docs/14`
draws — a field reads no cash — holds and does not need to bend.

### There is one pot, and `remaining` draws it down

**Claimed:** undistributed cash is a balance nobody keeps, so a waterfall
distributing more than once cannot state its own pot.

**Actually:** there is one pot per deal. Within a run `remaining` draws it down;
across periods the model states the window the `from` expression draws on, which
`docs/03` §3.2 keeps free for exactly that. Both shapes distribute the cash
exactly once:

```
end of hold, from series_sum(fcf, 0, time.t):  600 -> pref 400, gp 40, lp 160
every period, from available:                  each period's cash, in full
```

The wrong answers came from two malformed models: two waterfalls on one entity
each drawing the same pot, and a note asking to amortize faster than its
collateral can pay. Neither is something the engine should reconcile.

### A decision does not belong in a contract

**Claimed:** a contract needs `ends when` to model a clean-up call, and needs
`active when` so a repaid loan stops paying.

**Actually:** a contract records what was agreed, and its `term` states when the
obligations run. A prepayment, a termination, a default, a buy-out is a
modelling decision, and the language already splits it — an `option` holds a
right with `exercise when` and `payoff`, an `event` writes entity state, and a
stream's amount or guard reads that state. Putting a guard on the contract moves
a decision into the record of the agreement.

### `time.ppy` is the cadence-neutral divisor

**Claimed:** the calendar's grain is a pack privilege; a hand-written model must
restate `assume year_months = 12.0`.

**Actually:** `docs/03` §3 documents `time.ppy` and `time.days_in_period`.
`inputs.rent_year / time.ppy` pays 3000 a quarter on a quarterly book. The
original investigation probed five undocumented spellings and missed the
documented one.

### Distributions never reach a cash flow statement, by design

A distribution is not an operating, investing or financing flow. Subtotals
folding before waterfalls run is the correct order. A separate WATERFALL
statement may be added one day; it is a different statement.

---

## How to achieve a behavior

### A regime that turns on and off repeatedly

Events LATCH — at most one fire per run (`docs/01` §13.1) — so an event is not
the mechanism for something that recurs. Use a guard, which is level-triggered
and re-evaluated every period, and a field to publish the regime:

```cfdl
entity asset plant : Asset.Real {
  curtailed init 0.0 next if(curve_value("price", time.date) < 50.0, 1.0, 0.0)
}

stream plant.revenue on entity asset.plant inflow currency USD {
  schedule every year from 2026-01 to 2031-01
  active when asset.plant.curtailed == 0.0
  amount = curve_value("price", time.date)
}
```

The field flips both ways and publishes as a series; the stream follows it.

### Activating different things in different phases

Phases carry the date logic; schedules reference them. No guard is involved.

```cfdl
phase construction from 2026-01 to 2026-09
phase operations   from 2026-10 to 2028-12

stream plant.capex   ... { schedule every month from phase_start("construction")
                                                to phase_end("construction") }
stream plant.revenue ... { schedule every month from phase_start("operations")
                                                to phase_end("operations") }
stream plant.commissioning ... { schedule on phase_enter("operations") }
```

An expression reads the phase it is in as `time.phase`. A value that varies by
phase is a curve keyed to the boundaries.

### A date in an expression

Expression literals are numbers, booleans and strings (`docs/03` §2). A bare
`2022-01-01` in an expression is arithmetic — 2022 minus 1 minus 1. Write
`date(2022, 1, 1)` or `parse_date("2022-01-01")`.

### Stopping a contract's cash

A contract runs its term. To stop what it pays, write the decision as entity
state and have the amount read it — the expression environment binds `entity.*`
to the stream's owning entity, so no name is needed:

```cfdl
event called when <threshold> { set entity asset.pool.status = "called" }
```

```
amount_expr = "... * if(entity.status == \"called\", 0.0, 1.0)"
```

An event naming a lowered stream in `deactivate stream` does not resolve today
(§7.50).
