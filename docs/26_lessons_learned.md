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

### A covenant does not need a metrics reduction

**Claimed:** `domain.cre.dscr` is a per-period series but nothing reduces it, so
"never below 1.20" cannot be stated and the covenant cannot be asserted. Fixing
it needs a `min` reduction at the metrics layer.

**Actually:** the reduction is genuinely absent — `subtotal_total` sums and there
is no `min` — but the conclusion does not follow, and the reduction is not what
makes a covenant assertable.

A benchmark asserts subtotal COLUMNS, not just scalars.
`benchmarks/cre/office_two_tenant/expected.csv` carries `domain.cre.dscr` as a
per-period column, checked every period against a tolerance. Every year is
already tested, which is strictly stronger than testing the worst one.

The covenant's EFFECT is an ordering, not a construct. A cash trap is the
lender's step placed above the equity step; if the step above takes the cash,
there is nothing left to distribute. Conditional steps are ordinary — five ship
in `fixtures/valid/waterfall_abs_22_step`, e.g.
`pay class_b_final to asset.class_b = if(time.t >= 5, …, 0.0)`.

Note that `when` as a step modifier does not exist and is not missing.
`docs/17` §3 proposed six payment forms plus `when`; §13 records what was built
— ONE form, an arbitrary expression per step — because `if`, `min`, `max` and
`clamp` already cover all seven. The canonical grammar is
`waterfall_step = "pay" IDENT "to" entity_ref "=" expr ;`.

What was left over is a minimum-coverage scalar for a credit memo. That is an
underwriting summary statistic, and this language is not the loan-approval
surface: read it off the published series with the Python SDK.

**Do not add language surface for a structure with no terms.** A sweep that
amortises the loan — a turbo, as opposed to a cash trap, which leaves the
balance alone — is a contract with conditions and rules, and cash applied to
financing resolves at a period boundary: after that period's results, or at the
next period's open from prior activity. Until those terms are written down, the
requirement is phantom and the language should not move to meet it.

### A contract term that "needs to accept a stream reference" already does

**Claimed:** three CRE requirements — vacancy tracking the rent roll, a
management fee as a percentage of EGI, an expense stop resetting to a later
year's actual opex — each need a new pack term that accepts a stream
reference, or a new lowering rule that reads one.

**Actually:** none of the three needed a pack change. A contract term already
holds an expression, an expression may name another stream through
`series_sum`, and the term's text splices into the lowering rule's
`amount_expr`. The only thing missing was the ENGINE's evaluation order, which
allowed a reader to read only non-readers; once streams evaluated in
dependency-ordered waves, all three worked as written.

The reasoning failed in a specific and repeatable way: **a capability gap and a
demonstration gap look identical from the backlog.** All three items had real
provenance — someone hit a wall modelling a real deal — but the wall was one
layer down from where the item placed it, so the remedy each item proposed was
for the wrong layer. Probe the spelling before designing the term: if the shape
compiles and runs, the item is documentation, not development.

## How to achieve a behavior

### A line derived from other lines

A contract term holds an expression, and the expression may read another
stream. That is the whole mechanism behind "this line is a percentage of that
one" — no pack term is required for it, and the number never goes stale.

```cfdl
contract cre.vacancy_loss on entity asset.strip_center {
  terms {
    rate = if(time.date >= date(2030, 1, 1), 0.46, 0.03)
    potential_gross_year = series_sum("cre.unit.base_rent.*", time.t, time.t) * time.ppy
  }
}
```

`time.ppy` annualizes rather than a hard-coded twelve, so the term follows the
model's calendar. A window pinned to a period reads a specific year — an
expense stop resetting to the 2028 actual is
`series_sum("cre.opex.line", 24, 24) * time.ppy`. Outflow streams book signed
negative, so a read of one is usually negated.

Name the read as narrowly as the economics allow. A management fee reading
`cre.opex.line.*` reaches the recoveries that read the fee, and the engine
refuses the loop by name:

```
cyclic series reads: 'cre.opex.line.management' -> 'cre.unit.recoveries.anchor'
-> 'cre.opex.line.management'.
```

Reading `cre.opex.line` exactly is the fix, and the diagnostic says so rather
than answering with a number. `fixtures/valid/cre_derived_lines` carries all of
this; `packs/cre/templates.toml` ships the two patterns as templates.

### An assumption derived from other assumptions

`assume` values resolve in dependency order, so one may be computed from
others and stating a number once beats restating it:

```cfdl
assume gross_sf   = 10000.0
assume efficiency = 0.85
assume net_sf     = inputs.gross_sf * inputs.efficiency
```

Declaration order and name order are both irrelevant. Random assumptions
resolve first, as leaves, so a derived assumption may be built on one. A
circular derivation is refused with the cycle named — the same rule as series
reads one layer down, and for the same reason: no order satisfies it, and the
engine does not iterate toward a fixed point.


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
