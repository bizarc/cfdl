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

### A valuation date that is not on a fiscal-year boundary

**Claimed:** `time calendar <c> from <d> for <n>` produces periods of one
length, so a valuation dated off a fiscal-year boundary has a partial first
period the language cannot express. The calendar needs a leading stub, so
period lengths are `[stub, p, p, ...]`.

**Actually:** THE CALENDAR IS NOT THE DEAL'S FISCAL YEAR. It is a neutral
coordinate grid — a 120-month grid can host deals active for a few periods of
it — and a deal's fiscal convention is a property of the deal, mapped onto the
grid by DATE. Putting a stub in the calendar pushes one deal's fiscal shape
into the shared axis.

Calendar and schedule are separate concepts and each already carries its half:

- The **calendar** (`docs/01` §6.1) takes any start date, so the grid origin
  can simply be the valuation date.
- The **schedule** (`docs/01` §11.2) carries placement — `due` for the start of
  a period, the default for its end, `mid` for halfway — plus day rules
  (`on day <n>`, `on eom`), business-day conventions, and stub policies
  (`short_front`, `long_front`, and the back forms). The stub concept exists
  here, where the recurrence is, not on the grid.
- Discounting reads `(period + offset) / ppy` continuously (`docs/12`), so a
  fractional position is an ordinary number, not a special case.

A 30 September valuation with fiscal years ending 30 June, on a plain monthly
grid:

```cfdl
time calendar monthly from 2026-09-30 for 60

stream fy.cash on entity asset.co inflow currency USD {
  schedule every year start from 2027-06-30 to 2031-06-30
  amount = 1000.0
}
```

lands at 0.75, 1.75, 2.75, 3.75 and 4.75 years out — exactly the off-cycle
spacing a banker's DCF needs.

**The trap that produced the claim.** Omitting `due` places the payment at the
END of each annual period, roughly eleven months later, which reads as the
schedule "drifting" off the fiscal year end. It is not drift; it is the
documented default. Say where in the period the cash sits.

### One axis is one field, not three booleans

Where a flow sits in its period was spelled as three independent flags —
`due`, `mid`, `at_period_end` — in the parser, the IR, the engine and the pack
interface. Three consequences followed, and they are the signature of this
mistake wherever it appears:

- **A contradictory state was representable**, so it needed a runtime check
  (`E2109`) to reject `due mid`.
- **Coverage drifted between the layers.** A one-shot could say `mid` but not
  `end`, so a disposal's reversion could only be placed by a pack rule, never
  by a hand-written model — the error that discounted a CRE reversion a period
  short, worth 9% of it at 12%.
- **The vocabulary diverged.** The same position was `due` on a recurrence,
  a default on a one-shot, and `schedule_at_period_end` in a pack.

It is now `Placement { Start, Mid, End }`, one field, spelled `start`/`mid`/
`end` everywhere including pack rules. The contradictory state is unwritable
rather than rejected, so `E2109` shrank to what it should always have been:
clashes across DIFFERENT axes — a placement against a day rule, or against
`net` payment terms.

**Defaults that differ by form are not a reason to leave a position
unnameable.** A recurrence defaults to `end`, a one-shot to `start`, and no
single constant covers both — which is exactly why every position must be
statable in both forms, so a model never depends on which default applies.

The refactor is the evidence it was mechanical: 17 IR goldens changed shape and
**not one number moved** — the 17 results goldens that differ do so only in
hash fields, which necessarily follow the IR.

### Payment terms discount at bucket granularity, on purpose

**Claimed:** `net <n>` honours its lag only to whole periods and drops the
remainder from discounting, so `net 45` on a monthly grid loses fifteen days.
A fractional residual should be carried out of the bucketing step and added to
the stream's offset.

**Actually:** that is a stated convention, and `docs/12` §"Discounting is at
bucket granularity" says so in those words — *"This is a stated convention, not
an oversight."* A payment is discounted from the period it lands in; the
fraction between that period's boundary and the actual due date is not
modelled, and is worth roughly 0.5% on an affected flow on a monthly grid at
12%. The first-order effect — moving the cash two periods later — IS captured.
Measured, billing at the close of period 0 on a monthly grid:

| terms | lands in | discounted from |
|---|---|---|
| `net 0` | period 0 | 1.0 periods |
| `net 30` | period 2 | 3.0 periods |
| `net 45` | period 2 | 3.0 periods |
| `net 60` | period 3 | 4.0 periods |

`net 30` and `net 45` landing in the same bucket is the convention working, not
failing: within a bucket a stream has ONE position, which is its placement —
`end` by default for a recurrence, so the close.

**Two errors to avoid when reasoning about this.** The residual is not an extra
discount, so a `net 45` flow is not penalised fifteen days against a `net 30`
one; and the placement is not an over-discount either, because a period's cash
is summarised at a single point by design, and which point is what
`start`/`mid`/`end` selects.

Removing the convention is not a bug fix but an architectural change — giving
each PAYMENT its own discount offset rather than one per stream, in
`npv_with_offsets` — and it would move numbers in every model that uses `net`.
If that is ever wanted, it belongs as an item stated that way. It is also why
`mid` combined with `net` is refused (`E2109`): composing them means answering
this question, and picking a default quietly would answer it wrongly.

### A quantity quoted to a tick, and the staircase it makes

`round_to(x, step)` rounds to any tick — not just powers of ten, so an eighth,
a quarter-cent or a 25-unit lot all work. It rounds ONE value; the recurrence
is built around it.

**A rounded recurrence is an entity field**, because `next` reads `prev`:

```cfdl
entity asset home_project : CRE.Asset.RealProperty {
  opex_management init inputs.opex_management
       next round_to(prev * (1 + inputs.opex_trend), 1)
}
```

Each year escalates the previous year's ALREADY-ROUNDED figure, which is what
a published schedule does and what no closed form reproduces. A stream cannot
read its own prior period; a field can, and that is the whole difference.
`benchmarks/cre/hud_home_multifamily` reproduces its source this way on five
expense lines. Where the staircase happens to have a closed form — a statutory
credit escalating off a fixed base — one `round_to` call inside the amount is
enough, as `energy.ptc` does.

**Check the unit before choosing the step.** The production tax credit is
published "to the nearest 0.1 cent per kWh", which on a per-MWh quantity is
`1.00`, not `0.10`: 0.1 cent = $0.001/kWh x 1000 kWh = $1.00/MWh. Rounding a
per-MWh figure to 0.10 rounds to a hundredth of a cent, which is
indistinguishable from not rounding at all — the pack default is `1.00` for
exactly this reason, and only an external reference caught it.

**Why an omitted staircase hides.** Carrying the credit continuously was wrong
by up to 1.8% in a single year and about -0.3% over ten, because the error
ALTERNATES SIGN rather than drifting. In aggregate it looks like noise, so it
survives reconciliation against anything but a source that rounds — and a debt
sizing struck off one year's coverage feels the 1.8%.

## How to achieve a behavior

### A balance swept by the period's free cash flow

A recurrence cannot read the model's own streams — a field's `next` sees
`prev`, other fields' `prev`, `time.*`, `inputs`, `cfg`, `obs` and curves, and
no series at all. That does NOT make a cash sweep inexpressible. It means the
quantity being swept has to be something a field can see, and the way to get
that is to let the FIELD own it:

```cfdl
entity asset co : Asset.Financial {
  // Free cash flow, stated ONCE.
  fcf     init (curve_value("ebit", time.date) * (1 - inputs.tax_rate))
               - curve_value("capex", time.date)
          next (curve_value("ebit", time.date) * (1 - inputs.tax_rate))
               - curve_value("capex", time.date)

  balance init 3000.0 next max(0.0, prev - prev.asset.co.fcf)
}

// The published line reads the same field, so nothing is computed twice.
stream opco.fcf on entity asset.co inflow currency USD {
  amount = asset.co.fcf
}
```

The balance tracks the sweep to the cent, and the free-cash-flow build exists
in exactly one place. `prev.<entity>.<field>` inside a `next` is that field's
value in the period being left, so there is no off-by-one — but note `init`
must compute the first period too, or period 0 silently sweeps nothing.

**Express the operating drivers as curves.** A recurrence may read a curve and
may not read a series, so a curve-shaped operating case is what lets a balance
see the deal. `benchmarks/opco/lbo_financing_cases` sweeps a Term Loan B this
way and reproduces its reference's MoIC and IRR — and its balance/interest
circularity is affine, so collecting terms solves it in one substitution
instead of iterating.

**When the quantity is only knowable from stream output** — a pack-lowered
contract's cash, or what a waterfall actually paid under a short pot — no field
can see it and the arithmetic must be restated. That is a real cost, not a
missing capability: `americredit_2017_1` carries seven class balances each
duplicating its step-down expression. Reach for it only after checking whether
the quantity can be a field the streams read instead.

### A term whose value is derived from other inputs

A contract term holds an expression, so anything a modeller would work out on
paper before typing it belongs in the model instead. The rule: **state the
figures the source gives and the identity that combines them, never the
product.**

Taking an investment credit conventionally removes half the credit from the
depreciable basis, so a $100m project claiming a 30% ITC depreciates $85m:

```cfdl
assume installed_cost = 100000000
assume itc_rate       = 0.30

contract energy.itc on entity asset.pv {
  terms { credit = inputs.installed_cost * inputs.itc_rate }
}

contract energy.macrs_shield on entity asset.pv {
  terms { basis = inputs.installed_cost * (1 - 0.5 * inputs.itc_rate) ... }
}
```

A pasted `85000000` with the arithmetic in a comment beside it is the failure
mode: change the cost or the rate and the constant is silently wrong, and here
being wrong means depreciating the FULL basis, which overstates the shield by
17.6% for the life of the schedule with nothing to object.

**A pack declining to derive something is not a reason to hardcode it.**
`energy.macrs_shield` takes `basis` as an input deliberately — basis
adjustments are jurisdictional and a wrong default is worse than none — so the
model must say WHICH adjustment applies. Saying it and pre-computing it are
different things.

### A rate quoted on a different cadence than the term takes

Practitioners quote monthly SMM and MDR; the credit pack's `cpr`/`cdr` terms
take annual figures. The conversion does NOT have to be done by hand and
pasted in as `0.11361512828387077` — a term holds an expression, so state the
quoted figure and the identity that converts it:

```cfdl
contract credit.pool_level_pay on entity asset.pool {
  terms {
    cpr = 1 - pow(1 - 0.01, time.ppy)   // a 1% SMM pool
    cdr = 1 - pow(1 - 0.0005, time.ppy) // a 0.05% MDR pool
  }
}
```

`time.ppy` rather than a literal 12, so the conversion follows the model's
calendar instead of assuming a monthly grid. Verified byte-identical to the
hand-computed constant across every stream and all 361 periods of a 30-year
pool — this is a legibility idiom, not a different number.

The same shape covers any quoted-cadence mismatch: what belongs in the term is
the figure the source states plus the identity, never a pre-multiplied constant
that no reader can check and that goes stale silently if the quoted figure
changes.

### A liability stack — notes, subordination and a distribution waterfall

An ABS capital structure is ONE waterfall: one set of distribution
instructions, executed as ordered steps, on a stated distribution date. Free
cash accumulates between dates and cascades when the date arrives.
`benchmarks/credit/americredit_2017_1` is the worked example — 22 prospectus
clauses, 30 `pay` steps, reproducing the published grid and all 48 weighted
average lives.

**State a claim, never a payment.** A step says what a class is OWED; the
engine pays it out of what is left. Pre-computing a payment and then
reconciling it restates what `remaining` already decides, and the two drift the
moment the pot is short.

**A step has no guard.** `waterfall_step = "pay" IDENT "to" entity_ref "="
expr` — a step always executes and declines to fire by evaluating to zero.
`if(time.t >= 12.0, bal_a1, 0.0)` is a trigger written as arithmetic, and
`max(senior_balances - pool_prior, 0.0)` is a test that pays nothing when it
passes. On the AmeriCredit deal 13 of the 30 steps never pay a cent, which is
faithful to an indenture whose loss-cure clauses exist but are not reached.

**A capped fee's overflow is `owed.<step> - paid.<step>`**, paid at a lower
priority. Clause 21 of that deal is exactly this.

**Whether a class needs a BALANCE depends on one question: does what is
distributed equal what is produced?**

When it does — plain sequential pay — there is no balance. The outstanding is
derived from cumulative collections and the waterfall reads it:

```cfdl
stream notes.a_outstanding on entity asset.trust inflow currency USD {
  amount = max(0.0, inputs.a_face - series_sum("pool.principal", 0, time.t - 1))
}
```

Interest accrues on that stream, principal is capped by it, the class receives
exactly its face, and nothing is carried or restated. (A chain like this is a
depth-2 series read, which only became expressible when streams began
evaluating in dependency-ordered waves.)

When distribution DIVERGES from production — a step-down amount, an
overcollateralization redirection, losses, capitalizing interest — the
outstanding is knowable only from what was actually paid, and a field cannot
read a waterfall's steps. The balance then has to be a field whose `next`
restates the distribution arithmetic, which is why AmeriCredit carries seven
balance fields each duplicating its step-down expression. Do not reach for that
shape until the divergence is real: it costs the same formula written twice and
required to stay in sync.

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


### A regime that changes once and stays changed

The mirror of the entry below, and the LATCHING property decides which you
want. An event fires at most once per run (`docs/01` §13.1), which makes it
wrong for something that recurs and exactly right for a permanent transition:
a rent restriction expiring, a PPA term ending and revenue going merchant, a
teaser rate resetting. Carry the regime as a field, clear it with an event, and
let every line that cares read the field:

```cfdl
entity asset home : CRE.Asset.RealProperty {
  restricted init 1.0
}

event affordability_expires when time.t >= inputs.restricted_years {
  set entity asset.home.restricted = 0.0
}

contract cre.lease_unit.home on entity asset.home {
  terms {
    rent_year = if(asset.home.restricted == 1.0,
                   inputs.rent_restricted_y1,
                   inputs.rent_market_y1)
    escalation = inputs.rent_trend
  }
}
```

**Why this beats a pack term, and why no pack should add one.** The shape
recurs in every pack — CRE restriction to market, energy PPA to merchant,
credit fixed to floating — so a `reverts_after` term would be built four times,
each with its own sentinel for "never reverts", each less expressive than the
expression it replaced. The event form reverts on any condition a date, a
curve crossing or a balance can state, not just a year offset.

**It also makes the boundary auditable, which a convention cannot.** The run
publishes the transition:

```json
{"period": 14, "date": "2038-01-01", "entity": "asset.home_project",
 "field": "restricted", "from": "1", "to": "0", "event": "affordability_expires"}
```

`benchmarks/cre/hud_home_multifamily` reproduces its source workbook to the
cent this way. Its affordability period is the case in point: the workbook's
own switch fires a year earlier than its "15-year" label reads, and that
discrepancy used to survive only as a comment. It is now a record in the
results that can be checked against the source, rather than an off-by-one a
reader has to re-derive from a `<`.

Lines keyed to the regime read the STREAM, not the switch — vacancy is
`inputs.vacancy_rate * series_sum("cre.unit.base_rent.*", time.t, time.t)` —
so the transition is stated once and nothing restates it.

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
